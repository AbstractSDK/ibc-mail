use abstract_adapter::AdapterContract;

use crate::server::{
    error::ServerError,
    msg::{ServerExecuteMsg, ServerInstantiateMsg, ServerQueryMsg},
};

pub mod api;
pub mod error;
pub mod msg;

/// The type of the client that is used to build your client and access the Abstract SDK features.
pub type ServerAdapter =
    AdapterContract<ServerError, ServerInstantiateMsg, ServerExecuteMsg, ServerQueryMsg>;
