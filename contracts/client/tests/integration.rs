use abstract_app::objects::AccountId;
use abstract_app::objects::namespace::Namespace;
use abstract_client::AbstractClient;
use abstract_client::Application;
use cosmwasm_std::coins;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

use ibcmail::{IBCMAIL_NAMESPACE, Message, Recipient};
use ibcmail::server::msg::ServerInstantiateMsg;
use ibcmail_client::{
    *,
    msg::{ClientInstantiateMsg, ConfigResponse},
};
use ibcmail_client::contract::interface::ClientInterface;
use server::ServerInterface;
use speculoos::prelude::*;

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

fn create_test_message(from: AccountId, to: AccountId) -> Message {
    Message {
        id: "test-id".to_string(),
        sender: from.clone(),
        recipient: Recipient::account(to.clone()),
        subject: "test-subject".to_string(),
        body: "test-body".to_string(),
        timestamp: Default::default(),
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
    use abstract_app::objects::AccountId;
    use speculoos::assert_that;
    use ibcmail::{IBCMAIL_SERVER_ID, Message, Recipient};
    use ibcmail::client::error::ClientError;
    use super::*;

    #[test]
    fn can_receive_from_server() -> anyhow::Result<()> {
        let env = TestEnv::setup()?;
        let app = env.app;

        let server_account_id = app.account().id().unwrap();
        let app_account_id = app.account().id().unwrap();

        let msg = create_test_message(server_account_id.clone(), app_account_id.clone());
        let server_adr = app.account().module_addresses(vec![IBCMAIL_SERVER_ID.into()])?.modules[0].1.clone();
        let res = app.call_as(&server_adr).receive_message(msg);

        assert_that!(res).is_ok();
        Ok(())
    }

    #[test]
    fn cannot_receive_from_not_server() -> anyhow::Result<()> {
        let env = TestEnv::setup()?;
        let app = env.app;

        let app_account_id = app.account().id().unwrap();

        let msg = create_test_message(app_account_id.clone(), app_account_id.clone());
        let res = app.receive_message(msg);

        assert_that!(res).is_err().matches(|e| {
            e.root().to_string().contains("Sender is not mail server")
        });

        Ok(())
    }

}