pub mod contract;
pub mod error;
mod handlers;
pub mod msg;
mod replies;
pub mod state;

/// The version of your app
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the client
pub const IBCMAIL_CLIENT: &str = "ibcmail:client";


#[cfg(feature = "interface")]
pub use contract::interface::AppInterface;
#[cfg(feature = "interface")]
pub use msg::{AppExecuteMsgFns, AppQueryMsgFns};

