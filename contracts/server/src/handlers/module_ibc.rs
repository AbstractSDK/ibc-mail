use abstract_adapter::sdk::AbstractResponse;
use abstract_adapter::std::ibc::ModuleIbcInfo;
use cosmwasm_std::{Binary, DepsMut, Env, from_json};

use ibcmail::{
    IBCMAIL_SERVER_ID,
    server::{error::ServerError, msg::ServerIbcMessage, ServerAdapter},
};

use crate::{contract::ServerResult, handlers::execute::route_msg};

// ANCHOR: module_ibc_handler
pub fn module_ibc_handler(
    deps: DepsMut,
    _env: Env,
    mut app: ServerAdapter,
    module_info: ModuleIbcInfo,
    msg: Binary,
) -> ServerResult {
    // Assert IBC sender was the server
    if module_info.module.id().ne(IBCMAIL_SERVER_ID) {
        return Err(ServerError::UnauthorizedIbcModule(module_info.clone()));
    };

    let server_msg: ServerIbcMessage = from_json(msg)?;

    match server_msg {
        ServerIbcMessage::RouteMessage { msg, mut header } => {
            header.current_hop += 1;

            let msgs = route_msg(deps, &mut app, msg, header)?;

            Ok(app.response("module_ibc").add_attribute("method", "route").add_submessages(msgs))
        }
        _ => Err(ServerError::UnauthorizedIbcMessage {}),
    }
}
