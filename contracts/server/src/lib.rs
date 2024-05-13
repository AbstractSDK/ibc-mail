pub mod contract;
mod dependencies;
mod handlers;
mod replies;

#[cfg(feature = "interface")]
pub use contract::interface::ServerInterface;
#[cfg(feature = "interface")]
pub use ibcmail::server::msg::{ServerExecuteMsgFns, ServerQueryMsgFns};

/// The version of your client
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use ibcmail::server::error;
pub use ibcmail::server::msg;
pub use ibcmail::server::state;
pub use ibcmail::server::ServerAdapter as Adapter;
