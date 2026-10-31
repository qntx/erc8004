import { toHex } from "viem";
import { describe, expect, it } from "vite-plus/test";

import { classifyAgentUri } from "../../src/registration/agent-uri.ts";
import type { AgentUri } from "../../src/registration/agent-uri.ts";
import { expectError, loadVector, named } from "./helpers.ts";
import type { VectorCase } from "./helpers.ts";

const cases = loadVector("vectors/registration/agent-uri.json", "registration.agent-uri");

function normalize(uri: AgentUri): Record<string, unknown> {
  if (uri.kind === "Data") {
    return { kind: "Data", bytes: toHex(uri.bytes) };
  }
  return { ...uri };
}

function classify(vectorCase: VectorCase): AgentUri {
  const { gateway } = vectorCase;
  return gateway === undefined
    ? classifyAgentUri(vectorCase["input"] as string)
    : classifyAgentUri(vectorCase["input"] as string, gateway as string);
}

describe("registration.agent-uri", () => {
  it.each(named(cases))("classifies %s", (_title, vectorCase) => {
    expect(normalize(classify(vectorCase))).toStrictEqual(vectorCase["output"]);
  });

  it.each(named(cases, undefined, "err"))("rejects %s", (_title, vectorCase) => {
    expect.hasAssertions();
    expectError(() => classify(vectorCase), vectorCase["error"]);
  });
});
