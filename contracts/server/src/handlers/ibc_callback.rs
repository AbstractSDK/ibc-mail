use abstract_adapter::objects::module::ModuleInfo;
use abstract_adapter::sdk::{AbstractResponse, ModuleRegistryInterface};
use abstract_adapter::std::ibc::{Callback, IbcResult, ModuleIbcInfo};
use abstract_adapter::std::ibc_client;
use abstract_adapter::traits::ModuleIdentification;
use cosmwasm_std::{from_json, Binary, DepsMut, Env, Response, wasm_execute, Addr, to_json_binary, CosmosMsg};

use ibcmail::{server::{error::ServerError, msg::ServerIbcMessage, ServerAdapter}, IBCMAIL_SERVER_ID, MessageStatus, MessageHash};
use ibcmail::server::state::AWAITING;
use crate::{contract::ServerResult, handlers::execute::route_msg};
use crate::contract::IBC_CLIENT;

// ANCHOR: ibc_callback_handler
pub fn ibc_callback_handler(
    deps: DepsMut,
    _env: Env,
    mut app: ServerAdapter,
    callback: Callback,
    ibc_result: IbcResult
) -> ServerResult {
    // panic!("ibc_callback_handler: {:?}", callback);
    println!("ibc_callback_handler callback: {:?} result, env: {:?}", callback, _env);

    // let events = ibc_result.get_execute_events();
    // panic!("ibc_callback_handler events: {:?}", events);
    // let chain = events.into_iter().find(|e| {
    //     e.ty == "abstract-wasm"
    //         && e.attributes
    //         .iter()
    //         .any(|a| a.key == "contract" && a.value == "abstract:ibc-host")
    // });


    let msgs = match ibc_result {
        IbcResult::Execute { result: Ok(e), initiator_msg } => {
            let origin_msg: ServerIbcMessage = from_json(initiator_msg)?;

            match origin_msg {
                ServerIbcMessage::RouteMessage { msg, header } => {
                    println!("ibc_callback_handler success route_msg id: {:?}, header: {:?}", msg.id, header);
                    if header.current_hop > 0 {
                        vec![send_update(deps, &mut app, msg.id, MessageStatus::Received)?]
                    } else {
                        // Sent
                        AWAITING.remove(deps.storage, &msg.id);
                        vec![]
                    }
                },
                ServerIbcMessage::UpdateMessage { id, status } => {
                    println!("ibc_callback_handler success update_msg: {:?}", id);
                    vec![]
                }
                _ => {
                    println!("Unknown message");
                    vec![]
                },
            }
        },
        IbcResult::Execute { result: Err(e), initiator_msg } => {
            let origin_msg: ServerIbcMessage = from_json(initiator_msg)?;
            match origin_msg {
                ServerIbcMessage::RouteMessage { msg, header } => {
                    println!("ibc_callback_handler success route_msg id: {:?}, header: {:?}", msg.id, header);
                    vec![send_update(deps, &mut app, msg.id, MessageStatus::Failed)?]
                },
                ServerIbcMessage::UpdateMessage { id, status } => {
                    println!("ibc_callback_handler error update_msg: {:?}", id);
                    vec![]
                }
                _ => {
                    println!("unknown message")
                    vec![]
                },
            };
            println!("ibc_callback_handler error: {:?}", e);

        },
        IbcResult::FatalError(e) => {
            println!("ibc_callback_handler fatal error: {:?}", e);
            vec![]
        },
        _ => {
            println!("unexpected callback result");
            vec![]
        }
    };
    Ok(Response::default())
}
// ANCHOR_END: ibc_callback_handler

/// We only want to send a message if we're in the middle
pub(crate) fn send_update(
    deps: DepsMut,
    module: &mut ServerAdapter,
    id: MessageHash,
    status: MessageStatus,
) -> ServerResult<CosmosMsg> {
    println!("updating message: {:?}, status: {:?}", id, status);
    let from_chain = AWAITING.load(deps.storage, &id).map_err(|_| ServerError::AwaitedMsgNotFound(id))?;

    let current_module_info = ModuleInfo::from_id(module.module_id(), module.version().into())?;

    // Call IBC client
    let ibc_client_msg = ibc_client::ExecuteMsg::ModuleIbcAction {
        host_chain: from_chain.clone(),
        target_module: current_module_info,
        msg: to_json_binary(&ServerIbcMessage::UpdateMessage { id, status })?,
        callback: None
    };

    let ibc_client_addr: Addr = module
        .module_registry(deps.as_ref())?
        .query_module(ModuleInfo::from_id_latest(IBC_CLIENT.id)?)?
        .reference
        .unwrap_native()?;

    let msg: CosmosMsg = wasm_execute(ibc_client_addr, &ibc_client_msg, vec![])?.into();
    Ok::<CosmosMsg, ServerError>(msg)
}
