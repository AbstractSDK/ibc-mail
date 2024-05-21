use abstract_app::sdk::cw_helpers::load_many;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult};
use cw_paginate::{DEFAULT_LIMIT, MAX_LIMIT};
use cw_storage_plus::Bound;
use ibcmail::{
    client::{
        error::ClientError,
        msg::{MessageFilter, MessagesResponse},
        state::{RECEIVED, SENT},
    },
    Message, MessageId, MessageStatus,
};

use crate::{
    contract::{App, ClientResult},
    msg::{ClientQueryMsg, ConfigResponse},
    state::CONFIG,
};

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

    Ok(MessagesResponse {
        messages,
        next_key: None,
    })
}

fn query_messages_list(
    deps: Deps,
    status: MessageStatus,
    filter: Option<MessageFilter>,
    start: Option<MessageId>,
    limit: Option<u32>,
) -> ClientResult<MessagesResponse> {
    let map = match status {
        MessageStatus::Received => RECEIVED,
        MessageStatus::Sent => SENT,
        _ => return Err(ClientError::NotImplemented("message type".to_string())),
    };

    // Apply the filter on the map
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let response = {
        // Used to get the next key
        let mut next_key = None;
        // Messages that pass the filter
        let mut messages = vec![];
        // Total number of messages analyzed (used on to make sure there might be future results)
        let mut queried_messages_nb = 0;

        for i in map
            .range(
                deps.storage,
                start.as_ref().map(Bound::exclusive),
                None,
                Order::Ascending,
            )
            .take(limit)
        {
            queried_messages_nb += 1;
            let (_, message) = i?;
            next_key = Some(message.id.clone());

            if !matches_filter(&filter, &message)? {
                continue;
            }

            messages.push(message);
        }

        // If there is less results than the expected limit, that means we have exhausted the map
        if queried_messages_nb != limit {
            next_key = None;
        }

        MessagesResponse { messages, next_key }
    };

    // TODO REMOVE, This could run out of gas
    let len = map.keys(deps.storage, None, None, Order::Ascending).count();

    println!("Messages Response: {:?}, len: {:?}", response, len);

    Ok(response)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}

// Keeping a result here in case we need to parse things or use Addr in the future
/// Verifies that the given message matches the filter, if any
fn matches_filter(filter: &Option<MessageFilter>, message: &Message) -> StdResult<bool> {
    if let Some(filter) = &filter {
        if let Some(sender) = &filter.from {
            if message.sender != *sender {
                return Ok(false);
            }
        }
        if let Some(contains) = &filter.contains {
            if !message.body.contains(contains) && !message.subject.contains(contains) {
                return Ok(false);
            }
        }
        if let Some(sent_after) = &filter.sent_after {
            if message.timestamp.nanos() < sent_after.nanos() {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
