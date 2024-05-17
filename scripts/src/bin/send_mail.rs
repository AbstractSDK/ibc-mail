//! Publishes the module to the Abstract platform by uploading it and registering it on the client store.
//!
//! Info: The mnemonic used to register the module must be the same as the owner of the account that claimed the namespace.
//!
//! ## Example
//!
//! ```bash
//! $ just publish uni-6 osmo-test-5
//! ```
use abstract_app::objects::account::AccountTrace;
use abstract_app::objects::AccountId;
use abstract_app::objects::chain_name::ChainName;
use abstract_app::objects::module::{ModuleInfo, ModuleStatus, ModuleVersion};
use abstract_app::objects::module_reference::ModuleReference;
use abstract_app::objects::namespace::Namespace;
use abstract_app::std::app::BaseMigrateMsg;
use abstract_app::std::ibc_client::QueryMsgFns as IbcQueryFns;
use abstract_app::std::version_control::{ExecuteMsgFns, ModuleFilter, QueryMsgFns};
use abstract_app::std::{app, IBC_HOST};
use abstract_client::{AbstractClient, Account, Application};
use abstract_interface::{Abstract, VersionControl};
use clap::Parser;
use cw_orch::daemon::networks::{ARCHWAY_1, CONSTANTINE_3, NEUTRON_1};
use cw_orch::prelude::*;
use cw_orch::{anyhow, tokio::runtime::Runtime};
use cw_orch::environment::{ChainKind, NetworkInfo};
use client::{APP_VERSION, ClientInterface};

use ibcmail::client::msg::{ClientExecuteMsgFns, ClientInstantiateMsg, ClientQueryMsgFns};
use ibcmail::{NewMessage, IBCMAIL_NAMESPACE, IBCMAIL_CLIENT_ID, IBCMAIL_SERVER_ID, MessageStatus};
use ibcmail_scripts::MYOS;


const SRC: ChainInfo = MYOS;
const DST: ChainInfo = CONSTANTINE_3;

const TEST_NAMESPACE: &str = "mailtest010";

fn test(args: Arguments) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let interchain = DaemonInterchainEnv::new(
        rt.handle(),
        vec![(SRC, None), (DST, None)],
        &ChannelCreationValidator,
    )?;

    let src = interchain.chain(SRC.chain_id)?;
    let dst = interchain.chain(DST.chain_id)?;

    let src_abs = AbstractClient::new(src.clone())?;
    let dst_abs = AbstractClient::new(dst.clone())?;

    // let hosts = Abstract::load_from(src.clone())?
    //     .ibc
    //     .client
    //     .list_remote_hosts()?;
    // println!("hosts: {:?}", hosts);

    // update_ibc_host(abs_src.version_control())?;
    // update_ibc_host(abs_dst.version_control())?;

    approve_mail_modules(src_abs.version_control())?;
    approve_mail_modules(dst_abs.version_control())?;

    let src_acc = if let Some(seq) = args.sender_seq {
        src_abs.account_from(AccountId::local(seq))?
    } else {
        src_abs
            .account_builder()
            .install_on_sub_account(false)
            .namespace(Namespace::new(TEST_NAMESPACE)?)
            .build()?
    };

    let src_client = get_client(&src_acc)?;

    let dst_acc = if let Some(seq) = args.recipient_seq {
        dst_abs.account_from(AccountId::local(seq))?
    } else {
        dst_abs
            .account_builder()
            .install_on_sub_account(false)
            .namespace(Namespace::new(TEST_NAMESPACE)?)
            .build()?
    };

    let dst_client = get_client(&dst_acc)?;

    // let dst_acc = abs_dst.account_builder().sub_account(&abs_dst.account_from(AccountId::local(1))?).namespace(Namespace::new("mailtest")?).build()?;

    let send = src_client.send_message(
        NewMessage::new(dst_acc.id()?.into(), &args.subject, &args.body),
        Some(AccountTrace::Remote(vec![ChainName::from_chain_id(
            DST.chain_id,
        )])),
    )?;

    interchain.wait_ibc(SRC.chain_id, send)?;

    // Rollup has weird formatting of IBC messages
    if SRC.chain_id == MYOS.chain_id {
        // wait for 10 seconds
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    let mut sent_messages = src_client.list_messages(MessageStatus::Sent, None, None, None)?.messages;
    sent_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let sent_message = sent_messages.last().unwrap();
    println!("sent_message_id: {:?}", sent_message.id);

    let received_message = dst_client.messages(vec![sent_message.id.clone()], MessageStatus::Received)?.messages;
    println!("received_message: {:?}", received_message);

    Ok(())
}

fn get_client(acc: &Account<Daemon>) -> anyhow::Result<Application<Daemon, ClientInterface<Daemon>>> {
    let client = if let Some(client_module) = acc.module_infos()?.module_infos.iter().find(|m| m.id == IBCMAIL_CLIENT_ID) {
        let app = acc.application::<ClientInterface<_>>()?;
        // Upgrade if necessary
        if semver::Version::parse(APP_VERSION)? > client_module.version.version.parse()? {
            app.account().as_ref().manager.upgrade_module(IBCMAIL_CLIENT_ID, &app::MigrateMsg {
                base: BaseMigrateMsg {},
                module: Empty {}
            })?;
        }
        app
    } else {
        let app = acc.install_app_with_dependencies::<ClientInterface<_>>(&ClientInstantiateMsg {}, Empty {}, &[])?;
        app.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;
        app
    };
    Ok(client)
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
struct Arguments {
    /// Sender
    #[arg(long)]
    sender_seq: Option<u32>,
    /// Recipient
    #[arg(long)]
    recipient_seq: Option<u32>,
    /// Subject
    #[arg(long)]
    subject: String,
    #[arg(long)]
    body: String,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let args = Arguments::parse();
    dbg!(test(args)).unwrap();
}
