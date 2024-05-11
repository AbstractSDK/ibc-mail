use abstract_adapter::AdapterContract;
use crate::server::error::ServerError;
use crate::server::msg::{ServerExecuteMsg, ServerInstantiateMsg, ServerQueryMsg};

pub mod msg;
pub mod state;
pub mod error;
pub mod api;


/// The type of the client that is used to build your client and access the Abstract SDK features.
pub type ServerAdapter = AdapterContract<ServerError, ServerInstantiateMsg, ServerExecuteMsg, ServerQueryMsg>;
