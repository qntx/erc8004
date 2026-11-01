import { describe, expect, test } from "vite-plus/test";

import {
  identityRegistryAbi,
  reputationRegistryAbi,
  validationRegistryAbi,
} from "../src/abi/index.ts";

type AbiEntry = { type?: string; name?: string };

function names(abi: ReadonlyArray<AbiEntry>, type: string): Set<string> {
  return new Set(
    abi.flatMap((e) => (e.type === type && typeof e.name === "string" ? [e.name] : [])),
  );
}

describe("vendored registry ABIs", () => {
  test("identity exposes the registry functions and events", () => {
    const fns = names(identityRegistryAbi, "function");
    const events = names(identityRegistryAbi, "event");
    for (const name of ["getVersion", "register", "setAgentURI", "tokenURI", "ownerOf"]) {
      expect(fns).toContain(name);
    }
    for (const name of ["Registered", "Transfer"]) {
      expect(events).toContain(name);
    }
  });

  test("reputation exposes feedback functions and events", () => {
    const fns = names(reputationRegistryAbi, "function");
    const events = names(reputationRegistryAbi, "event");
    for (const name of [
      "getVersion",
      "getIdentityRegistry",
      "giveFeedback",
      "revokeFeedback",
      "getSummary",
      "readFeedback",
    ]) {
      expect(fns).toContain(name);
    }
    for (const name of ["NewFeedback", "FeedbackRevoked", "ResponseAppended"]) {
      expect(events).toContain(name);
    }
  });

  test("validation exposes request/response functions and events", () => {
    const fns = names(validationRegistryAbi, "function");
    const events = names(validationRegistryAbi, "event");
    for (const name of [
      "getVersion",
      "getIdentityRegistry",
      "validationRequest",
      "validationResponse",
      "getValidationStatus",
      "getSummary",
    ]) {
      expect(fns).toContain(name);
    }
    for (const name of ["ValidationRequest", "ValidationResponse"]) {
      expect(events).toContain(name);
    }
  });
});
