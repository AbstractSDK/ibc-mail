use abstract_app::objects::chain_name::ChainName;
use abstract_app::std::account_factory;
use abstract_app::std::ibc_client::{ExecuteMsgFns, QueryMsgFns};
use abstract_app::std::ibc_host::ExecuteMsgFns as IbcHostFns;
use abstract_cw_orch_polytone::Polytone;
use abstract_interface::Abstract;
use abstract_scripts::abstract_ibc::has_abstract_ibc;

use clap::Parser;
use cw_orch::anyhow;

use cw_orch::daemon::networks::parse_network;
use cw_orch::prelude::*;
use cw_orch::tokio::runtime::{Handle, Runtime};
use ibcmail_scripts::MYOS;

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Chain Id to connect to
    #[arg(long)]
    chain_id: String,
}

/// Connect IBC between two chains.
/// @TODO update this to take in the networks as arguments.
fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let args = Arguments::parse();

    let runtime = Runtime::new()?;

    let dst_chain = (MYOS, Some(std::env::var("MYOS_MNEMONIC")?.to_string()));
    let src_chain = (parse_network(&args.chain_id).unwrap(), None);

    if has_abstract_ibc(src_chain.0.clone(), dst_chain.0.clone(), runtime.handle()) {
        println!("IBC already connected");
        return Ok(());
    };

    connect(src_chain.clone(), dst_chain.clone(), runtime.handle())?;

    Ok(())
}

fn get_daemon(
    chain: ChainInfo,
    handle: &Handle,
    mnemonic: Option<String>,
    deployment_id: Option<String>,
) -> cw_orch::anyhow::Result<Daemon> {
    let mut builder = DaemonBuilder::default();
    builder.chain(chain).handle(handle);
    if let Some(mnemonic) = mnemonic {
        builder.mnemonic(mnemonic);
    }
    if let Some(deployment_id) = deployment_id {
        builder.deployment_id(deployment_id);
    }
    Ok(builder.build()?)
}

pub fn get_deployment_id(src_chain: &ChainInfo, dst_chain: &ChainInfo) -> String {
    format!("{}-->{}", src_chain.chain_id, dst_chain.chain_id)
}

fn connect(
    (src_chain, src_mnemonic): (ChainInfo, Option<String>),
    (dst_chain, dst_mnemonic): (ChainInfo, Option<String>),
    handle: &Handle,
) -> cw_orch::anyhow::Result<()> {
    let src_daemon = get_daemon(src_chain.clone(), handle, src_mnemonic.clone(), None)?;
    let dst_daemon = get_daemon(dst_chain.clone(), handle, dst_mnemonic, None)?;

    let src_abstract = Abstract::load_from(src_daemon.clone())?;
    let dst_abstract = Abstract::load_from(dst_daemon.clone())?;

    let src_polytone_daemon = get_daemon(
        src_chain.clone(),
        handle,
        src_mnemonic,
        Some(get_deployment_id(&src_chain, &dst_chain)),
    )?;

    let src_polytone = Polytone::load_from(src_polytone_daemon)?;

    let interchain = DaemonInterchainEnv::from_daemons(
        handle,
        vec![src_daemon, dst_daemon],
        &ChannelCreationValidator,
    );

    iabstract_ibc_connection_with(&src_abstract, &dst_abstract, &src_polytone, &interchain)?;

    Ok(())
}

fn iabstract_ibc_connection_with(
    src_abstract: &Abstract<Daemon>,
    dst_abstract: &Abstract<Daemon>,
    src_polytone: &Polytone<Daemon>,
    interchain: &DaemonInterchainEnv,
) -> anyhow::Result<()> {
    let chain1_id = src_abstract.ibc.client.get_chain().chain_id();
    let chain1_name = ChainName::from_chain_id(&chain1_id);

    let chain2_id = dst_abstract.ibc.client.get_chain().chain_id();
    let chain2_name = ChainName::from_chain_id(&chain2_id);

    println!("here!");

    /// check for existing infra
    let infras = src_abstract.ibc.client.list_ibc_infrastructures()?;
    println!("infra: {:?}", infras);
    if infras
        .counterparts
        .into_iter()
        .map(|x| x.0)
        .any(|x| x == chain2_name)
    {
        println!("infra exists");
    } else {
        // First, we register the host with the client.
        // We register the polytone note with it because they are linked
        // This triggers an IBC message that is used to get back the proxy address
        let proxy_tx_result = src_abstract.ibc.client.register_infrastructure(
            chain2_name.to_string(),
            dst_abstract.ibc.host.address()?.to_string(),
            src_polytone.note.address()?.to_string(),
        )?;
        // We make sure the IBC execution is done so that the proxy address is saved inside the Abstract contract
        interchain.wait_ibc(&chain1_id, proxy_tx_result).unwrap();
    }

    // Finally, we get the proxy address and register the proxy with the ibc host for the dest chain
    let proxy_address = src_abstract.ibc.client.host(chain2_name.to_string())?;

    println!("proxy_address: {:?}", proxy_address);

    dst_abstract.ibc.host.register_chain_proxy(
        chain1_name.to_string(),
        proxy_address.remote_polytone_proxy.unwrap(),
    )?;

    account_factory::ExecuteMsgFns::update_config(
        &dst_abstract.account_factory,
        None,
        Some(dst_abstract.ibc.host.address()?.to_string()),
        None,
        None,
    )?;
    Ok(())
}
