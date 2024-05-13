
use abstract_client::AbstractClient;
use abstract_client::Application;
use abstract_std::objects::namespace::Namespace;

use client::{
    *,
    error::ClientError,
    msg::{ClientInstantiateMsg, ConfigResponse},
};
use cw_controllers::AdminError;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

use cosmwasm_std::{Addr, coins};
use ibcmail::IBCMAIL_SERVER_ID;
use ibcmail::server::msg::{ServerInstantiateMsg, ServerQueryMsgFns};
use ibcmail_server::ServerInterface;

/// Set up the test environment with an Account that has the App installed
#[allow(clippy::type_complexity)]
fn setup(
    count: i32,
) -> anyhow::Result<(
    AbstractClient<MockBech32>,
    Application<MockBech32, ServerInterface<MockBech32>>,
)> {
    // Create a sender and mock env
    let mock = MockBech32::new("mock");
    let sender = mock.sender();
    let namespace = Namespace::from_id(IBCMAIL_SERVER_ID)?;

    // You can set up Abstract with a builder.
    let client = AbstractClient::builder(mock).build()?;
    // The client supports setting balances for addresses and configuring ANS.
    client.set_balance(sender, &coins(123, "ucosm"))?;

    // Build a Publisher Account
    let publisher = client.publisher_builder(namespace).build()?;

    publisher.publish_adapter::<ServerInstantiateMsg, ServerInterface<_>>(ServerInstantiateMsg {})?;

    let app = publisher
        .account()
        .install_adapter::<ServerInterface<_>>( &[])?;

    Ok((client, app))
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    let (_, app) = setup(0)?;

    let config = app.config()?;
    assert_eq!(config, ConfigResponse {});
    Ok(())
}
