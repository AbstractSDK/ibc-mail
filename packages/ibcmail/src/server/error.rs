use abstract_adapter::sdk::AbstractSdkError;
use abstract_adapter::std::AbstractError;
use abstract_adapter::AdapterError as AbstractAdapterError;
use abstract_std::objects::account::AccountTrace;
use abstract_std::objects::module::ModuleInfo;
use abstract_std::objects::namespace::Namespace;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ServerError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    AdapterError(#[from] AbstractAdapterError),

    #[error("{0} are not implemented")]
    NotImplemented(String),

    #[error("Unauthorized IBC message from module: {0}")]
    UnauthorizedIbcModule(ModuleInfo),

    #[error("Unauthorized IBC message")]
    UnauthorizedIbcMessage,

    #[error("Invalid route hop")]
    InvalidRoute { route: AccountTrace, hop: u32 },

    #[error("Unclaimed namespace: {0}")]
    UnclaimedNamespace(Namespace),
}
