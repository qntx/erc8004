#![allow(clippy::print_stdout)]
//! Register a new AI agent on-chain via the ERC-8004 Identity Registry.
//!
//! Usage:
//!   cargo run --example `register_agent`
//!
//! **Requirements:** A funded wallet on Base Sepolia testnet.
//! Set the `PRIVATE_KEY` environment variable before running.

use alloy::{
    network::EthereumWallet, providers::ProviderBuilder, signers::local::PrivateKeySigner,
};
use erc8004::{Erc8004, Network};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load signer from environment.
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let wallet = EthereumWallet::from(signer);

    // Build a provider with the signer attached (Base Sepolia testnet).
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http("https://sepolia.base.org".parse()?);

    let client = Erc8004::new(provider).with_network(Network::BaseSepolia);

    // Register an agent with a URI pointing to its registration file.
    let agent_uri = "https://weather-bot.example.com/erc8004.json";
    let agent_id = client.identity()?.register_with_uri(agent_uri).await?;
    println!("Registered agent #{agent_id}");

    // Verify the URI was stored correctly.
    let stored_uri = client.identity()?.token_uri(agent_id).await?;
    println!("Stored URI: {stored_uri}");

    Ok(())
}
