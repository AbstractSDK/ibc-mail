use crate::contract::{Adapter, AppResult};

use abstract_adapter::traits::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply};

pub fn instantiate_reply(_deps: DepsMut, _env: Env, app: Adapter, _reply: Reply) -> AppResult {
    Ok(app.response("instantiate_reply"))
}
