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
use abstract_interface::Abstract;
use clap::Parser;
use cw_orch::{
    anyhow,
    prelude::{networks::parse_network, DaemonBuilder, *},
    tokio::runtime::Runtime,
};

use ibcmail::IBCMAIL_CLIENT_ID;

fn publish(networks: Vec<ChainInfo>) -> anyhow::Result<()> {
    // run for each requested network
    for network in networks {
        // Setup
        let rt = Runtime::new()?;
        let chain = DaemonBuilder::new(network).handle(rt.handle()).build()?;

        let app_namespace = Namespace::from_id(IBCMAIL_CLIENT_ID)?;

        // Create an [`AbstractClient`]
        let abs = Abstract::new(chain.clone());

        abs.version_control
            .approve_all_modules_for_namespace(app_namespace)?;
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
