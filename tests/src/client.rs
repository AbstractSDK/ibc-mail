use abstract_app::objects::{namespace::Namespace, AccountId};
use abstract_client::{AbstractClient, Application};
use cw_orch::{anyhow, prelude::*};
use speculoos::prelude::*;

// Use prelude to get all the necessary imports
use client::{contract::interface::ClientInterface, msg::ClientInstantiateMsg, *};
use ibcmail::{
    server::msg::ServerInstantiateMsg, Header, IbcMailMessage, Message, Recipient, Route, Sender,
    IBCMAIL_NAMESPACE, IBCMAIL_SERVER_ID,
};
use server::ServerInterface;

struct TestEnv<Env: CwEnv> {
    env: Env,
    abs: AbstractClient<Env>,
    client1: Application<Env, ClientInterface<Env>>,
    client2: Application<Env, ClientInterface<Env>>,
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

        let acc = abs_client
            .account_builder()
            .install_on_sub_account(false)
            .build()?;

        let app = acc.install_app_with_dependencies::<ClientInterface<_>>(
            &ClientInstantiateMsg {},
            Empty {},
            &[],
        )?;
        app.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;
        // acc.install_adapter::<ServerInterface<_>>(&[])?;

        let acc2 = abs_client
            .account_builder()
            .install_on_sub_account(false)
            .build()?;
        let app2 = acc2.install_app_with_dependencies::<ClientInterface<_>>(
            &ClientInstantiateMsg {},
            Empty {},
            &[],
        )?;
        // acc2.install_adapter::<ServerInterface<_>>(&[])?;
        app2.authorize_on_adapters(&[IBCMAIL_SERVER_ID])?;

        Ok(TestEnv {
            env,
            abs: abs_client,
            client1: app,
            client2: app2,
        })
    }
}

fn create_server_test_msg(from: AccountId, to: AccountId) -> IbcMailMessage {
    IbcMailMessage {
        id: "test-id".to_string(),
        sender: Sender::account(from.clone(), None),
        recipient: Recipient::account(to.clone(), None),
        message: Message {
            subject: "test-subject".to_string(),
            body: "test-body".to_string(),
        },
        timestamp: Default::default(),
        version: "0.0.1".to_string(),
    }
}

fn temp_ibc_mail_msg_to_header(msg: IbcMailMessage, route: Route) -> Header {
    Header {
        route,
        recipient: msg.recipient.clone(),
        id: msg.id.clone(),
        version: msg.version.clone(),
        sender: msg.sender.clone(),
        timestamp: msg.timestamp,
    }
}

mod receive_msg {
    use speculoos::assert_that;

    use ibcmail::{MessageKind, IBCMAIL_SERVER_ID};

    use super::*;

    /// Sending a message from the same account to the same account
    /// TODO: this test is failing because of an issue with state management...
    // #[test]
    fn _can_receive_from_server() -> anyhow::Result<()> {
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

        let msg = create_server_test_msg(server_account_id.clone(), app_account_id.clone());
        let server_addr = app
            .account()
            .module_addresses(vec![IBCMAIL_SERVER_ID.into()])?
            .modules[0]
            .1
            .clone();

        println!("app_account_id: {:?}", app.account().id());
        let res = app.call_as(&server_addr).receive_message(
            temp_ibc_mail_msg_to_header(msg.clone(), Route::Local),
            msg.clone(),
        );

        assert_that!(res).is_ok();

        let messages = app.list_messages(MessageKind::Received, None, None, None)?;
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

        let msg = create_server_test_msg(app_account_id.clone(), app_account_id.clone());
        let res = app.receive_message(temp_ibc_mail_msg_to_header(msg.clone(), Route::Local), msg);

        assert_that!(res)
            .is_err()
            .matches(|e| e.root().to_string().contains("Sender is not mail server"));

        Ok(())
    }
}

mod send_msg {
    use std::str::FromStr;

    use abstract_app::objects::TruncatedChainId;
    use abstract_app::{objects::account::AccountTrace, std::version_control::ExecuteMsgFns};
    use abstract_cw_orch_polytone::PolytoneConnection;
    use abstract_interface::Abstract;
    use cw_orch_interchain::{InterchainEnv, MockBech32InterchainEnv};

    use ibcmail::{server::error::ServerError, Message, MessageKind, Route, IBCMAIL_CLIENT_ID};

    use super::*;

    #[test]
    fn can_send_local_message() -> anyhow::Result<()> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let env = TestEnv::setup(mock)?;
        let client1 = env.client1;
        let client2 = env.client2;

        let recipient = Recipient::account(client2.account().id()?, None);
        let msg = Message::new("test-subject", "test-body");

        let res = client1.send_message(msg, recipient, None);

        assert_that!(res).is_ok();

        Ok(())
    }

    #[test]
    fn local_message_gets_delivery_result() -> anyhow::Result<()> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let env = TestEnv::setup(mock)?;
        let client1 = env.client1;
        let client2 = env.client2;

        let recipient = Recipient::account(client2.account().id()?, None);
        let msg = Message::new("test-subject", "test-body");

        let res = client1.send_message(msg, recipient, None);
        assert_that!(res).is_ok();

        let received_messages = client2
            .list_messages(MessageKind::Received, None, None, None)?
            .messages;
        assert_that!(received_messages).has_length(1);

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

        let msg = Message::new("test-subject", "test-body");

        let res =
            client1.send_message(msg, Recipient::namespace(namespace.try_into()?, None), None);
        assert_that!(res).is_ok();

        Ok(())
    }

    #[test]
    fn send_to_non_existent_namespace_fails() -> anyhow::Result<()> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let env = TestEnv::setup(mock)?;
        let client1 = env.client1;

        let bad_namespace: Namespace = "nope".try_into()?;

        let msg = Message::new("test-subject", "test-body");

        let res =
            client1.send_message(msg, Recipient::namespace(bad_namespace.clone(), None), None);

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
        let interchain =
            MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("archway-1", "archway")]);

        let arch_env = TestEnv::setup(interchain.get_chain("archway-1")?)?;
        let juno_env = TestEnv::setup(interchain.get_chain("juno-1")?)?;

        arch_env.abs.connect_to(&juno_env.abs, &interchain)?;

        let arch_client = arch_env.client1;
        let juno_client = juno_env.client1;

        // the trait `From<&str>` is not implemented for `abstract_app::objects::chain_name::TruncatedChainId`
        let arch_to_juno_msg = Message::new("test-subject", "test-body");

        let res = arch_client.send_message(
            arch_to_juno_msg,
            Recipient::account(
                juno_client.account().id()?,
                Some(TruncatedChainId::from_string("juno".into())?),
            ),
            None,
        );

        assert_that!(res).is_ok();

        interchain.await_and_check_packets("archway-1", res?)?;

        let arch_messages = arch_client.list_messages(MessageKind::Received, None, None, None)?;
        assert_that!(arch_messages.messages).is_empty();

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
            juno_mail_client.list_messages(MessageKind::Received, None, None, None)?;
        assert_that!(juno_mail_client_messages.messages).has_length(1);

        let juno_messages = juno_client.list_messages(MessageKind::Received, None, None, None)?;
        assert_that!(juno_messages.messages).has_length(1);

        // Sanity check messages method
        let juno_message_id = juno_messages.messages.first().cloned().unwrap().id;
        let juno_message = juno_client.messages(vec![juno_message_id], MessageKind::Received)?;
        assert_that!(juno_message.messages).has_length(1);

        Ok(())
    }

    #[test]
    fn send_remote_message_1_hop_account_dne_updates_status_to_failed() -> anyhow::Result<()> {
        // Create a sender and mock env
        let interchain =
            MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("archway-1", "archway")]);

        let arch_env = TestEnv::setup(interchain.get_chain("archway-1")?)?;
        let juno_env = TestEnv::setup(interchain.get_chain("juno-1")?)?;

        arch_env.abs.connect_to(&juno_env.abs, &interchain)?;

        let arch_client = arch_env.client1;
        let juno_client = juno_env.client1;

        // the trait `From<&str>` is not implemented for `abstract_app::objects::chain_name::TruncatedChainId`
        let arch_to_juno_msg = Message::new("test-subject", "test-body");

        let res = arch_client.send_message(
            arch_to_juno_msg,
            Recipient::account(
                AccountId::local(420),
                Some(TruncatedChainId::from_string("juno".into())?),
            ),
            Some(Route::Remote(vec![TruncatedChainId::from_string(
                "juno".into(),
            )?])),
        );

        assert_that!(res).is_ok();

        let server = ServerInterface::new(IBCMAIL_SERVER_ID, arch_env.env.clone());
        println!("server: {:?}", server.address()?);
        let abstr = Abstract::new(arch_env.env.clone());
        println!("ibc_host: {:?}", abstr.ibc.host.address()?);
        let poly = PolytoneConnection::load_from(arch_env.env.clone(), juno_env.env.clone());
        println!("poly_note: {:?}", poly.note.address()?);

        let packets = interchain.await_packets("archway-1", res?)?;

        assert_that!(
            arch_client
                .list_messages(MessageKind::Received, None, None, None)?
                .messages
        )
        .is_empty();
        assert_that!(
            juno_client
                .list_messages(MessageKind::Received, None, None, None)?
                .messages
        )
        .is_empty();

        // interchain.await_packets("archway-1", res?)?;
        // println!("packets: {:?}", packets);

        Ok(())
    }

    #[test]
    fn can_send_remote_message_2_hop() -> anyhow::Result<()> {
        // Create a sender and mock env
        let interchain = MockBech32InterchainEnv::new(vec![
            ("juno-1", "juno"),
            ("archway-1", "archway"),
            ("neutron-1", "neutron"),
        ]);

        // /Users/adair/.cargo/registry/src/index.crates.io-6f17d22bba15001f/cw-orch-mock-0.22.0/src/queriers/env.rs:12:70:
        // index out of bounds: the len is 1 but the index is 1 (when initializing with "juno")
        let arch_env = TestEnv::setup(interchain.get_chain("archway-1")?)?;
        let juno_env = TestEnv::setup(interchain.get_chain("juno-1")?)?;
        let neutron_env = TestEnv::setup(interchain.get_chain("neutron-1")?)?;

        arch_env.abs.connect_to(&juno_env.abs, &interchain)?;
        juno_env.abs.connect_to(&neutron_env.abs, &interchain)?;

        // ibc_abstract_setup(&interchain, "archway-1", "juno-1")?;
        // ibc_abstract_setup(&interchain, "juno-1", "neutron-1")?;

        let arch_client = arch_env.client1;
        let _juno_client = juno_env.client1;
        let neutron_client = neutron_env.client1;

        // the trait `From<&str>` is not implemented for `abstract_app::objects::chain_name::TruncatedChainId`
        let arch_to_neutron_msg = Message::new("test-subject", "test-body");

        let res = arch_client.send_message(
            arch_to_neutron_msg,
            Recipient::account(
                neutron_client.account().id()?,
                Some(TruncatedChainId::from_string("neutron".into())?),
            ),
            Some(AccountTrace::Remote(vec![
                "juno".parse()?,
                TruncatedChainId::from_str("neutron")?,
            ])),
        )?;

        interchain.await_and_check_packets("archway-1", res.clone())?;

        let arch_messages = arch_client.list_messages(MessageKind::Received, None, None, None)?;
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
            neutron_mail_client.list_messages(MessageKind::Received, None, None, None)?;
        assert_that!(neutron_mail_client_messages.messages).has_length(1);

        // let juno_messages = neutron_client.list_messages(None, None, None)?;
        // assert_that!(juno_messages.messages).has_length(1);

        Ok(())
    }
}
