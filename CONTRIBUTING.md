# Contributing

## Prerequisites

- **Bun 1.4** — package scripts, tests, and tooling.
- **Rust 1.99** — via `rust-toolchain.toml` (kept in sync with
  `[workspace.package].rust-version`).
- **cargo-deny** for the supply-chain check (`cargo install cargo-deny`).

## Repository layout

```text
packages/erc8004/  @qntx/erc8004 — platform-neutral TypeScript SDK (layered:
                   abi, core, deployments, registration, events,
                   identity|reputation|validation); builds with vp pack
crates/erc8004/    erc8004 — the Rust SDK crate
data/              source data: vendored upstream ABIs (data/abis, see
                   data/SOURCE.md) and the on-chain-verified deployment
                   table (data/deployments.json)
vectors/           shared cross-language test vectors (see vectors/README.md)
parity.json        capability ledger: every shared capability and the
                   vectors/tests behind it
scripts/           repository tooling: gen.ts regenerates the TS modules
                   from data/ (`bun run gen`, checked in lint via --check),
                   deployments/verify.ts re-verifies deployments on-chain
                   (manual), plus version and parity checks
```

## Local gate

Run before opening a pull request:

```bash
bun run lint && bun run typecheck && bun run test   # lint includes taplo,
                                                    # gen --check, version
                                                    # and parity checks
bun run build
bun run --cwd packages/erc8004 check:pkg
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo deny check
```

## Versioning and release

`@qntx/erc8004` and the `erc8004` crate share one lockstep version.
`bun run release` runs `bumpp` (`bump.config.ts`), which bumps
`packages/erc8004/package.json` and the Cargo workspace version together,
refreshes `Cargo.lock`/`bun.lock`, and opens a `release/v<version>` pull
request. `scripts/check-version.ts` (part of `bun run lint`) guards against
drift: the package version must equal `[workspace.package].version` and the
version string may appear nowhere else in `Cargo.toml`.

After the release PR merges, tag the merge commit `v*.*.*` and push it by
hand; one tag publishes npm (`@qntx/erc8004`), crates.io (`erc8004`), and
the GitHub Release.

## Commits

English Conventional Commits: `type(scope): subject` (`feat`, `fix`, `docs`,
`style`, `refactor`, `perf`, `test`, `chore`, `ci`, `build`, `revert`), subject
in the imperative mood without a trailing period, ≤ 50 characters. Breaking
changes get a `BREAKING CHANGE:` footer.
