pub mod server;
pub mod client;

use abstract_sdk::std::objects::AccountId;
use abstract_std::objects::account::AccountTrace;
use abstract_std::objects::chain_name::ChainName;

use cosmwasm_std::Timestamp;

pub const IBCMAIL_NAMESPACE: &str = "ibcmail";
pub const IBCMAIL_CLIENT: &str = "ibcmail:client";
pub const IBCMAIL_SERVER_ID: &str = "ibcmail:server";

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
            body: body.into()
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
    pub version: String
}

#[cosmwasm_schema::cw_serde]
pub struct Metadata {
    pub current_hop: u32,
    pub route: AccountTrace
}

pub type Route = AccountTrace;

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum Recipient {
    Account {
        id: AccountId,
        chain: Option<ChainName>
    }
}

impl Recipient {
    pub fn account(account_id: AccountId, chain: Option<ChainName>) -> Self {
        Recipient::Account {
            id: account_id,
            chain
        }
    }
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum Sender {
    Account {
        id: AccountId,
        chain: Option<ChainName>
    }
}

impl Sender {
    pub fn account(account_id: AccountId, chain: Option<ChainName>) -> Self {
        Sender::Account {
            id: account_id,
            chain
        }
    }
}