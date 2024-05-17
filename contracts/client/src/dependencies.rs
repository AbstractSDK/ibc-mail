use abstract_app::objects::dependency::StaticDependency;


use ibcmail::IBCMAIL_SERVER_ID;

pub const MAIL_SERVER_DEP: StaticDependency =
    StaticDependency::new(IBCMAIL_SERVER_ID, &[">=0.0.1"]);

#[cfg(feature = "interface")]
use abstract_app::{objects::module::ModuleInfo, std::manager::ModuleInstallConfig};

#[cfg(feature = "interface")]
impl<Chain: cw_orch::environment::CwEnv> abstract_interface::DependencyCreation
    for crate::ClientInterface<Chain>
{
    type DependenciesConfig = cosmwasm_std::Empty;

    fn dependency_install_configs(
        _configuration: Self::DependenciesConfig,
    ) -> Result<Vec<ModuleInstallConfig>, abstract_interface::AbstractInterfaceError> {
        let adapter_install_config = ModuleInstallConfig::new(
            ModuleInfo::from_id_latest(ibcmail::IBCMAIL_SERVER_ID)?,
            None,
        );

        Ok(vec![adapter_install_config])
    }
}
