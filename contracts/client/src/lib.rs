pub mod contract;
mod dependencies;
mod handlers;

/// The version of your app
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "interface")]
pub use contract::interface::ClientInterface;
#[cfg(feature = "interface")]
pub use ibcmail::client::msg::{ClientExecuteMsgFns, ClientQueryMsgFns};
pub use ibcmail::client::{error, msg, state};
