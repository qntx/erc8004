import { describe, expect, it } from "vite-plus/test";

import { formatAgentRegistry, parseAgentRegistry } from "../../src/core/agent-registry.ts";
import { expectError, loadVector, named } from "./helpers.ts";

const cases = loadVector("vectors/core/agent-registry.json", "core.agent-registry");

describe("core.agent-registry", () => {
  it.each(named(cases, undefined, "ok"))("round-trips %s", (_title, vectorCase) => {
    expect(formatAgentRegistry(parseAgentRegistry(vectorCase["input"] as string))).toStrictEqual(
      vectorCase["output"],
    );
  });

  it.each(named(cases, undefined, "err"))("rejects %s", (_title, vectorCase) => {
    expect.hasAssertions();
    expectError(() => parseAgentRegistry(vectorCase["input"] as string), vectorCase["error"]);
  });
});
