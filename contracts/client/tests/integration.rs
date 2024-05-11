use abstract_app::objects::namespace::Namespace;

use abstract_client::AbstractClient;
use abstract_client::Application;

use ibcmail_client::{
    msg::{ClientInstantiateMsg, ConfigResponse},
    *,
};
use cw_controllers::AdminError;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

use cosmwasm_std::{coins, Addr};
use env_logger::Env;
use ibcmail::client::error::ClientError;
use ibcmail::{IBCMAIL_CLIENT, IBCMAIL_NAMESPACE};
use ibcmail::server::msg::ServerInstantiateMsg;
use ibcmail_client::contract::interface::ClientInterface;
use server::ServerInterface;

struct TestEnv<Env: CwEnv> {
    env: Env,
    abs: AbstractClient<Env>,
    app: Application<Env, ClientInterface<Env>>
}

impl TestEnv<MockBech32> {
    /// Set up the test environment with an Account that has the App installed
    #[allow(clippy::type_complexity)]
    fn setup() -> anyhow::Result<TestEnv<MockBech32>> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let sender = mock.sender();
        let namespace = Namespace::new(IBCMAIL_NAMESPACE)?;

        // You can set up Abstract with a builder.
        let abs_client = AbstractClient::builder(mock.clone()).build()?;
        // The client supports setting balances for addresses and configuring ANS.
        abs_client.set_balance(sender, &coins(123, "ucosm"))?;

        // Publish both the client and the server
        let publisher = abs_client.publisher_builder(namespace).build()?;
        publisher.publish_app::<ClientInterface<_>>()?;
        publisher.publish_adapter::<ServerInstantiateMsg, ServerInterface<_>>(ServerInstantiateMsg {})?;

        let app = publisher
            .account()
            .install_app_with_dependencies::<ClientInterface<_>>(&ClientInstantiateMsg {}, Empty {},&[])?;

        Ok(TestEnv {
            env: mock,
            abs: abs_client,
            app
        })
    }
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    let env = TestEnv::setup()?;
    let app = env.app;

    let config = app.config()?;
    assert_eq!(config, ConfigResponse {});
    Ok(())
}

mod receive_msg {
    use super::*;
}