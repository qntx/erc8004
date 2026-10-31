import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

import viemPkg from "viem/package.json" with { type: "json" };

const CHECK = process.argv.includes("--check");
const ROOT = new URL("../../../../", import.meta.url).pathname;
const VIEM_VERSION = viemPkg.version;

let drifted = 0;

export function vectorFile(
  relative: string,
  capability: string,
  cases: unknown[],
  generator: string,
): void {
  const envelope = {
    schema: 1,
    capability,
    source: { kind: "reference", name: "viem", version: VIEM_VERSION, generator },
    cases,
  };
  const path = `${ROOT}${relative}`;
  const content = `${JSON.stringify(envelope, null, 2)}\n`;
  if (CHECK) {
    let existing: string | undefined;
    try {
      existing = readFileSync(path, "utf8");
    } catch {
      existing = undefined;
    }
    if (existing !== content) {
      console.error(`drift: ${relative}`);
      drifted += 1;
    }
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
  console.log(`wrote ${relative} (${cases.length} cases)`);
}

export function finish(): void {
  if (CHECK) {
    if (drifted > 0) {
      console.error(`${drifted} vector file(s) out of date — run bun run vectors`);
      process.exitCode = 1;
    }
    console.log("vectors up to date");
  }
}
