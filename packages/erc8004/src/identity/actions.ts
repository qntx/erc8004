import { zeroAddress } from "viem";
import type { Address, Client, ContractFunctionArgs, Hash, Hex } from "viem";
import { getBlock, signTypedData, writeContract } from "viem/actions";

import { identityRegistryAbi } from "../abi/index.ts";
import { ClientGuardError, MissingEventError } from "../core/errors.ts";
import { withContractRevert } from "../core/revert-map.ts";
import { read, registry, sender, senderAddress } from "../deployments/resolve.ts";
import type { ActionArgs } from "../deployments/resolve.ts";
import { decodeErc8004Log } from "../events/index.ts";
import { agentWalletTypedData, assertAgentWalletDeadline } from "./agent-wallet.ts";

type Args = ActionArgs;

/** `{ agentId, owner, agentUri, agentWallet }` — `agentWallet` is `null` when unset (zero address). */
export type Agent = {
  agentId: bigint;
  owner: Address;
  agentUri: string;
  // oxlint-disable-next-line typescript/no-restricted-types -- spec: the zero address maps to null.
  agentWallet: Address | null;
};

/**
 * Concurrent `ownerOf`/`tokenURI`/`getAgentWallet` reads; reverts (e.g. `AgentNotFound`) as
 * `ContractRevertError`.
 */
export async function getAgent(client: Client, args: Args & { agentId: bigint }): Promise<Agent> {
  const address = registry(client, "identity", args);
  const [owner, agentUri, wallet] = await Promise.all([
    read(client, address, identityRegistryAbi, "ownerOf", [args.agentId]),
    read(client, address, identityRegistryAbi, "tokenURI", [args.agentId]),
    read(client, address, identityRegistryAbi, "getAgentWallet", [args.agentId]),
  ]);
  return {
    agentId: args.agentId,
    owner,
    agentUri,
    agentWallet: wallet === zeroAddress ? null : wallet,
  };
}

/** `tokenURI` for an agent. */
export async function getAgentUri(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<string> {
  return read(client, registry(client, "identity", args), identityRegistryAbi, "tokenURI", [
    args.agentId,
  ]);
}

/** `ownerOf` for an agent; reverts `AgentNotFound` when the id does not exist. */
export async function getAgentOwner(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<Address> {
  return read(client, registry(client, "identity", args), identityRegistryAbi, "ownerOf", [
    args.agentId,
  ]);
}

/** `getAgentWallet` — `null` when unset (zero address). */
export async function getAgentWallet(
  client: Client,
  args: Args & { agentId: bigint },
  // oxlint-disable-next-line typescript/no-restricted-types -- spec: unset wallets read back as null.
): Promise<Address | null> {
  const wallet = await read(
    client,
    registry(client, "identity", args),
    identityRegistryAbi,
    "getAgentWallet",
    [args.agentId],
  );
  return wallet === zeroAddress ? null : wallet;
}

/** `getMetadata` for one key. */
export async function getMetadata(
  client: Client,
  args: Args & { agentId: bigint; key: string },
): Promise<Hex> {
  return read(client, registry(client, "identity", args), identityRegistryAbi, "getMetadata", [
    args.agentId,
    args.key,
  ]);
}

/** `isAuthorizedOrOwner` — whether `spender` may act for `agentId`. */
export async function isAuthorizedOrOwner(
  client: Client,
  args: Args & { spender: Address; agentId: bigint },
): Promise<boolean> {
  return read(
    client,
    registry(client, "identity", args),
    identityRegistryAbi,
    "isAuthorizedOrOwner",
    [args.spender, args.agentId],
  );
}

/** `getApproved` — the single approved address for `agentId`. */
export async function getApproved(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<Address> {
  return read(client, registry(client, "identity", args), identityRegistryAbi, "getApproved", [
    args.agentId,
  ]);
}

/** `isApprovedForAll` — operator approval across an owner's agents. */
export async function isApprovedForAll(
  client: Client,
  args: Args & { owner: Address; operator: Address },
): Promise<boolean> {
  return read(client, registry(client, "identity", args), identityRegistryAbi, "isApprovedForAll", [
    args.owner,
    args.operator,
  ]);
}

/** `getVersion` of the identity registry. */
export async function getIdentityVersion(client: Client, args: Args = {}): Promise<string> {
  return read(client, registry(client, "identity", args), identityRegistryAbi, "getVersion", []);
}

function checkMetadataKey(key: string): void {
  if (key === "agentWallet") {
    throw new ClientGuardError("ReservedMetadataKey", 'metadata key "agentWallet" is reserved');
  }
}

/**
 * `register` overload by args: bare `register()` → `register(string)` → `register(string,
 * MetadataEntry[])`. `ClientGuardError` `ReservedMetadataKey` when `metadata` uses the reserved
 * `agentWallet` key.
 */
export async function registerAgent(
  client: Client,
  args: Args & {
    agentUri?: string;
    metadata?: ReadonlyArray<{ key: string; value: Hex }>;
  },
): Promise<Hash> {
  const metadata = args.metadata ?? [];
  for (const entry of metadata) {
    checkMetadataKey(entry.key);
  }
  const registerArgs: ContractFunctionArgs<
    typeof identityRegistryAbi,
    "nonpayable" | "payable",
    "register"
  > =
    metadata.length > 0
      ? [args.agentUri ?? "", metadata.map((m) => ({ metadataKey: m.key, metadataValue: m.value }))]
      : args.agentUri === undefined
        ? []
        : [args.agentUri];
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, "identity", args),
      functionName: "register",
      args: registerArgs,
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `setAgentURI` write. */
export async function setAgentUri(
  client: Client,
  args: Args & { agentId: bigint; agentUri: string },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, "identity", args),
      functionName: "setAgentURI",
      args: [args.agentId, args.agentUri],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `setMetadata` write; `ClientGuardError` `ReservedMetadataKey` for `agentWallet`. */
export async function setMetadata(
  client: Client,
  args: Args & { agentId: bigint; key: string; value: Hex },
): Promise<Hash> {
  checkMetadataKey(args.key);
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, "identity", args),
      functionName: "setMetadata",
      args: [args.agentId, args.key, args.value],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/**
 * `setAgentWallet` write — `deadline` checked against the latest block timestamp (±5 minutes)
 * before sending; `AgentWalletDeadlineError` on violation.
 */
export async function setAgentWallet(
  client: Client,
  args: Args & { agentId: bigint; newWallet: Address; deadline: bigint; signature: Hex },
): Promise<Hash> {
  const block = await getBlock(client);
  assertAgentWalletDeadline(args.deadline, block.timestamp);
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, "identity", args),
      functionName: "setAgentWallet",
      args: [args.agentId, args.newWallet, args.deadline, args.signature],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `unsetAgentWallet` write. */
export async function unsetAgentWallet(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, "identity", args),
      functionName: "unsetAgentWallet",
      args: [args.agentId],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `transferFrom` — `from` is the current owner read via `ownerOf` (reverts `AgentNotFound`). */
export async function transferAgent(
  client: Client,
  args: Args & { to: Address; agentId: bigint },
): Promise<Hash> {
  const address = registry(client, "identity", args);
  const from = await read(client, address, identityRegistryAbi, "ownerOf", [args.agentId]);
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address,
      functionName: "transferFrom",
      args: [from, args.to, args.agentId],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `approve` write — approve a single address for `agentId`. */
export async function approveAgent(
  client: Client,
  args: Args & { to: Address; agentId: bigint },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, "identity", args),
      functionName: "approve",
      args: [args.to, args.agentId],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `setApprovalForAll` write — operator approval across the sender's agents. */
export async function setAgentOperator(
  client: Client,
  args: Args & { operator: Address; approved: boolean },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, "identity", args),
      functionName: "setApprovalForAll",
      args: [args.operator, args.approved],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/**
 * Signs an `AgentWalletSet` typed data with the _new wallet's_ client — the signature
 * `setAgentWallet` requires. `MissingAccount` when the client has no account (unless `args.account`
 * supplies one).
 */
export async function signAgentWallet(
  client: Client,
  args: ActionArgs & {
    chainId: number;
    identityRegistry: Address;
    agentId: bigint;
    owner: Address;
    deadline: bigint;
  },
): Promise<Hex> {
  return signTypedData(client, {
    account: sender(client, args.account),
    ...agentWalletTypedData({
      chainId: args.chainId,
      identityRegistry: args.identityRegistry,
      message: {
        agentId: args.agentId,
        newWallet: senderAddress(client, args.account),
        owner: args.owner,
        deadline: args.deadline,
      },
    }),
  });
}

/** The `Registered` event's `agentId` from a register receipt; `MissingEvent` when absent. */
export function parseRegisteredAgentId(
  logs: ReadonlyArray<{ topics: ReadonlyArray<Hex>; data: Hex }>,
): bigint {
  for (const log of logs) {
    const decoded = decodeErc8004Log(log);
    if (decoded?.eventName === "Registered") {
      return decoded.args.agentId;
    }
  }
  throw new MissingEventError("Registered");
}
