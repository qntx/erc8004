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
  `data/`.

### Removed

- **BREAKING**: the `erc8004-events` archiver crate, its Hugging Face sync
  workflow, the `contracts` submodule, `config.toml`, `Justfile`, `Makefile`
  and the Go/Python RPC tooling are gone — event indexing is out of scope
  for the SDK.
