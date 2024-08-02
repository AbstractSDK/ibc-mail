use abstract_adapter::std::ibc::{Callback, IbcResult};
use cosmwasm_std::{DepsMut, Env, from_json, Response};

use ibcmail::{MessageStatus, server::{msg::ServerIbcMessage, ServerAdapter}};
use ibcmail::server::state::AWAITING;
use crate::contract::ServerResult;
use crate::handlers::execute;

// ANCHOR: ibc_callback_handler
/// Handler for message callbacks.
/// We use this handler for sending message delivery updates to our clients.
pub fn ibc_callback_handler(
    deps: DepsMut,
    _env: Env,
    mut app: ServerAdapter,
    callback: Callback,
    ibc_result: IbcResult
) -> ServerResult {
    // panic!("ibc_callback_handler: {:?}", callback);
    println!("ibc_callback_handler callback: {:?} result, env: {:?}", callback, _env);

    let msgs = match ibc_result {
        // The destination server successfully processed the message
        IbcResult::Execute { result: Ok(_response), initiator_msg } => {
            println!("ibc_callback_handler execute success");
            let origin_msg: ServerIbcMessage = from_json(initiator_msg)?;

            match origin_msg {
                // We successfully routed a packet, and need to send an update to the sender
                ServerIbcMessage::RouteMessage { msg, header } => {
                    println!("ibc_callback_handler success route_msg id: {:?}, header: {:?}", msg.id, header);
                    vec![execute::update_message_status(deps, &mut app, msg.id, header, MessageStatus::Received)?]
                },
                // We successfully updated a message status, we shouldn't need to do anything now
                ServerIbcMessage::UpdateMessage { id, status, header } => {
                    println!("ibc_callback_handler success update_msg: {:?}", id);
                    vec![]
                }
                _ => {
                    println!("Unknown message");
                    vec![]
                },
            }
        }
        // The destination server failed to process the message
        IbcResult::Execute { result: Err(e), initiator_msg } => {
            println!("ibc_callback_handler execute error: {:?}", e);
            let origin_msg: ServerIbcMessage = from_json(initiator_msg)?;
            match origin_msg {
                ServerIbcMessage::RouteMessage { msg, header } => {
                    println!("ibc_callback_handler execute error route_msg id: {:?}, header: {:?}", msg.id, header);
                    vec![execute::update_message_status(deps, &mut app, msg.id, header, MessageStatus::Failed)?]
                },
                // We failed to update a message...
                ServerIbcMessage::UpdateMessage { id, header, status } => {
                    println!("ibc_callback_handler execute error update_msg: {:?}", id);
                    vec![]
                }
                _ => {
                    println!("unknown message");
                    vec![]
                },
            }

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

    Ok(Response::default().add_messages(msgs))
}
// ANCHOR_END: ibc_callback_handler

