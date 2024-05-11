    use cw_storage_plus::{Item, Map};
    use crate::{Message, MessageId};

    #[cosmwasm_schema::cw_serde]
    pub struct Config {}

    // TODO: use an indexeed map in the future
    pub const RECEIVED: Map<MessageId, Message> = Map::new("received");



    pub const CONFIG: Item<Config> = Item::new("config");
    pub const COUNT: Item<i32> = Item::new("count");
