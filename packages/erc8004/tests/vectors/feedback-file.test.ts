import { describe, expect, it } from "vite-plus/test";

import { feedbackFileToJson, parseFeedbackFile } from "../../src/reputation/feedback-file.ts";
import { expectError, inputOf, loadVector, named } from "./helpers.ts";

const cases = loadVector("vectors/reputation/feedback-file.json", "reputation.feedback-file");

describe("reputation.feedback-file", () => {
  it.each(named(cases))("round-trips %s", (_title, vectorCase) => {
    expect(feedbackFileToJson(parseFeedbackFile(inputOf(vectorCase)))).toStrictEqual(
      vectorCase["output"],
    );
  });

  it.each(named(cases, undefined, "err"))("rejects %s", (_title, vectorCase) => {
    expect.hasAssertions();
    expectError(
      () => parseFeedbackFile(inputOf(vectorCase)),
      vectorCase["error"],
      vectorCase["field"],
    );
  });
});
