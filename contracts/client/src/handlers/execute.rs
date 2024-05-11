use abstract_app::traits::{AbstractResponse, AccountIdentification, ModuleInterface};
use cosmwasm_std::{DepsMut, ensure_eq, Env, MessageInfo};

use ibcmail::{IBCMAIL_SERVER_ID, Message, Recipient};
use ibcmail::client::state::RECEIVED;

use crate::contract::{App, ClientResult};
use crate::error::ClientError;
use crate::msg::ClientExecuteMsg;
use crate::state::CONFIG;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: App,
    msg: ClientExecuteMsg,
) -> ClientResult {
    match msg {
        ClientExecuteMsg::ReceiveMessage(message) => receive_msg(deps, info, message, app),
        ClientExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

/// Receive a message from the server
fn receive_msg(deps: DepsMut, info: MessageInfo, msg: Message, app: App) -> ClientResult {
   // check that the message sender is the server
    let server_addr = app.modules(deps.as_ref()).module_address(IBCMAIL_SERVER_ID)?;
    ensure_eq!(info.sender, server_addr, ClientError::NotMailServer {});

    match msg.recipient {
        Recipient::Account(ref account_id) => {
            let our_id = app.account_id(deps.as_ref())?;
            // check that the recipient is the current account
            ensure_eq!(account_id, &our_id, ClientError::NotRecipient {} );
        }
        _ => Err(ClientError::NotImplemented("recipients".to_string()))?,

    }

    RECEIVED.save(deps.storage, msg.id.clone(), &msg)?;

    Ok(app.response("received").add_attribute("message_id", &msg.id))
}

/// Update the configuration of the client
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: App) -> ClientResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}
