use abstract_app::sdk::ModuleRegistryInterface;
use abstract_app::traits::{AbstractResponse, AccountIdentification, ModuleInterface};
use cosmwasm_std::{DepsMut, ensure_eq, Env, MessageInfo};

use ibcmail::{IBCMAIL_SERVER_ID, Message, NewMessage, Recipient};
use ibcmail::client::ClientApp;
use ibcmail::client::state::{RECEIVED, SENT};
use ibcmail::server::api::ServerInterface;
use uuid::{uuid, Uuid};

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
        ClientExecuteMsg::SendMessage(message) => send_msg(deps, info, message, app),
        ClientExecuteMsg::ReceiveMessage(message) => receive_msg(deps, info, message, app),
        ClientExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

fn send_msg(deps: DepsMut, info: MessageInfo, msg: NewMessage, app: ClientApp) -> ClientResult {
    // validate basic fields of message, construct message to send to server
    let to_send = Message {
        id: Uuid::new_v4().to_string(),
        sender: app.account_id(deps.as_ref()).unwrap(),
        recipient: msg.recipient,
        subject: msg.subject,
        body: msg.body,
        timestamp: Default::default(),
        version: app.version().to_string()
    };

    SENT.save(deps.storage, to_send.id.clone(), &to_send)?;

    let server = app.mail_server(deps.as_ref());
    let route_msg = server.route_msg(to_send)?;

    Ok(app.response("send").add_message(route_msg))
}

/// Receive a message from the server
fn receive_msg(deps: DepsMut, info: MessageInfo, msg: Message, app: App) -> ClientResult {
    // check that the message sender is the server... this requires the server to be the proper version
    let sender_module = app.module_registry(deps.as_ref())?.module_info(info.sender).map_err(|_| ClientError::NotMailServer {})?;
    ensure_eq!(sender_module.info.id(), IBCMAIL_SERVER_ID, ClientError::NotMailServer {});

    match msg.recipient {
        Recipient::Account { id: ref account_id, .. } => {
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
