use abstract_app::std::ibc::ModuleIbcMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use crate::contract::{App, ClientResult};

pub fn module_ibc_handler(
    deps: DepsMut,
    _env: Env,
    app: App,
    msg: ModuleIbcMsg,
) -> ClientResult {
    todo!()
}