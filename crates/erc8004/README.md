# erc8004

Rust SDK for [ERC-8004](https://eips.ethereum.org/EIPS/eip-8004) _Trustless
Agents_: identity, reputation and validation registries, agentURI resolution,
registration and feedback files, and `AgentWalletSet` signatures. Part of the
[dual-language monorepo](../../README.md) — the TypeScript counterpart is
[`@qntx/erc8004`](https://www.npmjs.com/package/@qntx/erc8004) and both run the
same vectors in `vectors/`.

The core is provider-free and sans-IO; network access is opt-in behind
features. `--no-default-features` compiles for `wasm32-unknown-unknown` with no
`tokio`/`reqwest` in the tree.

## Features

| Feature    | Effect                                                                                                                                          |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `provider` | `Erc8004<P: Provider>` client: identity/reputation/validation handles, `TxOutcome`, `ProviderError` (alloy providers + network + rpc-types-eth) |
| `fetch`    | `Resolver` / `ResolverOptions` / `ResolveError`: `data:`/HTTPS/IPFS agentURI resolution with a streamed byte cap (reqwest, rustls)              |
| `default`  | `provider` + `fetch`                                                                                                                            |

## Quick start

```rust
use alloy::primitives::U256;
use erc8004::{deployments, Erc8004, NewFeedback, Resolver, ResolverOptions};

let erc = Erc8004::new(provider, deployments::BASE);
let agent = erc.identity().get_agent(U256::from(1u64)).await?;
let feedback = NewFeedback::new(agent.agent_id, value).tag1("quality");
let outcome = erc.reputation().give_feedback(&feedback).await?;
let registration = Resolver::new(ResolverOptions::default())?
    .resolve_agent_uri(&agent.agent_uri)
    .await?;
```

`deployments::{BASE, ALL, by_chain_id, Deployment}` is generated from the
monorepo's `data/deployments.json` — chains only land in the table after
on-chain `getVersion() == "2.0.0"` verification on all three registries.

## Error model

Every error type is `#[non_exhaustive]` and exposes the same `code` strings as
the TypeScript SDK: `ProviderError` (`Rpc`, `Revert` → `"ContractRevert"`,
`TransactionFailed`, `MissingEvent`, `InvalidResponse`, plus the client
guards), `ResolveError`, `AgentRegistryError`, `FeedbackValueError`,
`DataUriError`, `AgentUriError`, `JsonDocumentError`, `EventDecodeError`,
`AgentWalletDeadlineError` and `RevertReason`. See the monorepo
[README](../../README.md) for the shared code table.
