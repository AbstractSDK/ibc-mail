use abstract_app::objects::dependency::StaticDependency;
use ibcmail::IBCMAIL_SERVER;

pub const MAIL_SERVER_DEP: StaticDependency = StaticDependency::new(IBCMAIL_SERVER, &[">0.1.0"]);