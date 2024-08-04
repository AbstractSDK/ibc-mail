use abstract_adapter::objects::TruncatedChainId;
use abstract_adapter::sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply, SubMsgResult};

use ibcmail::server::msg::ServerMessage;
use ibcmail::server::state::AWAITING_DELIVERY;
use ibcmail::server::ServerAdapter;
use ibcmail::{DeliveryFailure, DeliveryStatus, Sender};

use crate::contract::ServerResult;
use crate::handlers::execute::route_msg;

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
    let (message_id, origin_header) = awaiting_msgs.remove(0);
    AWAITING_DELIVERY.save(deps.storage, &awaiting_msgs)?;

    let delivery_msg = ServerMessage::delivery_status(message_id.clone(), delivery_status);
    let delivery_header = origin_header.reverse(Sender::Server {
        address: env.contract.address.to_string(),
        chain: TruncatedChainId::new(&env),
    })?;

    let msg = route_msg(
        deps,
        &current_chain,
        &mut app,
        delivery_header,
        delivery_msg,
    )?;

    Ok(app
        .response("deliver_message_reply")
        .add_attribute("message_id", message_id)
        .add_submessages(msg))
}
