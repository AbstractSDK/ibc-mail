use abstract_adapter::traits::AbstractResponse;
use abstract_sdk::{AccountVerification, IbcInterface, ModuleInterface};
use abstract_sdk::features::ModuleIdentification;
use abstract_std::app::{AppExecuteMsg, ExecuteMsg};
use abstract_std::ibc::ModuleIbcMsg;
use abstract_std::{ibc_client, IBC_CLIENT, manager};
use abstract_std::ibc_client::InstalledModuleIdentification;
use abstract_std::manager::ModuleAddressesResponse;
use abstract_std::objects::account::{AccountId, AccountTrace};
use abstract_std::objects::module::ModuleInfo;
use cosmwasm_std::{CosmosMsg, DepsMut, Empty, Env, MessageInfo, to_json_binary, wasm_execute, WasmMsg};

use ibcmail::{IBCMAIL_CLIENT, Message, Recipient};
use ibcmail::client::error::ClientError;
use ibcmail::client::msg::ClientExecuteMsg;

use crate::contract::{Adapter, ServerResult};
use crate::error::ServerError;
use ibcmail::server::msg::{ServerExecuteMsg, ServerIbcMessage};
use crate::state::CONFIG;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: Adapter,
    msg: ServerExecuteMsg,
) -> ServerResult {
    match msg {
        ServerExecuteMsg::RouteMessage(msg) => route_msg(deps, info, msg, app),
        ServerExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

pub(crate) fn route_msg(deps: DepsMut, _info: MessageInfo, msg: Message, app: Adapter) -> ServerResult {
    let registry = app.account_registry(deps.as_ref())?;
    println!("routing message: {:?}", msg);

    let msg = match &msg.recipient {
        Recipient::Account { id: ref account_id, route: chain } => {
            match chain {
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
                    )?.into();

                    Ok::<CosmosMsg, ServerError>(exec_msg)
                },
                AccountTrace::Remote(chains) => {
                    println!("routing to chain: {:?}", chains);
                    // TODO verify that the chain is a valid chain

                    let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;

                    let ibc_msg = ibc_client::ExecuteMsg::ModuleIbcAction {
                        // TODO: why is host chain not chain name
                        // We take the first chain in the hop
                        host_chain: chains.first().clone().unwrap().to_string(),
                        target_module: current_module_info,
                        msg: to_json_binary(&ServerIbcMessage::RouteMessage(msg))?,
                        callback_info: None,
                    };

                    let ibc_client_addr = app.modules(deps.as_ref()).module_address(IBC_CLIENT)?;
                    let exec_msg = wasm_execute(ibc_client_addr, &ibc_msg, vec![])?.into();

                    Ok::<CosmosMsg, ServerError>(exec_msg)
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
fn update_config(deps: DepsMut, _msg_info: MessageInfo, app: Adapter) -> ServerResult {
    // Only the admin should be able to call this
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}
