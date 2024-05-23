use abstract_app::{
    objects::chain_name::ChainName,
    sdk::ModuleRegistryInterface,
    traits::{AbstractResponse, AccountIdentification},
};
use cosmwasm_std::{ensure_eq, Deps, DepsMut, Env, MessageInfo, Order};
use ibcmail::{
    client::{
        state::{RECEIVED, SENT},
        ClientApp,
    },
    server::api::ServerInterface,
    IbcMailMessage, Message, Recipient, Route, Sender, IBCMAIL_SERVER_ID,
};

use crate::{
    contract::{App, ClientResult},
    error::ClientError,
    msg::ClientExecuteMsg,
    state::CONFIG,
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: App,
    msg: ClientExecuteMsg,
) -> ClientResult {
    println!("Env: {:?}", env);
    match msg {
        ClientExecuteMsg::SendMessage { message, route } => {
            send_msg(deps, env, info, message, route, app)
        }
        ClientExecuteMsg::ReceiveMessage(message) => receive_msg(deps, info, message, app),
        ClientExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
        _ => Err(ClientError::NotImplemented("execute".to_string())),
    }
}

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

    let to_send = IbcMailMessage {
        // TODO: base64 encode this
        id: format!("{:x}", hash),
        sender: Sender::account(
            app.account_id(deps.as_ref()).unwrap(),
            Some(ChainName::new(&env)),
        ),
        recipient: msg.recipient,
        subject: msg.subject,
        body: msg.body,
        timestamp: env.block.time,
        version: app.version().to_string(),
    };

    SENT.save(deps.storage, to_send.id.clone(), &to_send)?;

    let server = app.mail_server(deps.as_ref());
    let route_msg = server.process_msg(to_send, route)?;

    Ok(app.response("send").add_message(route_msg))
}

/// Receive a message from the server
fn receive_msg(deps: DepsMut, info: MessageInfo, msg: IbcMailMessage, app: App) -> ClientResult {
    // TODO: remove print
    println!("Received message: {:?}", msg);
    // check that the message sender is the server... this requires the server to be the proper version
    // TODO, should we have a function that is able to check against a module ID directly in the SDK ?
    let sender_module = app
        .module_registry(deps.as_ref())?
        .module_info(info.sender)
        .map_err(|_| ClientError::NotMailServer {})?;
    ensure_eq!(
        sender_module.info.id(),
        IBCMAIL_SERVER_ID,
        ClientError::NotMailServer {}
    );

    ensure_correct_recipient(deps.as_ref(), &msg.recipient, &app)?;

    RECEIVED.save(deps.storage, msg.id.clone(), &msg)?;
    // TODO: remove, could run out of gas!!
    let len = RECEIVED
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    println!("Received length: {:?}", len);

    Ok(app
        .response("received")
        .add_attribute("message_id", &msg.id))
}

fn ensure_correct_recipient(
    deps: Deps,
    recipient: &Recipient,
    app: &ClientApp,
) -> ClientResult<()> {
    println!("Checking recipient: {:?}", recipient);
    match recipient {
        Recipient::Account {
            id: ref account_id, ..
        } => {
            let our_id = app.account_id(deps)?;
            println!("recipient_id: {:?}, our_id: {:?}", account_id, our_id);

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

/// Update the configuration of the client
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: App) -> ClientResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}
