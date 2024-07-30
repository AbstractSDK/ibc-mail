use abstract_app::objects::TruncatedChainId;
use cw_storage_plus::Map;
use crate::MessageHash;

pub const AWAITING: Map<&MessageHash, TruncatedChainId> = Map::new("awaiting");