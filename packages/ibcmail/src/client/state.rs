use cw_storage_plus::Map;

use crate::{IbcMailMessage, MessageHash, MessageStatus};

// TODO: use an indexed map in the future
pub const RECEIVED: Map<MessageHash, IbcMailMessage> = Map::new("received");
pub const SENT: Map<MessageHash, IbcMailMessage> = Map::new("sent");
pub const STATUS: Map<MessageHash, MessageStatus> = Map::new("status");