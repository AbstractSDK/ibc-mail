use abstract_adapter::traits::AbstractResponse;
use abstract_sdk::{AccountRegistry, AccountVerification, IbcInterface, ModuleInterface};
use abstract_sdk::features::ModuleIdentification;
use abstract_std::app::{AppExecuteMsg, ExecuteMsg};
use abstract_std::ibc::ModuleIbcMsg;
use abstract_std::{ibc_client, IBC_CLIENT, manager};
use abstract_std::ibc_client::InstalledModuleIdentification;
use abstract_std::manager::ModuleAddressesResponse;
use abstract_std::objects::account::{AccountId, AccountTrace};
use abstract_std::objects::module::ModuleInfo;
use cosmwasm_std::{CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, to_json_binary, wasm_execute, WasmMsg};

use ibcmail::{IBCMAIL_CLIENT, Message, Metadata, Recipient, Route};
use ibcmail::client::error::ClientError;
use ibcmail::client::msg::ClientExecuteMsg;

use crate::contract::{Adapter, ServerResult};
use crate::error::ServerError;
use ibcmail::server::msg::{ServerExecuteMsg, ServerIbcMessage};
use ibcmail::server::ServerAdapter;
use crate::state::CONFIG;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: Adapter,
    msg: ServerExecuteMsg,
) -> ServerResult {
    match msg {
        ServerExecuteMsg::ProcessMessage { msg, route } => process_message(deps, info, msg, route, app),
        ServerExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

fn process_message(deps: DepsMut, _info: MessageInfo, msg: Message, route: Option<Route>, app: Adapter) -> ServerResult {
    println!("processing message: {:?} with route {:?}", msg, route);

    let route = if let Some(route) = route {
        Ok::<_, ServerError>(route)
    } else {
        match msg.recipient.clone() {
            // TODO: add smarter routing
            Recipient::Account { id, chain } => {
                Ok(chain.map_or(AccountTrace::Local, |chain| AccountTrace::Remote(vec![chain.clone()])))
            },
            _ => return Err(ServerError::NotImplemented("Non-account recipients not supported".to_string()))
        }
    }?;

    let metadata = Metadata {
        current_hop: 0,
        route,
    };

    let msg = route_msg(deps, msg, metadata, &app)?;

    Ok(app.response("route").add_message(msg))
}

pub(crate) fn route_msg(deps: DepsMut, msg: Message, metadata: Metadata, app: &ServerAdapter) -> ServerResult<CosmosMsg> {
    println!("routing message: {:?}, metadata: {:?}", msg, metadata);

    match &msg.recipient {
        Recipient::Account { id: ref account_id, ref chain } => {
            match metadata.route {
                AccountTrace::Local => {
                    // TODO: fix clone
                    route_to_local_account(deps.as_ref(), msg.clone(), &account_id, app)
                },
                AccountTrace::Remote(ref chains) => {
                    println!("routing to chains: {:?}", chains);
                    // check index of hop. If we are on the final hop, route to local account
                    if metadata.current_hop == chains.len() as u32 {
                        return route_to_local_account(deps.as_ref(), msg.clone(), &account_id, app)
                    }
                    // TODO verify that the chain is a valid chain

                    let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;

                    let ibc_msg = ibc_client::ExecuteMsg::ModuleIbcAction {
                        // TODO: why is host chain not chain name
                        host_chain: chains.get(metadata.current_hop as usize).ok_or(ServerError::InvalidRoute { route: metadata.route.clone(), hop: metadata.current_hop.clone() })?.to_string(),
                        target_module: current_module_info,
                        msg: to_json_binary(&ServerIbcMessage::RouteMessage { msg, metadata })?,
                        callback_info: None,
                    };

                    let ibc_client_addr = app.modules(deps.as_ref()).module_address(IBC_CLIENT)?;
                    let exec_msg = wasm_execute(ibc_client_addr, &ibc_msg, vec![])?.into();

                    Ok::<CosmosMsg, ServerError>(exec_msg)
                }
            }
        },
        _ => {
            return Err(ServerError::NotImplemented("Non-account recipients not supported".to_string()))
        }
    }
}

fn route_to_local_account(deps: Deps, msg: Message, account_id: &AccountId, app: &ServerAdapter) -> ServerResult<CosmosMsg> {
    let registry = app.account_registry(deps)?;

    println!("routing to local account: {:?}", account_id);
    // This is a local message
    let manager = registry.manager_address(&account_id)?;
    let module_addresses = deps.querier.query_wasm_smart::<ModuleAddressesResponse>(manager, &manager::QueryMsg::ModuleAddresses {
        ids: vec![IBCMAIL_CLIENT.to_string()]
    })?;
    if module_addresses.modules.is_empty() {
        return Err(ServerError::NotImplemented("Module not installed".to_string()))
    }

    let client_address = module_addresses.modules[0].1.clone();

    // TODO allow for funds
    let exec_msg = wasm_execute(
        client_address,
        &ExecuteMsg::<ClientExecuteMsg, Empty>::from(ClientExecuteMsg::ReceiveMessage(msg)),
        vec![]
    )?.into();

    Ok::<CosmosMsg, ServerError>(exec_msg)
}

/// Update the configuration of the client
fn update_config(deps: DepsMut, _msg_info: MessageInfo, app: Adapter) -> ServerResult {
    // Only the admin should be able to call this
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}
