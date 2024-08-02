use abstract_adapter::objects::TruncatedChainId;
use abstract_adapter::sdk::{
    AccountVerification, features::ModuleIdentification, ModuleRegistryInterface,
};
use abstract_adapter::std::{
    ibc_client,
    IBC_CLIENT,
    objects::{account::AccountTrace, module::ModuleInfo}
    ,
};
use abstract_adapter::std::ibc::Callback;
use abstract_adapter::std::version_control::AccountBase;
use abstract_adapter::traits::AbstractResponse;
use cosmwasm_std::{Addr, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, to_json_binary, wasm_execute};

use ibcmail::{client::api::ClientInterface, Header, IbcMailMessage, MessageHash, MessageStatus, Recipient, Route, server::{
    msg::{ServerExecuteMsg, ServerIbcMessage},
    ServerAdapter,
}};
use ibcmail::client::api::MailClient;
use ibcmail::server::state::AWAITING;

use crate::{contract::{Adapter, ServerResult}, error::ServerError};

// ANCHOR: execute_handler
pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: Adapter,
    msg: ServerExecuteMsg,
) -> ServerResult {
    match msg {
        ServerExecuteMsg::ProcessMessage { msg, route } => {
            process_message(deps, env, info, msg, route, app)
        }
    }
}
// ANCHOR_END: execute_handler

fn process_message(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: IbcMailMessage,
    route: Option<Route>,
    mut app: Adapter,
) -> ServerResult {
    println!("processing message: {:?} with route {:?}", msg, route);

    let current_chain = TruncatedChainId::new(&env);

    let route: Route = if let Some(route) = route {
        Ok::<_, ServerError>(match route {
            Route::Local => Route::Local,
            Route::Remote(mut chains) => {
                println!("processing remote route: {:?}", chains);
                // Enforce that the route always contains every hop in the chain
                if chains.first() == Some(&current_chain) {
                    if chains.len() == 1 {
                        Route::Local
                    } else {
                        Route::Remote(chains)
                    }
                } else {
                    chains.insert(0, current_chain);
                    Route::Remote(chains)
                }
            }
        })
    } else {
        println!("processing message recipient: {:?}", msg.message.recipient);
        match msg.message.recipient.clone() {
            // TODO: add smarter routing
            Recipient::Account { id: _, chain } => Ok(chain.map_or(AccountTrace::Local, |chain| {
                if chain == current_chain {
                    AccountTrace::Local
                } else {
                    AccountTrace::Remote(vec![current_chain, chain.clone()])
                }
            })),
            Recipient::Namespace {
                chain,
                namespace: _,
            } => Ok(chain.map_or(AccountTrace::Local, |chain| {
                if chain == current_chain {
                    AccountTrace::Local
                } else {
                    AccountTrace::Remote(vec![current_chain, chain.clone()])
                }
            })),
            _ => {
                return Err(ServerError::NotImplemented(
                    "Non-account recipients not supported".to_string(),
                ))
            }
        }
    }?;

    let metadata = Header {
        current_hop: 0,
        route,
        recipient: msg.message.recipient.clone()
    };

    let msg = route_msg(deps, msg, metadata, &mut app)?;

    Ok(app.response("route").add_message(msg))
}

pub(crate) fn route_msg(
    deps: DepsMut,
    msg: IbcMailMessage,
    header: Header,
    app: &mut ServerAdapter,
) -> ServerResult<CosmosMsg> {
    println!("routing message: {:?}, metadata: {:?}", msg, header);

    match header.route {
        AccountTrace::Local => route_to_local_account(deps.as_ref(), app, msg, header),
        AccountTrace::Remote(ref chains) => {
            println!("routing to chains: {:?}", chains);
            // check index of hop. If we are on the final hop, route to local account
            if header.current_hop == (chains.len() - 1) as u32 {
                println!("routing to local account: {:?}", chains);
                return route_to_local_account(deps.as_ref(), app, msg.clone(), header);
            }
            // TODO verify that the chain is a valid chain


            let dest_chain =
                chains
                    .get(header.current_hop as usize + 1)
                    .ok_or(ServerError::InvalidRoute {
                        route: header.route.clone(),
                        hop: header.current_hop,
                    })?;

            // Awaiting callback
            AWAITING.save(deps.storage, &msg.id, dest_chain)?;

            let msg = remote_server_msg(deps, &app, &ServerIbcMessage::RouteMessage { msg, header: header.clone() }, dest_chain)?;
            Ok::<CosmosMsg, ServerError>(msg)
        }
    }
}

/// Route a mail message to an account on the local chain
fn route_to_local_account(
    deps: Deps,
    app: &mut ServerAdapter,
    msg: IbcMailMessage,
    header: Header,
) -> ServerResult<CosmosMsg> {
    println!("routing to local account: {:?}", msg.message.recipient);
    // This is a local message
    let mail_client = get_recipient_mail_client(deps, app, &msg.message.recipient)?;
    let receive_msg: CosmosMsg = mail_client.receive_msg(msg, header)?;

    Ok(receive_msg)
}

/// Route a mail message to an account on the local chain
fn update_local_message_status(
    deps: Deps,
    module: &mut ServerAdapter,
    recipient: &Recipient,
    id: MessageHash,
    status: MessageStatus,
) -> ServerResult<CosmosMsg> {
    println!("updating local message status to local account: {:?}", recipient);
    // This is a local message
    let mail_client = get_recipient_mail_client(deps, module, recipient)?;
    return Ok(mail_client.update_msg_status(id, status)?);
}


/// Send a status update for a given message.
pub(crate) fn update_message_status(
    deps: DepsMut,
    module: &mut ServerAdapter,
    id: MessageHash,
    header: Header,
    status: MessageStatus,
) -> ServerResult<CosmosMsg> {
    println!("updating message: {:?}, header: {:?}, status: {:?}", id, header, status);
    // let from_chain = AWAITING.load(deps.storage, &id).map_err(|_| ServerError::AwaitedMsgNotFound(id))?;

    match header.route {
        AccountTrace::Local => update_local_message_status(deps.as_ref(), module, &header.recipient, id, status),
        AccountTrace::Remote(ref chains) => {
            // we need to take the route and do it in reverse
            println!("updating to chains: {:?}", chains);

            // check index of hop. If we are on the final hop, route to local account
            if header.current_hop == 0 {
                println!("updating to local account: {:?}", chains);
                return update_local_message_status(deps.as_ref(), module, &header.recipient, id, status);
            }

            let dest_chain =
                chains
                    .get(header.current_hop as usize - 1)
                    .ok_or(ServerError::InvalidRoute {
                        route: header.route.clone(),
                        hop: header.current_hop,
                    })?;

            let msg = remote_server_msg(deps, &module, &ServerIbcMessage::UpdateMessage { id, header: header.clone(), status }, dest_chain)?;
            Ok(msg)
        }
    }
}

/// Set the target account for the message and get the mail client for the recipient
fn get_recipient_mail_client<'a>(deps: Deps<'a>, app: &'a mut ServerAdapter, recipient: &Recipient) -> ServerResult<MailClient<'a, ServerAdapter>> {
    let account_id = recipient.resolve_account_id(app.module_registry(deps)?)?;

    // ANCHOR: set_acc_and_send
    // Set target account for actions, is used by APIs to retrieve mail client address.
    let recipient_acc: AccountBase = app.account_registry(deps)?.account_base(&account_id)?;
    app.target_account = Some(recipient_acc);
    Ok(app.mail_client::<'a>(deps))
    // ANCHOR_END: set_acc_and_send
}


/// Build a message to send to a server on the destination chain
fn remote_server_msg(deps: DepsMut, module: &ServerAdapter, msg: &ServerIbcMessage, dest_chain: &TruncatedChainId) -> ServerResult<CosmosMsg> {
    // ANCHOR: ibc_client
    // Call IBC client
    let current_module_info = ModuleInfo::from_id(module.module_id(), module.version().into())?;

    let ibc_client_msg = ibc_client::ExecuteMsg::ModuleIbcAction {
        host_chain: dest_chain.clone(),
        target_module: current_module_info,
        msg: to_json_binary(msg)?,
        callback: Some(Callback::new(&Empty {})?)
    };

    let ibc_client_addr: Addr = module
        .module_registry(deps.as_ref())?
        .query_module(ModuleInfo::from_id_latest(IBC_CLIENT)?)?
        .reference
        .unwrap_native()?;

    let msg: CosmosMsg = wasm_execute(ibc_client_addr, &ibc_client_msg, vec![])?.into();
    // ANCHOR_END: ibc_client
    Ok(msg)
}