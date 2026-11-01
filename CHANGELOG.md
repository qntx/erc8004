# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

A `vX.Y.Z` tag runs `publish-npm.yml` (publishing `packages/erc8004` via npm
trusted publishing / OIDC) and `publish-crates.yml` (publishing `erc8004` to
crates.io). npm and crate versions are lockstep.

## [Unreleased]

## [0.5.1] - 2026-11-01

### Changed

- Release through the tag-triggered npm trusted publishing and crates.io
  workflows; no API changes since 0.5.0.

## [0.5.0] - 2026-11-01

### Added

- `@qntx/erc8004` (`packages/erc8004`): TypeScript SDK layered as `/abi`
  (generated registry ABIs), `/core` (agent-registry ids, feedback values,
  revert-reason decoding, shared error model), `/deployments` (verified
  deployment table + address resolution), `/registration` (registration-file
  model, conformance checks, data URIs, agentURI classification and
  resolution, endpoint-domain verification), `/identity`, `/reputation`
  (including the feedback-file model), `/validation`, and `/events`.
- TypeScript on-chain actions: viem-style free functions for all three
  registries, `erc8004PublicActions()`/`erc8004WalletActions()` client
  decorators, contract reverts decoded into shared `RevertReason`s
  (`ContractRevertError`), pre-RPC guards (`ClientGuardError`), receipt
  helpers `parseRegisteredAgentId`/`parseFeedbackIndex`, and
  `signAgentWallet` for the `AgentWalletSet` EIP-712 payload.
- Rust `provider` feature: `Erc8004<P: Provider>` with
  `identity()`/`reputation()`/`validation()` handles, `TxOutcome<T>` write
  results, request builders (`NewFeedback`, `FeedbackFilter`,
  `ValidationResponse`), and `ProviderError` mirroring the TypeScript error
  codes.
- agentURI resolution in both languages: TypeScript `resolveAgentUri`/
  `checkEndpointDomain` (injected fetch, `data:`/HTTPS/IPFS gateway, streamed
  byte cap, `ResolveError`); Rust `Resolver`/`ResolverOptions`/`ResolveError`
  behind the `fetch` feature. `default = ["provider", "fetch"]`;
  `--no-default-features` remains the pure sans-IO core.
- `data/` shared source data: upstream contracts vendored at `b9e466c`,
  `data/abis/` derived from them via `forge inspect` (`bun run abi`), and
  `data/deployments.json` restricted to chains where `getVersion() ==
  "2.0.0"` verifies on all three registries.
- Shared test vectors in `vectors/` consumed by both languages.
- `e2e/` Foundry project + `run.sh` driving matching TypeScript and Rust
  anvil suites over the same scenario.
- Rust examples (`query_agent`, `registration_file`, `give_feedback`) and a
  `docs/` site covering the protocol, both language surfaces, the shared
  error table, and generated deployment tables.

### Changed

- **BREAKING**: the repository is now a dual TypeScript + Rust monorepo. The
  Rust crate moved from `erc8004/` to `crates/erc8004/`; npm and crate
  versions are lockstep at `0.5.0`.
- **BREAKING**: the previous Rust API surface (`client`, `contracts`,
  `networks`, `identity`, `reputation`, `validation`, `types`, examples) is
  replaced by the layered provider/fetch API described above.

### Removed

- **BREAKING**: the `erc8004-events` archiver crate and its sync workflow,
  the `contracts` submodule, `config.toml`, `Justfile`/`Makefile`, and the
  Go/Python RPC tooling — event indexing is out of scope for the SDK.
