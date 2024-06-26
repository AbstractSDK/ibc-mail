use std::{env::current_dir, fs::create_dir_all};

use cosmwasm_schema::remove_schemas;
use ibcmail_server::contract::Adapter;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    #[cfg(feature = "schema")]
    Adapter::export_schema(&out_dir);
}
