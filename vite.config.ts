import { builtinModules } from "node:module";

import { defineConfig } from "vite-plus";
import type { UserConfig } from "vite-plus";

import { fmt } from "@qntx/oxfmt";
import { config as lintConfig, merge } from "@qntx/oxlint";

// The library runs on Node, browsers, and Hermes: Node builtins must not be
// imported from src.
const platformNeutralImports = {
  paths: builtinModules.map((name) => ({
    name,
    message: "Node builtin — the library must stay platform-neutral",
  })),
  patterns: [
    {
      group: ["node:*"],
      message: "Node builtin — the library must stay platform-neutral",
    },
  ],
};

// One-directional layer graph for packages/erc8004/src — mirrors the
// "TypeScript" section of AGENTS.md, which describes the same graph.
const LAYERS = {
  abi: [],
  core: [],
  deployments: ["core"],
  registration: ["core"],
  events: ["abi", "core"],
  identity: ["abi", "core", "deployments", "events"],
  reputation: ["abi", "core", "deployments", "events"],
  validation: ["abi", "core", "deployments", "events"],
} as const;

const LAYER_NAMES = Object.keys(LAYERS).filter(
  (name): name is keyof typeof LAYERS => name in LAYERS,
);

// Relative import specifiers that would reach the package-root index from a
// layer file. src/ is flat (no nested directories under a layer), so every
// cross-layer import is a single `../` hop.
const PACKAGE_ROOT = ["..", "../", "../index", "../index.ts"];

// eslint/no-restricted-imports options that forbid a layer from importing any
// layer outside its allow list or the package root. The platform-neutral
// restrictions still apply on top: oxlint replaces (not merges) a rule's
// config per override, so callers must combine these patterns with
// platformNeutralImports themselves.
function layerImports(layer: keyof typeof LAYERS) {
  const allowed: ReadonlyArray<string> = LAYERS[layer];
  const message =
    allowed.length > 0
      ? `${layer} may only import ${allowed.join(", ")} (AGENTS.md layering)`
      : `${layer} is the leaf layer (AGENTS.md layering)`;
  const forbidden = LAYER_NAMES.filter((name) => name !== layer && !allowed.includes(name));
  return {
    paths: [],
    patterns: [
      {
        group: [...PACKAGE_ROOT, ...forbidden.flatMap((name) => [`../${name}`, `../${name}/**`])],
        message,
      },
    ],
  };
}

type OxlintOverride = (typeof lintConfig)["overrides"][number];

// Layer overrides replace the platform-neutral config rather than merging
// into it, so each one re-adds those paths/patterns explicitly.
const layerOverrides: OxlintOverride[] = LAYER_NAMES.map((layer) => ({
  files: [`packages/erc8004/src/${layer}/**`],
  rules: {
    "eslint/no-restricted-imports": [
      "error",
      {
        paths: platformNeutralImports.paths,
        patterns: [...platformNeutralImports.patterns, ...layerImports(layer).patterns],
      },
    ],
  },
}));

const config: UserConfig = defineConfig({
  staged: {
    "*": "vp check --fix",
  },
  test: {
    include: ["scripts/**/*.test.ts"],
  },
  lint: merge(lintConfig, {
    // merge() concatenates arrays onto the preset's own ignorePatterns.
    ignorePatterns: ["target/**"],
    jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
    rules: {
      "vite-plus/prefer-vite-plus-imports": "error",
    },
    overrides: [
      {
        files: ["scripts/**", "packages/erc8004/scripts/**"],
        rules: {
          // Maintenance scripts print their results, and sequential
          // endpoint probing (first success wins) is intentional.
          "eslint/no-console": "off",
          "eslint/no-await-in-loop": "off",
        },
      },
      {
        files: ["packages/*/src/**"],
        rules: {
          // Hermes V1 (what React Native ships) has no ES2023 immutable array
          // methods, so the library must sort/reverse copies in place.
          "unicorn/no-array-sort": "off",
          "unicorn/no-array-reverse": "off",
          // @types/node leaks into the src program via tests and scripts, so
          // Node-only globals would typecheck silently; browser-only globals
          // are equally absent on Hermes. Both sets are banned — library code
          // must use globalThis lookups instead.
          "eslint/no-restricted-globals": [
            "error",
            { name: "Buffer", message: "Node-only global — use Uint8Array" },
            { name: "process", message: "Node-only global — not available in browsers or Hermes" },
            { name: "global", message: "Node-only global — use globalThis" },
            { name: "require", message: "CJS-only — use import" },
            { name: "module", message: "CJS-only global" },
            { name: "__dirname", message: "CJS-only global" },
            { name: "__filename", message: "CJS-only global" },
            { name: "setImmediate", message: "Node-only global — use setTimeout" },
            { name: "clearImmediate", message: "Node-only global — use clearTimeout" },
            { name: "window", message: "browser-only global — use globalThis" },
            { name: "document", message: "browser-only global — use globalThis" },
            { name: "navigator", message: "browser-only global — use globalThis" },
            { name: "location", message: "browser-only global — use globalThis" },
            { name: "localStorage", message: "browser-only global — inject a store" },
            { name: "sessionStorage", message: "browser-only global — inject a store" },
          ],
          "eslint/no-restricted-imports": ["error", platformNeutralImports],
        },
      },
      ...layerOverrides,
    ],
  }),
  fmt: {
    ...fmt,
    ignorePatterns: [
      ...fmt.ignorePatterns,
      "target/**",
      "bun.lock",
      // TOML is owned by taplo (.taplo.toml, align_entries); vendored ABIs
      // and shared vectors stay byte-frozen.
      "**/*.toml",
      "data/abis/**",
      "vectors/**",
    ],
  },
  run: { cache: process.env["CI"] === undefined || process.env["CI"] === "" },
});

export default config;
