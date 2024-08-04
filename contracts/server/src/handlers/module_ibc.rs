use abstract_adapter::objects::TruncatedChainId;
use abstract_adapter::sdk::AbstractResponse;
use abstract_adapter::std::ibc::ModuleIbcInfo;
use cosmwasm_std::{from_json, Binary, DepsMut, Env};

use ibcmail::{
    server::{error::ServerError, msg::ServerIbcMessage, ServerAdapter},
    IBCMAIL_SERVER_ID,
};

use crate::{contract::ServerResult, handlers::execute::route_msg};

// ANCHOR: module_ibc_handler
pub fn module_ibc_handler(
    deps: DepsMut,
    env: Env,
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
            let msgs = route_msg(deps, &TruncatedChainId::new(&env), &mut app, header, msg)?;

            Ok(app
                .response("module_ibc")
                .add_attribute("method", "route")
                .add_submessages(msgs))
        }
        _ => Err(ServerError::UnauthorizedIbcMessage {}),
    }
}
