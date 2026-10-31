import { readFileSync } from "node:fs";

import { getAddress } from "viem";
import { describe, expect, test } from "vite-plus/test";

import { deployments, getDeployment } from "../src/deployments/index.ts";
import type { Deployment } from "../src/deployments/index.ts";

const all = Object.values(deployments);
const registryFields = ["identityRegistry", "reputationRegistry", "validationRegistry"] as const;

describe("deployments table", () => {
  test("chainIds are unique", () => {
    const ids = all.map((d) => d.chainId);
    expect(new Set(ids).size).toBe(ids.length);
  });

  test("ids are unique kebab-case", () => {
    for (const d of all) {
      expect(d.id).toMatch(/^[a-z0-9]+(-[a-z0-9]+)*$/);
    }
    expect(new Set(all.map((d) => d.id)).size).toBe(all.length);
  });

  test("registry addresses are EIP-55", () => {
    for (const d of all) {
      for (const field of registryFields) {
        expect(d[field]).toBe(getAddress(d[field]));
      }
    }
  });

  test("mainnets come first, each group sorted by chainId", () => {
    const mainnets = all.filter((d) => !d.testnet);
    const testnets = all.filter((d) => d.testnet);
    expect(all).toStrictEqual([...mainnets, ...testnets]);
    const byChainId = (a: Deployment, b: Deployment): number => a.chainId - b.chainId;
    expect(mainnets).toStrictEqual(mainnets.toSorted(byChainId));
    expect(testnets).toStrictEqual(testnets.toSorted(byChainId));
  });

  test("table matches data/deployments.json", () => {
    const raw = JSON.parse(readFileSync("../../data/deployments.json", "utf8")) as {
      deployments: Array<Record<string, unknown>>;
    };
    expect(all).toHaveLength(raw.deployments.length);
    const byId = new Map(raw.deployments.map((d) => [d["id"], d]));
    for (const d of all) {
      const want = byId.get(d.id);
      expect(want, `${d.id} in deployments.json`).toBeDefined();
      expect(d).toStrictEqual({ ...want });
    }
  });

  test("the three registry addresses are distinct", () => {
    for (const d of all) {
      expect(new Set(registryFields.map((field) => d[field])).size).toBe(3);
    }
  });
});

describe("getDeployment", () => {
  test("round-trips every table entry by chainId", () => {
    for (const d of all) {
      expect(getDeployment(d.chainId)).toBe(d);
    }
  });

  test("returns undefined for an unknown chain", () => {
    expect(getDeployment(Number.MAX_SAFE_INTEGER)).toBeUndefined();
  });
});
