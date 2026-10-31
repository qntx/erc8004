import { describe, expect, it } from "vite-plus/test";

import { parseRegistrationFile, registrationFileToJson } from "../../src/registration/file.ts";
import { expectError, inputOf, loadVector, named } from "./helpers.ts";

const cases = loadVector("vectors/registration/file.json", "registration.file");

describe("registration.file", () => {
  it.each(named(cases))("round-trips %s", (_title, vectorCase) => {
    expect(registrationFileToJson(parseRegistrationFile(inputOf(vectorCase)))).toStrictEqual(
      vectorCase["output"],
    );
  });

  it.each(named(cases, undefined, "err"))("rejects %s", (_title, vectorCase) => {
    expect.hasAssertions();
    expectError(
      () => parseRegistrationFile(inputOf(vectorCase)),
      vectorCase["error"],
      vectorCase["field"],
    );
  });
});
