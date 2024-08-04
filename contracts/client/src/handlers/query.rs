use crate::{
    contract::{App, ClientResult},
    msg::ClientQueryMsg,
};
use abstract_app::sdk::cw_helpers::load_many;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env};
use cw_storage_plus::Bound;
use ibcmail::client::msg::SentMessagesResponse;
use ibcmail::{
    client::{
        error::ClientError,
        msg::{MessageFilter, ReceivedMessagesResponse},
        state::{RECEIVED, SENT},
    },
    MessageHash, MessageKind,
};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &App,
    msg: ClientQueryMsg,
) -> ClientResult<Binary> {
    match msg {
        ClientQueryMsg::ReceivedMessages { ids } => {
            to_json_binary(&query_received_messages(deps, ids)?)
        }
        ClientQueryMsg::ListSentMessages {
            filter,
            start_after,
            limit,
        } => to_json_binary(&query_sent_messages_list(deps, filter, start_after, limit)?),
        ClientQueryMsg::ListReceivedMessages {
            filter,
            start_after,
            limit,
        } => to_json_binary(&query_received_messages_list(
            deps,
            filter,
            start_after,
            limit,
        )?),
    }
    .map_err(Into::into)
}

fn query_received_messages(
    deps: Deps,
    ids: Vec<MessageHash>,
) -> ClientResult<ReceivedMessagesResponse> {
    let messages = load_many(RECEIVED, deps.storage, ids)?;
    let messages = messages.into_iter().map(|(_, m)| m).collect();

    Ok(ReceivedMessagesResponse { messages })
}

fn query_sent_messages_list(
    deps: Deps,
    _filter: Option<MessageFilter>,
    start: Option<MessageHash>,
    limit: Option<u32>,
) -> ClientResult<SentMessagesResponse> {
    let messages = cw_paginate::paginate_map(
        &SENT,
        deps.storage,
        start.as_ref().map(Bound::exclusive),
        limit,
        |_id, message| Ok::<_, ClientError>(message),
    )?;

    Ok(SentMessagesResponse { messages })
}

fn query_received_messages_list(
    deps: Deps,
    _filter: Option<MessageFilter>,
    start: Option<MessageHash>,
    limit: Option<u32>,
) -> ClientResult<ReceivedMessagesResponse> {
    let messages = cw_paginate::paginate_map(
        &RECEIVED,
        deps.storage,
        start.as_ref().map(Bound::exclusive),
        limit,
        |_id, message| Ok::<_, ClientError>(message),
    )?;

    Ok(ReceivedMessagesResponse { messages })
}
