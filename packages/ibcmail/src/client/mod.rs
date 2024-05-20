use abstract_app::AppContract;

use crate::client::{
    error::ClientError,
    msg::{AppMigrateMsg, ClientExecuteMsg, ClientInstantiateMsg, ClientQueryMsg},
};

pub mod api;
pub mod error;
pub mod msg;
pub mod state;

/// The type of the client that is used to build your client and access the Abstract SDK features.
pub type ClientApp =
    AppContract<ClientError, ClientInstantiateMsg, ClientExecuteMsg, ClientQueryMsg, AppMigrateMsg>;
