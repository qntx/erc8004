import { defineConfig } from "vite-plus";
import type { UserConfig } from "vite-plus";

/** ESM-only sugar flattens to a string; keep types/import/default for pack entries. */
export function applyPackExports(pkgExports: Record<string, unknown>): Record<string, unknown> {
  for (const [key, value] of Object.entries(pkgExports)) {
    if (typeof value !== "string" || !value.endsWith(".mjs")) {
      continue;
    }
    pkgExports[key] = {
      types: value.replace(/\.mjs$/, ".d.mts"),
      import: value,
      default: value,
    };
  }
  return pkgExports;
}

const config: UserConfig = defineConfig({
  pack: {
    entry: {
      index: "src/index.ts",
      abi: "src/abi/index.ts",
      core: "src/core/index.ts",
      deployments: "src/deployments/index.ts",
      events: "src/events/index.ts",
      identity: "src/identity/index.ts",
      registration: "src/registration/index.ts",
      reputation: "src/reputation/index.ts",
      validation: "src/validation/index.ts",
    },
    dts: {
      generator: "tsgo",
    },
    sourcemap: true,
    exports: {
      customExports: applyPackExports,
    },
  },
  test: {
    include: ["tests/**/*.test.ts"],
  },
});

export default config;
