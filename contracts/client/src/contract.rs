use crate::msg::AppMigrateMsg;
use crate::{IBCMAIL_CLIENT, APP_VERSION, error::AdapterError, handlers, msg::{AppExecuteMsg, AppInstantiateMsg, AppQueryMsg}, replies::{self, INSTANTIATE_REPLY_ID}};
use abstract_app::AppContract;
use cosmwasm_std::Response;

/// The type of the result returned by your client's entry points.
pub type AppResult<T = Response> = Result<T, AdapterError>;

/// The type of the client that is used to build your client and access the Abstract SDK features.
pub type App = AppContract<AdapterError, AppInstantiateMsg, AppExecuteMsg, AppQueryMsg, AppMigrateMsg>;

const APP: App = App::new(IBCMAIL_CLIENT, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_replies(&[(INSTANTIATE_REPLY_ID, replies::instantiate_reply)]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, App);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(APP, App, AppInterface);
