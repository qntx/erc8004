import { describe, expect, it } from "vite-plus/test";

import { parseAgentRegistry } from "../../src/core/agent-registry.ts";
import { verifyEndpointDomain, wellKnownRegistrationUrl } from "../../src/registration/domain.ts";
import { parseRegistrationFile } from "../../src/registration/file.ts";
import { loadVector, named } from "./helpers.ts";

const cases = loadVector(
  "vectors/registration/domain-verification.json",
  "registration.domain-verification",
);

describe("registration.domain-verification", () => {
  it.each(named(cases, "url"))("builds %s", (_title, vectorCase) => {
    expect(wellKnownRegistrationUrl(vectorCase["input"] as string)).toStrictEqual(
      vectorCase["output"],
    );
  });

  it.each(named(cases, "verify"))("verifies %s", (_title, vectorCase) => {
    const file = parseRegistrationFile(vectorCase["file"]);
    const result = verifyEndpointDomain(file, {
      agentRegistry: parseAgentRegistry(vectorCase["agentRegistry"] as string),
      agentId: BigInt(vectorCase["agentId"] as string),
    });
    expect(result).toBe(vectorCase["output"]);
  });
});
