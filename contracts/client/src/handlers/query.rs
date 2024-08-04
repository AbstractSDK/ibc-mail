use abstract_app::sdk::cw_helpers::load_many;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env};
use cw_storage_plus::Bound;
use ibcmail::{
    client::{
        error::ClientError,
        msg::{MessageFilter, MessagesResponse},
        state::{RECEIVED, SENT},
    },
    MessageHash, MessageKind,
};

use crate::{
    contract::{App, ClientResult},
    msg::ClientQueryMsg,
};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &App,
    msg: ClientQueryMsg,
) -> ClientResult<Binary> {
    match msg {
        ClientQueryMsg::Messages { kind: status, ids } => {
            to_json_binary(&query_messages(deps, status, ids)?)
        }
        ClientQueryMsg::ListMessages {
            kind: status,
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
    kind: MessageKind,
    ids: Vec<MessageHash>,
) -> ClientResult<MessagesResponse> {
    let map = match kind {
        MessageKind::Received => RECEIVED,
        MessageKind::Sent => SENT,
        _ => return Err(ClientError::NotImplemented("message type".to_string())),
    };

    let messages = load_many(map, deps.storage, ids)?;
    let messages = messages.into_iter().map(|(_, m)| m).collect();

    Ok(MessagesResponse { messages })
}

fn query_messages_list(
    deps: Deps,
    status: MessageKind,
    _filter: Option<MessageFilter>,
    start: Option<MessageHash>,
    limit: Option<u32>,
) -> ClientResult<MessagesResponse> {
    let map = match status {
        MessageKind::Received => RECEIVED,
        MessageKind::Sent => SENT,
        _ => return Err(ClientError::NotImplemented("message type".to_string())),
    };

    let messages = cw_paginate::paginate_map(
        &map,
        deps.storage,
        start.as_ref().map(Bound::exclusive),
        limit,
        |_id, message| Ok::<_, ClientError>(message),
    )?;

    Ok(MessagesResponse { messages })
}
