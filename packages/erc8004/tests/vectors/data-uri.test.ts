import { fromHex, toHex } from "viem";
import type { Hex } from "viem";
import { describe, expect, it } from "vite-plus/test";

import { decodeDataUri, encodeDataUri } from "../../src/registration/data-uri.ts";
import { expectError, loadVector, named } from "./helpers.ts";

const cases = loadVector("vectors/registration/data-uri.json", "registration.data-uri");

describe("registration.data-uri", () => {
  it.each(named(cases, "decode"))("decodes %s", (_title, vectorCase) => {
    expect(toHex(decodeDataUri(vectorCase["input"] as string))).toStrictEqual(vectorCase["output"]);
  });

  it.each(named(cases, "encode"))("encodes %s", (_title, vectorCase) => {
    expect(encodeDataUri(fromHex(vectorCase["input"] as Hex, "bytes"))).toStrictEqual(
      vectorCase["output"],
    );
  });

  it.each(named(cases, undefined, "err"))("rejects %s", (_title, vectorCase) => {
    expect.hasAssertions();
    expectError(() => decodeDataUri(vectorCase["input"] as string), vectorCase["error"]);
  });
});
