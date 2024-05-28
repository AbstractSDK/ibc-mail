use abstract_adapter::sdk::{
    features::ModuleIdentification, AccountVerification, ModuleRegistryInterface,
};
use abstract_adapter::std::{
    ibc_client,
    objects::{account::AccountTrace, chain_name::ChainName, module::ModuleInfo},
    version_control::NamespaceResponse,
    IBC_CLIENT,
};
use abstract_adapter::traits::AbstractResponse;
use cosmwasm_std::{to_json_binary, wasm_execute, CosmosMsg, Deps, DepsMut, Env, MessageInfo};
use ibcmail::{
    client::api::ClientInterface,
    server::{
        msg::{ServerExecuteMsg, ServerIbcMessage},
        ServerAdapter,
    },
    Header, IbcMailMessage, Recipient, Route,
};

use crate::{
    contract::{Adapter, ServerResult},
    error::ServerError,
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: Adapter,
    msg: ServerExecuteMsg,
) -> ServerResult {
    match msg {
        ServerExecuteMsg::ProcessMessage { msg, route } => {
            process_message(deps, env, info, msg, route, app)
        }
    }
}

fn process_message(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: IbcMailMessage,
    route: Option<Route>,
    mut app: Adapter,
) -> ServerResult {
    println!("processing message: {:?} with route {:?}", msg, route);

    let current_chain = ChainName::new(&env);

    let route = if let Some(route) = route {
        Ok::<_, ServerError>(route)
    } else {
        match msg.message.recipient.clone() {
            // TODO: add smarter routing
            Recipient::Account { id: _, chain } => Ok(chain.map_or(AccountTrace::Local, |chain| {
                if chain == current_chain {
                    AccountTrace::Local
                } else {
                    AccountTrace::Remote(vec![chain.clone()])
                }
            })),
            Recipient::Namespace {
                chain,
                namespace: _,
            } => Ok(chain.map_or(AccountTrace::Local, |chain| {
                if chain == current_chain {
                    AccountTrace::Local
                } else {
                    AccountTrace::Remote(vec![chain.clone()])
                }
            })),
            _ => {
                return Err(ServerError::NotImplemented(
                    "Non-account recipients not supported".to_string(),
                ))
            }
        }
    }?;

    let metadata = Header {
        current_hop: 0,
        route,
    };

    let msg = route_msg(deps, msg, metadata, &mut app)?;

    Ok(app.response("route").add_message(msg))
}

pub(crate) fn route_msg(
    deps: DepsMut,
    msg: IbcMailMessage,
    header: Header,
    app: &mut ServerAdapter,
) -> ServerResult<CosmosMsg> {
    println!("routing message: {:?}, metadata: {:?}", msg, header);

    match header.route {
        AccountTrace::Local => route_to_local_account(deps.as_ref(), msg, header, app),
        AccountTrace::Remote(ref chains) => {
            println!("routing to chains: {:?}", chains);
            // check index of hop. If we are on the final hop, route to local account
            if header.current_hop == chains.len() as u32 {
                return route_to_local_account(deps.as_ref(), msg.clone(), header, app);
            }
            // TODO verify that the chain is a valid chain

            let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;

            let dest_chain = chains
                .get(header.current_hop as usize)
                .ok_or(ServerError::InvalidRoute {
                    route: header.route.clone(),
                    hop: header.current_hop,
                })?
                .to_string();
            println!("routing to destination_chain: {:?}", dest_chain);

            let ibc_msg = ibc_client::ExecuteMsg::ModuleIbcAction {
                // TODO: why is host chain not chain name
                host_chain: dest_chain,
                target_module: current_module_info,
                msg: to_json_binary(&ServerIbcMessage::RouteMessage { msg, header })?,
                callback_info: None,
            };

            println!("ibc_msg: {:?}", ibc_msg);
            // TODO: suggested syntax
            // let ibc_msg = app.ibc_client().module_ibc_action(chain, target_module, msg, callback)
            // TODO: We could additionally have something like to avoid having to create the module info object
            // let ibc_msg = app.ibc_client().self_module_ibc_action(chain, msg, callback)

            let ibc_client_addr = app
                .module_registry(deps.as_ref())?
                .query_module(ModuleInfo::from_id_latest(IBC_CLIENT)?)?
                .reference
                .unwrap_native()?;
            let exec_msg = wasm_execute(ibc_client_addr, &ibc_msg, vec![])?.into();

            Ok::<CosmosMsg, ServerError>(exec_msg)
        }
    }
}

fn route_to_local_account(
    deps: Deps,
    msg: IbcMailMessage,
    header: Header,
    app: &mut ServerAdapter,
) -> ServerResult<CosmosMsg> {
    println!("routing to local account: {:?}", msg.message.recipient);
    // This is a local message

    let recipient = msg.message.recipient.clone();

    let account_id = match recipient {
        Recipient::Account { id: account_id, .. } => Ok(account_id),
        Recipient::Namespace { namespace, .. } => {
            // TODO: this only allows for addressing recipients via namespace of their email account directly.
            // If they have the email application installed on a sub-account, this will not be able to identify the sub-account.
            let namespace_status = app
                .module_registry(deps)?
                .query_namespace(namespace.clone())?;
            match namespace_status {
                NamespaceResponse::Claimed(info) => Ok(info.account_id),
                NamespaceResponse::Unclaimed {} => {
                    return Err(ServerError::UnclaimedNamespace(namespace));
                }
            }
        }
        _ => Err(ServerError::NotImplemented(
            "Non-account recipients not supported".to_string(),
        )),
    }?;

    let acc_base = app.account_registry(deps)?.account_base(&account_id)?;
    app.target_account = Some(acc_base);

    let mail_client = app.mail_client(deps);

    Ok(mail_client.receive_msg(msg, header)?)
}
