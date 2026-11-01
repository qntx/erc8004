/// <reference types="node" />
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { basename, join } from "node:path";

type JsonObject = Record<string, unknown>;

const ENTRY_KEYS = new Set(["id", "ts", "vectors", "rust"]);

function isRecord(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function asRecord(value: unknown): JsonObject {
  return isRecord(value) ? value : {};
}

function asStrings(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((v): v is string => typeof v === "string") : [];
}

/**
 * Validates parity.json against the vectors/ tree. Returns error messages; an empty array means the
 * file is consistent.
 */
export function checkParity(
  parity: unknown,
  vectorFiles: string[],
  fileExists: (path: string) => boolean,
  readFile: (path: string) => string,
): string[] {
  const errors: string[] = [];
  if (!Array.isArray(parity)) {
    return ["parity.json: top level must be an array"];
  }

  const seen = new Set<string>();
  const referencedVectors = new Set<string>();

  for (const item of parity) {
    const cap = asRecord(item);
    const { id: rawId } = cap;
    const id = typeof rawId === "string" ? rawId : "?";
    const label = `capability "${id}"`;

    if (typeof rawId !== "string" || rawId === "") {
      errors.push("capability without a string id");
    } else if (seen.has(id)) {
      errors.push(`${label}: duplicate id`);
    }
    seen.add(id);

    for (const key of Object.keys(cap)) {
      if (!ENTRY_KEYS.has(key)) {
        errors.push(`${label}: unknown key "${key}"`);
      }
    }

    const ts = asRecord(cap["ts"]);
    const tsModule = typeof ts["module"] === "string" ? ts["module"] : undefined;
    if (tsModule === undefined || tsModule === "") {
      errors.push(`${label}: ts.module must be a non-empty string`);
    }
    const tsTests = asStrings(ts["tests"]);
    if (tsTests.length === 0) {
      errors.push(`${label}: ts.tests must list at least one test file`);
    }

    // `rust` is `{ tests }` — the erc8004 test files (tests/*.rs or the
    // src/ files holding in-crate vector tests) that run this capability's
    // vectors. Every capability is dual-language; there is no TS-only escape.
    const rustValue = cap["rust"];
    const rust: { module?: string; tests: string[] } = { tests: [] };
    if (rustValue === null || rustValue === undefined) {
      errors.push(`${label}: rust must list at least one test file`);
    } else if (isRecord(rustValue)) {
      const rustModule = rustValue["module"];
      if (typeof rustModule === "string" && rustModule !== "") {
        rust.module = rustModule;
      }
      rust.tests = asStrings(rustValue["tests"]);
      if (rust.tests.length === 0) {
        errors.push(`${label}: rust.tests must list at least one test file`);
      }
    } else {
      errors.push(`${label}: rust must be an object`);
    }

    const testFiles = [...tsTests, ...rust.tests];
    if (rust.module !== undefined) {
      testFiles.push(rust.module);
    }

    for (const path of [tsModule, ...testFiles]) {
      if (typeof path === "string" && !fileExists(path)) {
        errors.push(`${label}: referenced path does not exist: ${path}`);
      }
    }

    const vectors = asStrings(cap["vectors"]);
    if (vectors.length === 0) {
      // vectorless capabilities are allowed only when both languages list a
      // concrete e2e or unit test file exercising the shared semantics
      const hasTest = (tests: string[]) =>
        tests.some((t) => /e2e|test/i.test(basename(t)) && fileExists(t));
      if (!hasTest(tsTests) || !hasTest(rust.tests)) {
        errors.push(
          `${label}: vectors may be empty only when both ts.tests and rust.tests list an e2e or unit test file`,
        );
      }
    }
    const citedBy = (tests: string[], vector: string): boolean => {
      const needles = [vector, basename(vector)];
      return tests.some((testFile) => {
        if (!fileExists(testFile)) {
          return false;
        }
        const content = readFile(testFile);
        return needles.some((needle) => content.includes(needle));
      });
    };
    for (const vector of vectors) {
      referencedVectors.add(vector);
      if (!fileExists(vector)) {
        errors.push(`${label}: vector file does not exist: ${vector}`);
        continue;
      }
      if (!citedBy(testFiles, vector)) {
        errors.push(`${label}: vector ${vector} is not referenced by any of its test files`);
      }
    }
  }

  for (const file of vectorFiles) {
    if (basename(file) === "README.md") {
      continue;
    }
    if (!referencedVectors.has(file)) {
      errors.push(`vector file is not referenced by any capability: ${file}`);
    }
  }

  return errors;
}

function listVectors(dir: string, prefix: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(prefix, entry.name);
    if (entry.isDirectory()) {
      out.push(...listVectors(join(dir, entry.name), path));
    } else {
      out.push(path);
    }
  }
  return out.toSorted();
}

if (import.meta.main) {
  const parity: unknown = JSON.parse(readFileSync("parity.json", "utf8"));
  const vectorFiles = existsSync("vectors") ? listVectors("vectors", "vectors") : [];
  const errors = checkParity(
    parity,
    vectorFiles,
    (path) => existsSync(path),
    (path) => readFileSync(path, "utf8"),
  );
  for (const error of errors) {
    console.error(`parity: ${error}`);
  }
  if (errors.length > 0) {
    process.exitCode = 1;
  }
}
