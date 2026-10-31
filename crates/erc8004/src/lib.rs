//! # ERC-8004: Trustless Agents Rust SDK
//!
//! A type-safe, ergonomic Rust SDK for interacting with
//! [ERC-8004](https://eips.ethereum.org/EIPS/eip-8004) on-chain registries.
//!
//! ERC-8004 enables **discovery, reputation, and validation** for AI agents
//! across organizational boundaries without pre-existing trust.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use alloy::providers::ProviderBuilder;
//! use erc8004::{Erc8004, Network};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 1. Create an alloy provider (any transport works: HTTP, WS, IPC)
//! let provider = ProviderBuilder::new()
//!     .connect_http("https://eth.llamarpc.com".parse()?);
//!
//! // 2. Wrap it with the ERC-8004 client
//! let client = Erc8004::new(provider)
//!     .with_network(Network::EthereumMainnet);
//!
//! // 3. Interact with the registries
//! let version = client.identity()?.get_version().await?;
//! println!("Contract version: {version}");
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The SDK is designed around the alloy provider abstraction:
//!
//! - **[`Erc8004`]** — The top-level client, generic over `P: Provider`.
//!   Accepts any alloy provider the user has already configured.
//! - **[`Identity`](identity::Identity)** — Identity Registry (ERC-721)
//!   operations: register agents, manage URIs, wallets, and metadata.
//! - **[`Reputation`](reputation::Reputation)** — Reputation Registry
//!   operations: submit/revoke feedback, read summaries.
//! - **[`Validation`](validation::Validation)** — Validation Registry
//!   operations: request/respond to validation, query status.
//! - **[`Network`]** — Pre-configured network addresses for known deployments.
//! - **[`types`]** — Off-chain JSON types (registration files, feedback, etc.).

pub mod client;
pub mod contracts;
pub mod error;
pub mod identity;
pub mod networks;
pub mod reputation;
pub mod types;
pub mod validation;

// Re-export primary public API at crate root.
pub use client::Erc8004;
pub use error::{Erc8004Error, Result};
pub use networks::Network;
#[cfg(test)]
use tokio as _;
