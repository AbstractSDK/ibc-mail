use abstract_adapter::sdk::AbstractResponse;
use abstract_adapter::std::ibc::ModuleIbcInfo;
use cosmwasm_std::{from_json, Binary, DepsMut, Env, StdResult, StdError};

use ibcmail::{
    server::{error::ServerError, msg::ServerIbcMessage, ServerAdapter},
    IBCMAIL_SERVER_ID,
};
use ibcmail::server::state::AWAITING;
use crate::{contract::ServerResult, handlers::execute::route_msg};
use crate::handlers::execute::update_message_status;

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

            let msg = route_msg(deps, msg, header, &mut app)?;

            Ok(app.response("module_ibc").add_attribute("method", "route").add_message(msg))
        }
        ServerIbcMessage::UpdateMessage { id, header, status } => {
            println!("module_ibc_handler update_msg: {:?}, status: {:?}", id, status);
            // TODO: custom error
            let from_chain = AWAITING.load(deps.storage, &id).map_err(|_| ServerError::Std(StdError::generic_err(format!("Message not found: {:?}", id))))?;
            AWAITING.remove(deps.storage, &id);

            let msg = update_message_status(deps, &mut app, id, header, status)?;

            Ok(app.response("module_ibc").add_attribute("method", "update_status").add_message(msg))
        }
        _ => Err(ServerError::UnauthorizedIbcMessage {}),
    }
}
