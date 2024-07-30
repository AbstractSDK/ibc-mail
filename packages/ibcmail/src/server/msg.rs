use cosmwasm_schema::QueryResponses;

use crate::{server::ServerAdapter, Header, IbcMailMessage, Route};

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_adapter::adapter_msg_types!(ServerAdapter, ServerExecuteMsg, ServerQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct ServerInstantiateMsg {}

/// App execute messages
#[cosmwasm_schema::cw_serde]
pub enum ServerExecuteMsg {
    /// Route a message
    ProcessMessage {
        msg: IbcMailMessage,
        route: Option<Route>,
    },
}

/// App execute messages
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub enum ServerIbcMessage {
    /// Route a message
    RouteMessage { msg: IbcMailMessage, header: Header },
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
