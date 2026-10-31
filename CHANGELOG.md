# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

A `vX.Y.Z` tag runs `publish-npm.yml` (publishing `packages/erc8004` via npm
trusted publishing / OIDC) and `publish-crates.yml` (publishing `erc8004` to
crates.io). npm and crate versions are lockstep.

## [Unreleased]

### Changed

- **BREAKING**: the repository is now a dual TypeScript + Rust monorepo. The
  Rust crate moved from `erc8004/` to `crates/erc8004/` and the workspace
  version is `0.5.0`; the crate's API is unchanged but will be rewritten in
  later phases.
- Contract addresses are no longer trust-me lists: `data/deployments.json`
  only contains chains where `scripts/deployments/verify.ts` confirmed
  `getVersion() == "2.0.0"` on all three registries on-chain.

### Added

- `@qntx/erc8004` (`packages/erc8004`): TypeScript SDK with the vendored
  registry ABIs (`/abi`), the pure core (`/core`: agent registry parse/format,
  feedback value, revert-reason decoding, JSON document errors, content hash),
  the registration layer (`/registration`: lenient/lossless file model,
  conformance checks, data URIs, agentURI classification, endpoint-domain
  verification), the event decoder (`/events`), the `AgentWalletSet` EIP-712
  helpers (`/identity`), the feedback-file model (`/reputation`), and the
  verified
  deployment table (`/deployments`) — all against shared vectors in
  `vectors/` (`bun run vectors` regenerates the reference vectors).
- `erc8004` (`crates/erc8004`) rewritten as the matching pure Rust core —
  same modules, same error codes, same vectors (`tests/vectors.rs`), plus
  a `sol!`-vs-vendored-ABI selector/topic parity test. The provider-era API
  (`client`, `contracts`, `networks`, `identity`, `reputation`, `validation`,
  `types`, `examples/`) is removed; provider work returns in a later phase.
  The crate now builds for `wasm32-unknown-unknown` with no `tokio`/`reqwest`
  in its dependency tree.
- `data/abis/` vendors the upstream `erc-8004-contracts` ABIs verbatim;
  `bun run gen` regenerates the TS ABI modules and deployment table from
  `data/`. (Upstream's `IdentityRegistry.json` at `b9e466c` omits
  `isAuthorizedOrOwner`, which exists in the contract source — the vendored
  file adds that entry; see `data/SOURCE.md`.)
- On-chain layer, both languages. TypeScript: viem-style action free
  functions for the three registries (`identity`, `reputation`, and the new
  `/validation` layer), contract-revert errors mapped to shared
  `RevertReason`s via `ContractRevertError`, pre-RPC guards
  (`ClientGuardError`: `ClientAddressesRequired`, `ResponseOutOfRange`,
  `ReservedMetadataKey`, `EmptyResponseUri`, wallet-deadline checks against
  the latest block), `signAgentWallet` and `parseRegisteredAgentId` /
  `parseFeedbackIndex` receipt helpers, plus `erc8004PublicActions()` /
  `erc8004WalletActions()` decorators for `client.extend()`. Rust: the
  `provider` feature adds `Erc8004<P: Provider>` with `identity()` /
  `reputation()` / `validation()` handles, `TxOutcome<T>` write results, and
  `ProviderError` covering RPC, revert, receipt-status, missing-event and the
  same client guards.
- agentURI resolution, both languages. TypeScript `resolveAgentUri` /
  `checkEndpointDomain` take an injected `fetch` (data/HTTPS/IPFS gateway,
  1 MiB default body cap, `ResolveError` codes `NotSet` / `UnsupportedScheme`
  / `HttpStatus` / `TooLarge` / `InvalidJson`). Rust `Resolver` /
  `ResolverOptions` / `ResolveError` behind the `fetch` feature with a
  streamed byte cap. `default = ["provider", "fetch"]`; `--no-default-features`
  stays the pure sans-IO core.
- `e2e/`: a Foundry project vendoring the upstream contracts at `b9e466c`
  (soldeer: OpenZeppelin 5.4.0 + `forge-std` 1.9.4) with `Deploy.s.sol`
  replicating the upstream proxy-upgrade deployment and `e2e/run.sh` driving
  matching TypeScript (`tests/e2e/core.test.ts`, env-gated) and Rust
  (`tests/e2e.rs`, `#[ignore]`) anvil suites over the same scenario.

### Removed

- **BREAKING**: the `erc8004-events` archiver crate, its Hugging Face sync
  workflow, the `contracts` submodule, `config.toml`, `Justfile`, `Makefile`
  and the Go/Python RPC tooling are gone — event indexing is out of scope
  for the SDK.
