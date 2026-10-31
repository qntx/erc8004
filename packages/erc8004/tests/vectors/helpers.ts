import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { expect } from "vite-plus/test";

import { Erc8004Error } from "../../src/core/errors.ts";

export type VectorCase = Readonly<Record<string, unknown>>;
export type NamedCase = readonly [title: string, vectorCase: VectorCase];

type VectorFile = { schema: number; capability: string; cases: VectorCase[] };

function isVectorFile(value: unknown): value is VectorFile {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return (
    candidate["schema"] === 1 &&
    typeof candidate["capability"] === "string" &&
    Array.isArray(candidate["cases"])
  );
}

/** Loads a `vectors/**` file and asserts the shared envelope. */
export function loadVector(relative: string, capability: string): VectorCase[] {
  const path = fileURLToPath(new URL(`../../../../${relative}`, import.meta.url));
  const parsed: unknown = JSON.parse(readFileSync(path, "utf8"));
  if (!isVectorFile(parsed)) {
    throw new Error(`invalid vector file ${relative}`);
  }
  if (parsed.capability !== capability) {
    throw new Error(`capability mismatch in ${relative}: ${parsed.capability} != ${capability}`);
  }
  return parsed.cases;
}

/** Title for a generated `it.each` case — the primary input spelled like JSON. */
export function titleOf(vectorCase: VectorCase): string {
  const primary =
    "input" in vectorCase
      ? vectorCase["input"]
      : (vectorCase["inputText"] ?? vectorCase["file"] ?? vectorCase);
  return JSON.stringify(primary).slice(0, 80);
}

/** The JSON value a vector case feeds to the parser — `input` verbatim, `inputText` JSON-parsed. */
export function inputOf(vectorCase: VectorCase): unknown {
  if ("inputText" in vectorCase) {
    return JSON.parse(String(vectorCase["inputText"]));
  }
  return vectorCase["input"];
}

/** `[title, case]` tuples for `it.each`, filtered by op and outcome. */
export function named(cases: VectorCase[], op?: string, outcome: "ok" | "err" = "ok"): NamedCase[] {
  return cases
    .filter((c) => {
      if (op !== undefined && (c["op"] ?? "run") !== op) {
        return false;
      }
      return outcome === "ok" ? "output" in c && !("error" in c) : "error" in c;
    })
    .map((c) => [titleOf(c), c] as const);
}

/** Runs `fn` expecting an {@link Erc8004Error} with the given `code` (and `field` when present). */
export function expectError(run: () => unknown, expected: unknown, field?: unknown): void {
  try {
    run();
  } catch (error) {
    expect(error).toBeInstanceOf(Erc8004Error);
    if (error instanceof Erc8004Error) {
      expect(error.code).toBe(expected);
      if (field !== undefined) {
        expect((error as { field?: unknown }).field).toBe(field);
      }
    }
    return;
  }
  expect.unreachable(`expected error ${String(expected)}`);
}
