import { JsonDocumentError } from "../core/errors.ts";
import { feedbackValue } from "../core/feedback-value.ts";
import type { FeedbackValue } from "../core/feedback-value.ts";
import {
  extras,
  isRecord,
  notAnObject,
  optionalObject,
  optionalString,
  requiredDecimals,
  requiredInt,
  requiredString,
  requiredUint,
  signedToJson,
  uintToJson,
} from "../core/json.ts";
import type { JsonObject } from "../core/json.ts";

/**
 * Lenient, lossless off-chain feedback file: required fields typed,
 * `mcp`/`a2a`/`oasf`/`proofOfPayment` kept as JSON maps, unknown keys preserved in `extra`.
 * `value`/`valueDecimals` are one validated {@link FeedbackValue}.
 */
export type FeedbackFile = {
  agentRegistry: string;
  agentId: bigint;
  clientAddress: string;
  createdAt: string;
  value: FeedbackValue;
  tag1?: string | undefined;
  tag2?: string | undefined;
  endpoint?: string | undefined;
  mcp?: JsonObject | undefined;
  a2a?: JsonObject | undefined;
  oasf?: JsonObject | undefined;
  proofOfPayment?: JsonObject | undefined;
  extra: JsonObject;
};

const FILE_KEYS = new Set([
  "agentRegistry",
  "agentId",
  "clientAddress",
  "createdAt",
  "value",
  "valueDecimals",
  "tag1",
  "tag2",
  "endpoint",
  "mcp",
  "a2a",
  "oasf",
  "proofOfPayment",
]);

/**
 * Parses a feedback file leniently — `value`/`valueDecimals` must form a valid
 * {@link FeedbackValue}; pair failures name `value` or `valueDecimals`.
 */
export function parseFeedbackFile(json: unknown): FeedbackFile {
  if (!isRecord(json)) {
    throw notAnObject(json);
  }
  const agentRegistry = requiredString(json, "", "agentRegistry");
  const agentId = requiredUint(json, "", "agentId");
  const clientAddress = requiredString(json, "", "clientAddress");
  const createdAt = requiredString(json, "", "createdAt");
  const rawValue = requiredInt(json, "", "value");
  const decimals = requiredDecimals(json, "", "valueDecimals");
  let value: FeedbackValue;
  try {
    value = feedbackValue(rawValue, decimals);
  } catch (error) {
    throw new JsonDocumentError(
      "InvalidField",
      error instanceof Error ? error.message : "invalid value/valueDecimals pair",
      error instanceof Error &&
        error.name === "FeedbackValueError" &&
        error.message.includes("decimals")
        ? "valueDecimals"
        : "value",
    );
  }
  return {
    agentRegistry,
    agentId,
    clientAddress,
    createdAt,
    value,
    tag1: optionalString(json, "", "tag1"),
    tag2: optionalString(json, "", "tag2"),
    endpoint: optionalString(json, "", "endpoint"),
    mcp: optionalObject(json, "", "mcp"),
    a2a: optionalObject(json, "", "a2a"),
    oasf: optionalObject(json, "", "oasf"),
    proofOfPayment: optionalObject(json, "", "proofOfPayment"),
    extra: extras(json, FILE_KEYS),
  };
}

/** Builder — validates `value`/`valueDecimals` into a {@link FeedbackValue}. */
export function createFeedbackFile(
  init: Readonly<Omit<FeedbackFile, "value"> & { value: FeedbackValue }>,
): FeedbackFile {
  return { ...init, extra: init.extra };
}

/**
 * Serializes back to a JSON value; `agentId`/`value` are numbers within ±(2^53-1), decimal strings
 * above.
 */
export function feedbackFileToJson(file: FeedbackFile): JsonObject {
  return {
    agentRegistry: file.agentRegistry,
    agentId: uintToJson(file.agentId),
    clientAddress: file.clientAddress,
    createdAt: file.createdAt,
    value: signedToJson(file.value.value),
    valueDecimals: file.value.decimals,
    ...(file.tag1 === undefined ? {} : { tag1: file.tag1 }),
    ...(file.tag2 === undefined ? {} : { tag2: file.tag2 }),
    ...(file.endpoint === undefined ? {} : { endpoint: file.endpoint }),
    ...(file.mcp === undefined ? {} : { mcp: file.mcp }),
    ...(file.a2a === undefined ? {} : { a2a: file.a2a }),
    ...(file.oasf === undefined ? {} : { oasf: file.oasf }),
    ...(file.proofOfPayment === undefined ? {} : { proofOfPayment: file.proofOfPayment }),
    ...file.extra,
  };
}
