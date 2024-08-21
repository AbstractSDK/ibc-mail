use cosmwasm_std::Response;
pub use ibcmail::server::ServerAdapter as Adapter;
use ibcmail::{server::error::ServerError, IBCMAIL_SERVER_ID};
use ibcmail::{server::msg::ServerInstantiateMsg, IBCMAIL_CLIENT_ID};

use crate::{handlers, replies, APP_VERSION};

use crate::replies::DELIVER_MESSAGE_REPLY;
use abstract_adapter::objects::dependency::StaticDependency;

pub const MAIL_CLIENT: StaticDependency = StaticDependency::new(IBCMAIL_CLIENT_ID, &[]);
pub const IBC_CLIENT: StaticDependency = StaticDependency::new("abstract:ibc-client", &[]);

/// The type of the result returned by your client's entry points.
pub type ServerResult<T = Response> = Result<T, ServerError>;

const ADAPTER: Adapter = Adapter::new(IBCMAIL_SERVER_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_module_ibc(handlers::module_ibc_handler)
    .with_ibc_callback(handlers::ibc_callback_handler)
    .with_replies(&[(DELIVER_MESSAGE_REPLY, replies::deliver_message_reply)])
    .with_dependencies(&[MAIL_CLIENT, IBC_CLIENT]);

// Export handlers
#[cfg(feature = "export")]
abstract_adapter::export_endpoints!(ADAPTER, Adapter);

#[cfg(feature = "interface")]
abstract_adapter::cw_orch_interface!(ADAPTER, Adapter, ServerInstantiateMsg, ServerInterface);
