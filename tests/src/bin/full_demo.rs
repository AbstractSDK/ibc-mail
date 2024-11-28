use abstract_app::objects::TruncatedChainId;
use abstract_app::{
    objects::{
        account::AccountTrace,
        module::{ModuleInfo, ModuleStatus, ModuleVersion},
        module_reference::ModuleReference,
        namespace::Namespace,
    },
    std::{
        ibc_client::QueryMsgFns as IbcQueryFns,
        registry::{ExecuteMsgFns, ModuleFilter, QueryMsgFns},
        IBC_HOST,
    },
};
use abstract_client::AbstractClient;
use abstract_interface::{Abstract, Registry};
use clap::Parser;
use cw_orch::tokio::runtime::Runtime;
use cw_orch::{anyhow, prelude::*};
use cw_orch_interchain::prelude::*;
use networks::{HARPOON_4, PION_1};

use client::ClientInterface;
use ibcmail::{client::msg::ClientExecuteMsgFns, Message, IBCMAIL_NAMESPACE};
use tests::TEST_NAMESPACE;

const SRC: ChainInfo = HARPOON_4;
const DST: ChainInfo = PION_1;

fn test() -> anyhow::Result<()> {
    let _rt = Runtime::new()?;
    let interchain = DaemonInterchain::new(vec![SRC, DST], &ChannelCreationValidator)?;

    let src = interchain.get_chain(SRC.chain_id)?;
    let dst = interchain.get_chain(DST.chain_id)?;

    let abs_src = AbstractClient::new(src.clone())?;
    let abs_dst = AbstractClient::new(dst.clone())?;

    let hosts = Abstract::load_from(src.clone())?
        .ibc
        .client
        .list_remote_hosts()?;

    println!("hosts: {:?}", hosts);

    // update_ibc_host(abs_src.registry())?;
    // update_ibc_host(abs_dst.registry())?;

    // approve_mail_modules(abs_src.registry())?;
    // approve_mail_modules(abs_dst.registry())?;

    let module_list = abs_src.registry().module_list(
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
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    // src_acc.install_app_with_dependencies::<ClientInterface<_>>(&ClientInstantiateMsg {}, Empty {}, &[])?;
    // let app = src_acc.install_app_with_dependencies::<ClientInterface<_>>(&ClientInstantiateMsg {}, Empty {},&[])?;
    let _app = src_acc.application::<ClientInterface<_>>()?;
    // app.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;
    let _src_client = src_acc.application::<ClientInterface<_>>()?;

    let dst_acc = abs_dst
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?;
    // let dst_acc = abs_dst.account_builder().sub_account(&abs_dst.account_from(AccountId::local(1))?).namespace(Namespace::new("mailtest")?).build()?;
    // let app = dst_acc.install_app_with_dependencies::<ClientInterface<_>>(
    //     &ClientInstantiateMsg {},
    //     Empty {},
    //     &[],
    // )?;

    let _app = dst_acc.application::<ClientInterface<_>>()?;
    // app.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;

    let dst_client = dst_acc.application::<ClientInterface<_>>()?;

    // let send = src_client.send_message(
    //     Message::new(dst_acc.id()?.into(), "test-subject", "test-body"),
    //     Some(AccountTrace::Remote(vec![ChainName::from_chain_id(
    //         DST.chain_id,
    //     )])),
    // )?;

    let send = dst_client.send_message(
        Message::new(src_acc.id()?.into(), "test-subject", "test-body"),
        Some(AccountTrace::Remote(vec![TruncatedChainId::from_chain_id(
            SRC.chain_id,
        )])),
    )?;

    interchain.await_and_check_packets(DST.chain_id, send)?;

    Ok(())
}

fn _update_ibc_host<Env: CwEnv>(vc: &Registry<Env>) -> anyhow::Result<()> {
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

pub fn approve_mail_modules<Env: CwEnv>(vc: &Registry<Env>) -> anyhow::Result<()> {
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
