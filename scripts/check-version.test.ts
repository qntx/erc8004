import { describe, expect, test } from "vite-plus/test";

import { checkVersion } from "./check-version.ts";

const ERC8004 = {
  path: "packages/erc8004/package.json",
  pkg: { name: "@qntx/erc8004", version: "0.9.0" },
};

const ERC8004_WASM = {
  path: "packages/erc8004-wasm/package.json",
  pkg: {
    name: "@qntx/erc8004-wasm",
    version: "0.9.0",
    peerDependencies: { "@qntx/erc8004": "^0.9.0" } as Record<string, string>,
    devDependencies: { "@qntx/erc8004": "0.9.0" } as Record<string, string>,
  },
};

const PACKAGES = [ERC8004, ERC8004_WASM];

const CARGO_TOML = `[workspace]
[workspace.package]
version = "0.9.0"
[workspace.dependencies]
erc8004 = { path = "crates/erc8004", version = "=0.9.0", default-features = false }
sha2 = "0.9"
`;

const PARSED = {
  workspace: {
    package: { version: "0.9.0" },
    dependencies: {
      erc8004: { version: "=0.9.0", path: "crates/erc8004", "default-features": false },
      sha2: "0.9",
    },
  },
};

const MEMBERS = {
  "crates/erc8004/Cargo.toml": { package: { version: { workspace: true } } },
};

const CARGO_TOML_11 = CARGO_TOML.replaceAll("0.9.0", "0.11.0");
const PARSED_11 = {
  workspace: {
    package: { version: "0.11.0" },
    dependencies: {
      erc8004: { version: "=0.11.0", path: "crates/erc8004", "default-features": false },
      sha2: "0.9",
    },
  },
};
const LOCK_11 = {
  package: [
    { name: "erc8004", version: "0.11.0" },
    {
      name: "sha2",
      version: "0.10.0",
      source: "registry+https://github.com/rust-lang/crates.io-index",
    },
  ],
};

const LOCK = {
  package: [
    { name: "erc8004", version: "0.9.0" },
    {
      name: "sha2",
      version: "0.10.0",
      source: "registry+https://github.com/rust-lang/crates.io-index",
    },
  ],
};

// bun.lock mirrors the workspace manifests under `workspaces` keyed by
// directory; the root entry ("") has no version and is skipped.
const BUN_LOCK = {
  workspaces: {
    "": { name: "@qntx/erc8004-monorepo" },
    "packages/erc8004": { name: "@qntx/erc8004", version: "0.9.0" },
    "packages/erc8004-wasm": { name: "@qntx/erc8004-wasm", version: "0.9.0" },
  },
};

const BUN_LOCK_11 = {
  workspaces: {
    "packages/erc8004": { name: "@qntx/erc8004", version: "0.11.0" },
    "packages/erc8004-wasm": { name: "@qntx/erc8004-wasm", version: "0.11.0" },
  },
};

const STALE_BUN_LOCK = {
  workspaces: {
    "packages/erc8004": { name: "@qntx/erc8004", version: "0.10.1" },
    "packages/erc8004-wasm": { name: "@qntx/erc8004-wasm", version: "0.10.1" },
  },
};

const PACKAGES_11 = [
  { path: "packages/erc8004/package.json", pkg: { name: "@qntx/erc8004", version: "0.11.0" } },
  {
    path: "packages/erc8004-wasm/package.json",
    pkg: {
      name: "@qntx/erc8004-wasm",
      version: "0.11.0",
      peerDependencies: { "@qntx/erc8004": "^0.11.0" },
      devDependencies: { "@qntx/erc8004": "0.11.0" },
    },
  },
];

describe("check-version", () => {
  test("accepts synced versions", () => {
    expect(checkVersion(PACKAGES, PARSED, CARGO_TOML, MEMBERS, LOCK, BUN_LOCK)).toStrictEqual([]);
  });

  test("rejects a missing version", () => {
    const errors = checkVersion(
      [ERC8004, { ...ERC8004_WASM, pkg: { name: "@qntx/erc8004-wasm" } }],
      PARSED,
      CARGO_TOML,
      MEMBERS,
      LOCK,
      BUN_LOCK,
    );
    expect(errors).toStrictEqual([
      'packages/erc8004-wasm/package.json: missing string "version"',
      'bun.lock: workspaces["packages/erc8004-wasm"] has version "0.9.0", expected undefined',
    ]);
  });

  test("rejects a peer range without a caret", () => {
    const pkg = structuredClone(ERC8004_WASM.pkg);
    pkg.peerDependencies["@qntx/erc8004"] = "0.9.0";
    const errors = checkVersion(
      [ERC8004, { ...ERC8004_WASM, pkg }],
      PARSED,
      CARGO_TOML,
      MEMBERS,
      LOCK,
      BUN_LOCK,
    );
    expect(errors).toStrictEqual([
      'packages/erc8004-wasm/package.json: peerDependencies["@qntx/erc8004"] must be "^0.9.0", got "0.9.0"',
    ]);
  });

  test("rejects a dev range with a caret", () => {
    const pkg = structuredClone(ERC8004_WASM.pkg);
    pkg.devDependencies["@qntx/erc8004"] = "^0.9.0";
    const errors = checkVersion(
      [ERC8004, { ...ERC8004_WASM, pkg }],
      PARSED,
      CARGO_TOML,
      MEMBERS,
      LOCK,
      BUN_LOCK,
    );
    expect(errors).toStrictEqual([
      'packages/erc8004-wasm/package.json: devDependencies["@qntx/erc8004"] must be "0.9.0", got "^0.9.0"',
    ]);
  });

  test("ignores ranges on external packages", () => {
    const pkg = structuredClone(ERC8004_WASM.pkg);
    pkg.devDependencies["typescript"] = "^7.0.2";
    expect(
      checkVersion(
        [ERC8004, { ...ERC8004_WASM, pkg }],
        PARSED,
        CARGO_TOML,
        MEMBERS,
        LOCK,
        BUN_LOCK,
      ),
    ).toStrictEqual([]);
  });

  test("rejects a stale bun.lock workspace version", () => {
    // The drift this check guards: manifests moved on while bun.lock still
    // records the old workspace versions.
    const errors = checkVersion(
      PACKAGES_11,
      PARSED_11,
      CARGO_TOML_11,
      MEMBERS,
      LOCK_11,
      STALE_BUN_LOCK,
    );
    expect(errors).toStrictEqual([
      'bun.lock: workspaces["packages/erc8004"] has version "0.10.1", expected "0.11.0"',
      'bun.lock: workspaces["packages/erc8004-wasm"] has version "0.10.1", expected "0.11.0"',
    ]);
  });

  test("rejects a bun.lock workspace entry that is missing", () => {
    const errors = checkVersion(PACKAGES, PARSED, CARGO_TOML, MEMBERS, LOCK, { workspaces: {} });
    expect(errors).toStrictEqual([
      'bun.lock: missing workspaces["packages/erc8004"] entry',
      'bun.lock: missing workspaces["packages/erc8004-wasm"] entry',
    ]);
  });

  test("accepts a bumped pair with a synced lock", () => {
    expect(
      checkVersion(PACKAGES_11, PARSED_11, CARGO_TOML_11, MEMBERS, LOCK_11, BUN_LOCK_11),
    ).toStrictEqual([]);
  });

  test("rejects a package.json/Cargo.toml version mismatch", () => {
    const errors = checkVersion(
      [ERC8004, { ...ERC8004_WASM, pkg: { ...ERC8004_WASM.pkg, version: "0.9.1" } }],
      PARSED,
      CARGO_TOML,
      MEMBERS,
      LOCK,
      BUN_LOCK,
    );
    expect(errors).toStrictEqual([
      "version mismatch: packages/erc8004-wasm/package.json has 0.9.1, Cargo.toml has 0.9.0",
      'bun.lock: workspaces["packages/erc8004-wasm"] has version "0.9.0", expected "0.9.1"',
    ]);
  });

  test("rejects a member that does not inherit the workspace version", () => {
    const errors = checkVersion(
      PACKAGES,
      PARSED,
      CARGO_TOML,
      { "crates/erc8004/Cargo.toml": { package: { version: "0.9.0" } } },
      LOCK,
      BUN_LOCK,
    );
    expect(errors).toStrictEqual([
      "crates/erc8004/Cargo.toml: must inherit version via version.workspace = true",
    ]);
  });

  test("rejects an internal dependency without an exact pin", () => {
    const parsed = structuredClone(PARSED);
    parsed.workspace.dependencies.erc8004.version = "^0.9.0";
    const errors = checkVersion(PACKAGES, parsed, CARGO_TOML, MEMBERS, LOCK, BUN_LOCK);
    expect(errors).toStrictEqual([
      'Cargo.toml: internal dependency "erc8004" must use version "=0.9.0", got "^0.9.0"',
    ]);
  });

  test("rejects a stray version string in Cargo.toml", () => {
    const errors = checkVersion(
      PACKAGES,
      PARSED,
      `${CARGO_TOML}\n# pinned at 0.9.0 for the release\n`,
      MEMBERS,
      LOCK,
      BUN_LOCK,
    );
    expect(errors).toStrictEqual([
      'Cargo.toml: "0.9.0" appears 3 times, expected 2 ([workspace.package] plus internal path dependencies)',
    ]);
  });

  test("rejects a stale Cargo.lock member version", () => {
    const lock = structuredClone(LOCK);
    lock.package[0] = { name: "erc8004", version: "0.9.1" };
    const errors = checkVersion(PACKAGES, PARSED, CARGO_TOML, MEMBERS, lock, BUN_LOCK);
    expect(errors).toStrictEqual([
      'Cargo.lock: workspace member "erc8004" has version "0.9.1", expected "0.9.0"',
    ]);
  });
});
