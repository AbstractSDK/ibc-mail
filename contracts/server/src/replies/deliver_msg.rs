use abstract_adapter::objects::TruncatedChainId;
use abstract_adapter::sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply, SubMsgResult};

use ibcmail::server::msg::ServerMessage;
use ibcmail::server::state::AWAITING_DELIVERY;
use ibcmail::server::ServerAdapter;
use ibcmail::{DeliveryFailure, DeliveryStatus, Header, Sender, ServerMetadata};

use crate::contract::ServerResult;
use crate::handlers::execute::route_message;

pub fn deliver_message_reply(
    deps: DepsMut,
    env: Env,
    mut app: ServerAdapter,
    reply: Reply,
) -> ServerResult {
    let current_chain = TruncatedChainId::new(&env);
    let delivery_status = match reply.result {
        SubMsgResult::Ok(_) => DeliveryStatus::Delivered,
        SubMsgResult::Err(error) => DeliveryFailure::Unknown(error).into(),
    };

    // Load the awaiting message
    let mut awaiting_msgs = AWAITING_DELIVERY.load(deps.storage)?;
    let (message_id, origin_header, origin_metadata) = awaiting_msgs.remove(0);
    AWAITING_DELIVERY.save(deps.storage, &awaiting_msgs)?;

    let delivery_message = ServerMessage::delivery_status(message_id.clone(), delivery_status);

    let delivery_header = Header {
        sender: Sender::Server {
            address: env.contract.address.to_string(),
            chain: TruncatedChainId::new(&env),
        },
        recipient: origin_header.sender.try_into()?,
        // TODO: new ID?
        id: message_id.clone(),
        // TODO: version?
        version: origin_header.version,
        timestamp: env.block.time,
        reply_to: None,
    };

    let delivery_metadata = ServerMetadata {
        route: origin_metadata.reverse_route()?,
    };

    let msg = route_message(
        deps,
        &mut app,
        &current_chain,
        delivery_header,
        delivery_metadata,
        delivery_message,
    )?;

    Ok(app
        .response("deliver_message_reply")
        .add_attribute("message_id", message_id)
        .add_submessages(msg))
}
