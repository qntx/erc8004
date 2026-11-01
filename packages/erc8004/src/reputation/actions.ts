import type { Address, Client, Hash, Hex } from "viem";
import { zeroAddress, zeroHash } from "viem";
import { writeContract } from "viem/actions";

import { reputationRegistryAbi } from "../abi/index.ts";
import { ClientGuardError, InvalidResponseError, MissingEventError } from "../core/errors.ts";
import type { FeedbackValue } from "../core/feedback-value.ts";
import { feedbackValue } from "../core/feedback-value.ts";
import { withContractRevert } from "../core/revert-map.ts";
import { read, registry, sender } from "../deployments/resolve.ts";
import type { ActionArgs } from "../deployments/resolve.ts";
import { decodeErc8004Log } from "../events/index.ts";

type Args = ActionArgs;

/** One `(value, valueDecimals)` pair from `readFeedback`. */
export type Feedback = {
  value: FeedbackValue;
  tag1: string;
  tag2: string;
  isRevoked: boolean;
};

/** One zipped entry from `readAllFeedback`. */
export type FeedbackEntry = {
  clientAddress: Address;
  feedbackIndex: bigint;
  value: FeedbackValue;
  tag1: string;
  tag2: string;
  isRevoked: boolean;
};

/** `getSummary` aggregate. */
export type ReputationSummary = {
  count: bigint;
  value: FeedbackValue;
};

/** On-chain `(value, valueDecimals)` pair checked against `FeedbackValue` bounds. */
function checkedValue(value: bigint, decimals: number): FeedbackValue {
  try {
    return feedbackValue(value, decimals);
  } catch (error) {
    throw new InvalidResponseError(
      `on-chain feedback value out of bounds: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

/** `readFeedback` — one feedback row by client/index. */
export async function readFeedback(
  client: Client,
  args: Args & { agentId: bigint; clientAddress: Address; feedbackIndex: bigint },
): Promise<Feedback> {
  const [value, valueDecimals, tag1, tag2, isRevoked] = await read(
    client,
    registry(client, "reputation", args),
    reputationRegistryAbi,
    "readFeedback",
    [args.agentId, args.clientAddress, args.feedbackIndex],
  );
  return { value: checkedValue(value, valueDecimals), tag1, tag2, isRevoked };
}

/**
 * `readAllFeedback` — parallel arrays zipped into entries; `InvalidResponseError` when the contract
 * returns mismatched lengths or out-of-bounds values.
 */
export async function readAllFeedback(
  client: Client,
  args: Args & {
    agentId: bigint;
    clientAddresses?: ReadonlyArray<Address>;
    tag1?: string;
    tag2?: string;
    includeRevoked?: boolean;
  },
): Promise<FeedbackEntry[]> {
  const result = await read(
    client,
    registry(client, "reputation", args),
    reputationRegistryAbi,
    "readAllFeedback",
    [
      args.agentId,
      [...(args.clientAddresses ?? [])],
      args.tag1 ?? "",
      args.tag2 ?? "",
      args.includeRevoked ?? false,
    ],
  );
  const [clients, feedbackIndexes, values, decimals, tag1s, tag2s, revokedStatuses] = result;
  const n = clients.length;
  if (
    feedbackIndexes.length !== n ||
    values.length !== n ||
    decimals.length !== n ||
    tag1s.length !== n ||
    tag2s.length !== n ||
    revokedStatuses.length !== n
  ) {
    throw new InvalidResponseError(
      `readAllFeedback: mismatched array lengths (${n}, ${feedbackIndexes.length}, ${values.length}, ${decimals.length}, ${tag1s.length}, ${tag2s.length}, ${revokedStatuses.length})`,
    );
  }
  return clients.map((clientAddress, i) => ({
    clientAddress,
    // oxlint-disable-next-line typescript/no-non-null-assertion -- length verified above.
    feedbackIndex: feedbackIndexes[i]!,
    // oxlint-disable-next-line typescript/no-non-null-assertion -- length verified above.
    value: checkedValue(values[i]!, decimals[i]!),
    // oxlint-disable-next-line typescript/no-non-null-assertion -- length verified above.
    tag1: tag1s[i]!,
    // oxlint-disable-next-line typescript/no-non-null-assertion -- length verified above.
    tag2: tag2s[i]!,
    // oxlint-disable-next-line typescript/no-non-null-assertion -- length verified above.
    isRevoked: revokedStatuses[i]!,
  }));
}

/**
 * `getSummary` aggregate over `clientAddresses` — `ClientGuardError` `ClientAddressesRequired` for
 * an empty list.
 */
export async function getReputationSummary(
  client: Client,
  args: Args & {
    agentId: bigint;
    clientAddresses: ReadonlyArray<Address>;
    tag1?: string;
    tag2?: string;
  },
): Promise<ReputationSummary> {
  if (args.clientAddresses.length === 0) {
    throw new ClientGuardError("ClientAddressesRequired", "getSummary: clients must be non-empty");
  }
  const [count, summaryValue, summaryValueDecimals] = await read(
    client,
    registry(client, "reputation", args),
    reputationRegistryAbi,
    "getSummary",
    [args.agentId, [...args.clientAddresses], args.tag1 ?? "", args.tag2 ?? ""],
  );
  return { count, value: checkedValue(summaryValue, summaryValueDecimals) };
}

/** `getClients` — every client that left feedback for `agentId`. */
export async function getClients(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<ReadonlyArray<Address>> {
  return read(client, registry(client, "reputation", args), reputationRegistryAbi, "getClients", [
    args.agentId,
  ]);
}

/** `getLastIndex` — a client's latest feedback index for `agentId`. */
export async function getLastIndex(
  client: Client,
  args: Args & { agentId: bigint; clientAddress: Address },
): Promise<bigint> {
  return read(client, registry(client, "reputation", args), reputationRegistryAbi, "getLastIndex", [
    args.agentId,
    args.clientAddress,
  ]);
}

/**
 * `getResponseCount` — contract semantics: zero `clientAddress` → every client; `feedbackIndex` 0 →
 * every feedback of that client; empty `responders` → every responder.
 */
export async function getResponseCount(
  client: Client,
  args: Args & {
    agentId: bigint;
    clientAddress?: Address;
    feedbackIndex?: bigint;
    responders?: ReadonlyArray<Address>;
  },
): Promise<bigint> {
  return read(
    client,
    registry(client, "reputation", args),
    reputationRegistryAbi,
    "getResponseCount",
    [
      args.agentId,
      args.clientAddress ?? zeroAddress,
      args.feedbackIndex ?? 0n,
      [...(args.responders ?? [])],
    ],
  );
}

/** `getVersion` of the reputation registry. */
export async function getReputationVersion(client: Client, args: Args = {}): Promise<string> {
  return read(
    client,
    registry(client, "reputation", args),
    reputationRegistryAbi,
    "getVersion",
    [],
  );
}

/** `giveFeedback` write — `value` is a pre-validated `FeedbackValue`. */
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
      address: registry(client, "reputation", args),
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

/** `revokeFeedback` write; reverts `AlreadyRevoked` on a second call. */
export async function revokeFeedback(
  client: Client,
  args: Args & { agentId: bigint; feedbackIndex: bigint },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: reputationRegistryAbi,
      address: registry(client, "reputation", args),
      functionName: "revokeFeedback",
      args: [args.agentId, args.feedbackIndex],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `appendResponse` write; `ClientGuardError` `EmptyResponseUri` for an empty URI. */
export async function appendResponse(
  client: Client,
  args: Args & {
    agentId: bigint;
    clientAddress: Address;
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
      address: registry(client, "reputation", args),
      functionName: "appendResponse",
      args: [
        args.agentId,
        args.clientAddress,
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
