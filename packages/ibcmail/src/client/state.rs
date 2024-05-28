use cw_storage_plus::Map;

use crate::{IbcMailMessage, MessageHash};

// TODO: use an indexed map in the future
pub const RECEIVED: Map<MessageHash, IbcMailMessage> = Map::new("received");
pub const SENT: Map<MessageHash, IbcMailMessage> = Map::new("sent");
