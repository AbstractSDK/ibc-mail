use crate::contract::{Adapter, ServerResult};

use abstract_adapter::traits::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply};

pub fn instantiate_reply(_deps: DepsMut, _env: Env, app: Adapter, _reply: Reply) -> ServerResult {
    Ok(app.response("instantiate_reply"))
}
