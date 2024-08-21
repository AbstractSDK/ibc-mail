pub mod contract;
mod handlers;
mod replies;

#[cfg(feature = "interface")]
pub use contract::interface::ServerInterface;
#[cfg(feature = "interface")]
pub use ibcmail::server::msg::ServerQueryMsgFns;

/// The version of your client
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use ibcmail::server::{error, msg, ServerAdapter as Adapter};
