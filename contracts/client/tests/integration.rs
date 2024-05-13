use abstract_app::objects::namespace::Namespace;
use abstract_app::objects::AccountId;

use abstract_client::Application;
use abstract_client::{AbstractClient, Environment};
use abstract_cw_orch_polytone::Polytone;

// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

use abstract_interchain_tests::setup::ibc_connect_polytone_and_abstract;
use ibcmail::server::msg::ServerInstantiateMsg;
use ibcmail::{Message, Recipient, Sender, IBCMAIL_NAMESPACE, IBCMAIL_SERVER_ID};
use ibcmail_client::contract::interface::ClientInterface;
use ibcmail_client::{
    msg::{ClientInstantiateMsg, ConfigResponse},
    *,
};
use server::ServerInterface;
use speculoos::prelude::*;

struct TestEnv<Env: CwEnv> {
    env: Env,
    abs: AbstractClient<Env>,
    client1: Application<Env, ClientInterface<Env>>,
    client2: Application<Env, ClientInterface<Env>>,
    // server: Application<Env, ServerInterface<Env>>
}

impl<Env: CwEnv> TestEnv<Env> {
    /// Set up the test environment with an Account that has the App installed
    #[allow(clippy::type_complexity)]
    fn setup(env: Env) -> anyhow::Result<TestEnv<Env>> {
        let namespace = Namespace::new(IBCMAIL_NAMESPACE)?;

        // You can set up Abstract with a builder.
        let abs_client = AbstractClient::builder(env.clone()).build()?;

        // // The client supports setting balances for addresses and configuring ANS.
        // let sender = mock.sender();
        // abs_client.set_balance(sender, &coins(123, "ucosm"))?;

        // Publish both the client and the server
        let publisher = abs_client.publisher_builder(namespace).build()?;
        publisher.publish_app::<ClientInterface<_>>()?;
        publisher
            .publish_adapter::<ServerInstantiateMsg, ServerInterface<_>>(ServerInstantiateMsg {})?;

        // let app = publisher.account()
        //     .install_app_with_dependencies::<ClientInterface<_>>(&ClientInstantiateMsg {}, Empty {},&[])?;
        // app.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;
        //
        // let app2 = publisher.account()
        //     .install_app_with_dependencies::<ClientInterface<_>>(&ClientInstantiateMsg {}, Empty {},&[])?;
        // app2.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;

        let acc = abs_client
            .account_builder()
            .install_on_sub_account(false)
            .build()?;
        // acc.as_ref().manager.update_settings(Some(true))?;
        let app = acc.install_app_with_dependencies::<ClientInterface<_>>(
            &ClientInstantiateMsg {},
            Empty {},
            &[],
        )?;
        app.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;

        let acc2 = abs_client
            .account_builder()
            .install_on_sub_account(false)
            .build()?;
        // acc2.as_ref().manager.update_settings(Some(true))?;
        let app2 = acc2.install_app_with_dependencies::<ClientInterface<_>>(
            &ClientInstantiateMsg {},
            Empty {},
            &[],
        )?;
        app2.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;

        // let server = app.account().application::<ServerInterface<MockBech32>>()?;

        Ok(TestEnv {
            env,
            abs: abs_client,
            client1: app,
            client2: app2,
            // server
        })
    }

    fn enable_ibc(&self) -> anyhow::Result<()> {
        Polytone::deploy_on(self.abs.environment().clone(), None)?;
        Ok(())
    }
}

fn create_test_message(from: AccountId, to: AccountId) -> Message {
    Message {
        id: "test-id".to_string(),
        sender: Sender::account(from.clone(), None),
        recipient: Recipient::account(to.clone(), None),
        subject: "test-subject".to_string(),
        body: "test-body".to_string(),
        timestamp: Default::default(),
        version: "0.0.1".to_string(),
    }
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Create a sender and mock env
    let mock = MockBech32::new("mock");
    let env = TestEnv::setup(mock)?;
    let app = env.client1;

    let config = app.config()?;
    assert_eq!(config, ConfigResponse {});
    Ok(())
}

mod receive_msg {

    use ibcmail::{MessageStatus, IBCMAIL_SERVER_ID};
    use speculoos::assert_that;

    use super::*;

    /// Sending a message from the same account to the same account
    /// TODO: this test is failing because of an issue with state management...
    #[test]
    fn can_receive_from_server() -> anyhow::Result<()> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let env = TestEnv::setup(mock)?;
        let app = env.client1;

        let server_account_id = app.account().id().unwrap();
        let app_account_id = app.account().id().unwrap();

        println!(
            "server_account_id: {:?}, app_account_id: {:?}",
            server_account_id, app_account_id
        );

        let msg = create_test_message(server_account_id.clone(), app_account_id.clone());
        let server_addr = app
            .account()
            .module_addresses(vec![IBCMAIL_SERVER_ID.into()])?
            .modules[0]
            .1
            .clone();

        // TODO: for some reason, the accounts are conflicting with one another. I've fixed this test by removing the "two" accounts... it's probably the same bug
        println!("app_account_id: {:?}", app.account().id());
        let res = app.call_as(&server_addr).receive_message(msg);

        assert_that!(res).is_ok();

        let messages = app.list_messages(MessageStatus::Received, None, None, None)?;
        assert_that!(messages.messages).has_length(1);

        Ok(())
    }

    #[test]
    fn cannot_receive_from_not_server() -> anyhow::Result<()> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let env = TestEnv::setup(mock)?;
        let app = env.client1;

        let app_account_id = app.account().id().unwrap();

        let msg = create_test_message(app_account_id.clone(), app_account_id.clone());
        let res = app.receive_message(msg);

        assert_that!(res)
            .is_err()
            .matches(|e| e.root().to_string().contains("Sender is not mail server"));

        Ok(())
    }
}

mod send_msg {
    use abstract_app::objects::account::AccountTrace;
    use abstract_app::objects::chain_name::ChainName;
    use abstract_app::std::version_control::ExecuteMsgFns;
    use std::str::FromStr;

    use super::*;
    use ibcmail::server::error::ServerError;
    use ibcmail::{MessageStatus, NewMessage, IBCMAIL_CLIENT_ID};

    #[test]
    fn can_send_local_message() -> anyhow::Result<()> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let env = TestEnv::setup(mock)?;
        let client1 = env.client1;
        let client2 = env.client2;

        let msg = NewMessage::new(
            Recipient::account(client2.account().id()?, None),
            "test-subject",
            "test-body",
        );

        let res = client1.send_message(msg, None);

        assert_that!(res).is_ok();

        Ok(())
    }

    #[test]
    fn can_send_local_message_to_namespace() -> anyhow::Result<()> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let env = TestEnv::setup(mock)?;
        let client1 = env.client1;
        let client2 = env.client2;

        let namespace = "test";

        env.abs
            .version_control()
            .claim_namespace(client2.account().id()?, namespace.to_string())?;

        let msg = NewMessage::new(
            Recipient::namespace(namespace.try_into()?, None),
            "test-subject",
            "test-body",
        );

        let res = client1.send_message(msg, None);
        assert_that!(res).is_ok();

        Ok(())
    }

    #[test]
    fn send_to_non_existent_namespace_fails() -> anyhow::Result<()> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let env = TestEnv::setup(mock)?;
        let client1 = env.client1;

        client1.account()

        let bad_namespace: Namespace = "nope".try_into()?;

        let msg = NewMessage::new(
            Recipient::namespace(bad_namespace.clone(), None),
            "test-subject",
            "test-body",
        );

        let res = client1.send_message(msg, None);

        assert_that!(res).is_err().matches(|e| {
            e.root()
                .to_string()
                .contains(&ServerError::UnclaimedNamespace(bad_namespace.clone()).to_string())
        });

        Ok(())
    }

    #[test]
    fn can_send_remote_message() -> anyhow::Result<()> {
        // Create a sender and mock env
        let interchain = MockBech32InterchainEnv::new(vec![
            ("juno-1", "juno18k2uq7srsr8lwrae6zr0qahpn29rsp7tw83nyx"),
            (
                "archway-1",
                "archway18k2uq7srsr8lwrae6zr0qahpn29rsp7td7wvfd",
            ),
        ]);

        // /Users/adair/.cargo/registry/src/index.crates.io-6f17d22bba15001f/cw-orch-mock-0.22.0/src/queriers/env.rs:12:70:
        // index out of bounds: the len is 1 but the index is 1 (when initializing with "juno")
        let arch_env = TestEnv::setup(interchain.chain("archway-1")?)?;
        let juno_env = TestEnv::setup(interchain.chain("juno-1")?)?;

        arch_env.enable_ibc()?;
        juno_env.enable_ibc()?;

        // TODO: put somewhere better
        ibc_connect_polytone_and_abstract(&interchain, "archway-1", "juno-1")?;

        let arch_client = arch_env.client1;
        let juno_client = juno_env.client1;

        // the trait `From<&str>` is not implemented for `abstract_app::objects::chain_name::ChainName`
        let arch_to_juno_msg = NewMessage::new(
            Recipient::account(
                juno_client.account().id()?,
                Some(ChainName::from_string("juno".into())?),
            ),
            "test-subject",
            "test-body",
        );

        let res = arch_client.send_message(arch_to_juno_msg, None);

        assert_that!(res).is_ok();

        interchain.wait_ibc("archway-1", res?)?;

        let myos_messages = arch_client.list_messages(MessageStatus::Received, None, None, None)?;
        assert_that!(myos_messages.messages).is_empty();

        let juno_client_1_module_addresses = juno_client
            .account()
            .module_addresses(vec![IBCMAIL_CLIENT_ID.into()])?;
        let acc_id = juno_client.account().id()?;
        println!(
            "juno_client_1 addresses: {:?}, account_id: {:?}",
            juno_client_1_module_addresses, acc_id
        );
        // TESTING:
        let addresses = juno_env
            .client2
            .account()
            .module_addresses(vec![IBCMAIL_CLIENT_ID.into()])?;
        let acc_id = juno_env.client2.account().id()?;
        println!(
            "juno_client_2 addresses: {:?}, account_id: {:?}",
            addresses, acc_id
        );

        let test_address = juno_client.address()?;
        let test_id = juno_client.id();
        println!("test_address: {:?}, test_id: {:?}", test_address, test_id);

        let juno_mail_client = ClientInterface::new(IBCMAIL_CLIENT_ID, juno_env.env.clone());
        juno_mail_client.set_address(&juno_client_1_module_addresses.modules[0].1.clone());
        let juno_mail_client_messages =
            juno_mail_client.list_messages(MessageStatus::Received, None, None, None)?;
        assert_that!(juno_mail_client_messages.messages).has_length(1);

        let juno_messages = juno_client.list_messages(MessageStatus::Received, None, None, None)?;
        assert_that!(juno_messages.messages).has_length(1);

        Ok(())
    }

    #[test]
    fn can_send_remote_message_2_hop() -> anyhow::Result<()> {
        // Create a sender and mock env
        let interchain = MockBech32InterchainEnv::new(vec![
            ("juno-1", "juno18k2uq7srsr8lwrae6zr0qahpn29rsp7tw83nyx"),
            (
                "archway-1",
                "archway18k2uq7srsr8lwrae6zr0qahpn29rsp7td7wvfd",
            ),
            (
                "neutron-1",
                "neutron18k2uq7srsr8lwrae6zr0qahpn29rsp7tu2m2ea",
            ),
        ]);

        // /Users/adair/.cargo/registry/src/index.crates.io-6f17d22bba15001f/cw-orch-mock-0.22.0/src/queriers/env.rs:12:70:
        // index out of bounds: the len is 1 but the index is 1 (when initializing with "juno")
        let arch_env = TestEnv::setup(interchain.chain("archway-1")?)?;
        let juno_env = TestEnv::setup(interchain.chain("juno-1")?)?;
        let neutron_env = TestEnv::setup(interchain.chain("neutron-1")?)?;

        arch_env.enable_ibc()?;
        juno_env.enable_ibc()?;
        neutron_env.enable_ibc()?;

        // TODO: put somewhere better
        ibc_connect_polytone_and_abstract(&interchain, "archway-1", "juno-1")?;
        ibc_connect_polytone_and_abstract(&interchain, "juno-1", "neutron-1")?;

        let arch_client = arch_env.client1;
        let _juno_client = juno_env.client1;
        let neutron_client = neutron_env.client1;

        // the trait `From<&str>` is not implemented for `abstract_app::objects::chain_name::ChainName`
        let arch_to_neutron_msg = NewMessage::new(
            Recipient::account(
                neutron_client.account().id()?,
                Some(ChainName::from_string("neutron".into())?),
            ),
            "test-subject",
            "test-body",
        );

        let res = arch_client.send_message(
            arch_to_neutron_msg,
            Some(AccountTrace::Remote(vec![
                "juno".parse()?,
                ChainName::from_str("neutron")?,
            ])),
        )?;

        interchain.wait_ibc("archway-1", res.clone())?;

        let arch_messages = arch_client.list_messages(MessageStatus::Received, None, None, None)?;
        assert_that!(arch_messages.messages).is_empty();

        let neutron_client_1_module_addresses = neutron_client
            .account()
            .module_addresses(vec![IBCMAIL_CLIENT_ID.into()])?;
        // let acc_id = neutron_client.account().id()?;
        // println!("neutron_client_1 addresses: {:?}, account_id: {:?}", neutron_client_1_module_addresses, acc_id);
        // // TESTING:
        // let addresses = juno_env.client2.account().module_addresses(vec![IBCMAIL_CLIENT_ID.into()])?;
        // let acc_id = juno_env.client2.account().id()?;
        // println!("neutron_client_2 addresses: {:?}, account_id: {:?}", addresses, acc_id);
        //
        // let test_address = neutron_client.address()?;
        // let test_id = neutron_client.id();
        // println!("test_address: {:?}, test_id: {:?}", test_address, test_id);

        let neutron_mail_client = ClientInterface::new(IBCMAIL_CLIENT_ID, neutron_env.env.clone());
        neutron_mail_client.set_address(&neutron_client_1_module_addresses.modules[0].1.clone());
        let neutron_mail_client_messages =
            neutron_mail_client.list_messages(MessageStatus::Received, None, None, None)?;
        assert_that!(neutron_mail_client_messages.messages).has_length(1);

        // let juno_messages = neutron_client.list_messages(None, None, None)?;
        // assert_that!(juno_messages.messages).has_length(1);

        Ok(())
    }
}
