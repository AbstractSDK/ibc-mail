use abstract_app::objects::TruncatedChainId;
use abstract_app::{
    objects::{account::AccountTrace, namespace::Namespace},
    std::registry::QueryMsgFns,
};
use abstract_client::AbstractClient;
use cw_orch::{anyhow, prelude::*};
use cw_orch_interchain::prelude::*;
use networks::{HARPOON_4, PION_1};

use client::ClientInterface;
use ibcmail::{client::msg::ClientExecuteMsgFns, Message};
use tests::TEST_NAMESPACE;

const SRC: ChainInfo = PION_1;
const DST: ChainInfo = HARPOON_4;

fn test() -> anyhow::Result<()> {
    let interchain =
        DaemonInterchain::new(vec![SRC, DST], &ChannelCreationValidator)?;

    let src = interchain.get_chain(SRC.chain_id)?;
    let dst = interchain.get_chain(DST.chain_id)?;

    let abs_src = AbstractClient::new(src.clone())?;
    let abs_dst = AbstractClient::new(dst.clone())?;

    let src_acc = abs_src
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    let src_client = src_acc.application::<ClientInterface<_>>()?;

    let dst_acc = abs_dst
        .account_builder()
        .namespace(Namespace::new(TEST_NAMESPACE)?)
        .build()?;

    let mail_msg = Message::new(dst_acc.id()?.into(), "test-subject", "test-body");

    let send = src_client.send_message(
        mail_msg,
        Some(AccountTrace::Remote(vec![TruncatedChainId::from_chain_id(
            DST.chain_id,
        )])),
    )?;

    interchain.await_and_check_packets(SRC.chain_id, send)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    test()
}
