import { zeroAddress } from "viem";
import type { Account, Address, Client, Hash, Hex } from "viem";
import { parseAccount } from "viem/accounts";
import { getBlock, readContract, signTypedData, writeContract } from "viem/actions";

import { identityRegistryAbi } from "../abi/index.ts";
import { ClientGuardError, MissingEventError } from "../core/errors.ts";
import { withContractRevert } from "../core/revert-map.ts";
import type { Deployment } from "../deployments/deployment.ts";
import { resolveRegistryAddress } from "../deployments/resolve.ts";
import { decodeErc8004Log } from "../events/index.ts";
import { agentWalletTypedData, assertAgentWalletDeadline } from "./agent-wallet.ts";

type Args = {
  address?: Address | undefined;
  deployment?: Deployment | undefined;
  account?: Account | Address | undefined;
};

function sender(client: Client, account?: Account | Address): Account {
  const found = account ?? client.account;
  if (found === undefined) {
    throw new ClientGuardError(
      "MissingAccount",
      "no account on the client; pass args.account or use a wallet client",
    );
  }
  return parseAccount(found);
}

function senderAddress(client: Client, account?: Account | Address): Address {
  return sender(client, account).address;
}

function registry(client: Pick<Client, "chain">, args: Args): Address {
  return resolveRegistryAddress(client, "identity", args.address, args.deployment);
}

/** `{ agentId, owner, agentUri, agentWallet }` — `agentWallet` is `null` when unset (zero address). */
export type Agent = {
  agentId: bigint;
  owner: Address;
  agentUri: string;
  // oxlint-disable-next-line typescript/no-restricted-types -- spec: the zero address maps to null.
  agentWallet: Address | null;
};

export async function getAgent(client: Client, args: Args & { agentId: bigint }): Promise<Agent> {
  const address = registry(client, args);
  const [owner, agentUri, wallet] = await Promise.all([
    withContractRevert(
      readContract(client, {
        abi: identityRegistryAbi,
        address,
        functionName: "ownerOf",
        args: [args.agentId],
      }),
    ),
    withContractRevert(
      readContract(client, {
        abi: identityRegistryAbi,
        address,
        functionName: "tokenURI",
        args: [args.agentId],
      }),
    ),
    withContractRevert(
      readContract(client, {
        abi: identityRegistryAbi,
        address,
        functionName: "getAgentWallet",
        args: [args.agentId],
      }),
    ),
  ]);
  return {
    agentId: args.agentId,
    owner,
    agentUri,
    agentWallet: wallet === zeroAddress ? null : wallet,
  };
}

export async function getAgentUri(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<string> {
  return withContractRevert(
    readContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "tokenURI",
      args: [args.agentId],
    }),
  );
}

export async function getAgentOwner(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<Address> {
  return withContractRevert(
    readContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "ownerOf",
      args: [args.agentId],
    }),
  );
}

/** The agent wallet, `null` when unset (zero address). */
export async function getAgentWallet(
  client: Client,
  args: Args & { agentId: bigint },
  // oxlint-disable-next-line typescript/no-restricted-types -- spec: unset wallets read back as null.
): Promise<Address | null> {
  const wallet = await withContractRevert(
    readContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "getAgentWallet",
      args: [args.agentId],
    }),
  );
  return wallet === zeroAddress ? null : wallet;
}

export async function getMetadata(
  client: Client,
  args: Args & { agentId: bigint; key: string },
): Promise<Hex> {
  return withContractRevert(
    readContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "getMetadata",
      args: [args.agentId, args.key],
    }),
  );
}

export async function isAuthorizedOrOwner(
  client: Client,
  args: Args & { spender: Address; agentId: bigint },
): Promise<boolean> {
  return withContractRevert(
    readContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "isAuthorizedOrOwner",
      args: [args.spender, args.agentId],
    }),
  );
}

export async function getApproved(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<Address> {
  return withContractRevert(
    readContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "getApproved",
      args: [args.agentId],
    }),
  );
}

export async function isApprovedForAll(
  client: Client,
  args: Args & { owner: Address; operator: Address },
): Promise<boolean> {
  return withContractRevert(
    readContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "isApprovedForAll",
      args: [args.owner, args.operator],
    }),
  );
}

export async function getIdentityVersion(client: Client, args: Args = {}): Promise<string> {
  return withContractRevert(
    readContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "getVersion",
    }),
  );
}

function checkMetadataKey(key: string): void {
  if (key === "agentWallet") {
    throw new ClientGuardError("ReservedMetadataKey", 'metadata key "agentWallet" is reserved');
  }
}

/** `register()`/`register(string)`/`register(string, metadata)` — returns the tx hash. */
export async function registerAgent(
  client: Client,
  args: Args & {
    agentUri?: string;
    metadata?: ReadonlyArray<{ key: string; value: Hex }>;
  },
): Promise<Hash> {
  const address = registry(client, args);
  const metadata = args.metadata ?? [];
  for (const entry of metadata) {
    checkMetadataKey(entry.key);
  }
  if (metadata.length > 0) {
    return withContractRevert(
      writeContract(client, {
        abi: identityRegistryAbi,
        address,
        functionName: "register",
        args: [
          args.agentUri ?? "",
          metadata.map((m) => ({ metadataKey: m.key, metadataValue: m.value })),
        ],
        account: sender(client, args.account),
        chain: client.chain,
      }),
    );
  }
  if (args.agentUri !== undefined) {
    return withContractRevert(
      writeContract(client, {
        abi: identityRegistryAbi,
        address,
        functionName: "register",
        args: [args.agentUri],
        account: sender(client, args.account),
        chain: client.chain,
      }),
    );
  }
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address,
      functionName: "register",
      args: [],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

export async function setAgentUri(
  client: Client,
  args: Args & { agentId: bigint; agentUri: string },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "setAgentURI",
      args: [args.agentId, args.agentUri],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

export async function setMetadata(
  client: Client,
  args: Args & { agentId: bigint; key: string; value: Hex },
): Promise<Hash> {
  checkMetadataKey(args.key);
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "setMetadata",
      args: [args.agentId, args.key, args.value],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `deadline` checked against the latest block timestamp (±5 minutes), like the contract. */
export async function setAgentWallet(
  client: Client,
  args: Args & { agentId: bigint; newWallet: Address; deadline: bigint; signature: Hex },
): Promise<Hash> {
  const block = await getBlock(client);
  assertAgentWalletDeadline(args.deadline, block.timestamp);
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "setAgentWallet",
      args: [args.agentId, args.newWallet, args.deadline, args.signature],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

export async function unsetAgentWallet(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "unsetAgentWallet",
      args: [args.agentId],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `transferFrom` — `from` defaults to the sending account. */
export async function transferAgent(
  client: Client,
  args: Args & { to: Address; agentId: bigint; from?: Address },
): Promise<Hash> {
  const from = await readContract(client, {
    abi: identityRegistryAbi,
    functionName: "ownerOf",
    args: [args.agentId],
    address: registry(client, args),
  });
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "transferFrom",
      args: [from, args.to, args.agentId],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

export async function approveAgent(
  client: Client,
  args: Args & { to: Address; agentId: bigint },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "approve",
      args: [args.to, args.agentId],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

export async function setAgentOperator(
  client: Client,
  args: Args & { operator: Address; approved: boolean },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: identityRegistryAbi,
      address: registry(client, args),
      functionName: "setApprovalForAll",
      args: [args.operator, args.approved],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/**
 * Signs an `AgentWalletSet` typed data with the _new wallet's_ client — the signature for
 * `setAgentWallet`/`unsetAgentWallet` must come from the wallet being set.
 */
export async function signAgentWallet(
  client: Client,
  args: {
    chainId: number;
    identityRegistry: Address;
    agentId: bigint;
    owner: Address;
    deadline: bigint;
  },
): Promise<Hex> {
  return signTypedData(client, {
    account: sender(client),
    ...agentWalletTypedData({
      chainId: args.chainId,
      identityRegistry: args.identityRegistry,
      message: {
        agentId: args.agentId,
        newWallet: senderAddress(client),
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
