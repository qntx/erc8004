#![allow(
    unused_crate_dependencies,
    clippy::print_stdout,
    reason = "example prints results to stdout"
)]

//! Reads an agent from the identity registry over a public RPC URL and resolves
//! its agentURI registration file.
//!
//! `RPC_URL` defaults to a public Base endpoint; `AGENT_ID` defaults to `1`.

use alloy::{primitives::U256, providers::ProviderBuilder, transports::http::reqwest::Url};
use erc8004::{Erc8004, Resolver, ResolverOptions, deployments};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url: Url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://mainnet.base.org".into())
        .parse()?;
    let agent_id = std::env::var("AGENT_ID")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(1);

    let provider = ProviderBuilder::new().connect_http(rpc_url);
    let erc = Erc8004::new(provider, deployments::BASE);

    let agent = erc.identity().get_agent(U256::from(agent_id)).await?;
    println!("agent #{agent_id}");
    println!("  owner: {}", agent.owner);
    println!("  agentURI: {}", agent.agent_uri);
    match agent.agent_wallet {
        Some(wallet) => println!("  agentWallet: {wallet}"),
        None => println!("  agentWallet: (unset)"),
    }

    if agent.agent_uri.is_empty() {
        println!("no agentURI set — nothing to resolve");
        return Ok(());
    }
    let resolver = Resolver::new(ResolverOptions::default())?;
    let file = resolver.resolve_agent_uri(&agent.agent_uri).await?;
    println!("registration file:");
    println!("{}", serde_json::to_string_pretty(&file.to_value())?);
    Ok(())
}
