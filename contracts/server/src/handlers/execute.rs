use abstract_adapter::objects::TruncatedChainId;
use abstract_adapter::sdk::{
    features::ModuleIdentification, AccountVerification, ModuleRegistryInterface,
};
use abstract_adapter::std::ibc::Callback;
use abstract_adapter::std::version_control::AccountBase;
use abstract_adapter::std::{
    ibc_client,
    objects::{account::AccountTrace, module::ModuleInfo},
    IBC_CLIENT,
};
use abstract_adapter::traits::{AbstractResponse, AccountIdentification};
use cosmwasm_std::{
    to_json_binary, wasm_execute, Addr, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    StdResult, SubMsg,
};

use ibcmail::client::api::MailClient;
use ibcmail::client::state::FEATURES;
use ibcmail::features::DELIVERY_STATUS_FEATURE;
use ibcmail::server::msg::ServerMessage;
use ibcmail::server::state::{AWAITING, AWAITING_DELIVERY};
use ibcmail::{
    client::api::ClientInterface,
    server::{
        msg::{ServerExecuteMsg, ServerIbcMessage},
        ServerAdapter,
    },
    Header, IbcMailMessage, Recipient, Route, Sender,
};

use crate::replies::DELIVER_MESSAGE_REPLY;
use crate::{
    contract::{Adapter, ServerResult},
    error::ServerError,
};

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

    let sender_acc_id = app
        .account_id(deps.as_ref())
        .map_err(|_| ServerError::NoSenderAccount)?;

    let current_chain = TruncatedChainId::new(&env);
    let sender = Sender::account(sender_acc_id, Some(current_chain.clone()));

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

    let header = Header {
        current_hop: 0,
        route,
        recipient: msg.message.recipient.clone(),
        sender,
    };

    let msgs = route_msg(deps, &mut app, ServerMessage::mail(msg), header)?;

    Ok(app.response("route").add_submessages(msgs))
}

pub(crate) fn route_msg(
    deps: DepsMut,
    app: &mut ServerAdapter,
    msg: ServerMessage,
    header: Header,
) -> ServerResult<Vec<SubMsg>> {
    println!("routing message: {:?}, metadata: {:?}", msg, header);

    match header.route {
        AccountTrace::Local => route_to_local_account(deps, app, msg, header),
        AccountTrace::Remote(ref chains) => {
            println!("routing to chains: {:?}", chains);
            // check index of hop. If we are on the final hop, route to local account
            if header.current_hop == (chains.len() - 1) as u32 {
                println!("routing to local account: {:?}", chains.last().unwrap());
                return route_to_local_account(deps, app, msg.clone(), header);
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
            // Save that we're awaiting callbacks from dest chain onwards.
            AWAITING.save(deps.storage, &msg.id(), dest_chain)?;

            let msg = remote_server_msg(
                deps,
                &app,
                &ServerIbcMessage::RouteMessage {
                    msg,
                    header: header.clone(),
                },
                dest_chain,
            )?;
            Ok::<Vec<SubMsg>, ServerError>(vec![SubMsg::new(msg)])
        }
    }
}

/// Route a mail message to an account on the local chain
fn route_to_local_account(
    deps: DepsMut,
    app: &mut ServerAdapter,
    msg: ServerMessage,
    header: Header,
) -> ServerResult<Vec<SubMsg>> {
    println!("routing to local account: {:?}", header.recipient);
    // This is a local message
    match msg {
        ServerMessage::Mail { message } => {
            AWAITING_DELIVERY.update(deps.storage, |mut awaiting| -> StdResult<Vec<_>> {
                awaiting.push((message.id.clone(), header.clone()));
                Ok(awaiting)
            })?;

            let mail_client = get_recipient_mail_client(deps.as_ref(), app, &header.recipient)?;
            Ok(vec![SubMsg::reply_always(
                mail_client.receive_msg(message, header)?,
                DELIVER_MESSAGE_REPLY,
            )])
        }
        ServerMessage::DeliveryStatus { id, status } => {
            println!(
                "updating local delivery message status: recipient: {:?} status: {:?}",
                header.recipient, status
            );

            let mail_client = get_recipient_mail_client(deps.as_ref(), app, &header.recipient)?;
            let is_delivery_enabled = FEATURES
                .query(
                    &deps.querier,
                    mail_client.module_address()?,
                    DELIVERY_STATUS_FEATURE.to_string(),
                )
                .is_ok_and(|f| f.is_some_and(|f| f));

            if is_delivery_enabled {
                Ok(vec![SubMsg::new(
                    mail_client.update_msg_status(id, status)?,
                )])
            } else {
                Ok(vec![])
            }
        }
        _ => Err(ServerError::NotImplemented(
            "Unknown message type".to_string(),
        )),
    }
}

/// Set the target account for the message and get the mail client for the recipient
fn get_recipient_mail_client<'a>(
    deps: Deps<'a>,
    app: &'a mut ServerAdapter,
    recipient: &Recipient,
) -> ServerResult<MailClient<'a, ServerAdapter>> {
    let account_id = recipient.resolve_account_id(app.module_registry(deps)?)?;

    // ANCHOR: set_acc_and_send
    // Set target account for actions, is used by APIs to retrieve mail client address.
    let recipient_acc: AccountBase = app.account_registry(deps)?.account_base(&account_id)?;
    app.target_account = Some(recipient_acc);
    Ok(app.mail_client::<'a>(deps))
    // ANCHOR_END: set_acc_and_send
}

/// Build a message to send to a server on the destination chain
fn remote_server_msg(
    deps: DepsMut,
    module: &ServerAdapter,
    msg: &ServerIbcMessage,
    dest_chain: &TruncatedChainId,
) -> ServerResult<CosmosMsg> {
    // ANCHOR: ibc_client
    // Call IBC client
    let current_module_info = ModuleInfo::from_id(module.module_id(), module.version().into())?;

    let ibc_client_msg = ibc_client::ExecuteMsg::ModuleIbcAction {
        host_chain: dest_chain.clone(),
        target_module: current_module_info,
        msg: to_json_binary(msg)?,
        callback: Some(Callback::new(&Empty {})?),
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
