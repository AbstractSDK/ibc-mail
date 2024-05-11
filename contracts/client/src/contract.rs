use abstract_app::objects::module::{ModuleInfo, ModuleVersion};
use abstract_app::std::manager::ModuleInstallConfig;
use cosmwasm_std::Response;

pub use ibcmail::client::ClientApp as App;
use ibcmail::IBCMAIL_CLIENT;

use crate::{APP_VERSION, error::ClientError, handlers};
use crate::dependencies::{MAIL_SERVER_DEP};

/// The type of the result returned by your client's entry points.
pub type ClientResult<T = Response> = Result<T, ClientError>;

const APP: App = App::new(IBCMAIL_CLIENT, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_module_ibc(handlers::module_ibc_handler)
    .with_dependencies(&[
        MAIL_SERVER_DEP,
    ]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, App);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(APP, App, ClientInterface);

#[cfg(feature = "interface")]
impl<Chain: cw_orch::environment::CwEnv> abstract_interface::DependencyCreation
for crate::ClientInterface<Chain>
{
    type DependenciesConfig = cosmwasm_std::Empty;

    fn dependency_install_configs(
        _configuration: Self::DependenciesConfig,
    ) -> Result<
        Vec<ModuleInstallConfig>,
        abstract_interface::AbstractInterfaceError,
    > {
        let adapter_install_config = ModuleInstallConfig::new(
            ModuleInfo::from_id(
                ibcmail::IBCMAIL_SERVER_ID,
                APP_VERSION.into()
            )?,
            None,
        );

        Ok(
            vec![adapter_install_config],
            )
    }
}