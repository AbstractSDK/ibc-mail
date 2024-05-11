use abstract_sdk::std::objects::AccountId;
use cosmwasm_std::Timestamp;

pub type MessageId = String;

#[cosmwasm_schema::cw_serde]
pub struct Message {
    pub id: MessageId,
    pub sender: AccountId,
    pub recipient: Recipient,
    pub subject: String,
    pub body: String,
    pub timestamp: Timestamp
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum Recipient {
    Account(AccountId),
}