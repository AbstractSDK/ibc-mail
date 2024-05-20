use cosmwasm_schema::QueryResponses;

use crate::{client::ClientApp, Message, MessageId, MessageStatus, NewMessage, Route, Sender};

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_app::app_msg_types!(ClientApp, ClientExecuteMsg, ClientQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct ClientInstantiateMsg {}

/// App execute messages
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum ClientExecuteMsg {
    /// Receive a message from the server
    ReceiveMessage(Message),
    /// Send a message
    SendMessage {
        message: NewMessage,
        route: Option<Route>,
    },
    /// Update the client configuration
    UpdateConfig {},
}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ClientQueryMsg {
    #[returns(MessagesResponse)]
    ListMessages {
        status: MessageStatus,
        filter: Option<MessageFilter>,
        limit: Option<u32>,
        start_after: Option<MessageId>,
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
    pub messages: Vec<Message>,
}
