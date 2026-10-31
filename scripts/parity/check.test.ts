import { describe, expect, test } from "vite-plus/test";

import { checkParity } from "./check.ts";

const TRUE = (): boolean => true;
const EMPTY = (): string => "";
const CITES = (): string => "vectors/x.json";
const mapReader =
  (files: Map<string, string>) =>
  (p: string): string =>
    files.get(p) ?? "";

function cap(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  const c: Record<string, unknown> = {
    id: "core.example",
    ts: { module: "src/x.ts", tests: ["tests/x.test.ts"] },
    vectors: ["vectors/x.json"],
    rust: { tests: ["tests/x.rs"] },
    ...overrides,
  };
  return c;
}

describe("parity check", () => {
  test("accepts a minimal valid document", () => {
    expect(checkParity([cap()], ["vectors/x.json"], TRUE, CITES)).toStrictEqual([]);
  });

  test("rejects a non-array top level", () => {
    const errors = checkParity({ schema: 1, capabilities: [cap()] }, [], TRUE, CITES);
    expect(errors).toStrictEqual(["parity.json: top level must be an array"]);
  });

  test("rejects unknown entry keys", () => {
    const errors = checkParity([cap({ tier: "A", status: "stable" })], [], TRUE, CITES);
    expect(errors).toStrictEqual([
      'capability "core.example": unknown key "tier"',
      'capability "core.example": unknown key "status"',
    ]);
  });

  test("rejects duplicate ids", () => {
    const errors = checkParity([cap(), cap()], ["vectors/x.json"], TRUE, CITES);
    expect(errors).toStrictEqual(['capability "core.example": duplicate id']);
  });

  test("rejects an empty vectors list", () => {
    const errors = checkParity([cap({ vectors: [] })], [], TRUE, CITES);
    expect(errors).toStrictEqual([
      'capability "core.example": vectors must list at least one vector file',
    ]);
  });

  test("rejects an empty ts.tests list", () => {
    const errors = checkParity(
      [cap({ ts: { module: "src/x.ts", tests: [] } })],
      ["vectors/x.json"],
      TRUE,
      CITES,
    );
    expect(errors).toStrictEqual([
      'capability "core.example": ts.tests must list at least one test file',
    ]);
  });

  test("rejects a missing ts.module", () => {
    const errors = checkParity(
      [cap({ ts: { tests: ["tests/x.test.ts"] } })],
      ["vectors/x.json"],
      TRUE,
      CITES,
    );
    expect(errors).toStrictEqual([
      'capability "core.example": ts.module must be a non-empty string',
    ]);
  });

  test("rejects missing referenced paths", () => {
    const errors = checkParity([cap()], ["vectors/x.json"], () => false, EMPTY);
    expect(errors).toStrictEqual([
      'capability "core.example": referenced path does not exist: src/x.ts',
      'capability "core.example": referenced path does not exist: tests/x.test.ts',
      'capability "core.example": referenced path does not exist: tests/x.rs',
      'capability "core.example": vector file does not exist: vectors/x.json',
    ]);
  });

  test("rejects rust null — every capability is dual-language", () => {
    const errors = checkParity([cap({ rust: null })], ["vectors/x.json"], TRUE, CITES);
    expect(errors).toStrictEqual([
      'capability "core.example": rust must list at least one test file',
    ]);
  });

  test("rejects a rust object without tests", () => {
    const errors = checkParity([cap({ rust: {} })], [], TRUE, CITES);
    expect(errors).toStrictEqual([
      'capability "core.example": rust.tests must list at least one test file',
    ]);
  });

  test("rejects a non-object rust value", () => {
    const errors = checkParity([cap({ rust: "missing" })], [], TRUE, CITES);
    expect(errors).toStrictEqual(['capability "core.example": rust must be an object']);
  });

  test("rejects unreferenced vector files", () => {
    const errors = checkParity([cap()], ["vectors/x.json", "vectors/orphan.json"], TRUE, CITES);
    expect(errors).toStrictEqual([
      "vector file is not referenced by any capability: vectors/orphan.json",
    ]);
  });

  test("requires each vector to be cited in a test file", () => {
    const files = new Map([
      ["src/x.ts", ""],
      ["tests/x.test.ts", "reads nothing relevant"],
      ["tests/x.rs", "reads nothing relevant"],
      ["vectors/x.json", "{}"],
    ]);
    const errors = checkParity([cap()], ["vectors/x.json"], (p) => files.has(p), mapReader(files));
    expect(errors).toStrictEqual([
      'capability "core.example": vector vectors/x.json is not referenced by any of its test files',
    ]);
  });

  test("accepts a vector cited by test file contents", () => {
    const files = new Map([
      ["src/x.ts", ""],
      ["tests/x.test.ts", 'readFileSync("vectors/x.json")'],
      ["tests/x.rs", ""],
      ["vectors/x.json", "{}"],
    ]);
    const errors = checkParity([cap()], ["vectors/x.json"], (p) => files.has(p), mapReader(files));
    expect(errors).toStrictEqual([]);
  });

  test("accepts a vector cited only by rust tests (rust object keeps ts vectors)", () => {
    const c = cap({
      rust: { tests: ["tests/x.rs"] },
    });
    const files = new Map([
      ["src/x.ts", ""],
      ["tests/x.test.ts", "no citation"],
      ["tests/x.rs", "x.json"],
      ["vectors/x.json", "{}"],
    ]);
    const errors = checkParity([c], ["vectors/x.json"], (p) => files.has(p), mapReader(files));
    expect(errors).toStrictEqual([]);
  });
});
