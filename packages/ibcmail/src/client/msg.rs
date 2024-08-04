use cosmwasm_schema::QueryResponses;

use crate::{client::ClientApp, DeliveryStatus, Header, IbcMailMessage, Message, MessageHash, MessageKind, Recipient, Route, Sender};

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
    ReceiveMessage {
        header: Header,
        message: IbcMailMessage
    },
    /// Update the status of a message. only callable by the server
    UpdateDeliveryStatus {
        id: MessageHash,
        status: DeliveryStatus,
    },
    /// Send a message
    SendMessage {
        recipient: Recipient,
        message: Message,
        route: Option<Route>,
    },
}
// # ANCHOR_END: execute_msg

/// App query messages
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::QueryFns)]
#[cw_orch(impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ClientQueryMsg {
    #[returns(MessagesResponse)]
    ListMessages {
        kind: MessageKind,
        filter: Option<MessageFilter>,
        limit: Option<u32>,
        start_after: Option<MessageHash>,
    },
    #[returns(MessagesResponse)]
    Messages {
        kind: MessageKind,
        ids: Vec<MessageHash>,
    },
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
