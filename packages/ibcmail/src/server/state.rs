use crate::{Header, MessageHash, ServerMetadata};
use abstract_app::objects::TruncatedChainId;
use cw_storage_plus::{Item, Map};

pub const AWAITING: Map<&MessageHash, TruncatedChainId> = Map::new("awaiting");
pub const AWAITING_DELIVERY: Item<Vec<(Header, ServerMetadata)>> = Item::new("awaiting_delivery");
