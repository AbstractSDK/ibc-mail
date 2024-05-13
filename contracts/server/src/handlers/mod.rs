pub mod execute;
pub mod instantiate;
pub mod module_ibc;
pub mod query;

pub use crate::handlers::{
    execute::execute_handler, instantiate::instantiate_handler, module_ibc::module_ibc_handler,
    query::query_handler,
};
