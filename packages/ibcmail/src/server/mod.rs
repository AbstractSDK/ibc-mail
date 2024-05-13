use crate::server::error::ServerError;
use crate::server::msg::{ServerExecuteMsg, ServerInstantiateMsg, ServerQueryMsg};
use abstract_adapter::AdapterContract;

pub mod api;
pub mod error;
pub mod msg;
pub mod state;

/// The type of the client that is used to build your client and access the Abstract SDK features.
pub type ServerAdapter =
    AdapterContract<ServerError, ServerInstantiateMsg, ServerExecuteMsg, ServerQueryMsg>;
