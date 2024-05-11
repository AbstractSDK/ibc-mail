use crate::{
    error::ServerError,
    handlers,
    msg::{ServerExecuteMsg, ServerInstantiateMsg, ServerQueryMsg},
    replies::{self, INSTANTIATE_REPLY_ID},
};
use abstract_adapter::AdapterContract;
use cosmwasm_std::Response;

/// The version of your client
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the client
pub const APP_ID: &str = "ibcmail:client";

/// The type of the result returned by your client's entry points.
pub type AppResult<T = Response> = Result<T, ServerError>;

/// The type of the client that is used to build your client and access the Abstract SDK features.
pub type Adapter = AdapterContract<ServerError, ServerInstantiateMsg, ServerExecuteMsg, ServerQueryMsg>;

const ADAPTER: Adapter = Adapter::new(APP_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_replies(&[(INSTANTIATE_REPLY_ID, replies::instantiate_reply)]);

// Export handlers
#[cfg(feature = "export")]
abstract_adapter::export_endpoints!(ADAPTER, Adapter);

#[cfg(feature = "interface")]
abstract_adapter::cw_orch_interface!(ADAPTER, Adapter, ServerInterface);
