use abstract_adapter::{
    sdk::{
        features::{Dependencies, ModuleIdentification},
        AbstractSdkResult,
    },
    std::objects::module::ModuleId,
};
use abstract_sdk::{features::AbstractRegistryAccess, AccountVerification};
use abstract_std::{
    app::ExecuteMsg, manager, manager::ModuleAddressesResponse, objects::AccountId,
};
use cosmwasm_std::{wasm_execute, Addr, CosmosMsg, Deps, Empty};

use crate::{client::msg::ClientExecuteMsg, Header, Message, NewMessage, Route, IBCMAIL_CLIENT_ID};

// API for Abstract SDK users
/// Interact with the hub adapter in your module.
pub trait ClientInterface: Dependencies + ModuleIdentification + AbstractRegistryAccess {
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

impl<T: Dependencies + ModuleIdentification + AbstractRegistryAccess> ClientInterface for T {}

#[derive(Clone)]
pub struct MailClient<'a, T: ClientInterface> {
    pub base: &'a T,
    pub module_id: ModuleId<'a>,
    pub account_id: &'a AccountId,
    pub deps: Deps<'a>,
}

// impl ModuleIdentification for MailClient<'_, dyn ClientInterface> {
//     fn module_id(&self) -> ModuleId {
//         self.module_id
//     }
// }

impl<'a, T: ClientInterface> MailClient<'a, T> {
    /// Set the module id for the MONEY_MARKET
    pub fn with_module_id(self, module_id: ModuleId<'a>) -> Self {
        Self { module_id, ..self }
    }

    /// returns the module id
    fn module_id(&self) -> ModuleId {
        self.module_id
    }

    /// Returns the module addresss
    fn module_address(&self) -> AbstractSdkResult<Addr> {
        // TODO: we theoretically could implement AccountIdentification for this, but it would require implementing Dependencies as well
        let registry = self.base.account_registry(self.deps)?;
        let manager = registry.manager_address(self.account_id)?;
        let module_addresses = self
            .deps
            .querier
            .query_wasm_smart::<ModuleAddressesResponse>(
                manager,
                &manager::QueryMsg::ModuleAddresses {
                    ids: vec![self.module_id().to_string()],
                },
            )?;

        // TODO: this panics if the module is not found

        Ok(module_addresses.modules[0].1.clone())
    }

    /// Executes a [MoneyMarketRawAction] in th MONEY_MARKET
    fn request(&self, msg: ClientExecuteMsg) -> AbstractSdkResult<CosmosMsg> {
        let client_address = self.module_address()?;

        // TODO allow for funds
        Ok(wasm_execute(
            client_address,
            &ExecuteMsg::<ClientExecuteMsg, Empty>::from(msg),
            vec![],
        )?
        .into())
    }

    /// Send message
    pub fn send_msg(
        &self,
        message: NewMessage,
        route: Option<Route>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(ClientExecuteMsg::SendMessage { message, route })
    }

    /// Receive message
    pub fn receive_msg(&self, message: Message, _header: Header) -> AbstractSdkResult<CosmosMsg> {
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
