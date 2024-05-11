use abstract_adapter::traits::AbstractResponse;
use abstract_sdk::AccountVerification;
use abstract_std::manager;
use abstract_std::manager::ModuleAddressesResponse;
use abstract_std::objects::account::AccountTrace;
use cosmwasm_std::{DepsMut, Env, MessageInfo};

use client::IBCMAIL_CLIENT;
use ibcmail::{Message, Recipient};

use crate::contract::{Adapter, AppResult};
use crate::error::ServerError;
use crate::msg::ServerExecuteMsg;
use crate::state::CONFIG;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: Adapter,
    msg: ServerExecuteMsg,
) -> AppResult {
    match msg {
        ServerExecuteMsg::RouteMessage(msg) => route_msg(deps, info, msg, app),
        ServerExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

fn route_msg(deps: DepsMut, info: MessageInfo, msg: Message, app: Adapter) -> AppResult {

    let registry = app.account_registry(deps.as_ref())?;

    match msg.recipient {
        Recipient::Account(account_id) => {
            match account_id.trace() {
                AccountTrace::Local => {
                    // This is a local message
                    let manager = registry.manager_address(&account_id)?;
                    let module_addresses = deps.querier.query_wasm_smart::<ModuleAddressesResponse>(manager, &manager::QueryMsg::ModuleAddresses {
                        ids: vec![IBCMAIL_CLIENT.to_string()]
                    })?;
                    if module_addresses.modules.is_empty() {
                        return Err(ServerError::NotImplemented("Module not installed".to_string()))
                    }
                    let client_address = module_addresses.modules[0].1.clone();
                    // TODO: send the message to the local client
                    panic!();
                },
                _ => {
                    // This is a remote message
                    return Err(ServerError::NotImplemented("Remote messages".to_string()))
                }
            }
        },
        _ => {
            return Err(ServerError::NotImplemented("Recipient not supported".to_string()))
        }
    }


    Ok(app.response("reset"))
}

/// Update the configuration of the client
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: Adapter) -> AppResult {
    // Only the admin should be able to call this
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}
