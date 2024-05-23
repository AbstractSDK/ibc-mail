use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::{IbcMailMessage, MessageHash};

#[cosmwasm_schema::cw_serde]
pub struct Config {}

// TODO: use an indexeed map in the future
pub const RECEIVED: Map<MessageHash, IbcMailMessage> = Map::new("received");
pub const SENT: Map<MessageHash, IbcMailMessage> = Map::new("sent");

pub const CONFIG: Item<Config> = Item::new("config");

pub const TEST: Item<Addr> = Item::new("test");
