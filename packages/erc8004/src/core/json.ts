import { JsonDocumentError } from "./errors.ts";

/** `Record<string, unknown>` minus `Array`/primitives — the JSON object shape. */
export type JsonObject = Record<string, unknown>;

export function isRecord(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function notAnObject(value: unknown): JsonDocumentError {
  return new JsonDocumentError(
    "NotAnObject",
    `document must be a JSON object, got ${typeof value}`,
  );
}

export function fieldPath(parent: string | undefined, key: string): string {
  return parent === undefined || parent === "" ? key : `${parent}.${key}`;
}

export function indexPath(parent: string, index: number): string {
  return `${parent}[${index}]`;
}

function invalidField(field: string, expectation: string): JsonDocumentError {
  return new JsonDocumentError("InvalidField", `${field} must ${expectation}`, field);
}

function missing(obj: JsonObject, key: string): boolean {
  return obj[key] === undefined;
}

/** Required `obj[key]` as string. */
export function requiredString(obj: JsonObject, parent: string, key: string): string {
  const field = fieldPath(parent, key);
  const value = obj[key];
  if (value === undefined) {
    throw invalidField(field, "present");
  }
  if (typeof value !== "string") {
    throw invalidField(field, "a string");
  }
  return value;
}

/** Optional `obj[key]` as string. */
export function optionalString(obj: JsonObject, parent: string, key: string): string | undefined {
  const value = obj[key];
  if (value === undefined) {
    return undefined;
  }
  if (typeof value !== "string") {
    throw invalidField(fieldPath(parent, key), "a string");
  }
  return value;
}

/** Optional `obj[key]` as boolean. */
export function optionalBool(obj: JsonObject, parent: string, key: string): boolean | undefined {
  const value = obj[key];
  if (value === undefined) {
    return undefined;
  }
  if (typeof value !== "boolean") {
    throw invalidField(fieldPath(parent, key), "a boolean");
  }
  return value;
}

/** Optional `obj[key]` as a JSON array of strings. */
export function optionalStringArray(
  obj: JsonObject,
  parent: string,
  key: string,
): string[] | undefined {
  const field = fieldPath(parent, key);
  const value = obj[key];
  if (value === undefined) {
    return undefined;
  }
  if (!Array.isArray(value)) {
    throw invalidField(field, "an array of strings");
  }
  const out: string[] = [];
  for (const item of value) {
    if (typeof item !== "string") {
      throw invalidField(field, "an array of strings");
    }
    out.push(item);
  }
  return out;
}

/** Optional `obj[key]` as a JSON object — `null` and arrays are not objects. */
export function optionalObject(
  obj: JsonObject,
  parent: string,
  key: string,
): JsonObject | undefined {
  const value = obj[key];
  if (value === undefined) {
    return undefined;
  }
  if (!isRecord(value)) {
    throw invalidField(fieldPath(parent, key), "an object");
  }
  return value;
}

const MAX_SAFE = BigInt("9007199254740991"); // 2^53 - 1
const UINT256_MAX = 2n ** 256n - 1n;

function isCanonicalDecimal(s: string): boolean {
  return s === "0" || /^[1-9][0-9]*$/.test(s);
}

/**
 * Required `obj[key]`: JSON number integral in `0..=2^53-1` (any spelling) or canonical decimal
 * string ≤ uint256 max.
 */
export function requiredUint(obj: JsonObject, parent: string, key: string): bigint {
  const field = fieldPath(parent, key);
  const value = obj[key];
  if (value === undefined) {
    throw invalidField(field, "present");
  }
  if (typeof value === "number") {
    if (!Number.isInteger(value) || value < 0 || value > Number.MAX_SAFE_INTEGER) {
      throw invalidField(field, "a JSON integer in 0..=2^53-1 or a canonical decimal string");
    }
    return BigInt(value);
  }
  if (typeof value === "string" && isCanonicalDecimal(value)) {
    const parsed = BigInt(value);
    if (parsed <= UINT256_MAX) {
      return parsed;
    }
  }
  throw invalidField(field, "a JSON integer in 0..=2^53-1 or a canonical decimal string");
}

/**
 * Required `obj[key]`: JSON number integral within ±(2^53-1) (any spelling) or canonical signed
 * decimal string.
 */
export function requiredInt(obj: JsonObject, parent: string, key: string): bigint {
  const field = fieldPath(parent, key);
  const value = obj[key];
  if (value === undefined) {
    throw invalidField(field, "present");
  }
  if (typeof value === "number") {
    if (
      !Number.isInteger(value) ||
      value < -Number.MAX_SAFE_INTEGER ||
      value > Number.MAX_SAFE_INTEGER
    ) {
      throw invalidField(
        field,
        "a JSON integer within ±(2^53-1) or a canonical signed decimal string",
      );
    }
    return BigInt(value);
  }
  if (typeof value === "string" && /^-?(0|[1-9][0-9]*)$/.test(value)) {
    return BigInt(value);
  }
  throw invalidField(field, "a JSON integer within ±(2^53-1) or a canonical signed decimal string");
}

/** Required `obj[key]` as a JSON integer in `0..=18` — for `valueDecimals`. */
export function requiredDecimals(obj: JsonObject, parent: string, key: string): number {
  const field = fieldPath(parent, key);
  const value = obj[key];
  if (value === undefined) {
    throw invalidField(field, "present");
  }
  if (typeof value !== "number" || !Number.isInteger(value) || value < 0 || value > 18) {
    throw invalidField(field, "a JSON integer in 0..=18");
  }
  return value;
}

/** Bigints serialize as JSON numbers up to 2^53-1, decimal strings above. */
export function uintToJson(value: bigint): number | string {
  return value <= MAX_SAFE ? Number(value) : value.toString();
}

export function signedToJson(value: bigint): number | string {
  return value >= -MAX_SAFE && value <= MAX_SAFE ? Number(value) : value.toString();
}

/** Keys of `obj` not in `known`, preserving their values. */
export function extras(obj: JsonObject, known: ReadonlySet<string>): JsonObject {
  const out: JsonObject = {};
  for (const [key, value] of Object.entries(obj)) {
    if (!known.has(key)) {
      out[key] = value;
    }
  }
  return out;
}

export { missing };
