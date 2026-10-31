import type { Account, Address, Client, Hash, Hex } from "viem";
import { zeroHash } from "viem";
import { parseAccount } from "viem/accounts";
import { readContract, writeContract } from "viem/actions";

import { reputationRegistryAbi } from "../abi/index.ts";
import { ClientGuardError, MissingEventError } from "../core/errors.ts";
import type { FeedbackValue } from "../core/feedback-value.ts";
import { withContractRevert } from "../core/revert-map.ts";
import type { Deployment } from "../deployments/deployment.ts";
import { resolveRegistryAddress } from "../deployments/resolve.ts";
import { decodeErc8004Log } from "../events/index.ts";

type Args = {
  address?: Address | undefined;
  deployment?: Deployment | undefined;
  account?: Account | Address | undefined;
};

function registry(client: Pick<Client, "chain">, args: Args): Address {
  return resolveRegistryAddress(client, "reputation", args.address, args.deployment);
}

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

/** One `(value, valueDecimals)` pair from `readFeedback`. */
export type Feedback = {
  value: FeedbackValue;
  tag1: string;
  tag2: string;
  isRevoked: boolean;
};

/** One zipped entry from `readAllFeedback`. */
export type FeedbackEntry = {
  client: Address;
  feedbackIndex: bigint;
  value: FeedbackValue;
  tag1: string;
  tag2: string;
  isRevoked: boolean;
};

export async function readFeedback(
  client: Client,
  args: Args & { agentId: bigint; client: Address; feedbackIndex: bigint },
): Promise<Feedback> {
  const [value, valueDecimals, tag1, tag2, isRevoked] = await withContractRevert(
    readContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "readFeedback",
      args: [args.agentId, args.client, args.feedbackIndex],
    }),
  );
  return { value: { value, decimals: valueDecimals }, tag1, tag2, isRevoked };
}

export async function readAllFeedback(
  client: Client,
  args: Args & {
    agentId: bigint;
    clients?: ReadonlyArray<Address>;
    tag1?: string;
    tag2?: string;
    includeRevoked?: boolean;
  },
): Promise<FeedbackEntry[]> {
  const result = await withContractRevert(
    readContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "readAllFeedback",
      args: [
        args.agentId,
        args.clients ?? [],
        args.tag1 ?? "",
        args.tag2 ?? "",
        args.includeRevoked ?? false,
      ],
    }),
  );
  const [clients, feedbackIndexes, values, decimals, tag1s, tag2s, revokedStatuses] = result;
  return clients.map((entry, i) => ({
    client: entry,
    feedbackIndex: feedbackIndexes[i] ?? 0n,
    value: { value: values[i] ?? 0n, decimals: decimals[i] ?? 0 },
    tag1: tag1s[i] ?? "",
    tag2: tag2s[i] ?? "",
    isRevoked: revokedStatuses[i] ?? false,
  }));
}

/** `getSummary` — `clients` must be non-empty (`ClientAddressesRequired`). */
export async function getReputationSummary(
  client: Client,
  args: Args & { agentId: bigint; clients: ReadonlyArray<Address>; tag1?: string; tag2?: string },
): Promise<{ count: bigint; value: FeedbackValue }> {
  if (args.clients.length === 0) {
    throw new ClientGuardError("ClientAddressesRequired", "getSummary: clients must be non-empty");
  }
  const [count, summaryValue, summaryValueDecimals] = await withContractRevert(
    readContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "getSummary",
      args: [args.agentId, [...args.clients], args.tag1 ?? "", args.tag2 ?? ""],
    }),
  );
  return { count, value: { value: summaryValue, decimals: summaryValueDecimals } };
}

export async function getClients(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<ReadonlyArray<Address>> {
  return withContractRevert(
    readContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "getClients",
      args: [args.agentId],
    }),
  );
}

export async function getLastIndex(
  client: Client,
  args: Args & { agentId: bigint; client: Address },
): Promise<bigint> {
  return withContractRevert(
    readContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "getLastIndex",
      args: [args.agentId, args.client],
    }),
  );
}

/** `client`/`feedbackIndex`/`responders` are paired: all set or all unset. */
export async function getResponseCount(
  client: Client,
  args: Args & {
    agentId: bigint;
    client?: Address;
    feedbackIndex?: bigint;
    responders?: ReadonlyArray<Address>;
  },
): Promise<bigint> {
  return withContractRevert(
    readContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "getResponseCount",
      args: [
        args.agentId,
        args.client ?? "0x0000000000000000000000000000000000000000",
        args.feedbackIndex ?? 0n,
        args.responders ? [...args.responders] : [],
      ],
    }),
  );
}

export async function getReputationVersion(client: Client, args: Args = {}): Promise<string> {
  return withContractRevert(
    readContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "getVersion",
    }),
  );
}

export async function giveFeedback(
  client: Client,
  args: Args & {
    agentId: bigint;
    value: FeedbackValue;
    tag1?: string;
    tag2?: string;
    endpoint?: string;
    feedbackUri?: string;
    feedbackHash?: Hex;
  },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "giveFeedback",
      args: [
        args.agentId,
        args.value.value,
        args.value.decimals,
        args.tag1 ?? "",
        args.tag2 ?? "",
        args.endpoint ?? "",
        args.feedbackUri ?? "",
        args.feedbackHash ?? zeroHash,
      ],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

export async function revokeFeedback(
  client: Client,
  args: Args & { agentId: bigint; feedbackIndex: bigint },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "revokeFeedback",
      args: [args.agentId, args.feedbackIndex],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `responseUri` must be non-empty (`EmptyResponseUri`). */
export async function appendResponse(
  client: Client,
  args: Args & {
    agentId: bigint;
    client: Address;
    feedbackIndex: bigint;
    responseUri: string;
    responseHash?: Hex;
  },
): Promise<Hash> {
  if (args.responseUri === "") {
    throw new ClientGuardError("EmptyResponseUri", "appendResponse: responseUri must be non-empty");
  }
  return withContractRevert(
    writeContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, args),
      functionName: "appendResponse",
      args: [
        args.agentId,
        args.client,
        args.feedbackIndex,
        args.responseUri,
        args.responseHash ?? zeroHash,
      ],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/**
 * The `NewFeedback` event's `feedbackIndex` from a giveFeedback receipt; `MissingEvent` when
 * absent.
 */
export function parseFeedbackIndex(
  logs: ReadonlyArray<{ topics: ReadonlyArray<Hex>; data: Hex }>,
): bigint {
  for (const log of logs) {
    const decoded = decodeErc8004Log(log);
    if (decoded?.eventName === "NewFeedback") {
      return decoded.args.feedbackIndex;
    }
  }
  throw new MissingEventError("NewFeedback");
}
