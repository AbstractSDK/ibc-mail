use crate::client::error::ClientError;
use crate::client::msg::{AppMigrateMsg, ClientExecuteMsg, ClientInstantiateMsg, ClientQueryMsg};
use abstract_app::AppContract;

pub mod api;
pub mod error;
pub mod msg;
pub mod state;

/// The type of the client that is used to build your client and access the Abstract SDK features.
pub type ClientApp =
    AppContract<ClientError, ClientInstantiateMsg, ClientExecuteMsg, ClientQueryMsg, AppMigrateMsg>;
