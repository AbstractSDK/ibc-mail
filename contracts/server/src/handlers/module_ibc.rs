use abstract_sdk::AbstractResponse;
use abstract_std::ibc::ModuleIbcMsg;
use abstract_std::objects::account::AccountTrace;
use abstract_std::objects::AccountId;
use abstract_std::objects::chain_name::ChainName;
use cosmwasm_std::{DepsMut, Env, from_json, MessageInfo, Response};
use ibcmail::{IBCMAIL_SERVER_ID, Message, Recipient, Sender};
use ibcmail::server::error::ServerError;
use ibcmail::server::msg::{ExecuteMsg, ServerExecuteMsg, ServerIbcMessage};
use ibcmail::server::ServerAdapter;
use crate::contract::{execute, ServerResult};

pub fn module_ibc_handler(
    deps: DepsMut,
    _env: Env,
    app: ServerAdapter,
    ibc_msg: ModuleIbcMsg,
) -> ServerResult {
    println!("module_ibc_handler 1 : {:?}", ibc_msg);
    // First check that we received the message from the server
    if ibc_msg.source_module.id().ne(IBCMAIL_SERVER_ID) {
        println!("UnauthorizedIbcModule: {:?}", ibc_msg.source_module.clone());
        return Err(ServerError::UnauthorizedIbcModule(ibc_msg.source_module.clone()));
    };

    let server_msg: ServerIbcMessage = from_json(&ibc_msg.msg)?;

    println!("parsed_msg: {:?}", server_msg);

    match server_msg {
        ServerIbcMessage::RouteMessage(msg) => {
            // Update the sender to the proper remote account?
            let updated_sender = match &msg.sender {
                // Update the sender
                Sender::Account { id, route: mut chain } => {
                    chain.push_chain(ibc_msg.client_chain);
                    Sender::Account { id: id.clone(), chain: route }
                }
                _ => msg.sender
            };

            let updated_recipient = match &msg.recipient {
                // Update the recipient
                Recipient::Account { id, route: mut chain } => {
                    chain.verify()?;
                    chain = match chain {
                        // Unreachable because we just sent it to this chain
                        AccountTrace::Local => unreachable!(),
                        AccountTrace::Remote(trace) => {
                            let mut new_trace = trace.clone();
                            let _popped = new_trace.remove(0);
                            // TODO: could verify trace is correct

                            if new_trace.is_empty() {
                                AccountTrace::Local
                            } else {
                                new_trace.into()
                            }
                        }
                    };
                    Recipient::Account { id: id.clone(), chain: route }
                }
                _ => msg.recipient
            };

            let updated = Message {
                sender: updated_sender,
                recipient: updated_recipient,
                ..msg
            };
            crate::handlers::execute::route_msg(deps, MessageInfo { funds: vec![], sender: _env.contract.address }, updated, app)
        }
        _ => Err(ServerError::UnauthorizedIbcMessage {})
    }
}