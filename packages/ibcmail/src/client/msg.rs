
use cosmwasm_schema::QueryResponses;
use crate::client::ClientApp;
use crate::Message;


// This is used for type safety and re-exporting the contract endpoint structs.
abstract_app::app_msg_types!(ClientApp, ClientExecuteMsg, ClientQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    /// Initial count
    pub count: i32,
}


/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum ClientExecuteMsg {
    /// Receive a message from the server
    ReceiveMessage(Message),
    UpdateConfig {},
}

#[cosmwasm_schema::cw_serde]
pub struct AppMigrateMsg {}


/// App query messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ClientQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(CountResponse)]
    Count {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {}

#[cosmwasm_schema::cw_serde]
pub struct CountResponse {
    pub count: i32,
}


