pub mod client;
pub mod features;
pub mod server;

use crate::server::error::ServerError;
use abstract_adapter::sdk::ModuleRegistryInterface;
use abstract_adapter::std::version_control::NamespaceResponse;
use abstract_app::objects::TruncatedChainId;
use abstract_app::sdk::ModuleRegistry;
use abstract_app::std::objects::AccountId;
use abstract_app::std::objects::{account::AccountTrace, namespace::Namespace};
use const_format::concatcp;
use cosmwasm_std::{Addr, StdError, StdResult, Timestamp};
use std::fmt;
use std::fmt::{Display, Formatter};
use thiserror::Error;

pub const IBCMAIL_NAMESPACE: &str = "ibcmail";
pub const IBCMAIL_CLIENT_ID: &str = concatcp!(IBCMAIL_NAMESPACE, ":", "client");
pub const IBCMAIL_SERVER_ID: &str = concatcp!(IBCMAIL_NAMESPACE, ":", "server");

pub const EMAIL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type MessageHash = String;

/// Struct representing new message to send to another client
// # ANCHOR: message
#[cosmwasm_schema::cw_serde]
pub struct Message {
    pub subject: String,
    pub body: String,
}
// # ANCHOR_END: message

impl Message {
    pub fn new(subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            body: body.into(),
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct IbcMailMessage {
    pub id: MessageHash,
    pub sender: Sender,
    pub recipient: Recipient,
    pub version: String,
    pub timestamp: Timestamp,
    pub message: Message,
}

#[cosmwasm_schema::cw_serde]
pub struct Header {
    // TODO: remove current hop
    pub route: Route,
    pub sender: Sender,
    pub recipient: Recipient,
    pub id: MessageHash,
    pub version: String,
    pub timestamp: Timestamp
}

impl Header {
    pub fn reverse(self, sender: Sender) -> StdResult<Header> {
        let reverse_route = match self.route {
            Route::Remote(mut route) => {
                route.reverse();
                Route::Remote(route)
            }
            Route::Local => Route::Local,
        };
        Ok(Header {

            route: reverse_route,
            recipient: self.sender.clone().try_into()?,
            sender,
            id: self.id,
            version: self.version,
            timestamp: self.timestamp,
        })
    }

    pub fn current_hop(&self, current_chain: &TruncatedChainId) -> StdResult<u32> {
        match self.route {
            Route::Local => Ok(0),
            Route::Remote(ref route) => {
                let position = route.iter().position(|chain| chain == current_chain);
                match position {
                    Some(position) => Ok(position as u32),
                    None => Err(StdError::generic_err("Current chain not in route"))
                }
            }
        }
    }
}

pub type Route = AccountTrace;

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum Recipient {
    Account {
        id: AccountId,
        chain: Option<TruncatedChainId>,
    },
    Namespace {
        namespace: Namespace,
        chain: Option<TruncatedChainId>,
    },
    Server {
        chain: TruncatedChainId,
        address: String,
    },
}

impl From<AccountId> for Recipient {
    fn from(account_id: AccountId) -> Self {
        Recipient::Account {
            id: account_id,
            chain: None,
        }
    }
}

impl Recipient {
    pub fn account(account_id: AccountId, chain: Option<TruncatedChainId>) -> Self {
        Recipient::Account {
            id: account_id,
            chain,
        }
    }
    pub fn namespace(namespace: Namespace, chain: Option<TruncatedChainId>) -> Self {
        Recipient::Namespace { namespace, chain }
    }

    pub fn resolve_account_id<T: ModuleRegistryInterface>(
        &self,
        module_registry: ModuleRegistry<T>,
    ) -> Result<AccountId, ServerError> {
        Ok(match self {
            Recipient::Account { id: account_id, .. } => Ok(account_id.clone()),
            Recipient::Namespace { namespace, .. } => {
                // TODO: this only allows for addressing recipients via namespace of their email account directly.
                // If they have the email application installed on a sub-account, this will not be able to identify the sub-account.
                let namespace_status = module_registry.query_namespace(namespace.clone())?;
                match namespace_status {
                    NamespaceResponse::Claimed(info) => Ok(info.account_id),
                    NamespaceResponse::Unclaimed {} => {
                        return Err(ServerError::UnclaimedNamespace(namespace.clone()));
                    }
                }
            }
            _ => Err(ServerError::NotImplemented(
                "Non-account recipients not supported".to_string(),
            )),
        }?)
    }
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum Sender {
    Account {
        id: AccountId,
        chain: Option<TruncatedChainId>,
    },
    Server {
        chain: TruncatedChainId,
        // String because it's a different chain
        address: String,
    },
}

impl Sender {
    pub fn account(account_id: AccountId, chain: Option<TruncatedChainId>) -> Self {
        Sender::Account {
            id: account_id,
            chain,
        }
    }
}

impl TryFrom<Sender> for Recipient {
    type Error = StdError;

    fn try_from(sender: Sender) -> Result<Self, Self::Error> {
        match sender {
            Sender::Account { id, chain } => Ok(Recipient::Account { id, chain }),
            Sender::Server { chain, address } => Ok(Recipient::Server { chain, address }),
            _ => Err(StdError::generic_err("Cannot convert Sender to Recipient").into()),
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub enum MessageKind {
    Sent,
    Received,
}

impl Display for MessageKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MessageKind::Sent => write!(f, "Sent"),
            MessageKind::Received => write!(f, "Received"),
        }
    }
}

#[derive(Error)]
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum DeliveryFailure {
    #[error("Recipient not found")]
    RecipientNotFound,
    #[error("Unknown failure: {0}")]
    Unknown(String),
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Failure(DeliveryFailure),
}

impl From<DeliveryFailure> for DeliveryStatus {
    fn from(failure: DeliveryFailure) -> Self {
        DeliveryStatus::Failure(failure)
    }
}

impl Display for DeliveryStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DeliveryStatus::Sent => write!(f, "Sent"),
            DeliveryStatus::Delivered => write!(f, "Received"),
            DeliveryStatus::Failure(failure) => write!(f, "Failed: {}", failure),
        }
    }
}
