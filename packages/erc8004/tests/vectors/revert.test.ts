import type { Hex } from "viem";
import { describe, expect, it } from "vite-plus/test";

import { decodeRevert } from "../../src/core/revert.ts";
import type { RevertReason } from "../../src/core/revert.ts";
import { loadVector, named } from "./helpers.ts";

const cases = loadVector("vectors/core/revert.json", "core.revert");

function normalize(reason: RevertReason): Record<string, unknown> {
  const out: Record<string, unknown> = { reason: reason.reason };
  for (const [key, value] of Object.entries(reason)) {
    if (key !== "reason") {
      out[key] = typeof value === "bigint" ? value.toString() : value;
    }
  }
  return out;
}

describe("core.revert", () => {
  it.each(named(cases))("decodes %s", (_title, vectorCase) => {
    const input = vectorCase["input"] as { data: Hex };
    expect(normalize(decodeRevert(input.data))).toStrictEqual(vectorCase["output"]);
  });
});
