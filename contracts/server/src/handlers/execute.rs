use abstract_adapter::traits::AbstractResponse;
use abstract_sdk::ModuleRegistryInterface;

use abstract_sdk::features::ModuleIdentification;

use abstract_std::{ibc_client, IBC_CLIENT};

use abstract_std::objects::account::{AccountId, AccountTrace};
use abstract_std::objects::module::ModuleInfo;
use cosmwasm_std::{to_json_binary, wasm_execute, CosmosMsg, Deps, DepsMut, Env, MessageInfo};

use ibcmail::{Header, Message, Recipient, Route};

use crate::contract::{Adapter, ServerResult};
use crate::error::ServerError;
use ibcmail::server::msg::{ServerExecuteMsg, ServerIbcMessage};
use ibcmail::server::ServerAdapter;

use crate::state::CONFIG;
use ibcmail::client::api::ClientInterface;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: Adapter,
    msg: ServerExecuteMsg,
) -> ServerResult {
    match msg {
        ServerExecuteMsg::ProcessMessage { msg, route } => {
            process_message(deps, info, msg, route, app)
        }
        ServerExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

fn process_message(
    deps: DepsMut,
    _info: MessageInfo,
    msg: Message,
    route: Option<Route>,
    app: Adapter,
) -> ServerResult {
    println!("processing message: {:?} with route {:?}", msg, route);

    let route = if let Some(route) = route {
        Ok::<_, ServerError>(route)
    } else {
        match msg.recipient.clone() {
            // TODO: add smarter routing
            Recipient::Account { id: _, chain } => Ok(chain.map_or(AccountTrace::Local, |chain| {
                AccountTrace::Remote(vec![chain.clone()])
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

    let msg = route_msg(deps, msg, metadata, &app)?;

    Ok(app.response("route").add_message(msg))
}

pub(crate) fn route_msg(
    deps: DepsMut,
    msg: Message,
    header: Header,
    app: &ServerAdapter,
) -> ServerResult<CosmosMsg> {
    println!("routing message: {:?}, metadata: {:?}", msg, header);

    match &msg.recipient {
        Recipient::Account {
            id: ref account_id,
            chain: _,
        } => {
            match header.route {
                AccountTrace::Local => {
                    // TODO: fix clone
                    route_to_local_account(deps.as_ref(), msg.clone(), account_id, header, app)
                }
                AccountTrace::Remote(ref chains) => {
                    println!("routing to chains: {:?}", chains);
                    // check index of hop. If we are on the final hop, route to local account
                    if header.current_hop == chains.len() as u32 {
                        return route_to_local_account(
                            deps.as_ref(),
                            msg.clone(),
                            account_id,
                            header,
                            app,
                        );
                    }
                    // TODO verify that the chain is a valid chain

                    let current_module_info =
                        ModuleInfo::from_id(app.module_id(), app.version().into())?;

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
                    // let ibc_msg = app.ibc_client().module_ibc_action(chain, module,msg, callback)

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
        _ => Err(ServerError::NotImplemented(
            "Non-account recipients not supported".to_string(),
        )),
    }
}

fn route_to_local_account(
    deps: Deps,
    msg: Message,
    account_id: &AccountId,
    header: Header,
    app: &ServerAdapter,
) -> ServerResult<CosmosMsg> {
    println!("routing to local account: {:?}", account_id);
    // This is a local message

    let mail_client = app.mail_client(deps, account_id);

    Ok(mail_client.receive_msg(msg, header)?)
}

/// Update the configuration of the client
fn update_config(deps: DepsMut, _msg_info: MessageInfo, app: Adapter) -> ServerResult {
    // Only the admin should be able to call this
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}
