use abstract_client::{AbstractClient, Application};
use abstract_std::objects::namespace::Namespace;
use cosmwasm_std::coins;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};
use ibcmail::{server::msg::ServerInstantiateMsg, IBCMAIL_SERVER_ID};
use ibcmail_server::ServerInterface;

/// Set up the test environment with an Account that has the App installed
#[allow(clippy::type_complexity)]
fn _setup(
    _count: i32,
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

    publisher
        .publish_adapter::<ServerInstantiateMsg, ServerInterface<_>>(ServerInstantiateMsg {})?;

    let app = publisher
        .account()
        .install_adapter::<ServerInterface<_>>(&[])?;

    Ok((client, app))
}
