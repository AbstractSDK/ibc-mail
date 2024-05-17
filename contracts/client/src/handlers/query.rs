use abstract_app::sdk::cw_helpers::load_many;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;
use ibcmail::{
    client::{
        error::ClientError,
        msg::{MessageFilter, MessagesResponse},
        state::{RECEIVED, SENT},
    },
    MessageId, MessageStatus,
};
use crate::{
    contract::{App, ClientResult},
    msg::{ClientQueryMsg, ConfigResponse},
    state::CONFIG,
};
pub const DEFAULT_LIMIT: u32 = 50;

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &App,
    msg: ClientQueryMsg,
) -> ClientResult<Binary> {
    match msg {
        ClientQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        ClientQueryMsg::Messages { status, ids } => {
            to_json_binary(&query_messages(deps, status, ids)?)
        }
        ClientQueryMsg::ListMessages {
            status,
            filter,
            start_after,
            limit,
        } => to_json_binary(&query_messages_list(
            deps,
            status,
            filter,
            start_after,
            limit,
        )?),
    }
    .map_err(Into::into)
}

fn query_messages(
    deps: Deps,
    status: MessageStatus,
    ids: Vec<MessageId>,
) -> ClientResult<MessagesResponse> {
    let map = match status {
        MessageStatus::Received => RECEIVED,
        MessageStatus::Sent => SENT,
        _ => return Err(ClientError::NotImplemented("message type".to_string())),
    };

    let messages = load_many(map, deps.storage, ids)?;
    let messages = messages.into_iter().map(|(_, m)| m).collect();

    Ok(MessagesResponse { messages })
}

fn query_messages_list(
    deps: Deps,
    status: MessageStatus,
    _filter: Option<MessageFilter>,
    start: Option<MessageId>,
    limit: Option<u32>,
) -> ClientResult<MessagesResponse> {
    let map = match status {
        MessageStatus::Received => RECEIVED,
        MessageStatus::Sent => SENT,
        _ => return Err(ClientError::NotImplemented("message type".to_string())),
    };

    let messages = cw_paginate::paginate_map(
        &map,
        deps.storage,
        start.as_ref().map(Bound::exclusive),
        limit,
        |_id, message| Ok::<_, ClientError>(message),
    )?;

    // TODO REMOVE, This could run out of gas
    let len = map.keys(deps.storage, None, None, Order::Ascending).count();

    println!("Messages: {:?}, len: {:?}", messages, len);

    Ok(MessagesResponse { messages })
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}
