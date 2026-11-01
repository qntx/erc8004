import { execFileSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, test } from "vite-plus/test";

const root = join(import.meta.dirname, "..");

// Package sources are .ts; a stray .d.ts under src/ means a tool (tsgo dts with
// a `paths` mapping, a bare tsc emit) wrote declarations next to the sibling
// sources. Tracked declarations (wasm shims) are allowed.
describe("no stray declarations in packages/*/src", () => {
  test("every .d.ts under packages/*/src is git-tracked", () => {
    const tracked = new Set(
      execFileSync("git", ["ls-files", "packages"], { cwd: root, encoding: "utf8" })
        .split("\n")
        .filter((line) => line !== ""),
    );
    const strays = readdirSync(join(root, "packages"), { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .flatMap((entry) =>
        readdirSync(join(root, "packages", entry.name, "src"), {
          recursive: true,
          encoding: "utf8",
        }).map((file) => join("packages", entry.name, "src", file)),
      )
      .filter((file) => file.endsWith(".d.ts"))
      .filter((file) => !tracked.has(file));
    expect(strays).toStrictEqual([]);
  });
});
