pub mod execute;
pub mod migrate;
pub mod query;
pub mod instantiate;

pub use crate::handlers::{
    execute::execute_handler, migrate::migrate_handler, query::query_handler, instantiate::instantiate_handler
};
