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
  registry ABIs (`/abi`), the shared error base (`/core`), and the verified
  on-chain deployment table (`/deployments`).
- `data/abis/` vendors the upstream `erc-8004-contracts` ABIs verbatim;
  `bun run gen` regenerates the TS ABI modules and deployment table from
  `data/`.

### Removed

- **BREAKING**: the `erc8004-events` archiver crate, its Hugging Face sync
  workflow, the `contracts` submodule, `config.toml`, `Justfile`, `Makefile`
  and the Go/Python RPC tooling are gone — event indexing is out of scope
  for the SDK.
