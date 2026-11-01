/**
 * Regenerates `data/abis/*.json` from the vendored Solidity in `data/contracts/` via `forge
 * inspect`. `bun run abi` rewrites; `bun run abi --check` verifies. Requires the Foundry toolchain
 * (the e2e project's `soldeer` dependencies are installed idempotently).
 */
import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";

const CONTRACTS = [
  ["IdentityRegistryUpgradeable", "IdentityRegistry"],
  ["ReputationRegistryUpgradeable", "ReputationRegistry"],
  ["ValidationRegistryUpgradeable", "ValidationRegistry"],
] as const;

const check = process.argv.includes("--check");

function forge(args: string[]): string {
  const result = spawnSync("forge", args, { cwd: "e2e", encoding: "utf8" });
  if (result.error !== undefined) {
    throw new Error(`forge not found: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`forge ${args.join(" ")} failed:\n${result.stderr}${result.stdout}`);
  }
  return result.stdout;
}

// soldeer deps are idempotent — the e2e project needs OpenZeppelin sources to inspect
forge(["soldeer", "install"]);

let failed = false;
for (const [contract, file] of CONTRACTS) {
  const out = forge(["inspect", `../data/contracts/${contract}.sol:${contract}`, "abi", "--json"]);
  const rendered = `${JSON.stringify(JSON.parse(out), null, 2)}\n`;
  const path = `data/abis/${file}.json`;
  if (check) {
    if (readFileSync(path, "utf8") !== rendered) {
      console.error(`abi: ${path} out of date — run bun run abi`);
      failed = true;
    }
  } else {
    writeFileSync(path, rendered);
    console.error(`abi: wrote ${path}`);
  }
}
if (failed) {
  throw new Error("abi: files out of date — run bun run abi");
}
