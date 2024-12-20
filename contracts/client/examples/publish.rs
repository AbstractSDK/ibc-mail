//! Publishes the module to the Abstract platform by uploading it and registering it on the client store.
//!
//! Info: The mnemonic used to register the module must be the same as the owner of the account that claimed the namespace.
//!
//! ## Example
//!
//! ```bash
//! $ just publish uni-6 osmo-test-5
//! ```

use abstract_app::objects::namespace::Namespace;
use abstract_client::{AbstractClient, Publisher};
use clap::Parser;
use cw_orch::daemon::TxSender;
use cw_orch::{
    anyhow,
    environment::TxHandler,
    prelude::{networks::parse_network, DaemonBuilder, *},
    tokio::runtime::Runtime,
};
use ibcmail::IBCMAIL_CLIENT_ID;
use ibcmail_client::ClientInterface;

fn publish(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    // run for each requested network
    for network in networks {
        // Setup
        let rt = Runtime::new()?;
        let chain = DaemonBuilder::new(network).handle(rt.handle()).build()?;

        let app_namespace = Namespace::from_id(IBCMAIL_CLIENT_ID)?;

        // Create an [`AbstractClient`]
        let abstract_client: AbstractClient<Daemon> = AbstractClient::new(chain.clone())?;

        // Get the [`Publisher`] that owns the namespace, otherwise create a new one and claim the namespace
        let publisher_acc = abstract_client
            .fetch_or_build_account(app_namespace.clone(), |builder| {
                builder.namespace(app_namespace)
            })?;
        let publisher = Publisher::new(&publisher_acc)?;

        if publisher.account().owner()? != chain.sender().address() {
            panic!("The current sender can not publish to this namespace. Please use the wallet that owns the Account that owns the Namespace.")
        }

        // Publish the App to the Abstract Platform
        publisher.publish_app::<ClientInterface<Daemon>>()?;
    }
    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Network Id to publish on
    #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
    network_ids: Vec<String>,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let args = Arguments::parse();
    let networks = args
        .network_ids
        .iter()
        .map(|n| parse_network(n).unwrap())
        .collect();
    publish(networks).unwrap();
}
