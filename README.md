# erc8004

[![CI][ci-badge]][ci-url]
[![crates.io][crate-badge]][crate-url]
[![docs.rs][doc-badge]][doc-url]
[![License][license-badge]][license-url]
[![Rust][rust-badge]][rust-url]

[ci-badge]: https://github.com/qntx/erc8004/actions/workflows/rust.yml/badge.svg
[ci-url]: https://github.com/qntx/erc8004/actions/workflows/rust.yml
[crate-badge]: https://img.shields.io/crates/v/erc8004.svg
[crate-url]: https://crates.io/crates/erc8004
[doc-badge]: https://img.shields.io/docsrs/erc8004.svg
[doc-url]: https://docs.rs/erc8004
[license-badge]: https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg
[license-url]: LICENSE-MIT
[rust-badge]: https://img.shields.io/badge/rust-edition%202024-orange.svg
[rust-url]: https://doc.rust-lang.org/edition-guide/

**Type-safe Rust SDK for the [ERC-8004](https://eips.ethereum.org/EIPS/eip-8004) Trustless Agents standard — on-chain identity, reputation, and validation registries for AI agents.**

ERC-8004 enables **discovery, reputation, and validation** of AI agents across organizational boundaries without pre-existing trust. This SDK provides ergonomic, alloy-native bindings for all three registries, with 18 pre-configured network deployments (CREATE2 deterministic addresses) and full off-chain type support (registration files, service endpoints, feedback).

See [Security](SECURITY.md) before using in production.

## Quick Start

### Query an Agent (Read-Only)

```rust
use alloy::{primitives::U256, providers::ProviderBuilder};
use erc8004::{Erc8004, Network};

let provider = ProviderBuilder::new()
    .connect_http("https://eth.llamarpc.com".parse()?);

let client = Erc8004::new(provider)
    .with_network(Network::EthereumMainnet);

// Identity Registry — ERC-721 agent identity
let identity = client.identity()?;
let owner  = identity.owner_of(U256::from(1)).await?;
let uri    = identity.token_uri(U256::from(1)).await?;
let wallet = identity.get_agent_wallet(U256::from(1)).await?;
```

### Register an Agent (Write)

```rust
use alloy::{network::EthereumWallet, providers::ProviderBuilder, signers::local::PrivateKeySigner};
use erc8004::{Erc8004, Network};

let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
let wallet = EthereumWallet::from(signer);

let provider = ProviderBuilder::new()
    .wallet(wallet)
    .connect_http("https://sepolia.base.org".parse()?);

let client = Erc8004::new(provider)
    .with_network(Network::BaseSepolia);

let agent_id = client.identity()?
    .register_with_uri("https://my-agent.example.com/erc8004.json")
    .await?;
```

### Build a Registration File (Offline)

```rust
use erc8004::types::{RegistrationFile, ServiceEndpoint};

let mut reg = RegistrationFile::new(
    "WeatherBot",
    "An AI agent that provides real-time weather forecasts.",
);

reg.services.push(ServiceEndpoint {
    name: "A2A".to_owned(),
    endpoint: "https://weather-bot.example.com/.well-known/agent.json".to_owned(),
    version: Some("0.2".to_owned()),
    skills: None,
    domains: None,
});

reg.x402_support = true;
let json = reg.to_json()?;
```

## Architecture

| Module | Description |
| --- | --- |
| **[`Erc8004`](erc8004/src/client.rs)** | Top-level client — generic over `P: Provider`, builder pattern for network / address configuration |
| **[`Identity`](erc8004/src/identity.rs)** | Identity Registry (ERC-721) — register agents, manage URIs, wallets, metadata, EIP-712 signatures |
| **[`Reputation`](erc8004/src/reputation.rs)** | Reputation Registry — submit / revoke feedback, read aggregated summaries, list clients |
| **[`Validation`](erc8004/src/validation.rs)** | Validation Registry — request / respond to validation, query status and summaries |
| **[`Network`](erc8004/src/networks.rs)** | 18 pre-configured deployments (10 mainnet + 8 testnet) with CREATE2 deterministic addresses |
| **[`types`](erc8004/src/types.rs)** | Off-chain JSON types — `RegistrationFile`, `ServiceEndpoint`, `Feedback`, `ReputationSummary` |
| **[`contracts`](erc8004/src/contracts.rs)** | Inline Solidity bindings (`sol!` macro) — alloy-recommended, preserves full type information |

## Supported Networks

Contracts are deployed via CREATE2, so all mainnets share the same addresses and all testnets share the same addresses.

| Network | Type | Chain ID |
| --- | --- | --- |
| Ethereum | mainnet | 1 |
| Base | mainnet | 8453 |
| Polygon | mainnet | 137 |
| Arbitrum One | mainnet | 42161 |
| Celo | mainnet | 42220 |
| Gnosis | mainnet | 100 |
| Scroll | mainnet | 534352 |
| Taiko (Alethia) | mainnet | 167000 |
| Monad | mainnet | 143 |
| BNB Smart Chain | mainnet | 56 |
| Ethereum Sepolia | testnet | 11155111 |
| Base Sepolia | testnet | 84532 |
| Polygon Amoy | testnet | 80002 |
| Arbitrum Sepolia | testnet | 421614 |
| Celo Alfajores | testnet | 44787 |
| Scroll Sepolia | testnet | 534351 |
| Monad Testnet | testnet | 10143 |
| BNB Smart Chain Testnet | testnet | 97 |

## Design

- **Zero `async_trait`** — pure RPITIT, no trait-object overhead
- **Inline Solidity bindings** — `sol!` macro preserves struct names, enums, and visibility; no JSON ABI files
- **Provider-generic** — works with any alloy transport (HTTP, WebSocket, IPC) and any signer configuration
- **Strict linting** — `pedantic` + `nursery` + `correctness` (deny), see [`clippy.toml`](clippy.toml)
- **Lightweight instances** — each `Identity` / `Reputation` / `Validation` call creates a zero-alloc contract handle

## Examples

| Example | Description |
| --- | --- |
| [`query_agent`](erc8004/examples/query_agent.rs) | Read agent identity from Ethereum mainnet |
| [`register_agent`](erc8004/examples/register_agent.rs) | Register a new agent on Base Sepolia testnet |
| [`reputation_summary`](erc8004/examples/reputation_summary.rs) | Query aggregated reputation and feedback entries |
| [`registration_file`](erc8004/examples/registration_file.rs) | Build and serialize an off-chain registration file |
| [`multi_network`](erc8004/examples/multi_network.rs) | Query the same registry across multiple chains |

```bash
cargo run --example query_agent
cargo run --example registration_file
```

## Security

See [`SECURITY.md`](SECURITY.md) for disclaimers, supported versions, and vulnerability reporting.

## Acknowledgments

- [ERC-8004 Specification](https://eips.ethereum.org/EIPS/eip-8004) — Trustless Agents standard
- [erc-8004/erc-8004-contracts](https://github.com/erc-8004/erc-8004-contracts) — official Solidity contracts
- [alloy](https://github.com/alloy-rs/alloy) — Rust Ethereum toolkit

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.
