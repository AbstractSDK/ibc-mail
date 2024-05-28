use abstract_adapter::sdk::AbstractResponse;
use abstract_adapter::std::ibc::ModuleIbcMsg;
use cosmwasm_std::{from_json, DepsMut, Env};
use ibcmail::{
    server::{error::ServerError, msg::ServerIbcMessage, ServerAdapter},
    IBCMAIL_SERVER_ID,
};

use crate::{contract::ServerResult, handlers::execute::route_msg};

// ANCHOR: module_ibc_handler
pub fn module_ibc_handler(
    deps: DepsMut,
    _env: Env,
    mut app: ServerAdapter,
    ibc_msg: ModuleIbcMsg,
) -> ServerResult {
    // Assert IBC sender was the server
    if ibc_msg.source_module.id().ne(IBCMAIL_SERVER_ID) {
        return Err(ServerError::UnauthorizedIbcModule(
            ibc_msg.source_module.clone(),
        ));
    };

    let server_msg: ServerIbcMessage = from_json(&ibc_msg.msg)?;

    match server_msg {
        ServerIbcMessage::RouteMessage { msg, mut header } => {
            header.current_hop += 1;

            let msg = route_msg(deps, msg, header, &mut app)?;

            Ok(app.response("module_ibc").add_message(msg))
        }
        _ => Err(ServerError::UnauthorizedIbcMessage {}),
    }
}
// ANCHOR_END: module_ibc_handler
