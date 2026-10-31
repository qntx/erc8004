# AGENTS.md

- Do not preserve backward compatibility. Remove obsolete paths instead of adding compatibility layers, fallbacks, or migrations.
- Choose the simplest implementation that fully meets the current requirements. Avoid speculative abstractions, configuration, and indirection.
- Grow the system in layers. Start from the smallest version that works end to end, and add each new capability on top of a product that already works. Never trade a working product for unfinished complexity.
- Keep components modular and concerns clearly separated.
- Prefer established, well-maintained libraries when they reduce overall complexity or improve reliability. Do not reimplement common functionality without a clear reason.
- Lean on the dependencies already in the project before writing your own implementation or adding packages. Do not assume a library lacks a capability without checking its documentation and types.
- Make architectural decisions for the long term. Do not accept a stopgap that only works for now and is meant to be replaced later.

## Scope

ERC-8004 (Trustless Agents) protocol SDK in two languages: `@qntx/erc8004` (TypeScript, `packages/erc8004`) and `erc8004` (Rust, `crates/erc8004`). Protocol layer only: the three registries (Identity, Reputation, Validation) at contract version `2.0.0`, the off-chain registration and feedback files, agentURI resolution, endpoint-domain verification, and the `AgentWalletSet` EIP-712 message. No IPFS pinning, no subgraph, no indexer, no payment rails.

## Source data

- `data/contracts/*.sol` are the upstream registry contracts vendored verbatim from `erc-8004/erc-8004-contracts` at the commit recorded in `data/SOURCE.md`. `data/abis/*.json` are **derived**: `bun run abi` regenerates them via `forge inspect` (upstream's committed `abis/` is stale relative to its own sources, so it is never used). The `e2e` Foundry project compiles them via `src = "../data/contracts"` + a `contracts/` remapping.
- `data/deployments.json` is the single source of truth for known deployments. A chain is listed only when `scripts/deployments/verify.ts` has confirmed on-chain that all three registries answer `getVersion() == "2.0.0"` and that Reputation and Validation `getIdentityRegistry()` return the Identity address. Placeholder proxies (`MinimalUUPS` `0.0.1`, `MinimalUUPSMainnet` `1.0.0`) and unreachable chains are excluded. Each entry carries `id` (kebab-case), `name`, `chainId`, `testnet`, and the three registry addresses (EIP-55).
- `bun run gen` regenerates `packages/erc8004/src/abi/*.ts` (`as const`) from `data/abis`, and the TS and Rust deployment tables from `data/deployments.json`; `bun run lint` runs it in check mode so generated files never drift.

## TypeScript (`packages/erc8004`)

- `viem` is a peer dependency; `@scure/base` is the only other runtime dependency (caret range). devDependencies are exact-pinned.
- `src/` is platform-neutral and Hermes V1-safe: no Node or browser globals, no `node:*` imports, no ES2023+ array methods (`tsconfig.src.json` compiles `src` against `es2022`). All I/O is injected (`fetch` defaults to a call-time `globalThis.fetch` lookup); no ambient singletons.
- One-directional layering, enforced by lint (`LAYERS` in `vite.config.ts`): `abi` is the leaf (pure data); `core` imports `abi` (revert decoding needs the ABI errors); `deployments` and `registration` import `core`; `events` imports `abi`, `core`; `identity`, `reputation`, `validation` import `abi`, `core`, `deployments`, `events`; the package root composes everything and exports the `erc8004PublicActions()` / `erc8004WalletActions()` client decorators. Every layer is a subpath export.
- Actions are viem-style free functions `(client, args)`. Registry addresses resolve from `args.address`, else the decorator's bound deployment, else `client.chain.id` via the deployment table; otherwise `UnsupportedChainError`. Write actions return the transaction `Hash`; receipt helpers (`parseRegisteredAgentId`, `parseFeedbackIndex`, ...) decode results from logs.

## Rust (`crates/erc8004`)

- One crate. Features gate dependencies only: `provider` (alloy RPC actions) and `fetch` (reqwest agentURI resolution); `default = ["provider", "fetch"]`. `--no-default-features` is the sans-IO core and must build for `wasm32-unknown-unknown` with no `tokio` in the tree.
- Contract bindings are inline `sol!` interfaces without `#[sol(rpc)]`; a test asserts every selector and event topic matches `data/abis`. The provider layer uses one generic `eth_call` helper and one send helper that waits for the receipt, checks its status, and decodes reverts.
- Public API types are std, `serde_json::Value`/`Map`, alloy primitives (`Address`, `U256`, `B256`, `Bytes`, `Log`, `Signature`), and `Eip712Domain` (alloy-sol-types) only; no other third-party types. One `#[non_exhaustive]` error enum per module.
- Follow the Rust API Guidelines. No panics and no `unwrap`/`expect` outside tests.

## Shared semantics

Both languages implement the same capabilities with the same rules; `parity.json` lists each capability with its TS module and tests, its Rust module and tests, and its vectors.

- Error codes are shared: every failure case in `vectors/` names a code (`InvalidChainId`, `BadChecksum`, `TooManyDecimals`, ...) that equals the Rust error variant name and the TS error `code`.
- `AgentRegistry` is `eip155:{chainId}:{address}`. Only the `eip155` namespace is accepted; `chainId` is canonical decimal in `1..=2^53-1`; the address is all-lowercase, all-uppercase, or valid EIP-55. It always formats EIP-55.
- `FeedbackValue` is `(value: int128, decimals: u8)` with `decimals <= 18` and `|value| <= 1e38` (contract bounds). Decimal strings `-?\d+(\.\d+)?` convert exactly, keeping the written number of fraction digits (`"99.70"` is `9970` with `2` decimals, because `getSummary` aggregates by the mode of `valueDecimals`). No rounding, clamping, exponents, or floats.
- Registration files parse leniently and round-trip losslessly: the top level must be a JSON object; spec fields (`type`, `name`, `description`, `image`, `services`, `x402Support`, `active`, `registrations`, `supportedTrust`) are optional but must have the spec type when present; unknown fields are preserved. No defaults are invented for absent fields. `registrations[].agentRegistry` stays a raw string with a typed accessor; `registrations[].agentId` accepts a JSON integer up to `2^53-1` or a canonical decimal string, and serializes as a number up to `2^53-1` and as a decimal string above it. Conformance checks are a separate function, not a parse failure.
- agentURI classification: empty is `NotSet`; `data:application/json[;charset=...];base64,` is decoded inline; `ipfs://` goes through a configurable gateway (default `https://ipfs.io/ipfs/`); `https://` is fetched; everything else (including `http://`) is `UnsupportedScheme`. Resolution caps the body at 1 MiB by default.
- Event decoding covers the protocol events of the vendored ABIs and decides by `topic0`: unknown topics are "not ours" (`null`/`None`), malformed data under a known topic is an error.
- Revert decoding maps `Error(string)` reasons and the ABI custom errors to one shared reason enum; unrecognised data stays available raw.
- Clients reject calls the contract is guaranteed to revert before sending (`getSummary` with no client addresses, out-of-range feedback values, validation `response > 100`, `AgentWalletSet` deadlines more than 5 minutes ahead).

## Versioning and release

- npm and crate versions are lockstep: `[workspace.package].version` equals `packages/erc8004/package.json`; `scripts/check-version.ts` and `scripts/sync-versions.ts` guard drift; `bumpp` opens a `release/v<version>` PR and the merge commit is tagged by hand. Tags publish to npm (`publish-npm.yml`, OIDC) and crates.io (`publish-crates.yml`).
- TOML is formatted by taplo (`.taplo.toml`, aligned `=`); keep `taplo fmt --check` green.

## Commands

Local gate before a pull request:

```bash
bun run lint && bun run typecheck && bun run test   # lint includes taplo,
                                                    # version, layer and
                                                    # parity checks
bun run build && bun run --cwd packages/erc8004 check:pkg
bun run abi --check                                 # data/abis derived from the
                                                    # vendored contracts via
                                                    # forge inspect
taplo fmt --check
cargo fmt --all --check
cargo clippy -p erc8004 --all-targets --all-features -- -D warnings
cargo clippy -p erc8004 --all-targets --no-default-features -- -D warnings
cargo test -p erc8004 --all-features
cargo test -p erc8004 --no-default-features
cargo hack check -p erc8004 --each-feature --no-dev-deps
cargo check -p erc8004 --target wasm32-unknown-unknown --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc -p erc8004 --all-features --no-deps
cargo deny check
cargo tree -p erc8004 -e normal --no-default-features | rg -i 'tokio|reqwest'
                                                    # must print nothing
cargo tree -p erc8004 -e normal -d | rg reqwest    # no duplicate reqwest
e2e/run.sh                                          # anvil + forge + bun +
                                                    # cargo; leaves anvil
                                                    # stopped
```
