/// <reference types="node" />
import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

type JsonObject = Record<string, unknown>;

function isRecord(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function asRecord(value: unknown): JsonObject {
  return isRecord(value) ? value : {};
}

/**
 * Rewrites a manifest's internal dependency ranges to `version`: `^<v>` in peerDependencies, `<v>`
 * in devDependencies. Returns the (possibly mutated) manifest; bumpp rewrites each package's own
 * `version` field, this covers the ranges bumpp cannot express in JSON.
 */
export function syncInternalDeps(
  pkg: JsonObject,
  version: string,
  internalNames: ReadonlySet<string>,
): JsonObject {
  for (const [section, prefix] of [
    ["peerDependencies", "^"],
    ["devDependencies", ""],
  ] as const) {
    const deps = asRecord(pkg[section]);
    for (const name of Object.keys(deps)) {
      if (internalNames.has(name)) {
        deps[name] = `${prefix}${version}`;
      }
    }
  }
  return pkg;
}

if (import.meta.main) {
  const dirs = readdirSync("packages", { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => join("packages", entry.name));

  const manifests = new Map<string, JsonObject>();
  const internalNames = new Set<string>();
  for (const dir of dirs) {
    const path = join(dir, "package.json");
    const pkg: unknown = JSON.parse(readFileSync(path, "utf8"));
    manifests.set(path, asRecord(pkg));
    const { name } = asRecord(pkg);
    if (typeof name === "string") {
      internalNames.add(name);
    }
  }

  const version = manifests.get("packages/erc8004/package.json")?.["version"];
  if (typeof version === "string") {
    for (const [path, pkg] of manifests) {
      if (path === "packages/erc8004/package.json") {
        continue;
      }
      syncInternalDeps(pkg, version, internalNames);
      writeFileSync(path, `${JSON.stringify(pkg, null, 2)}\n`);
    }

    // bun.lock mirrors the workspace versions under `workspaces`; refresh
    // Cargo.lock must record the new workspace version. bumpp runs the
    // `execute` hook without a shell — `&&` would reach bun as arguments —
    // so the refresh is spawned here instead of chained in bump.config.ts.
    const update = spawnSync("cargo", ["update", "--workspace"], { stdio: "inherit" });
    if (update.status !== 0) {
      const why =
        update.error === undefined ? `exit code ${String(update.status)}` : update.error.message;
      console.error(`sync-versions: cargo update --workspace failed: ${why}`);
      process.exitCode = 1;
    }
    // the lockfile without touching dependencies.
    const lock = spawnSync("bun", ["install", "--lockfile-only"], { stdio: "inherit" });
    if (lock.status !== 0) {
      const why =
        lock.error === undefined ? `exit code ${String(lock.status)}` : lock.error.message;
      console.error(`sync-versions: bun install --lockfile-only failed: ${why}`);
      process.exitCode = 1;
    }
  } else {
    console.error('sync-versions: packages/erc8004/package.json has no string "version"');
    process.exitCode = 1;
  }
}
