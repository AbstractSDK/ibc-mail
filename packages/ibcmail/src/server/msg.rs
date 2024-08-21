use cosmwasm_schema::QueryResponses;

use crate::{
    server::ServerAdapter, ClientMetadata, DeliveryStatus, Header, MailMessage, MessageHash,
    ServerMetadata,
};

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_adapter::adapter_msg_types!(ServerAdapter, ServerExecuteMsg, ServerQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct ServerInstantiateMsg {}

/// App execute messages
#[cosmwasm_schema::cw_serde]
pub enum ServerExecuteMsg {
    /// Process a message sent by the client
    ProcessMessage {
        message: MailMessage,
        header: Header,
        metadata: Option<ClientMetadata>,
    },
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum ServerMessage {
    Mail {
        message: MailMessage,
    },
    DeliveryStatus {
        id: MessageHash,
        status: DeliveryStatus,
    },
}

impl ServerMessage {
    pub fn mail(message: MailMessage) -> Self {
        ServerMessage::Mail { message }
    }

    pub fn delivery_status(id: MessageHash, status: DeliveryStatus) -> Self {
        ServerMessage::DeliveryStatus { id, status }
    }
}

/// App execute messages
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum ServerIbcMessage {
    /// Route a message
    RouteMessage {
        msg: ServerMessage,
        header: Header,
        metadata: ServerMetadata,
    },
}

/// App execute messages
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum ServerCallbackMessage {
    /// Update a message
    UpdateMessage {
        id: MessageHash,
        header: Header,
        status: DeliveryStatus,
    },
}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::QueryFns)]
#[cw_orch(impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ServerQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {}

#[cosmwasm_schema::cw_serde]
pub struct CountResponse {
    pub count: i32,
}
