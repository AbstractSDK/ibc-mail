use abstract_adapter::traits::AbstractResponse;
use abstract_sdk::AccountVerification;
use abstract_std::app::{AppExecuteMsg, ExecuteMsg};
use abstract_std::manager;
use abstract_std::manager::ModuleAddressesResponse;
use abstract_std::objects::account::AccountTrace;
use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, wasm_execute, WasmMsg};

use ibcmail::{IBCMAIL_CLIENT, Message, Recipient};
use ibcmail::client::error::ClientError;
use ibcmail::client::msg::ClientExecuteMsg;

use crate::contract::{Adapter, AppResult};
use crate::error::ServerError;
use ibcmail::server::msg::ServerExecuteMsg;
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

fn route_msg(deps: DepsMut, _info: MessageInfo, msg: Message, app: Adapter) -> AppResult {

    let registry = app.account_registry(deps.as_ref())?;

    let msg = match msg.recipient {
        Recipient::Account(ref account_id) => {
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

                    // TODO allow for funds
                    let exec_msg = wasm_execute(
                        client_address,
                        &ExecuteMsg::<ClientExecuteMsg, Empty>::from(ClientExecuteMsg::ReceiveMessage(msg)),
                        vec![]
                    )?;

                    Ok::<WasmMsg, ServerError>(exec_msg)
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
    }?;


    Ok(app.response("route").add_message(msg))
}

/// Update the configuration of the client
fn update_config(deps: DepsMut, _msg_info: MessageInfo, app: Adapter) -> AppResult {
    // Only the admin should be able to call this
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}
