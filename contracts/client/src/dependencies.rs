use abstract_app::objects::dependency::StaticDependency;
use ibcmail::IBCMAIL_SERVER_ID;

pub const MAIL_SERVER_DEP: StaticDependency = StaticDependency::new(IBCMAIL_SERVER_ID, &[">=0.0.1"]);