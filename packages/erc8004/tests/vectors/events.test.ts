import type { Hex } from "viem";
import { describe, expect, it } from "vite-plus/test";

import { decodeErc8004Log } from "../../src/events/index.ts";
import { expectError, loadVector, named } from "./helpers.ts";

const cases = loadVector("vectors/events/decode.json", "events.decode");

function normalize(value: unknown): unknown {
  // Normal form: every integer (any width, incl. uint8 as a JS number) is a decimal string.
  if (typeof value === "bigint" || typeof value === "number") {
    return value.toString();
  }
  if (Array.isArray(value)) {
    return value.map(normalize);
  }
  if (typeof value === "object" && value !== null) {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, normalize(v)]));
  }
  return value;
}

describe("events.decode", () => {
  it.each(named(cases).filter(([, c]) => c["output"] === null))(
    "ignores %s",
    (_title, vectorCase) => {
      const input = vectorCase["input"] as { topics: Hex[]; data: Hex };
      expect(decodeErc8004Log(input)).toBeNull();
    },
  );

  it.each(named(cases).filter(([, c]) => c["output"] !== null))(
    "decodes %s",
    (_title, vectorCase) => {
      const input = vectorCase["input"] as { topics: Hex[]; data: Hex };
      const expected = vectorCase["output"] as { event: string; args: Record<string, unknown> };
      const decoded = decodeErc8004Log(input);
      expect(decoded?.eventName).toStrictEqual(expected.event);
      expect(normalize(decoded?.args)).toStrictEqual(expected.args);
    },
  );

  it.each(named(cases, undefined, "err"))("rejects %s", (_title, vectorCase) => {
    expect.hasAssertions();
    const input = vectorCase["input"] as { topics: Hex[]; data: Hex };
    expectError(() => decodeErc8004Log(input), vectorCase["error"]);
  });
});
