import { describe, expect, it } from "vite-plus/test";

import {
  feedbackValue,
  formatFeedbackValue,
  parseFeedbackValue,
} from "../../src/core/feedback-value.ts";
import { expectError, loadVector, named } from "./helpers.ts";
import type { VectorCase } from "./helpers.ts";

const cases = loadVector("vectors/core/feedback-value.json", "core.feedback-value");

function construct(vectorCase: VectorCase): { value: bigint; decimals: number } {
  const input = vectorCase["input"] as { value: string; decimals: string };
  return vectorCase["op"] === "parse"
    ? parseFeedbackValue(vectorCase["input"] as string)
    : feedbackValue(BigInt(input.value), Number(input.decimals));
}

describe("core.feedback-value", () => {
  it.each(named(cases).filter(([, c]) => c["op"] !== "format"))("%s %s", (_title, vectorCase) => {
    const parsed = construct(vectorCase);
    const expected = vectorCase["output"] as { value: string; decimals: string };
    expect(parsed.value.toString()).toStrictEqual(expected.value);
    expect(parsed.decimals.toString()).toStrictEqual(expected.decimals);
  });

  it.each(named(cases, "format"))("formats %s", (_title, vectorCase) => {
    const input = vectorCase["input"] as { value: string; decimals: string };
    expect(
      formatFeedbackValue({ value: BigInt(input.value), decimals: Number(input.decimals) }),
    ).toStrictEqual(vectorCase["output"]);
  });

  it.each(named(cases, undefined, "err"))("rejects %s", (_title, vectorCase) => {
    expect.hasAssertions();
    expectError(() => construct(vectorCase), vectorCase["error"]);
  });
});
