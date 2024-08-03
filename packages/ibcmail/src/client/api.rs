use abstract_adapter::{sdk::AbstractSdkResult, std::objects::module::ModuleId};

use abstract_app::sdk::AppInterface;

use cosmwasm_std::{Addr, CosmosMsg, Deps};

use crate::{client::msg::ClientExecuteMsg, Header, IbcMailMessage, Message, Route, IBCMAIL_CLIENT_ID, MessageHash, DeliveryStatus};

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

    pub fn module_address(&self) -> AbstractSdkResult<Addr> {
        self.base.modules(self.deps).module_address(self.module_id())
    }

    // Execute a request on the ibc mail client
    fn request(&self, msg: ClientExecuteMsg) -> AbstractSdkResult<CosmosMsg> {
        let apps = self.base.apps(self.deps);
        apps.execute(self.module_id(), msg)
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

    /// Receive message
    pub fn update_msg_status(
        &self,
        id: MessageHash,
        status: DeliveryStatus,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.request(ClientExecuteMsg::UpdateDeliveryStatus { id, status })
    }
}
