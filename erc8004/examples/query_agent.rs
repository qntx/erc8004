#![allow(clippy::print_stdout)]
//! Query an agent's on-chain identity from the ERC-8004 Identity Registry.
//!
//! Usage:
//!   cargo run --example `query_agent`
//!
//! This example connects to Ethereum mainnet via a public RPC endpoint and
//! reads basic identity information for a given agent ID.

use alloy::{primitives::U256, providers::ProviderBuilder};
use erc8004::{Erc8004, Network};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Ethereum mainnet via a public RPC.
    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse()?);

    // Initialize the ERC-8004 client with mainnet addresses.
    let client = Erc8004::new(provider).with_network(Network::EthereumMainnet);

    // Query the Identity Registry contract version.
    let identity = client.identity()?;
    let version = identity.get_version().await?;
    println!("Identity Registry version: {version}");

    // Look up agent #1 (if it exists).
    let agent_id = U256::from(1);
    let owner = identity.owner_of(agent_id).await?;
    let uri = identity.token_uri(agent_id).await?;
    let wallet = identity.get_agent_wallet(agent_id).await?;

    println!("Agent #{agent_id}");
    println!("  Owner:  {owner}");
    println!("  URI:    {uri}");
    println!("  Wallet: {wallet}");

    Ok(())
}
