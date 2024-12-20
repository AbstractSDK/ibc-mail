use abstract_adapter::objects::TruncatedChainId;
use abstract_adapter::sdk::{
    features::ModuleIdentification, AccountVerification, ModuleRegistryInterface,
};
use abstract_adapter::std::registry::Account;
use abstract_adapter::std::{
    ibc_client,
    objects::{account::AccountTrace, module::ModuleInfo},
    registry::NamespaceResponse,
    IBC_CLIENT,
};
use abstract_adapter::traits::AbstractResponse;
use cosmwasm_std::{
    to_json_binary, wasm_execute, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
};
use ibcmail::client::api::MailClient;
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

// ANCHOR: execute_handler
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
// ANCHOR_END: execute_handler

fn process_message(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: IbcMailMessage,
    route: Option<Route>,
    mut app: Adapter,
) -> ServerResult {
    println!("processing message: {:?} with route {:?}", msg, route);

    let current_chain = TruncatedChainId::new(&env);

    let route: Route = if let Some(route) = route {
        Ok::<_, ServerError>(match route {
            Route::Local => Route::Local,
            Route::Remote(mut chains) => {
                println!("processing remote route: {:?}", chains);
                // Enforce that the route always contains every hop in the chain
                if chains.first() == Some(&current_chain) {
                    if chains.len() == 1 {
                        Route::Local
                    } else {
                        Route::Remote(chains)
                    }
                } else {
                    chains.insert(0, current_chain);
                    Route::Remote(chains)
                }
            }
        })
    } else {
        println!("processing message recipient: {:?}", msg.message.recipient);
        match msg.message.recipient.clone() {
            // TODO: add smarter routing
            Recipient::Account { id: _, chain } => Ok(chain.map_or(AccountTrace::Local, |chain| {
                if chain == current_chain {
                    AccountTrace::Local
                } else {
                    AccountTrace::Remote(vec![current_chain, chain.clone()])
                }
            })),
            Recipient::Namespace {
                chain,
                namespace: _,
            } => Ok(chain.map_or(AccountTrace::Local, |chain| {
                if chain == current_chain {
                    AccountTrace::Local
                } else {
                    AccountTrace::Remote(vec![current_chain, chain.clone()])
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
            if header.current_hop == (chains.len() - 1) as u32 {
                println!("routing to local account: {:?}", chains);
                return route_to_local_account(deps.as_ref(), msg.clone(), header, app);
            }
            // TODO verify that the chain is a valid chain

            let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;

            let dest_chain =
                chains
                    .get(header.current_hop as usize + 1)
                    .ok_or(ServerError::InvalidRoute {
                        route: header.route.clone(),
                        hop: header.current_hop,
                    })?;

            // ANCHOR: ibc_client
            // Call IBC client
            let ibc_client_msg = ibc_client::ExecuteMsg::ModuleIbcAction {
                host_chain: dest_chain.clone(),
                target_module: current_module_info,
                msg: to_json_binary(&ServerIbcMessage::RouteMessage { msg, header })?,
                callback: None,
            };

            let ibc_client_addr: Addr = app
                .module_registry(deps.as_ref())?
                .query_module(ModuleInfo::from_id_latest(IBC_CLIENT)?)?
                .reference
                .unwrap_native()?;

            let msg: CosmosMsg = wasm_execute(ibc_client_addr, &ibc_client_msg, vec![])?.into();
            // ANCHOR_END: ibc_client
            Ok::<CosmosMsg, ServerError>(msg)
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

    // ANCHOR: set_acc_and_send
    // Set target account for actions, is used by APIs to retrieve mail client address.
    let recipient_acc: Account = app.account_registry(deps)?.account(&account_id)?;
    app.target_account = Some(recipient_acc);

    let mail_client: MailClient<_> = app.mail_client(deps);
    let msg: CosmosMsg = mail_client.receive_msg(msg, header)?;
    // ANCHOR_END: set_acc_and_send

    Ok(msg)
}
