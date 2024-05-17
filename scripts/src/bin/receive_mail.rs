//! Publishes the module to the Abstract platform by uploading it and registering it on the client store.
//!
//! Info: The mnemonic used to register the module must be the same as the owner of the account that claimed the namespace.
//!
//! ## Example
//!
//! ```bash
//! $ just publish uni-6 osmo-test-5
//! ```

use abstract_app::objects::AccountId;

use abstract_app::std::app::BaseMigrateMsg;
use abstract_app::std::ibc_client::QueryMsgFns as IbcQueryFns;

use abstract_app::std::app;
use abstract_client::{AbstractClient, Account, Application};

use clap::Parser;
use cw_orch::daemon::networks::parse_network;
use cw_orch::prelude::*;
use cw_orch::{anyhow, tokio::runtime::Runtime};

use client::{ClientInterface, APP_VERSION};

use ibcmail::client::msg::{ClientInstantiateMsg, ClientQueryMsgFns};
use ibcmail::{MessageStatus, IBCMAIL_CLIENT_ID, IBCMAIL_SERVER_ID};
use ibcmail_scripts::MYOS;

const TEST_NAMESPACE: &str = "mailtest010";

fn received_messages(args: Arguments) -> anyhow::Result<()> {
    let rt = Runtime::new()?;

    let network = if args.chain_id == MYOS.chain_id {
        MYOS
    } else {
        parse_network(&args.chain_id).unwrap()
    };

    let dst = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;

    let dst_abs = AbstractClient::new(dst.clone())?;

    let dst_acc = dst_abs.account_from(AccountId::local(args.account_seq))?;

    let dst_client = get_client(&dst_acc)?;

    let mut received_messages = dst_client
        .list_messages(MessageStatus::Received, None, None, None)?
        .messages;
    received_messages.sort_by_key(|m| m.timestamp);
    let last_message = received_messages.last().unwrap();
    println!("last_message: {:?}", last_message);

    Ok(())
}

fn get_client(
    acc: &Account<Daemon>,
) -> anyhow::Result<Application<Daemon, ClientInterface<Daemon>>> {
    let client = if let Some(client_module) = acc
        .module_infos()?
        .module_infos
        .iter()
        .find(|m| m.id == IBCMAIL_CLIENT_ID)
    {
        let app = acc.application::<ClientInterface<_>>()?;
        // Upgrade if necessary
        if semver::Version::parse(APP_VERSION)? > client_module.version.version.parse()? {
            app.account().as_ref().manager.upgrade_module(
                IBCMAIL_CLIENT_ID,
                &app::MigrateMsg {
                    base: BaseMigrateMsg {},
                    module: Empty {},
                },
            )?;
        }
        app
    } else {
        let app = acc.install_app_with_dependencies::<ClientInterface<_>>(
            &ClientInstantiateMsg {},
            Empty {},
            &[],
        )?;
        app.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;
        app
    };
    Ok(client)
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Sender
    #[arg(long)]
    chain_id: String,
    /// Recipient
    #[arg(long)]
    account_seq: u32,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let args = Arguments::parse();
    received_messages(args).unwrap();
}
