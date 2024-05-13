use abstract_sdk::AbstractResponse;
use abstract_std::ibc::ModuleIbcMsg;



use cosmwasm_std::{DepsMut, Env, from_json};
use ibcmail::{IBCMAIL_SERVER_ID};
use ibcmail::server::error::ServerError;
use ibcmail::server::msg::{ServerIbcMessage};
use ibcmail::server::ServerAdapter;
use crate::contract::{ServerResult};
use crate::handlers::execute::route_msg;

pub fn module_ibc_handler(
    deps: DepsMut,
    _env: Env,
    app: ServerAdapter,
    ibc_msg: ModuleIbcMsg,
) -> ServerResult {
    println!("module_ibc_handler 1 : {:?}", ibc_msg);
    // First check that we received the message from the server
    if ibc_msg.source_module.id().ne(IBCMAIL_SERVER_ID) {
        println!("UnauthorizedIbcModule: {:?}", ibc_msg.source_module.clone());
        return Err(ServerError::UnauthorizedIbcModule(ibc_msg.source_module.clone()));
    };

    let server_msg: ServerIbcMessage = from_json(&ibc_msg.msg)?;

    println!("parsed_msg: {:?}", server_msg);

    match server_msg {
        ServerIbcMessage::RouteMessage { msg, header: mut header } => {
            // We've hopped one more time
            header.current_hop += 1;
            let msg = dbg!(route_msg(deps, msg, header, &app))?;

            println!("routed_msg: {:?}", msg);

            Ok(app.response("module_ibc").add_message(msg))
        }
        _ => Err(ServerError::UnauthorizedIbcMessage {})
    }
}