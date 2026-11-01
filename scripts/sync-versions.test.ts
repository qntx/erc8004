import { describe, expect, test } from "vite-plus/test";

import { syncInternalDeps } from "./sync-versions.ts";

const INTERNAL = new Set(["@qntx/erc8004", "@qntx/erc8004-wasm"]);

describe("syncInternalDeps", () => {
  test("rewrites peer ranges to ^<version> and dev ranges to <version>", () => {
    const pkg = {
      name: "@qntx/erc8004-wasm",
      peerDependencies: { "@qntx/erc8004": "^0.9.0" },
      devDependencies: { "@qntx/erc8004": "0.9.0", typescript: "7.0.2" },
    };
    expect(syncInternalDeps(pkg, "0.9.1", INTERNAL)).toStrictEqual({
      name: "@qntx/erc8004-wasm",
      peerDependencies: { "@qntx/erc8004": "^0.9.1" },
      devDependencies: { "@qntx/erc8004": "0.9.1", typescript: "7.0.2" },
    });
  });

  test("leaves manifests without internal deps untouched", () => {
    const pkg = { name: "@qntx/erc8004", dependencies: { "@noble/curves": "^2.4.0" } };
    expect(syncInternalDeps(pkg, "0.9.1", INTERNAL)).toStrictEqual(pkg);
  });
});
