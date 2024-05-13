use crate::{Message, MessageId};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cosmwasm_schema::cw_serde]
pub struct Config {}

// TODO: use an indexeed map in the future
pub const RECEIVED: Map<MessageId, Message> = Map::new("received");
pub const SENT: Map<MessageId, Message> = Map::new("sent");

pub const CONFIG: Item<Config> = Item::new("config");

pub const TEST: Item<Addr> = Item::new("test");
