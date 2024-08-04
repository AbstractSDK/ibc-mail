use abstract_adapter::objects::TruncatedChainId;
use abstract_adapter::std::ibc::{Callback, IbcResult};
use cosmwasm_std::{from_json, DepsMut, Env, Response, SubMsg};

use ibcmail::server::msg::ServerMessage;
use ibcmail::{
    server::{msg::ServerIbcMessage, ServerAdapter},
    DeliveryFailure, Header, Route, Sender, ServerMetadata,
};

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
    ibc_result: IbcResult,
) -> ServerResult {
    // panic!("ibc_callback_handler: {:?}", callback);
    println!(
        "ibc_callback_handler callback: {:?} result, env: {:?}",
        callback, _env
    );

    let msgs: Vec<SubMsg> = match ibc_result {
        // The destination server successfully processed the message
        IbcResult::Execute {
            result: Ok(_response),
            initiator_msg,
        } => {
            println!("ibc_callback_handler execute success");
            let origin_msg: ServerIbcMessage = from_json(initiator_msg)?;

            match origin_msg {
                // We successfully routed a packet, and need to send an update to the sender
                ServerIbcMessage::RouteMessage {
                    msg,
                    header,
                    metadata,
                } => {
                    println!(
                        "ibc_callback_handler success route_msg id: {:?}, header: {:?}, metadata: {:?}",
                        msg.id(),
                        header,
                        metadata
                    );
                    vec![]
                    // vec![execute::update_message_status(deps, &mut app, msg.id, header, MessageStatus::Received)?]
                }
                _ => {
                    println!("Unknown message");
                    vec![]
                }
            }
        }
        // The destination server failed to process the message
        IbcResult::Execute {
            result: Err(e),
            initiator_msg,
        } => {
            println!("ibc_callback_handler execute error");
            // println!("ibc_callback_handler execute error: {:?}", e);
            let origin_msg: ServerIbcMessage = from_json(initiator_msg)?;
            match origin_msg {
                ServerIbcMessage::RouteMessage {
                    msg,
                    header,
                    metadata,
                } => {
                    println!(
                        "ibc_callback_handler execute error route_msg id: {:?}, header: {:?}, metadata: {:?}",
                        msg.id(),
                        header,
                        metadata
                    );
                    let current_chain = TruncatedChainId::new(&_env);
                    let current_hop = metadata.current_hop(&current_chain)?;
                    match msg {
                        // We failed to deliver a message, we send a failed status update to the sender
                        ServerMessage::Mail { ref message } => {
                            // archway juno neutron
                            // juno -> neutron failed current hop 1
                            // expected: juno archway
                            // need to remove anything after the current hop
                            let status_header = Header {
                                sender: Sender::Server {
                                    chain: current_chain.clone(),
                                    address: _env.contract.address.to_string(),
                                },
                                recipient: message.sender.clone().try_into()?,
                                // TODO: new message id
                                id: message.id.clone(),
                                version: message.version.clone(),
                                timestamp: _env.block.time,
                                reply_to: None,
                            };

                            let status_metadata = ServerMetadata {
                                route: match metadata.route {
                                    Route::Remote(mut chains) => {
                                        // keep the current hop but remove everything after it
                                        chains.truncate(current_hop as usize + 1);
                                        chains.reverse();
                                        Route::Remote(chains)
                                    }
                                    _ => Route::Local,
                                },
                            };

                            let status_message = ServerMessage::delivery_status(
                                msg.id(),
                                DeliveryFailure::Unknown(e).into(),
                            );
                            execute::route_message(
                                deps,
                                &mut app,
                                &current_chain,
                                status_header,
                                status_metadata,
                                status_message,
                            )?
                        }
                        _ => {
                            println!(
                                "ibc_callback_handler execute error route_msg unknown message"
                            );
                            vec![]
                        }
                    }
                    // We failed to route a message, we send a failed status update to the sender
                }
                _ => {
                    println!("unknown message");
                    vec![]
                }
            }
        }
        IbcResult::FatalError(e) => {
            println!("ibc_callback_handler fatal error: {:?}", e);
            vec![]
        }
        _ => {
            println!("unexpected callback result");
            vec![]
        }
    };

    Ok(Response::default().add_submessages(msgs))
}
// ANCHOR_END: ibc_callback_handler
