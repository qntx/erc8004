<!-- markdownlint-disable MD033 MD041 -->

# erc8004

[ERC-8004](https://eips.ethereum.org/EIPS/eip-8004) _Trustless Agents_ — an identity/reputation/validation
registry protocol — for TypeScript and Rust:
[`@qntx/erc8004`](https://www.npmjs.com/package/@qntx/erc8004) (npm, `packages/erc8004`) and
[`erc8004`](https://crates.io/crates/erc8004) (crates.io, `crates/erc8004`). Both
implementations share one protocol layer — registries, agentURIs, feedback
values, registration files, AgentWalletSet signatures — driven by the same
test vectors in `vectors/` and tracked in `parity.json`.

| Package         | Registry  | Language   |
| --------------- | --------- | ---------- |
| `@qntx/erc8004` | npm       | TypeScript |
| `erc8004`       | crates.io | Rust       |

## Quick start — TypeScript

```ts
import { createPublicClient, createWalletClient, http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { base } from "viem/chains";

import {
  erc8004PublicActions,
  erc8004WalletActions,
  parseRegisteredAgentId,
  parseFeedbackValue,
  resolveAgentUri,
} from "@qntx/erc8004";

const publicClient = createPublicClient({ chain: base, transport: http() }).extend(
  erc8004PublicActions(),
);

const agent = await publicClient.erc8004GetAgent({ agentId: 1n });
const registration = await resolveAgentUri(agent.agentUri);
const summary = await publicClient.erc8004GetReputationSummary({
  agentId: 1n,
  clientAddresses: ["0x..."],
});

const wallet = createWalletClient({
  account: privateKeyToAccount("0x..."),
  chain: base,
  transport: http(),
}).extend(erc8004WalletActions());

const hash = await wallet.erc8004RegisterAgent({
  agentUri: "https://example.com/agent.json",
});
const receipt = await publicClient.waitForTransactionReceipt({ hash });
const agentId = parseRegisteredAgentId(receipt.logs);
```

## Quick start — Rust

```rust
use erc8004::{deployments, Erc8004, NewFeedback, Resolver, ResolverOptions};

let erc = Erc8004::new(provider, deployments::BASE);
let agent = erc.identity().get_agent(agent_id).await?;
let feedback = NewFeedback::new(agent.agent_id, value).tag1("quality");
let outcome = erc.reputation().give_feedback(&feedback).await?;
let registration = Resolver::new(ResolverOptions::default())?
    .resolve_agent_uri(&agent.agent_uri)
    .await?;
```

`deployments::BASE` / `by_chain_id` / `ALL` are generated from
[`data/deployments.json`](data/deployments.json); a chain is included only
after `scripts/deployments/verify.ts` confirms `getVersion() == "2.0.0"` on
all three registries on-chain.

## Repository layout

```
packages/erc8004   TypeScript library (@qntx/erc8004)
crates/erc8004     Rust crate (erc8004)
data/              Vendored ABIs + contracts + deployments
vectors/           Shared conformance vectors
scripts/           Codegen, parity, versioning
e2e/               Foundry project + anvil end-to-end suite
docs/              Published documentation source
```

## Development

Full local gate lives in [AGENTS.md](AGENTS.md) — `bun run lint && bun run
typecheck && bun run test`, `cargo clippy -- -D warnings`, `cargo test`,
`cargo deny check`, `e2e/run.sh` (anvil + forge + bun + cargo).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.

---

<div align="center">

A **[QuantX](https://qntx.org)** open-source project.

<a href="https://qntx.org"><img alt="QuantX" width="369" src="https://raw.githubusercontent.com/qntx/.github/main/profile/qntx.svg" /></a>

Code is law. We write both.

</div>
