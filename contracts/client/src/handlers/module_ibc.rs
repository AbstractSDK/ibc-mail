use abstract_app::std::ibc::ModuleIbcMsg;
use cosmwasm_std::{DepsMut, Env};
use crate::contract::{App, ClientResult};

pub fn module_ibc_handler(
    _deps: DepsMut,
    _env: Env,
    _app: App,
    _msg: ModuleIbcMsg,
) -> ClientResult {
    todo!()
}