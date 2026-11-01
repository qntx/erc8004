/// <reference types="node" />
import { readFileSync } from "node:fs";

import { describe, expect, test } from "vite-plus/test";

const BUMP_CONFIG = readFileSync("bump.config.ts", "utf8");

describe("bump.config", () => {
  test("the execute hook is a single command with no shell operators", () => {
    const execute = /execute:\s*"([^"]*)"/.exec(BUMP_CONFIG)?.[1];
    expect(execute).toBeDefined();
    // bumpp spawns `execute` without a shell (first token is the binary, the
    // rest become its arguments), so `&&`, `||`, `;` and `|` never chain —
    // multi-step work must live inside the executed script.
    expect(execute).not.toMatch(/&&|\|\||;|\|/);
  });
});
