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
use abstract_app::std::version_control::ExecuteMsgFns;
use abstract_client::{AbstractClient, Publisher};
use clap::Parser;
use cw_orch::prelude::*;
use cw_orch::{
    anyhow,
    environment::TxHandler,
    prelude::{networks::parse_network, DaemonBuilder},
    tokio::runtime::Runtime,
};
use cw_orch::daemon::networks::CONSTANTINE_3;
use ibcmail::IBCMAIL_CLIENT_ID;
use client::ClientInterface;
use ibcmail::server::msg::ServerInstantiateMsg;
use ibcmail_scripts::MYOS;
use server::ServerInterface;

fn publish(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    // run for each requested network
    for network in networks {
        // Setup
        let rt = Runtime::new()?;
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network)
            .build()?;

        let app_namespace = Namespace::from_id(IBCMAIL_CLIENT_ID)?;

        // Create an [`AbstractClient`]
        let abstract_client: AbstractClient<Daemon> = AbstractClient::new(chain.clone())?;

        // Get the [`Publisher`] that owns the namespace, otherwise create a new one and claim the namespace
        let publisher: Publisher<_> = abstract_client.publisher_builder(app_namespace).build()?;

        if publisher.account().owner()? != chain.sender() {
            panic!("The current sender can not publish to this namespace. Please use the wallet that owns the Account that owns the Namespace.")
        }

        // Publish the App to the Abstract Platform
        publisher.publish_app::<ClientInterface<Daemon>>()?;

        // // Publish the App to the Abstract Platform
        // publisher.publish_adapter::<ServerInstantiateMsg, ServerInterface<Daemon>>(
        //     ServerInstantiateMsg {},
        // )?;

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
    // let networks = args
    //     .network_ids
    //     .iter()
    //     .map(|n| parse_network(n).unwrap())
    //     .collect();
    // publish(vec![MYOS]).unwrap();
    publish(vec![CONSTANTINE_3, MYOS]).unwrap();
}
