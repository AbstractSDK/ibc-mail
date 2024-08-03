use abstract_adapter::objects::TruncatedChainId;
use abstract_adapter::std::ibc::{Callback, IbcResult};
use cosmwasm_std::{CosmosMsg, DepsMut, Env, from_json, Response, SubMsg};

use ibcmail::{Header, MessageStatus, Recipient, Route, Sender, server::{msg::ServerIbcMessage, ServerAdapter}};
use ibcmail::server::msg::ServerMessage;
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

    let msgs: Vec<SubMsg> = match ibc_result {
        // The destination server successfully processed the message
        IbcResult::Execute { result: Ok(_response), initiator_msg } => {
            println!("ibc_callback_handler execute success");
            let origin_msg: ServerIbcMessage = from_json(initiator_msg)?;

            match origin_msg {
                // We successfully routed a packet, and need to send an update to the sender
                ServerIbcMessage::RouteMessage { msg, header } => {
                    println!("ibc_callback_handler success route_msg id: {:?}, header: {:?}", msg.id(), header);
                    vec![]
                    // vec![execute::update_message_status(deps, &mut app, msg.id, header, MessageStatus::Received)?]
                },
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
                    println!("ibc_callback_handler execute error route_msg id: {:?}, header: {:?}", msg.id(), header);
                    match msg {
                        ServerMessage::Mail { ref message } => {
                            // archway juno neutron
                            // juno -> neutron failed current hop 1
                            // expected: juno archway
                            // need to remove anything after the current hop
                            let status_header = Header {
                                current_hop: 0,
                                route: match header.route {
                                    Route::Remote(mut chains) => {
                                        chains.truncate(header.current_hop as usize);
                                        chains.reverse();
                                        Route::Remote(chains)
                                    },
                                    _ => Route::Local
                                },
                                recipient: message.sender.clone().try_into()?,
                                sender: Sender::Server {
                                    chain: TruncatedChainId::new(&_env),
                                    address: _env.contract.address.to_string(),
                                }
                            };
                            execute::route_msg(deps, &mut app, ServerMessage::delivery_status(msg.id(), MessageStatus::Failed), status_header)?
                        },
                        _ => {
                            println!("ibc_callback_handler execute error route_msg unknown message");
                            vec![]
                        },
                    }
                    // We failed to route a message, we send a failed status update to the sender
                },
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

    Ok(Response::default().add_submessages(msgs))
}
// ANCHOR_END: ibc_callback_handler

