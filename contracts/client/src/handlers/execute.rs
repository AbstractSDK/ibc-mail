use crate::{
    contract::{App, ClientResult},
    error::ClientError,
    msg::ClientExecuteMsg,
};
use abstract_app::objects::TruncatedChainId;
use abstract_app::{
    sdk::ModuleRegistryInterface,
    traits::{AbstractResponse, AccountIdentification},
};
use base64::prelude::*;
use cosmwasm_std::{ensure_eq, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo};
use ibcmail::client::state::STATUS;
use ibcmail::{
    client::{
        state::{RECEIVED, SENT},
        ClientApp,
    },
    server::api::{MailServer, ServerInterface},
    ClientMetadata, DeliveryStatus, Header, MailMessage, MessageHash, ReceivedMessage, Recipient,
    Sender, ServerMetadata, IBCMAIL_SERVER_ID,
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
        ClientExecuteMsg::SendMessage {
            message,
            recipient,
            metadata,
        } => send_msg(deps, env, info, app, message, recipient, metadata),
        ClientExecuteMsg::ReceiveMessage(message) => receive_msg(deps, info, app, message),
        ClientExecuteMsg::UpdateDeliveryStatus { id, status } => {
            update_delivery_status(deps, info, app, id, status)
        }
    }
}
// # ANCHOR_END: execute_handler

// # ANCHOR: send_msg
fn send_msg(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    app: ClientApp,
    message: MailMessage,
    recipient: Recipient,
    metadata: Option<ClientMetadata>,
) -> ClientResult {
    // validate basic fields of message, construct message to send to server
    let to_hash = format!("{:?}{:?}{:?}", env.block.time, message.subject, recipient);
    let hash = <sha2::Sha256 as sha2::Digest>::digest(to_hash);
    let base_64_hash = BASE64_STANDARD.encode(hash);

    let sender = Sender::account(
        app.account_id(deps.as_ref()).unwrap(),
        Some(TruncatedChainId::new(&env)),
    );
    let version = app.version().to_string();

    let client_header = Header {
        sender,
        recipient,
        id: base_64_hash,
        version,
        timestamp: env.block.time,
        reply_to: None,
    };

    SENT.save(
        deps.storage,
        client_header.id.clone(),
        &(message.clone(), client_header.clone()),
    )?;

    let server: MailServer<_> = app.mail_server(deps.as_ref());
    let route_msg: CosmosMsg = server.process_msg(message, client_header, metadata)?;

    Ok(app.response("send").add_message(route_msg))
}
// # ANCHOR_END: send_msg

/// Receive a message from the server
// # ANCHOR: receive_msg
fn receive_msg(
    deps: DepsMut,
    info: MessageInfo,
    app: App,
    received: ReceivedMessage,
) -> ClientResult {
    ensure_server_sender(deps.as_ref(), &app, info.sender)?;
    ensure_correct_recipient(deps.as_ref(), &received.header.recipient, &app)?;

    let msg_id = received.header.id.clone();
    RECEIVED.save(deps.storage, msg_id.clone(), &received)?;

    Ok(app
        .response("received")
        .add_attribute("message_id", &msg_id))
}
// # ANCHOR_END: receive_msg

fn update_delivery_status(
    deps: DepsMut,
    info: MessageInfo,
    app: App,
    id: MessageHash,
    status: DeliveryStatus,
) -> ClientResult {
    ensure_server_sender(deps.as_ref(), &app, info.sender)?;

    // ensure that the message exists
    SENT.load(deps.storage, id.clone())
        .map_err(|_| ClientError::MessageNotFound(id.clone()))?;
    STATUS.save(deps.storage, id.clone(), &status)?;

    Ok(app
        .response("update_msg_status")
        .add_attribute("message_id", &id)
        .add_attribute("status", status.to_string()))
}

fn ensure_server_sender(deps: Deps, app: &ClientApp, sender: Addr) -> Result<(), ClientError> {
    let sender_module = app
        .module_registry(deps)?
        .module_info(sender)
        .map_err(|_| ClientError::NotMailServer {})?;

    ensure_eq!(
        sender_module.info.id(),
        IBCMAIL_SERVER_ID,
        ClientError::NotMailServer {}
    );
    Ok(())
}

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
