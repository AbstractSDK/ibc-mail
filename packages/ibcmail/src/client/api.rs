use abstract_adapter::{sdk::AbstractSdkResult, std::objects::module::ModuleId};

use abstract_app::sdk::AppInterface;
use abstract_app::std::app;
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps};

use crate::{
    client::msg::ClientExecuteMsg, Header, IbcMailMessage, Message, Route, IBCMAIL_CLIENT_ID,
};

// API for Abstract SDK users
pub trait ClientInterface: AppInterface {
    /// Construct a new mail_client interface.
    fn mail_client<'a>(&'a self, deps: Deps<'a>) -> MailClient<Self> {
        MailClient {
            base: self,
            deps,
            module_id: IBCMAIL_CLIENT_ID,
        }
    }
}

impl<T: AppInterface> ClientInterface for T {}

#[derive(Clone)]
pub struct MailClient<'a, T: ClientInterface> {
    pub base: &'a T,
    pub module_id: ModuleId<'a>,
    pub deps: Deps<'a>,
}

impl<'a, T: ClientInterface> MailClient<'a, T> {
    /// returns the module id
    fn module_id(&self) -> ModuleId {
        self.module_id
    }

    // Execute a request on the ibc mail client
    fn request(&self, msg: ClientExecuteMsg) -> AbstractSdkResult<CosmosMsg> {
        let app_msg: app::ExecuteMsg<_> = msg.into();

        let modules = self.base.modules(self.deps);
        let app_address = modules.module_address(self.module_id())?;

        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Send message
    pub fn send_msg(&self, message: Message, route: Option<Route>) -> AbstractSdkResult<CosmosMsg> {
        self.request(ClientExecuteMsg::SendMessage { message, route })
    }

    /// Receive message
    pub fn receive_msg(
        &self,
        message: IbcMailMessage,
        _header: Header,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(ClientExecuteMsg::ReceiveMessage(message))
    }
}
