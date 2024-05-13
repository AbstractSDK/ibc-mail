use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{Adapter, ServerResult};
use ibcmail::server::msg::ServerInstantiateMsg;
use crate::state::{Config, CONFIG};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: Adapter,
    _msg: ServerInstantiateMsg,
) -> ServerResult {
    let config: Config = Config {};

    CONFIG.save(deps.storage, &config)?;

    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
