use crate::{APP_VERSION, handlers};

use cosmwasm_std::Response;
use ibcmail::IBCMAIL_SERVER_ID;
use ibcmail::server::error::ServerError;
use ibcmail::server::msg::{ServerInstantiateMsg};
pub use ibcmail::server::ServerAdapter as Adapter;
use crate::dependencies::IBC_CLIENT_DEP;

/// The type of the result returned by your client's entry points.
pub type ServerResult<T = Response> = Result<T, ServerError>;


const ADAPTER: Adapter = Adapter::new(IBCMAIL_SERVER_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_module_ibc(handlers::module_ibc_handler)
    .with_dependencies(&[
        // IBC_CLIENT_DEP
    ]);

// Export handlers
#[cfg(feature = "export")]
abstract_adapter::export_endpoints!(ADAPTER, Adapter);

#[cfg(feature = "interface")]
abstract_adapter::cw_orch_interface!(ADAPTER, Adapter, ServerInstantiateMsg, ServerInterface);
