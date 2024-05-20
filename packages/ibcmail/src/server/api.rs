use abstract_adapter::{
    sdk::{
        features::{AccountIdentification, Dependencies, ModuleIdentification},
        AbstractSdkResult, AdapterInterface,
    },
    std::objects::module::ModuleId,
};
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{CosmosMsg, Deps, Uint128};

use crate::{
    server::msg::{ServerExecuteMsg, ServerQueryMsg},
    Message, Route, IBCMAIL_SERVER_ID,
};

// API for Abstract SDK users
/// Interact with the hub adapter in your module.
pub trait ServerInterface: AccountIdentification + Dependencies + ModuleIdentification {
    /// Construct a new money_market interface.
    fn mail_server<'a>(&'a self, deps: Deps<'a>) -> MailServer<Self> {
        MailServer {
            base: self,
            deps,
            module_id: IBCMAIL_SERVER_ID,
        }
    }
}

impl<T: AccountIdentification + Dependencies + ModuleIdentification> ServerInterface for T {}

#[derive(Clone)]
pub struct MailServer<'a, T: ServerInterface> {
    pub base: &'a T,
    pub module_id: ModuleId<'a>,
    pub deps: Deps<'a>,
}

impl<'a, T: ServerInterface> MailServer<'a, T> {
    /// Set the module id for the MONEY_MARKET
    pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
        Self { module_id, ..self }
    }

    /// returns the HUB module id
    fn module_id(&self) -> ModuleId {
        self.module_id
    }

    /// Executes a [MoneyMarketRawAction] in th MONEY_MARKET
    fn request(&self, msg: ServerExecuteMsg) -> AbstractSdkResult<CosmosMsg> {
        let adapters = self.base.adapters(self.deps);

        adapters.execute(self.module_id(), msg)
    }

    /// Route message
    pub fn process_msg(&self, msg: Message, route: Option<Route>) -> AbstractSdkResult<CosmosMsg> {
        self.request(ServerExecuteMsg::ProcessMessage { msg, route })
    }
}

/// Queries
impl<'a, T: ServerInterface> MailServer<'a, T> {
    /// Do a query in the MONEY_MARKET
    pub fn query<R: DeserializeOwned>(&self, query_msg: ServerQueryMsg) -> AbstractSdkResult<R> {
        let adapters = self.base.adapters(self.deps);
        adapters.query(self.module_id(), query_msg)
    }

    // Queries
    pub fn config(&self) -> AbstractSdkResult<Uint128> {
        self.query(ServerQueryMsg::Config {})
    }
}
