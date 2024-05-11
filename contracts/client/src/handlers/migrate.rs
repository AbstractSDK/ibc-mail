use crate::contract::{App, ClientResult};
use crate::msg::AppMigrateMsg;
use abstract_app::traits::AbstractResponse;
use cosmwasm_std::{DepsMut, Env};

/// Handle the client migrate msg
/// The top-level Abstract client does version checking and dispatches to this handler
pub fn migrate_handler(_deps: DepsMut, _env: Env, app: App, _msg: AppMigrateMsg) -> ClientResult {
    Ok(app.response("migrate"))
}
