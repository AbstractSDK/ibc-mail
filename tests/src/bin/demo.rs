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

const SRC: ChainInfo = PION_1;
const DST: ChainInfo = HARPOON_4;

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

    let mail_msg = Message::new(dst_acc.id()?.into(), "test-subject", "test-body");

    let send = src_client.send_message(
        mail_msg,
        Some(AccountTrace::Remote(vec![ChainName::from_chain_id(
            DST.chain_id,
        )])),
    )?;

    interchain.wait_ibc(SRC.chain_id, send)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    test()
}
