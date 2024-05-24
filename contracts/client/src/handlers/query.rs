use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order};
use cw_storage_plus::Bound;
use ibcmail::{
    client::{
        error::ClientError,
        msg::{MessageFilter, MessagesResponse},
        state::{RECEIVED, SENT},
    },
    MessageHash, MessageStatus,
};

use crate::{
    contract::{App, ClientResult},
    msg::{ClientQueryMsg},
};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &App,
    msg: ClientQueryMsg,
) -> ClientResult<Binary> {
    match msg {
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
        // ClientQueryMsg::Messages {
        //     ids,
        // } => to_json_binary(&query_messages_list(deps, ids)?),
    }
    .map_err(Into::into)
}

fn query_messages_list(
    deps: Deps,
    status: MessageStatus,
    _filter: Option<MessageFilter>,
    start: Option<MessageHash>,
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
    let _len = map.keys(deps.storage, None, None, Order::Ascending).count();

    Ok(MessagesResponse { messages })
}
