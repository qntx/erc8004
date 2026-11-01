import { describe, expect, test } from "vite-plus/test";

import { Erc8004Error, UnsupportedChainError } from "../src/core/index.ts";

describe("UnsupportedChainError", () => {
  test("carries the shared code and the chainId", () => {
    const error = new UnsupportedChainError(999);
    expect(error).toBeInstanceOf(Erc8004Error);
    expect(error.name).toBe("UnsupportedChainError");
    expect(error.code).toBe("UnsupportedChain");
    expect(error.chainId).toBe(999);
    expect(error.message).toContain("999");
  });
});
