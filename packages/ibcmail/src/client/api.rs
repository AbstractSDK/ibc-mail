use abstract_adapter::{
    sdk::{
        features::{ModuleIdentification},
        AbstractSdkResult,
    },
    std::objects::module::ModuleId,
};
use abstract_app::traits::AccountIdentification;
use abstract_sdk::{AppInterface};
use abstract_std::{
    objects::AccountId,
};
use cosmwasm_std::{CosmosMsg, Deps};

use crate::{client::msg::ClientExecuteMsg, Header, IbcMailMessage, Message, Route, IBCMAIL_CLIENT_ID};

// API for Abstract SDK users
pub trait ClientInterface: AppInterface {
    /// Construct a new mail_client interface.
    fn mail_client<'a>(&'a self, deps: Deps<'a>, account_id: &'a AccountId) -> MailClient<Self> {
        MailClient {
            base: self,
            deps,
            module_id: IBCMAIL_CLIENT_ID,
            account_id,
        }
    }
}

impl<T: AppInterface> ClientInterface for T {}

#[derive(Clone)]
pub struct MailClient<'a, T: ClientInterface> {
    pub base: &'a T,
    pub module_id: ModuleId<'a>,
    pub account_id: &'a AccountId,
    pub deps: Deps<'a>,
}

impl<'a, T: ClientInterface> MailClient<'a, T> {
    /// Set the module id for the MONEY_MARKET
    pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
        Self { module_id, ..self }
    }

    /// returns the module id
    fn module_id(&self) -> ModuleId {
        self.module_id
    }

    // Execute a request on the ibc mail client
    fn request(&self, msg: ClientExecuteMsg) -> AbstractSdkResult<CosmosMsg> {
        let apps = self.base.apps(self.deps);
        apps.execute(self.module_id(), msg)
    }

    /// Send message
    pub fn send_msg(
        &self,
        message: Message,
        route: Option<Route>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(ClientExecuteMsg::SendMessage { message, route })
    }

    /// Receive message
    pub fn receive_msg(&self, message: IbcMailMessage, _header: Header) -> AbstractSdkResult<CosmosMsg> {
        self.request(ClientExecuteMsg::ReceiveMessage(message))
    }
}

// /// Queries
// impl<'a, T: ClientInterface> MailClient<'a, T> {
//     /// Do a query in the MONEY_MARKET
//     pub fn query<R: DeserializeOwned>(
//         &self,
//         query_msg: ServerQueryMsg,
//     ) -> AbstractSdkResult<R> {
//         let address = self.module_address()?;
//
//         self.deps.querier.query_wasm_smart(address, &QueryMsg::<ClientQueryMsg>::from(query_msg))
//     }
//
//     // Queries
//     pub fn config(
//         &self,
//     ) -> AbstractSdkResult<Uint128> {
//         self.query(ServerQueryMsg::Config {})
//     }
// }
