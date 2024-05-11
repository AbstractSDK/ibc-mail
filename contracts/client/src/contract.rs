use cosmwasm_std::Response;

pub use ibcmail::client::ClientApp as App;

use crate::{APP_VERSION, error::ClientError, handlers, IBCMAIL_CLIENT};
use crate::dependencies::MAIL_SERVER_DEP;

/// The type of the result returned by your client's entry points.
pub type AppResult<T = Response> = Result<T, ClientError>;



const APP: App = App::new(IBCMAIL_CLIENT, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_dependencies(&[MAIL_SERVER_DEP]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, App);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(APP, App, AppInterface);
