use cosmwasm_std::Response;
pub use ibcmail::server::ServerAdapter as Adapter;
use ibcmail::{server::error::ServerError, IBCMAIL_SERVER_ID};
use ibcmail::{server::msg::ServerInstantiateMsg, IBCMAIL_CLIENT_ID};

use crate::{handlers, APP_VERSION};

use abstract_adapter::objects::dependency::StaticDependency;

pub const MAIL_CLIENT: StaticDependency = StaticDependency::new(IBCMAIL_CLIENT_ID, &[]);

/// The type of the result returned by your client's entry points.
pub type ServerResult<T = Response> = Result<T, ServerError>;

const ADAPTER: Adapter = Adapter::new(IBCMAIL_SERVER_ID, APP_VERSION, None)
    .with_execute(handlers::execute_handler)
    .with_module_ibc(handlers::module_ibc_handler)
    .with_dependencies(&[]);

// Export handlers
#[cfg(feature = "export")]
abstract_adapter::export_endpoints!(ADAPTER, Adapter);

#[cfg(feature = "interface")]
abstract_adapter::cw_orch_interface!(ADAPTER, Adapter, ServerInstantiateMsg, ServerInterface);
