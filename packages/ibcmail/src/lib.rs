pub mod client;
pub mod server;

use abstract_app::objects::TruncatedChainId;
use abstract_app::std::objects::AccountId;
use abstract_app::std::objects::{account::AccountTrace, namespace::Namespace};
use const_format::concatcp;
use cosmwasm_std::Timestamp;

pub const IBCMAIL_NAMESPACE: &str = "ibcmail";
pub const IBCMAIL_CLIENT_ID: &str = concatcp!(IBCMAIL_NAMESPACE, ":", "client");
pub const IBCMAIL_SERVER_ID: &str = concatcp!(IBCMAIL_NAMESPACE, ":", "server");

pub const EMAIL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type MessageHash = String;

/// Struct representing new message to send to another client
// # ANCHOR: message
#[cosmwasm_schema::cw_serde]
pub struct Message {
    pub recipient: Recipient,
    pub subject: String,
    pub body: String,
}
// # ANCHOR_END: message

impl Message {
    pub fn new(recipient: Recipient, subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            recipient,
            subject: subject.into(),
            body: body.into(),
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct IbcMailMessage {
    pub id: MessageHash,
    pub sender: Sender,
    pub version: String,
    pub timestamp: Timestamp,
    pub message: Message,
}

#[cosmwasm_schema::cw_serde]
pub struct Header {
    pub current_hop: u32,
    pub route: Route,
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
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum Sender {
    Account {
        id: AccountId,
        chain: Option<TruncatedChainId>,
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

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum MessageStatus {
    Sent,
    Received,
}
