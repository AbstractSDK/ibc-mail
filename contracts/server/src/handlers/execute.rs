use abstract_adapter::{
    objects::TruncatedChainId,
    sdk::{features::ModuleIdentification, AccountVerification, ModuleRegistryInterface},
    std::ibc::Callback,
    std::version_control::AccountBase,
    std::{
        ibc_client,
        objects::{account::AccountTrace, module::ModuleInfo},
        IBC_CLIENT,
    },
    traits::{AbstractResponse, AccountIdentification},
};
use cosmwasm_std::{
    ensure_eq, to_json_binary, wasm_execute, Addr, CosmosMsg, Deps, DepsMut, Empty, Env,
    MessageInfo, StdResult, SubMsg,
};

use ibcmail::{
    client::{api::ClientInterface, api::MailClient, state::FEATURES},
    features::DELIVERY_STATUS_FEATURE,
    server::{
        msg::{ServerExecuteMsg, ServerIbcMessage, ServerMessage},
        state::{AWAITING, AWAITING_DELIVERY},
        ServerAdapter,
    },
    ClientMetadata, DeliveryFailure, DeliveryStatus, Header, MailMessage, Recipient, Route, Sender,
    ServerMetadata,
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
        ServerExecuteMsg::ProcessMessage {
            message,
            header,
            metadata,
        } => process_message(deps, env, info, app, message, header, metadata),
    }
}
// ANCHOR_END: execute_handler

fn check_sender(
    deps: Deps,
    module: &Adapter,
    current_chain: &TruncatedChainId,
    sender_to_check: Sender,
) -> ServerResult<Sender> {
    let expected_sender = module
        .account_id(deps)
        .map_err(|_| ServerError::NoSenderAccount)?;

    match sender_to_check {
        Sender::Account { id, chain } => {
            ensure_eq!(
                id,
                expected_sender,
                ServerError::MismatchedSender {
                    expected: expected_sender,
                    actual: id,
                }
            );

            Ok(Sender::account(id, Some(current_chain.clone())))
        }
        Sender::Server { address, chain } => {
            panic!("Server sender not implemented");
        }
        _ => Err(ServerError::NotImplemented(
            "Non-account senders not supported".to_string(),
        )),
    }
}

fn process_message(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    mut module: Adapter,
    message: MailMessage,
    header: Header,
    metadata: Option<ClientMetadata>,
) -> ServerResult {
    println!(
        "processing message: {:?} with header: {:?}, metadata {:?}",
        message, header, metadata
    );

    let current_chain = TruncatedChainId::new(&env);
    let checked_sender = check_sender(
        deps.as_ref(),
        &module,
        &current_chain,
        header.sender.clone(),
    )?;

    let client_metadata = metadata.unwrap_or_default();

    let route: Route = if let Some(route) = client_metadata.route {
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
                    chains.insert(0, current_chain.clone());
                    Route::Remote(chains)
                }
            }
        })
    } else {
        // We weren't provided a route
        println!("processing message recipient: {:?}", header.recipient);
        match header.recipient.clone() {
            // TODO: add smarter routing
            Recipient::Account { id: _, chain } => Ok(chain.map_or(AccountTrace::Local, |chain| {
                if chain == current_chain {
                    AccountTrace::Local
                } else {
                    AccountTrace::Remote(vec![current_chain.clone(), chain.clone()])
                }
            })),
            Recipient::Namespace {
                chain,
                namespace: _,
            } => Ok(chain.map_or(AccountTrace::Local, |chain| {
                if chain == current_chain {
                    AccountTrace::Local
                } else {
                    AccountTrace::Remote(vec![current_chain.clone(), chain.clone()])
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
        recipient: header.recipient.clone(),
        id: header.id.clone(),
        version: header.version.clone(),
        sender: checked_sender,
        timestamp: header.timestamp,
        reply_to: None,
    };

    let server_metadata = ServerMetadata { route };

    let msgs = route_message(
        deps.branch(),
        &env,
        &mut module,
        &current_chain,
        header.clone(),
        server_metadata.clone(),
        ServerMessage::mail(message),
    )?;

    Ok(module.response("route").add_submessages(msgs))
}

pub(crate) fn route_message(
    mut deps: DepsMut,
    env: &Env,
    module: &mut ServerAdapter,
    current_chain: &TruncatedChainId,
    header: Header,
    metadata: ServerMetadata,
    message: ServerMessage,
) -> ServerResult<Vec<SubMsg>> {
    println!("routing message: {:?}, metadata: {:?}", message, header);

    let current_hop = metadata.current_hop(current_chain)?;

    let branched_deps = deps.branch();

    let routing = match metadata.route {
        AccountTrace::Local => route_to_local_account(
            branched_deps,
            module,
            message,
            header.clone(),
            metadata.clone(),
        ),
        AccountTrace::Remote(ref chains) => {
            println!("routing to chains: {:?}", chains);
            // check index of hop. If we are on the final hop, route to local account
            if current_hop == (chains.len() - 1) as u32 {
                println!("routing to local account: {:?}", chains.last().unwrap());
                route_to_local_account(
                    branched_deps,
                    module,
                    message.clone(),
                    header.clone(),
                    metadata.clone(),
                )
            } else {
                // TODO verify that the chain is a valid chain
                let dest_chain =
                    chains
                        .get(current_hop as usize + 1)
                        .ok_or(ServerError::InvalidRoute {
                            route: metadata.route.clone(),
                            hop: current_hop,
                        })?;

                // Awaiting callback
                // Save that we're awaiting callbacks from dest chain onwards.
                AWAITING.save(branched_deps.storage, &header.id, dest_chain)?;

                let msg = remote_server_msg(
                    branched_deps,
                    module,
                    &ServerIbcMessage::RouteMessage {
                        msg: message,
                        header: header.clone(),
                        metadata: metadata.clone(),
                    },
                    dest_chain,
                )?;
                Ok::<Vec<SubMsg>, ServerError>(vec![SubMsg::new(msg)])
            }
        }
    };

    // ensure that the route is valid, otherwise send a status update
    match routing {
        Ok(msgs) => Ok(msgs),
        Err(e) => match e {
            ServerError::DeliveryFailure(delivery_failure) => send_delivery_status(
                deps,
                env,
                module,
                current_chain,
                header,
                metadata,
                delivery_failure.into(),
            ),
            _ => Err(e),
        },
    }
}

/// Route a mail message to an account on the local chain
fn route_to_local_account(
    deps: DepsMut,
    app: &mut ServerAdapter,
    msg: ServerMessage,
    header: Header,
    metadata: ServerMetadata,
) -> ServerResult<Vec<SubMsg>> {
    println!("routing to local account: {:?}", header.recipient);
    // This is a local message
    match msg {
        ServerMessage::Mail { message } => {
            AWAITING_DELIVERY.update(deps.storage, |mut awaiting| -> StdResult<Vec<_>> {
                awaiting.push((header.clone(), metadata.clone()));
                Ok(awaiting)
            })?;

            let mail_client = get_recipient_mail_client(deps.as_ref(), app, &header.recipient)
                .map_err(|e| DeliveryFailure::RecipientNotFound)?;

            Ok(vec![SubMsg::reply_always(
                mail_client.receive_msg(message, header, metadata)?,
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

/// Send a delivery status update to the
pub(crate) fn send_delivery_status(
    deps: DepsMut,
    env: &Env,
    module: &mut ServerAdapter,
    current_chain: &TruncatedChainId,
    origin_header: Header,
    origin_metadata: ServerMetadata,
    delivery_status: DeliveryStatus,
) -> Result<Vec<SubMsg>, ServerError> {
    let message_id = origin_header.id.clone();
    let delivery_message = ServerMessage::delivery_status(message_id.clone(), delivery_status);

    let delivery_header = Header {
        sender: Sender::Server {
            address: env.contract.address.to_string(),
            chain: TruncatedChainId::new(env),
        },
        recipient: origin_header.sender.try_into()?,
        // TODO: new ID?
        id: message_id.clone(),
        // TODO: version?
        version: origin_header.version,
        timestamp: env.block.time,
        reply_to: None,
    };

    let delivery_metadata = ServerMetadata {
        route: origin_metadata.reverse_route()?,
    };

    let msg = route_message(
        deps,
        env,
        module,
        current_chain,
        delivery_header,
        delivery_metadata,
        delivery_message,
    )?;
    Ok(msg)
}
