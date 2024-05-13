use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use ibcmail::client::state::{TEST};

use crate::contract::{App, ClientResult};
use crate::msg::ClientInstantiateMsg;
use crate::state::{Config, CONFIG};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: App,
    _msg: ClientInstantiateMsg,
) -> ClientResult {
    let config: Config = Config {};

    CONFIG.save(deps.storage, &config)?;
    TEST.save(deps.storage, &_env.contract.address)?;

    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
