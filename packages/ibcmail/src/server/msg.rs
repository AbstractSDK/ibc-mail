use cosmwasm_schema::QueryResponses;

use crate::{server::ServerAdapter, Header, IbcMailMessage, Route, MessageHash, MessageStatus};

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
        msg: IbcMailMessage,
        route: Option<Route>,
    },
}

#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum ServerMessage {
    Mail {
        message: IbcMailMessage,
    },
    DeliveryStatus {
        id: MessageHash,
        status: MessageStatus,
    }
}

impl ServerMessage {
    pub fn id(&self) -> MessageHash {
        match self {
            ServerMessage::Mail { message } => message.id.clone(),
            ServerMessage::DeliveryStatus { id, .. } => id.clone(),
        }
    }

    pub fn mail(message: IbcMailMessage) -> Self {
        ServerMessage::Mail { message }
    }

    pub fn delivery_status(id: MessageHash, status: MessageStatus) -> Self {
        ServerMessage::DeliveryStatus { id, status }
    }
}

/// App execute messages
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum ServerIbcMessage {
    /// Route a message
    RouteMessage { msg: ServerMessage, header: Header },
}

/// App execute messages
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum ServerCallbackMessage {
    /// Update a message
    UpdateMessage { id: MessageHash, header: Header, status: MessageStatus },
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
