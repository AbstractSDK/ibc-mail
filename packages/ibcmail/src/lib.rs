pub mod client;
pub mod server;

use abstract_sdk::std::objects::AccountId;

use abstract_std::objects::account::AccountTrace;
use abstract_std::objects::chain_name::ChainName;
use abstract_std::objects::namespace::Namespace;
use const_format::concatcp;

use cosmwasm_std::Timestamp;

pub const IBCMAIL_NAMESPACE: &str = "ibcmail";
pub const IBCMAIL_CLIENT_ID: &str = concatcp!(IBCMAIL_NAMESPACE, ":", "client");
pub const IBCMAIL_SERVER_ID: &str = concatcp!(IBCMAIL_NAMESPACE, ":", "server");

pub type MessageId = String;

/// STruct representing new message to send to another client
#[cosmwasm_schema::cw_serde]
pub struct NewMessage {
    pub recipient: Recipient,
    pub subject: String,
    pub body: String,
}

impl NewMessage {
    pub fn new(recipient: Recipient, subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            recipient,
            subject: subject.into(),
            body: body.into(),
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct Message {
    pub id: MessageId,
    pub sender: Sender,
    pub recipient: Recipient,
    pub subject: String,
    pub body: String,
    pub timestamp: Timestamp,
    // TODO : use semver
    pub version: String,
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
        chain: Option<ChainName>,
    },
    Namespace {
        namespace: Namespace,
        chain: Option<ChainName>,
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
    pub fn account(account_id: AccountId, chain: Option<ChainName>) -> Self {
        Recipient::Account {
            id: account_id,
            chain,
        }
    }
    pub fn namespace(namespace: Namespace, chain: Option<ChainName>) -> Self {
        Recipient::Namespace { namespace, chain }
    }
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum Sender {
    Account {
        id: AccountId,
        chain: Option<ChainName>,
    },
}

impl Sender {
    pub fn account(account_id: AccountId, chain: Option<ChainName>) -> Self {
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
