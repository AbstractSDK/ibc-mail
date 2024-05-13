pub mod contract;
mod handlers;
mod replies;
mod dependencies;

#[cfg(feature = "interface")]
pub use contract::interface::ServerInterface;
#[cfg(feature = "interface")]
pub use ibcmail::server::msg::{ServerExecuteMsgFns, ServerQueryMsgFns};

/// The version of your client
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use ibcmail::server::error as error;
pub use ibcmail::server::msg as msg;
pub use ibcmail::server::state as state;
pub use ibcmail::server::ServerAdapter as Adapter;