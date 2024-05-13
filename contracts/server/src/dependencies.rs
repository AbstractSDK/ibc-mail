use abstract_adapter::objects::dependency::StaticDependency;
use abstract_std::IBC_CLIENT;

pub const IBC_CLIENT_DEP: StaticDependency = StaticDependency::new(IBC_CLIENT, &[">=0.22.0"]);