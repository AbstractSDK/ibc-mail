use cosmwasm_schema::QueryResponses;

use crate::{client::ClientApp, IbcMailMessage, MessageHash, MessageStatus, Message, Route, Sender};

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_app::app_msg_types!(ClientApp, ClientExecuteMsg, ClientQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct ClientInstantiateMsg {}

/// App execute messages
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
#[impl_into(ExecuteMsg)]
pub enum ClientExecuteMsg {
    /// Receive a message from the server
    ReceiveMessage(IbcMailMessage),
    /// Send a message
    SendMessage {
        message: Message,
        route: Option<Route>,
    },
    /// Update the client configuration
    UpdateConfig {},
}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::QueryFns)]
#[impl_into(QueryMsg)]
#[derive(QueryResponses)]
pub enum ClientQueryMsg {
    #[returns(MessagesResponse)]
    ListMessages {
        status: MessageStatus,
        filter: Option<MessageFilter>,
        limit: Option<u32>,
        start_after: Option<MessageHash>,
    },
    // #[returns(MessagesResponse)]
    // Messages {
    //     status: MessageStatus,
    //     ids: Vec<MessageId>,
    // },
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub struct MessageFilter {
    pub from: Option<Sender>,
}

#[cosmwasm_schema::cw_serde]
pub struct AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {}

#[cosmwasm_schema::cw_serde]
pub struct MessagesResponse {
    pub messages: Vec<IbcMailMessage>,
}
