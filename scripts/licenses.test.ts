import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, test } from "vite-plus/test";

const root = join(import.meta.dirname, "..");

const cases: string[][] = readdirSync(join(root, "packages"), { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .flatMap((entry) =>
    ["LICENSE-MIT", "LICENSE-APACHE"].map((license) => [`packages/${entry.name}`, license]),
  );

describe("package licenses", () => {
  test.each(cases)("%s/%s is byte-identical to the root copy", (dir, license) => {
    expect(readFileSync(join(root, dir, license))).toStrictEqual(readFileSync(join(root, license)));
  });
});
