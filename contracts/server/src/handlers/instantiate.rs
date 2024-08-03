use abstract_adapter::sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, MessageInfo};

use ibcmail::server::msg::ServerInstantiateMsg;
use ibcmail::server::ServerAdapter;
use ibcmail::server::state::AWAITING_DELIVERY;

use crate::contract::ServerResult;

pub fn instantiate_handler(deps: DepsMut, _env: Env, info: MessageInfo, app: ServerAdapter, _msg: ServerInstantiateMsg) -> ServerResult {

    AWAITING_DELIVERY.save(deps.storage, &vec![])?;
    Ok(app.response("instantiate"))
}
