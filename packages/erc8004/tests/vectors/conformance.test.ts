import { describe, expect, it } from "vite-plus/test";

import { checkRegistrationFile } from "../../src/registration/conformance.ts";
import { parseRegistrationFile } from "../../src/registration/file.ts";
import { inputOf, loadVector, named } from "./helpers.ts";

function normalizeExpected(output: unknown): Array<{ code: string; index?: number }> {
  return (output as Array<{ code: string; index?: string }>).map((issue) =>
    issue.index === undefined
      ? { code: issue.code }
      : { code: issue.code, index: Number(issue.index) },
  );
}

const cases = loadVector("vectors/registration/conformance.json", "registration.conformance");

describe("registration.conformance", () => {
  it.each(named(cases))("reports %s", (_title, vectorCase) => {
    const issues = checkRegistrationFile(parseRegistrationFile(inputOf(vectorCase)));
    expect(issues).toStrictEqual(normalizeExpected(vectorCase["output"]));
  });
});
