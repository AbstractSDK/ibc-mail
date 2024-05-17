use cw_orch::environment::{ChainInfo, ChainKind, NetworkInfo};

pub const MYOS_NETWORK: NetworkInfo = NetworkInfo {
    chain_name: "celeswasm",
    pub_address_prefix: "wasm",
    coin_type: 118u32,
};

/// Archway Docs: <https://docs.archway.io/resources/networks>
/// Parameters: <https://testnet.mintscan.io/archway-testnet/parameters>
pub const MYOS: ChainInfo = ChainInfo {
    kind: ChainKind::Testnet,
    chain_id: "celeswasm",
    gas_denom: "uwasm",
    gas_price: 0.025,
    grpc_urls: &["http://ec2-100-25-222-131.compute-1.amazonaws.com:9290"],
    network_info: MYOS_NETWORK,
    lcd_url: None,
    fcd_url: None,
};