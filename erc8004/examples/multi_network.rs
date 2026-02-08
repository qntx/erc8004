#![allow(clippy::print_stdout)]
//! Demonstrate using the ERC-8004 SDK across multiple networks.
//!
//! Usage:
//!   cargo run --example `multi_network`
//!
//! This example queries the Identity Registry version on both Ethereum mainnet
//! and Base mainnet, showing how the same SDK types work across chains.

use alloy::providers::ProviderBuilder;
use erc8004::{Erc8004, Network};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let networks = [
        (
            "Ethereum",
            Network::EthereumMainnet,
            "https://eth.llamarpc.com",
        ),
        ("Base", Network::BaseMainnet, "https://mainnet.base.org"),
    ];

    for (name, network, rpc) in networks {
        let provider = ProviderBuilder::new().connect_http(rpc.parse()?);
        let client = Erc8004::new(provider).with_network(network);

        let version = client.identity()?.get_version().await?;
        println!(
            "[{name}] chain_id={}, version={version}",
            network.chain_id()
        );
    }

    Ok(())
}
