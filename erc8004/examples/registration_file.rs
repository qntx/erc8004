#![allow(clippy::print_stdout)]
//! Build and serialize an ERC-8004 agent registration file (pure offline).
//!
//! Usage:
//!   cargo run --example `registration_file`
//!
//! This example demonstrates how to construct the off-chain JSON registration
//! file that an agent publishes at its `agentURI`. No RPC connection is needed.

use erc8004::types::{RegistrationFile, ServiceEndpoint};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a registration file using the typed builder.
    let mut reg = RegistrationFile::new(
        "WeatherBot",
        "An AI agent that provides real-time weather forecasts.",
    );

    // Advertise an A2A endpoint.
    reg.services.push(ServiceEndpoint {
        name: "A2A".to_owned(),
        endpoint: "https://weather-bot.example.com/.well-known/agent.json".to_owned(),
        version: Some("0.2".to_owned()),
        skills: None,
        domains: None,
    });

    // Advertise an MCP endpoint.
    reg.services.push(ServiceEndpoint {
        name: "MCP".to_owned(),
        endpoint: "https://weather-bot.example.com/mcp".to_owned(),
        version: Some("2025-03-26".to_owned()),
        skills: None,
        domains: None,
    });

    reg.x402_support = true;

    // Serialize to JSON.
    let json = reg.to_json()?;
    println!("{json}");

    // Round-trip: deserialize back and verify.
    let parsed = RegistrationFile::from_json(&json)?;
    println!(
        "\nRound-trip OK: name={:?}, {} service(s)",
        parsed.name,
        parsed.services.len()
    );

    Ok(())
}
