use abstract_app::{
    objects::{
        account::AccountTrace,
        chain_name::ChainName,
        module::{ModuleInfo, ModuleStatus, ModuleVersion},
        module_reference::ModuleReference,
        namespace::Namespace,
    },
    std::{
        ibc_client::QueryMsgFns as IbcQueryFns,
        version_control::{ExecuteMsgFns, ModuleFilter, QueryMsgFns},
        IBC_HOST,
    },
};
use abstract_client::AbstractClient;
use abstract_interface::{Abstract, DependencyCreation, VersionControl};
use clap::Parser;
use client::{msg::ClientInstantiateMsg, ClientInterface};
use cw_orch::{anyhow, prelude::*, tokio::runtime::Runtime};
use ibcmail::{client::msg::ClientExecuteMsgFns, Message, IBCMAIL_NAMESPACE, IBCMAIL_SERVER_ID};
use networks::{HARPOON_4, PION_1};

const SRC: ChainInfo = HARPOON_4;
const DST: ChainInfo = PION_1;

const TEST_NAMESPACE: &str = "ibcmail-demo";

fn test() -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let interchain = DaemonInterchainEnv::new(
        rt.handle(),
        vec![(SRC, None), (DST, None)],
        &ChannelCreationValidator,
    )?;

    let src = interchain.chain(SRC.chain_id)?;
    let dst = interchain.chain(DST.chain_id)?;

    let abs_src = AbstractClient::new(src.clone())?;
    let abs_dst = AbstractClient::new(dst.clone())?;

    let hosts = Abstract::load_from(src.clone())?
        .ibc
        .client
        .list_remote_hosts()?;

    let module_list = abs_src.version_control().module_list(
        Some(ModuleFilter {
            namespace: Some(IBCMAIL_NAMESPACE.to_string()),
            name: None,
            version: None,
            status: None,
        }),
        None,
        None,
    )?;
    println!("module_list: {:?}", module_list);

    let src_acc = abs_src
        .account_builder()
        .install_on_sub_account(false)
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    let src_client = src_acc.application::<ClientInterface<_>>()?;

    let dst_acc = abs_dst
        .account_builder()
        .install_on_sub_account(false)
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    let _dst_client = dst_acc.application::<ClientInterface<_>>()?;

    let send = src_client.send_message(
        Message::new(dst_acc.id()?.into(), "test-subject", "test-body"),
        Some(AccountTrace::Remote(vec![ChainName::from_chain_id(
            DST.chain_id,
        )])),
    )?;

    interchain.wait_ibc(SRC.chain_id, send)?;

    Ok(())
}

fn update_ibc_host<Env: CwEnv>(vc: &VersionControl<Env>) -> anyhow::Result<()> {
    let ibc_host_module = vc
        .modules(vec![ModuleInfo::from_id_latest(IBC_HOST)?])?
        .modules
        .first()
        .unwrap()
        .to_owned()
        .module;
    let version = ibc_host_module.info.version.clone();
    if version == ModuleVersion::Version("0.22.1".into()) {
        return Ok(());
    }
    let ibc_host_addr = ibc_host_module.reference.unwrap_native()?;
    vc.propose_modules(vec![(
        ModuleInfo::from_id(IBC_HOST, ModuleVersion::Version("0.22.1".into()))?,
        ModuleReference::Native(ibc_host_addr),
    )])?;
    vc.approve_any_abstract_modules()?;
    Ok(())
}

pub fn approve_mail_modules<Env: CwEnv>(vc: &VersionControl<Env>) -> anyhow::Result<()> {
    let proposed_abstract_modules = vc.module_list(
        Some(ModuleFilter {
            namespace: Some(IBCMAIL_NAMESPACE.to_string()),
            status: Some(ModuleStatus::Pending),
            ..Default::default()
        }),
        None,
        None,
    )?;

    if proposed_abstract_modules.modules.is_empty() {
        return Ok(());
    }

    vc.approve_or_reject_modules(
        proposed_abstract_modules
            .modules
            .into_iter()
            .map(|m| m.module.info)
            .collect(),
        vec![],
    )?;
    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    let _args = Arguments::parse();
    test()
}
