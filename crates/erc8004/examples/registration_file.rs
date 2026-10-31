#![allow(
    unused_crate_dependencies,
    clippy::print_stdout,
    reason = "example prints results to stdout"
)]

//! Builds a registration file offline, serializes it to JSON and packs it into a
//! `data:` URI — the same shape `resolve_agent_uri` parses back.

use alloy::primitives::U256;
use erc8004::{Registration, RegistrationFile, Service, decode_data_uri, encode_data_uri};
use serde_json::Map;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = RegistrationFile {
        services: Some(vec![Service {
            name: "web".into(),
            endpoint: "https://agent.example".into(),
            version: None,
            skills: None,
            domains: None,
            extra: Map::new(),
        }]),
        active: Some(true),
        registrations: Some(vec![Registration {
            agent_id: U256::from(42u64),
            agent_registry: "eip155:8453:0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".into(),
            extra: Map::new(),
        }]),
        supported_trust: Some(vec!["reputation".into()]),
        ..RegistrationFile::new("Example Agent", "Offline-built registration file")
    };

    let json = serde_json::to_string_pretty(&file.to_value())?;
    println!("{json}");

    let data_uri = encode_data_uri(json.as_bytes());
    println!("data URI ({} bytes): {}…", data_uri.len(), &data_uri[..64]);

    let round_trip = decode_data_uri(&data_uri)?;
    if round_trip != json.as_bytes() {
        return Err("data URI round-trip mismatch".into());
    }
    let reparsed = RegistrationFile::from_slice(&round_trip)?;
    println!(
        "round-trip parse ok ({} field(s) preserved)",
        reparsed.to_value().as_object().map_or(0, Map::len)
    );
    Ok(())
}
