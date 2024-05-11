use abstract_app::AppContract;
use crate::client::error::ClientError;
use crate::client::msg::{ClientInstantiateMsg, AppMigrateMsg, ClientExecuteMsg, ClientQueryMsg};

pub mod msg;
pub mod state;
pub mod error;


/// The type of the client that is used to build your client and access the Abstract SDK features.
pub type ClientApp = AppContract<ClientError, ClientInstantiateMsg, ClientExecuteMsg, ClientQueryMsg, AppMigrateMsg>;
