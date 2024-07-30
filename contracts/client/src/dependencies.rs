use abstract_app::objects::dependency::StaticDependency;
#[cfg(feature = "interface")]
use abstract_app::{objects::module::ModuleInfo, std::manager::ModuleInstallConfig};
use ibcmail::IBCMAIL_SERVER_ID;

pub const MAIL_SERVER_DEP: StaticDependency =
    StaticDependency::new(IBCMAIL_SERVER_ID, &[">=0.0.1"]);

#[cfg(feature = "interface")]
impl<Chain: cw_orch::environment::CwEnv> abstract_app::abstract_interface::DependencyCreation
    for crate::ClientInterface<Chain>
{
    type DependenciesConfig = cosmwasm_std::Empty;

    fn dependency_install_configs(
        _configuration: Self::DependenciesConfig,
    ) -> Result<Vec<ModuleInstallConfig>, abstract_app::abstract_interface::AbstractInterfaceError>
    {
        let adapter_install_config = ModuleInstallConfig::new(
            ModuleInfo::from_id(ibcmail::IBCMAIL_SERVER_ID, crate::APP_VERSION.into())?,
            None,
        );

        // The IBC client is depended upon by the server
        let ibc_client = ModuleInstallConfig::new(
            ModuleInfo::from_id_latest("abstract:ibc-client")?,
            None,
        );

        Ok(vec![adapter_install_config, ibc_client])
    }
}
