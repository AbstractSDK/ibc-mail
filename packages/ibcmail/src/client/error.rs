use abstract_app::{sdk::AbstractSdkError, std::AbstractError, AppError as AbstractAppError};
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use thiserror::Error;
use crate::MessageHash;

#[derive(Error, Debug, PartialEq)]
pub enum ClientError {
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
    DappError(#[from] AbstractAppError),

    #[error("Sender is not mail server")]
    NotMailServer {},

    #[error("Recipient is not the current account")]
    NotRecipient {},

    #[error("Message not found: {0}")]
    MessageNotFound(MessageHash),

    #[error("{0} is not implemented")]
    NotImplemented(String),
}
