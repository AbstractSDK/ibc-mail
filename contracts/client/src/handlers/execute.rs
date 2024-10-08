use abstract_app::objects::TruncatedChainId;
use abstract_app::{
    sdk::ModuleRegistryInterface,
    traits::{AbstractResponse, AccountIdentification},
};
use base64::prelude::*;
use cosmwasm_std::{ensure_eq, CosmosMsg, Deps, DepsMut, Env, MessageInfo};
use ibcmail::{
    client::{
        state::{RECEIVED, SENT},
        ClientApp,
    },
    server::api::{MailServer, ServerInterface},
    IbcMailMessage, Message, Recipient, Route, Sender, IBCMAIL_SERVER_ID,
};

use crate::{
    contract::{App, ClientResult},
    error::ClientError,
    msg::ClientExecuteMsg,
};

// # ANCHOR: execute_handler
pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: App,
    msg: ClientExecuteMsg,
) -> ClientResult {
    match msg {
        ClientExecuteMsg::SendMessage { message, route } => {
            send_msg(deps, env, info, message, route, app)
        }
        ClientExecuteMsg::ReceiveMessage(message) => receive_msg(deps, info, message, app),
    }
}
// # ANCHOR_END: execute_handler

// # ANCHOR: send_msg
fn send_msg(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: Message,
    route: Option<Route>,
    app: ClientApp,
) -> ClientResult {
    // validate basic fields of message, construct message to send to server
    let to_hash = format!("{:?}{:?}{:?}", env.block.time, msg.subject, msg.recipient);
    let hash = <sha2::Sha256 as sha2::Digest>::digest(to_hash);
    let base_64_hash = BASE64_STANDARD.encode(hash);
    let to_send = IbcMailMessage {
        id: base_64_hash,
        sender: Sender::account(
            app.account_id(deps.as_ref()).unwrap(),
            Some(TruncatedChainId::new(&env)),
        ),
        message: Message {
            recipient: msg.recipient,
            subject: msg.subject,
            body: msg.body,
        },
        timestamp: env.block.time,
        version: app.version().to_string(),
    };

    SENT.save(deps.storage, to_send.id.clone(), &to_send)?;

    let server: MailServer<_> = app.mail_server(deps.as_ref());
    let route_msg: CosmosMsg = server.process_msg(to_send, route)?;

    Ok(app.response("send").add_message(route_msg))
}
// # ANCHOR_END: send_msg

/// Receive a message from the server
// # ANCHOR: receive_msg
fn receive_msg(deps: DepsMut, info: MessageInfo, msg: IbcMailMessage, app: App) -> ClientResult {
    let sender_module = app
        .module_registry(deps.as_ref())?
        .module_info(info.sender)
        .map_err(|_| ClientError::NotMailServer {})?;
    ensure_eq!(
        sender_module.info.id(),
        IBCMAIL_SERVER_ID,
        ClientError::NotMailServer {}
    );

    ensure_correct_recipient(deps.as_ref(), &msg.message.recipient, &app)?;

    RECEIVED.save(deps.storage, msg.id.clone(), &msg)?;

    Ok(app
        .response("received")
        .add_attribute("message_id", &msg.id))
}
// # ANCHOR_END: receive_msg

fn ensure_correct_recipient(
    deps: Deps,
    recipient: &Recipient,
    app: &ClientApp,
) -> ClientResult<()> {
    match recipient {
        Recipient::Account {
            id: ref account_id, ..
        } => {
            let our_id = app.account_id(deps)?;

            // check that the recipient is the current account
            ensure_eq!(account_id, &our_id, ClientError::NotRecipient {});
        }
        Recipient::Namespace {
            namespace,
            chain: _,
        } => {
            let _namespace = app
                .module_registry(deps)?
                .query_namespace(namespace.to_owned())?;
        }
        _ => Err(ClientError::NotImplemented("recipients".to_string()))?,
    }
    Ok(())
}
