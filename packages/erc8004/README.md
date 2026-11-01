# @qntx/erc8004

TypeScript SDK for [ERC-8004](https://eips.ethereum.org/EIPS/eip-8004) _Trustless
Agents_: identity, reputation and validation registries, agentURI resolution,
registration and feedback files, and `AgentWalletSet` signatures. Part of the
[dual-language monorepo](../../README.md) — the Rust counterpart is
[`erc8004`](https://crates.io/crates/erc8004) and both run the same vectors in
`vectors/`.

## Install

```bash
bun add @qntx/erc8004 viem
```

## Subpath exports

| Subpath                     | Contents                                                                                                              |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `@qntx/erc8004`             | Protocol core, on-chain actions (`erc8004PublicActions` / `erc8004WalletActions`), agentURI resolver, events          |
| `@qntx/erc8004/abi`         | Vendored registry ABIs (`identityRegistryAbi`, `reputationRegistryAbi`, `validationRegistryAbi`, `protocolEventsAbi`) |
| `@qntx/erc8004/core`        | Sans-IO protocol core: agent registry ids, feedback values, revert decoding, errors                                   |
| `@qntx/erc8004/deployments` | Verified on-chain deployment table (`deployments`, `getDeployment`, `Deployment`)                                     |
| `@qntx/erc8004/validation`  | Validation-registry actions layer                                                                                     |

## Quick start

```ts
import { createPublicClient, createWalletClient, http } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { base } from "viem/chains";

import {
  erc8004PublicActions,
  erc8004WalletActions,
  parseRegisteredAgentId,
  resolveAgentUri,
} from "@qntx/erc8004";

const publicClient = createPublicClient({
  chain: base,
  transport: http(),
}).extend(erc8004PublicActions());

const agent = await publicClient.erc8004GetAgent({ agentId: 1n });
const registration = await resolveAgentUri(agent.agentUri);

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

## Error model

All SDK errors extend `Erc8004Error` with a stable `code` shared with the Rust
implementation:

| Class / shape              | `code`                                                                                                       |
| -------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `AgentRegistryError`       | `Malformed`, `UnsupportedNamespace`, `InvalidChainId`, `InvalidAddress`, `BadChecksum`                       |
| `FeedbackValueError`       | `InvalidDecimalString`, `TooManyDecimals`, `ValueOutOfRange`, `InvalidDecimals`                              |
| `DataUriError`             | `NotDataUri`, `UnsupportedMediaType`, `NotBase64`, `InvalidBase64`                                           |
| `AgentUriError`            | `InvalidGateway`                                                                                             |
| `JsonDocumentError`        | `NotAnObject`, `InvalidField`                                                                                |
| `ContractRevertError`      | `ContractRevert` (carries `reason: RevertReason`)                                                            |
| `ClientGuardError`         | `ClientAddressesRequired`, `MissingAccount`, `ResponseOutOfRange`, `ReservedMetadataKey`, `EmptyResponseUri` |
| `AgentWalletDeadlineError` | `DeadlineExpired`, `DeadlineTooFar`                                                                          |
| `MissingEventError`        | `MissingEvent`                                                                                               |
| `ResolveError`             | `NotSet`, `UnsupportedScheme`, `HttpStatus`, `TooLarge`, `InvalidJson`                                       |
| `UnsupportedChainError`    | `UnsupportedChain`                                                                                           |
| `EventDecodeError`         | `MalformedLog`                                                                                               |

See the monorepo [README](../../README.md) and [`AGENTS.md`](../../AGENTS.md)
for the architecture invariants and the shared Rust error mapping.
