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

// #[cfg(feature = "interface")]
// use { cw_orch::environment::CwEnv, server::ServerInterface, cw_orch::prelude::Deploy, ibcmail::{IBCMAIL_CLIENT_ID, IBCMAIL_SERVER_ID} };
//
// #[cfg(feature = "interface")]
// pub struct IbcMail<Env: CwEnv> {
//     pub client: ClientInterface<Env>,
//     pub server: ServerInterface<Env>,
// }
//
// #[cfg(feature = "interface")]
// impl<Chain: CwEnv> Deploy<Chain> for IbcMail<Chain> {
//     type Error = ();
//     type DeployData = ();
//
//     fn store_on(chain: Chain) -> Result<Self, Self::Error> {
//         let client = ClientInterface::new(IBCMAIL_CLIENT_ID, chain.clone());
//         let server = ServerInterface::new(IBCMAIL_SERVER_ID, chain.clone());
//
//         client.upload_if_needed()?;
//         server.upload_if_needed()?;
//
//         Ok(IbcMail {
//             client,
//             server,
//         })
//     }
//
//     fn deployed_state_file_path() -> Option<String> {
//         todo!()
//     }
//
//     fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
//         vec![
//             Box::new(&mut self.client),
//             Box::new(&mut self.server),
//         ]
//     }
//
//     fn load_from(chain: Chain) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }
