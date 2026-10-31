# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

A `vX.Y.Z` tag runs `publish-npm.yml` (publishing `packages/erc8004` via npm
trusted publishing / OIDC) and `publish-crates.yml` (publishing `erc8004` to
crates.io). npm and crate versions are lockstep.

## [Unreleased]

### Added

- `@qntx/erc8004` (`packages/erc8004`): TypeScript SDK with the vendored
  registry ABIs (`/abi`), the pure core (`/core`: agent-registry ids, feedback
  values, revert-reason decoding, JSON document errors, content hashing), the
  registration layer (`/registration`: lenient/lossless registration-file and
  feedback-file models, conformance checks, data URIs, agentURI
  classification, endpoint-domain verification), the event decoder (`/events`),
  the `AgentWalletSet` EIP-712 helpers (`/identity`), and the verified
  deployment table (`/deployments`) — all exercised by the shared vectors in
  `vectors/` (`bun run vectors` regenerates the reference vectors).
- On-chain actions in both languages. TypeScript: viem-style free functions for
  the three registries (`identity`, `reputation` and the `/validation` layer),
  contract-revert errors mapped to shared `RevertReason`s via
  `ContractRevertError`, pre-RPC guards (`ClientGuardError`:
  `ClientAddressesRequired`, `MissingAccount`, `ResponseOutOfRange`,
  `ReservedMetadataKey`, `EmptyResponseUri`; wallet-deadline checks against the
  latest block), `signAgentWallet` plus `parseRegisteredAgentId` /
  `parseFeedbackIndex` receipt helpers, and the
  `erc8004PublicActions()`/`erc8004WalletActions()` client decorators.
  Rust: the `provider` feature adds `Erc8004<P: Provider>` with
  `identity()`/`reputation()`/`validation()` handles, `TxOutcome<T>` write
  results and `ProviderError` covering RPC, revert (`"ContractRevert"`),
  receipt-status, missing-event and the same client guards.
- agentURI resolution in both languages. TypeScript `resolveAgentUri` /
  `checkEndpointDomain` take an injected `fetch` (`data:`/HTTPS/IPFS gateway,
  1 MiB default body cap checked on `content-length` and the actual body,
  `ResolveError` codes `NotSet`/`UnsupportedScheme`/`HttpStatus`/`TooLarge`/
  `InvalidJson`). Rust `Resolver`/`ResolverOptions`/`ResolveError` behind the
  `fetch` feature with a streamed byte cap. `default = ["provider", "fetch"]`;
  `--no-default-features` stays the pure sans-IO core.
- `erc8004` (`crates/erc8004`) rewritten as the matching Rust SDK — same
  modules, same error codes, same vectors (`tests/vectors.rs`), plus a
  `sol!`-vs-vendored-ABI selector/topic parity test.
- `data/abis/` is derived from the vendored upstream Solidity sources at
  `b9e466c` (`forge inspect` via `bun run abi`/`--check`; upstream's committed
  `abis/` omits `isAuthorizedOrOwner`, which exists in the deployed contracts —
  see `data/SOURCE.md`). `bun run gen` regenerates the TS ABI modules, the
  deployment tables and the docs deployment page from `data/`.
- `e2e/`: a Foundry project vendoring the upstream contracts at `b9e466c`
  (soldeer: OpenZeppelin 5.4.0 + `forge-std` 1.9.4) with `Deploy.s.sol`
  replicating the upstream proxy-upgrade deployment and `e2e/run.sh` driving
  matching TypeScript (`tests/e2e/core.test.ts`, env-gated) and Rust
  (`tests/e2e.rs`, `#[ignore]`) anvil suites over the same scenario.
- Rust examples: `query_agent`, `registration_file`, `give_feedback`
  (`cargo run -p erc8004 --example <name>`).
- `docs/` published documentation: overview, protocol, TypeScript, Rust, the
  shared error table and the generated deployments page.

### Changed

- **BREAKING**: the repository is now a dual TypeScript + Rust monorepo. The
  Rust crate moved from `erc8004/` to `crates/erc8004/` and the workspace
  version is `0.5.0`.
- **BREAKING**: the provider-era Rust API (`client`, `contracts`, `networks`,
  `identity`, `reputation`, `validation`, `types`, `examples/`) is replaced by
  the layered surface above.
- Contract addresses are no longer trust-me lists: `data/deployments.json`
  only contains chains where `scripts/deployments/verify.ts` confirmed
  `getVersion() == "2.0.0"` on all three registries on-chain.
- The crate builds for `wasm32-unknown-unknown` under
  `--no-default-features` with no `tokio`/`reqwest` in the dependency tree.

### Removed

- **BREAKING**: the `erc8004-events` archiver crate, its Hugging Face sync
  workflow, the `contracts` submodule, `config.toml`, `Justfile`, `Makefile`
  and the Go/Python RPC tooling are gone — event indexing is out of scope
  for the SDK.
