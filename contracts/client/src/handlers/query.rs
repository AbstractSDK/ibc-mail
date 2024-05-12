use crate::contract::{App, ClientResult};
use crate::msg::{ClientQueryMsg, ConfigResponse};
use crate::state::{CONFIG};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};
use cw_storage_plus::Bound;
use ibcmail::client::error::ClientError;
use ibcmail::client::msg::{MessageFilter, MessagesResponse};
use ibcmail::client::state::RECEIVED;
use ibcmail::MessageId;

pub const DEFAULT_LIMIT: u32 = 50;

pub fn query_handler(deps: Deps, _env: Env, _app: &App, msg: ClientQueryMsg) -> ClientResult<Binary> {
    match msg {
        ClientQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        ClientQueryMsg::Messages { filter, start_after, limit } => to_json_binary(&query_messages(deps, filter, start_after, limit)?),
    }
    .map_err(Into::into)
}

fn query_messages(deps: Deps, filter: Option<MessageFilter>, start: Option<MessageId>, limit: Option<u32>) -> ClientResult<MessagesResponse> {
    let messages = cw_paginate::paginate_map(
        &RECEIVED,
        deps.storage,
        start.as_ref().map(Bound::exclusive),
        limit,
        |_id, message| Ok::<_, ClientError>(message),
    )?;

    Ok(MessagesResponse { messages })
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}

