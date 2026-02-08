#![allow(clippy::print_stdout)]
//! Query an agent's reputation summary from the ERC-8004 Reputation Registry.
//!
//! Usage:
//!   cargo run --example `reputation_summary`
//!
//! Connects to Ethereum mainnet and reads reputation data for a given agent.

use alloy::{primitives::U256, providers::ProviderBuilder};
use erc8004::{Erc8004, Network};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse()?);

    let client = Erc8004::new(provider).with_network(Network::EthereumMainnet);
    let reputation = client.reputation()?;

    // Print the Reputation Registry version.
    let version = reputation.get_version().await?;
    println!("Reputation Registry version: {version}");

    // List all clients who left feedback for agent #1.
    let agent_id = U256::from(1);
    let clients = reputation.get_clients(agent_id).await?;
    println!("Agent #{agent_id} has {} feedback client(s)", clients.len());

    if clients.is_empty() {
        println!("No feedback yet — nothing to summarize.");
        return Ok(());
    }

    // Get the aggregated summary (filtering by known clients to avoid Sybil).
    let summary = reputation
        .get_summary(agent_id, clients.clone(), "", "")
        .await?;

    println!(
        "Summary: count={}, value={} (decimals={})",
        summary.count, summary.summary_value, summary.summary_value_decimals,
    );

    // Read the first feedback entry from the first client.
    let feedback = reputation.read_feedback(agent_id, clients[0], 0).await?;
    println!(
        "First feedback: value={}, tag1={:?}, tag2={:?}, revoked={}",
        feedback.value, feedback.tag1, feedback.tag2, feedback.is_revoked,
    );

    Ok(())
}
