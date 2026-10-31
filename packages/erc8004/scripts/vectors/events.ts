import {
  encodeAbiParameters,
  getAddress,
  keccak256,
  pad,
  stringToHex,
  toEventSelector,
} from "viem";
import type { Hex } from "viem";

import {
  identityRegistryAbi,
  reputationRegistryAbi,
  validationRegistryAbi,
} from "../../src/abi/index.ts";

const ABIS = {
  identity: identityRegistryAbi,
  reputation: reputationRegistryAbi,
  validation: validationRegistryAbi,
} as const;

type AbiRegistry = (typeof ABIS)[keyof typeof ABIS];
type AbiEvent = {
  type: string;
  name: string;
  inputs: Array<{ type: string; name?: string; indexed?: boolean }>;
};

const OWNER = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";
const OTHER = "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359";
const THIRD = "0x8004A169FB4a3325136EB29fA0ceB6D2e539a432";
const H32 = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

function eventOf(abi: AbiRegistry, name: string): AbiEvent {
  for (const entry of abi) {
    if (entry.type === "event" && entry.name === name) {
      return entry as unknown as AbiEvent;
    }
  }
  throw new Error(`event ${name} not found`);
}

// Topics are built manually: topic0 = keccak signature, indexed params ABI-
// encoded into 32-byte topics (string params keep their keccak hash).
function indexedTopic(param: AbiEvent["inputs"][number], value: unknown): Hex {
  if (param.type === "string") {
    return keccak256(stringToHex(value as string));
  }
  if (param.type === "address") {
    return pad(getAddress(value as string).toLowerCase() as Hex);
  }
  if (param.type === "bool") {
    return `0x${"00".repeat(31)}0${value === true ? "1" : "0"}`;
  }
  const encoded = encodeAbiParameters([{ type: param.type }], [value]);
  return encoded.slice(0, 66) as Hex;
}

function buildLog(
  abi: AbiRegistry,
  name: string,
  args: Record<string, unknown>,
): { topics: Hex[]; data: Hex } {
  const event = eventOf(abi, name);
  const signature = `${event.name}(${event.inputs.map((i) => i.type).join(",")})`;
  const topics: Hex[] = [toEventSelector(signature)];
  const indexed = event.inputs.filter((i) => i.indexed === true);
  for (const param of indexed) {
    if (param.name === undefined) {
      throw new Error("unnamed indexed param");
    }
    topics.push(indexedTopic(param, args[param.name]));
  }
  const nonIndexed = event.inputs.filter((i) => i.indexed !== true);
  const data =
    nonIndexed.length === 0
      ? ("0x" as Hex)
      : encodeAbiParameters(
          nonIndexed.map((i) => ({ type: i.type, name: i.name })),
          nonIndexed.map((i) => args[i.name ?? ""]),
        );
  return { topics, data };
}

function normalize(value: unknown, type: string): unknown {
  if (type.startsWith("uint") || type.startsWith("int")) {
    return BigInt(value as bigint | number | string).toString();
  }
  if (type === "address") {
    return getAddress(value as string);
  }
  if (type === "bytes" || type.startsWith("bytes")) {
    return (value as string).toLowerCase();
  }
  return value;
}

function expectedArgs(
  abi: AbiRegistry,
  name: string,
  args: Record<string, unknown>,
  topics: Hex[],
): Record<string, unknown> {
  const event = eventOf(abi, name);
  const out: Record<string, unknown> = {};
  const indexed = event.inputs.filter((i) => i.indexed === true);
  for (const [i, param] of indexed.entries()) {
    if (param.name === undefined) {
      throw new Error("unnamed indexed param");
    }
    const topic = topics[i + 1];
    if (topic === undefined) {
      throw new Error("missing topic");
    }
    if (param.type === "string") {
      out[param.name] = topic.toLowerCase();
    } else if (param.type === "address") {
      out[param.name] = getAddress(`0x${topic.slice(-40)}`);
    } else if (param.type.startsWith("bytes")) {
      out[param.name] = topic.toLowerCase();
    } else {
      out[param.name] = normalize(BigInt(topic), param.type);
    }
  }
  for (const param of event.inputs) {
    if (param.indexed === true || param.name === undefined) {
      continue;
    }
    out[param.name] = normalize(args[param.name], param.type);
  }
  return out;
}

export function build(): { capability: string; cases: unknown[] } {
  const cases: Array<Record<string, unknown>> = [];
  const push = (abi: AbiRegistry, name: string, args: Record<string, unknown>) => {
    const log = buildLog(abi, name, args);
    cases.push({
      input: { topics: log.topics, data: log.data },
      output: { event: name, args: expectedArgs(abi, name, args, log.topics) },
    });
    return log;
  };

  // Protocol events — Identity.
  push(ABIS.identity, "Registered", {
    agentId: 1n,
    agentURI: "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
    owner: OWNER,
  });
  push(ABIS.identity, "Registered", { agentId: 0n, agentURI: "", owner: OTHER });
  push(ABIS.identity, "URIUpdated", {
    agentId: 5n,
    newURI: "data:application/json;base64,eyJ9",
    updatedBy: THIRD,
  });
  push(ABIS.identity, "MetadataSet", {
    agentId: 7n,
    indexedMetadataKey: "wallet",
    metadataKey: "wallet",
    metadataValue: "0xdeadbeef",
  });
  push(ABIS.identity, "Transfer", { from: OWNER, to: OTHER, tokenId: 9n });
  push(ABIS.identity, "Transfer", {
    from: "0x0000000000000000000000000000000000000000",
    to: OWNER,
    tokenId: 115792089237316195423570985008687907853269984665640564039457584007913129639935n,
  });
  push(ABIS.identity, "Approval", { owner: OWNER, approved: OTHER, tokenId: 3n });
  push(ABIS.identity, "ApprovalForAll", { owner: OWNER, operator: THIRD, approved: true });
  push(ABIS.identity, "ApprovalForAll", { owner: OTHER, operator: OWNER, approved: false });
  push(ABIS.identity, "MetadataUpdate", { _tokenId: 11n });
  push(ABIS.identity, "BatchMetadataUpdate", { _fromTokenId: 10n, _toTokenId: 20n });

  // Protocol events — Reputation.
  push(ABIS.reputation, "NewFeedback", {
    agentId: 5n,
    clientAddress: OWNER,
    feedbackIndex: 1n,
    value: 9970n,
    valueDecimals: 2,
    indexedTag1: "quality",
    tag1: "quality",
    tag2: "speed",
    endpoint: "https://example.com",
    feedbackURI: "ipfs://QmFeedback",
    feedbackHash: H32,
  });
  push(ABIS.reputation, "NewFeedback", {
    agentId: 0n,
    clientAddress: OTHER,
    feedbackIndex: 0n,
    value: -5n,
    valueDecimals: 0,
    indexedTag1: "",
    tag1: "",
    tag2: "",
    endpoint: "",
    feedbackURI: "",
    feedbackHash: `0x${"00".repeat(32)}`,
  });
  push(ABIS.reputation, "FeedbackRevoked", {
    agentId: 5n,
    clientAddress: OWNER,
    feedbackIndex: 3n,
  });
  push(ABIS.reputation, "ResponseAppended", {
    agentId: 5n,
    clientAddress: OWNER,
    feedbackIndex: 1n,
    responder: THIRD,
    responseURI: "ipfs://QmResponse",
    responseHash: H32,
  });

  // Protocol events — Validation.
  push(ABIS.validation, "ValidationRequest", {
    validatorAddress: OWNER,
    agentId: 12n,
    requestURI: "ipfs://QmRequest",
    requestHash: H32,
  });
  push(ABIS.validation, "ValidationResponse", {
    validatorAddress: OTHER,
    agentId: 12n,
    requestHash: H32,
    response: 100,
    responseURI: "ipfs://QmResp",
    responseHash: `0x${"ff".repeat(32)}`,
    tag: "success",
  });

  // Admin events and unknown topic0 → null.
  for (const [abi, name, args] of [
    [ABIS.identity, "Initialized", { version: 2n }],
    [ABIS.identity, "OwnershipTransferred", { previousOwner: OWNER, newOwner: OTHER }],
    [ABIS.identity, "Upgraded", { implementation: THIRD }],
    [ABIS.identity, "EIP712DomainChanged", {}],
    [ABIS.reputation, "Initialized", { version: 1n }],
    [ABIS.validation, "OwnershipTransferred", { previousOwner: OWNER, newOwner: THIRD }],
  ] as const) {
    const log = buildLog(abi, name, args);
    cases.push({ input: { topics: log.topics, data: log.data }, output: null });
  }
  cases.push(
    { input: { topics: [`0x${"aa".repeat(32)}`], data: "0x" }, output: null },
    { input: { topics: [], data: "0x" }, output: null },
    { input: { topics: [`0x${"aa".repeat(32)}`], data: "0xdeadbeef" }, output: null },
  );

  // Malformed logs under known topics → MalformedLog.
  const registered = buildLog(ABIS.identity, "Registered", {
    agentId: 1n,
    agentURI: "ipfs://x",
    owner: OWNER,
  });
  cases.push(
    { input: { topics: registered.topics, data: "0x" }, error: "MalformedLog" },
    { input: { topics: registered.topics, data: "0x1234" }, error: "MalformedLog" },
    {
      input: { topics: [registered.topics[0]], data: registered.data },
      error: "MalformedLog",
    },
    {
      input: { topics: [...registered.topics, `0x${"11".repeat(32)}`], data: registered.data },
      error: "MalformedLog",
    },
  );
  const transfer = buildLog(ABIS.identity, "Transfer", { from: OWNER, to: OTHER, tokenId: 1n });
  cases.push({ input: { topics: transfer.topics.slice(0, 3), data: "0x" }, error: "MalformedLog" });

  return { capability: "events.decode", cases };
}
