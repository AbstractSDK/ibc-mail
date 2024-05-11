use crate::contract::{Adapter, ServerResult};
use ibcmail::server::msg::{ServerQueryMsg, ConfigResponse};
use crate::state::{CONFIG};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};

pub fn query_handler(deps: Deps, _env: Env, _app: &Adapter, msg: ServerQueryMsg) -> ServerResult<Binary> {
    match msg {
        ServerQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}
