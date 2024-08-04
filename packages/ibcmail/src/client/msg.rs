use cosmwasm_schema::QueryResponses;

use crate::{
    client::ClientApp, ClientMetadata, DeliveryStatus, Header, MailMessage, MessageHash,
    MessageKind, ReceivedMessage, Recipient, Sender, SentMessage, ServerMetadata,
};

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_app::app_msg_types!(ClientApp, ClientExecuteMsg, ClientQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct ClientInstantiateMsg {}

/// App execute messages
// # ANCHOR: execute_msg
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
#[cw_orch(impl_into(ExecuteMsg))]
pub enum ClientExecuteMsg {
    /// Receive a message from the server.
    ReceiveMessage(ReceivedMessage),
    /// Update the status of a message. only callable by the server
    UpdateDeliveryStatus {
        id: MessageHash,
        status: DeliveryStatus,
    },
    /// Send a message
    SendMessage {
        recipient: Recipient,
        message: MailMessage,
        metadata: Option<ClientMetadata>,
    },
}
// # ANCHOR_END: execute_msg

/// App query messages
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::QueryFns)]
#[cw_orch(impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ClientQueryMsg {
    #[returns(SentMessagesResponse)]
    ListSentMessages {
        filter: Option<MessageFilter>,
        limit: Option<u32>,
        start_after: Option<MessageHash>,
    },
    #[returns(ReceivedMessagesResponse)]
    ListReceivedMessages {
        filter: Option<MessageFilter>,
        limit: Option<u32>,
        start_after: Option<MessageHash>,
    },
    #[returns(ReceivedMessagesResponse)]
    ReceivedMessages { ids: Vec<MessageHash> },
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
pub struct SentMessagesResponse {
    pub messages: Vec<SentMessage>,
}

#[cosmwasm_schema::cw_serde]
pub struct ReceivedMessagesResponse {
    pub messages: Vec<ReceivedMessage>,
}

#[cosmwasm_schema::cw_serde]
pub struct MessageStatusesResponse {
    pub statuses: Vec<(MessageHash, DeliveryStatus)>,
}
