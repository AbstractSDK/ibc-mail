use abstract_adapter::objects::TruncatedChainId;
use abstract_adapter::sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply, SubMsgResult};

use ibcmail::server::state::AWAITING_DELIVERY;
use ibcmail::server::ServerAdapter;
use ibcmail::{DeliveryFailure, DeliveryStatus};

use crate::contract::ServerResult;
use crate::handlers::execute;
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
    let (origin_header, origin_metadata) = awaiting_msgs.remove(0);
    AWAITING_DELIVERY.save(deps.storage, &awaiting_msgs)?;

    let message_id = origin_header.id.clone();

    let msg = execute::send_delivery_status(
        deps,
        &env,
        &mut app,
        &current_chain,
        origin_header,
        origin_metadata,
        delivery_status,
    )?;

    Ok(app
        .response("deliver_message_reply")
        .add_attribute("message_id", &message_id)
        .add_submessages(msg))
}
