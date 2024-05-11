use abstract_std::ibc::ModuleIbcMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use ibcmail::server::ServerAdapter;
use crate::contract::{ServerResult};

pub fn module_ibc_handler(
    deps: DepsMut,
    _env: Env,
    app: ServerAdapter,
    msg: ModuleIbcMsg,
) -> ServerResult {
    todo!()
}