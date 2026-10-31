import { FeedbackValueError } from "./errors.ts";

/**
 * `(value: int128, decimals: u8)` — the contract's feedback magnitude; `getSummary` aggregates by
 * the mode of `valueDecimals`.
 */
export type FeedbackValue = {
  readonly value: bigint;
  readonly decimals: number;
};

const MAX_DECIMALS = 18;
const MAX_VALUE = 10n ** 38n;
const DECIMAL = /^-?[0-9]+(\.[0-9]+)?$/;

/** Checked constructor: `decimals <= 18` and `|value| <= 1e38` (contract bounds). */
export function feedbackValue(value: bigint, decimals: number): FeedbackValue {
  if (!Number.isInteger(decimals) || decimals < 0) {
    throw new FeedbackValueError(
      "InvalidDecimals",
      `decimals must be a non-negative integer, got ${decimals}`,
    );
  }
  if (decimals > MAX_DECIMALS) {
    throw new FeedbackValueError("TooManyDecimals", `decimals ${decimals} exceeds ${MAX_DECIMALS}`);
  }
  if (value > MAX_VALUE || value < -MAX_VALUE) {
    throw new FeedbackValueError("ValueOutOfRange", `value ${value} exceeds ±1e38`);
  }
  return { value, decimals };
}

/** `-?[0-9]+(\.[0-9]+)?` — leading zeros allowed; fraction digits kept as written. */
export function parseFeedbackValue(text: string): FeedbackValue {
  if (!DECIMAL.test(text)) {
    throw new FeedbackValueError(
      "InvalidDecimalString",
      `invalid decimal string ${JSON.stringify(text)}`,
    );
  }
  const negative = text.startsWith("-");
  const body = negative ? text.slice(1) : text;
  const dot = body.indexOf(".");
  const digits = dot === -1 ? body : body.slice(0, dot) + body.slice(dot + 1);
  const decimals = dot === -1 ? 0 : body.length - dot - 1;
  if (decimals > MAX_DECIMALS) {
    throw new FeedbackValueError("TooManyDecimals", `decimals ${decimals} exceeds ${MAX_DECIMALS}`);
  }
  let value = BigInt(digits);
  if (negative && value !== 0n) {
    value = -value;
  }
  if (value > MAX_VALUE || value < -MAX_VALUE) {
    throw new FeedbackValueError("ValueOutOfRange", `value ${value} exceeds ±1e38`);
  }
  return { value, decimals };
}

/** Prints exactly `decimals` fraction digits: `(0,2)` → `"0.00"`, `(-5,2)` → `"-0.05"`. */
export function formatFeedbackValue({ value, decimals }: FeedbackValue): string {
  if (decimals === 0) {
    return value.toString();
  }
  const negative = value < 0n;
  const digits = (negative ? -value : value).toString().padStart(decimals + 1, "0");
  const whole = digits.slice(0, digits.length - decimals);
  const fraction = digits.slice(digits.length - decimals);
  return `${negative ? "-" : ""}${whole}.${fraction}`;
}
