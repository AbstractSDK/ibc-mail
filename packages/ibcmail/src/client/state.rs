use cw_storage_plus::Map;

use crate::{DeliveryStatus, Header, MailMessage, MessageHash, ReceivedMessage};

// TODO: use an indexed map in the future
pub const RECEIVED: Map<MessageHash, ReceivedMessage> = Map::new("received");
pub const SENT: Map<MessageHash, (MailMessage, Header)> = Map::new("sent");
pub const SENT_STATUS: Map<MessageHash, DeliveryStatus> = Map::new("status");

/// Set of features supported by the client
pub const FEATURES: Map<String, bool> = Map::new("features");
