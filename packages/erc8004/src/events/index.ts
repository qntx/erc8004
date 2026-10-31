import { decodeEventLog, AbiEventSignatureNotFoundError } from "viem";
import type { DecodeEventLogReturnType, Hex } from "viem";

import { protocolEventsAbi } from "../abi/index.ts";
import { EventDecodeError } from "../core/errors.ts";

/** One decoded protocol event — viem's typed `{ eventName, args }` union over the 13 events. */
export type Erc8004Event = DecodeEventLogReturnType<typeof protocolEventsAbi>;

/**
 * Decodes one protocol log by `topic0`: unknown topics (including admin events) return `null`;
 * malformed data under a known topic throws {@link EventDecodeError} (`MalformedLog`). Indexed
 * `string` params surface as their topic hash (bytes32 hex), matching the on-chain value.
 */
export function decodeErc8004Log(log: {
  topics: ReadonlyArray<Hex>;
  data: Hex;
  // oxlint-disable-next-line typescript/no-restricted-types -- spec: unknown or admin topics decode to null.
}): Erc8004Event | null {
  const [topic0, ...rest] = log.topics;
  if (topic0 === undefined) {
    return null;
  }
  const topics: [Hex, ...Hex[]] = [topic0, ...rest];
  try {
    const decoded = decodeEventLog({
      abi: protocolEventsAbi,
      topics,
      data: log.data,
      strict: true,
    });
    // viem tolerates extra indexed topics; the normal form requires an exact count.
    const eventDef = protocolEventsAbi.find(
      (entry) => entry.type === "event" && entry.name === decoded.eventName,
    );
    const indexedCount = eventDef?.inputs.filter((input) => input.indexed).length ?? -1;
    if (indexedCount < 0 || log.topics.length - 1 !== indexedCount) {
      throw new EventDecodeError(
        `${decoded.eventName}: expected ${indexedCount} indexed topics, got ${log.topics.length - 1}`,
      );
    }
    return decoded;
  } catch (error) {
    if (error instanceof AbiEventSignatureNotFoundError) {
      return null;
    }
    throw error instanceof EventDecodeError
      ? error
      : new EventDecodeError(error instanceof Error ? error.message : String(error));
  }
}
