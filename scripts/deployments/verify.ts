/// <reference types="node" />
// Probes every candidate chain over JSON-RPC and rewrites
// data/deployments.json with the chains whose three registries answer
// getVersion() == "2.0.0" and whose Reputation/Validation registries point
// back at the Identity address. Manual tool — not run in CI.
//
//   bun scripts/deployments/verify.ts
import { writeFileSync } from "node:fs";
import { join } from "node:path";

import { decodeAbiParameters, getAddress, isHex, toFunctionSelector } from "viem";
import type { Hex } from "viem";

// Upstream scripts/addresses.ts (erc-8004/erc-8004-contracts @
// b9e466c250744a7e06b13dff9d3c2844ed64f825): one CREATE2 address set per
// network type.
const MAINNET_ADDRESSES = {
  identityRegistry: getAddress("0x8004A169FB4a3325136EB29fA0ceB6D2e539a432"),
  reputationRegistry: getAddress("0x8004BAa17C55a88189AE136b182e5fdA19dE9b63"),
  validationRegistry: getAddress("0x8004Cc8439f36fd5F9F049D9fF86523Df6dAAB58"),
} as const;

const TESTNET_ADDRESSES = {
  identityRegistry: getAddress("0x8004A818BFB912233c491871b3d84c89A494BD9e"),
  reputationRegistry: getAddress("0x8004B663056A597Dffe9eCcC1965A193B7388713"),
  validationRegistry: getAddress("0x8004Cb1BF31DAf7788923b405b754f57acEB4272"),
} as const;

const EXPECTED_VERSION = "2.0.0";
const GET_VERSION = toFunctionSelector("getVersion()");
const GET_IDENTITY_REGISTRY = toFunctionSelector("getIdentityRegistry()");
const TIMEOUT_MS = 10_000;

type Candidate = {
  id: string;
  name: string;
  chainId: number;
  testnet: boolean;
  rpcs: string[];
};

// Candidates: every upstream MAINNET_CHAIN_IDS / TESTNET_CHAIN_IDS entry
// (31337 skipped) plus the chains the upstream README lists but the deploy
// scripts do not. RPC sources: upstream hardhat.config.ts defaults,
// chainid.network (viem/chains equivalents), and the retired root
// config.toml RPC lists.
const CANDIDATES: Candidate[] = [
  {
    id: "ethereum",
    name: "Ethereum",
    chainId: 1,
    testnet: false,
    rpcs: [
      "https://mainnet.gateway.tenderly.co",
      "https://0xrpc.io/eth",
      "https://ethereum.rpc.subquery.network/public",
      "https://ethereum-public.nodies.app",
      "https://1rpc.io/eth",
      "https://eth.merkle.io",
      "https://ethereum-rpc.publicnode.com",
      "https://eth.drpc.org",
    ],
  },
  {
    id: "optimism",
    name: "Optimism",
    chainId: 10,
    testnet: false,
    rpcs: [
      "https://optimism.gateway.tenderly.co",
      "https://optimism-rpc.publicnode.com",
      "https://optimism.drpc.org",
      "https://mainnet.optimism.io",
      "https://1rpc.io/op",
    ],
  },
  {
    id: "bsc",
    name: "BSC",
    chainId: 56,
    testnet: false,
    rpcs: [
      "https://bsc-dataseed.binance.org",
      "https://bsc-rpc.publicnode.com",
      "https://bsc.drpc.org",
      "https://binance.llamarpc.com",
      "https://bsc-mainnet.public.blastapi.io",
    ],
  },
  {
    id: "gnosis",
    name: "Gnosis",
    chainId: 100,
    testnet: false,
    rpcs: [
      "https://rpc.gnosischain.com",
      "https://gnosis-rpc.publicnode.com",
      "https://gnosis.drpc.org",
      "https://1rpc.io/gnosis",
    ],
  },
  {
    id: "polygon",
    name: "Polygon",
    chainId: 137,
    testnet: false,
    rpcs: [
      "https://polygon-rpc.com",
      "https://polygon.gateway.tenderly.co",
      "https://polygon-bor-rpc.publicnode.com",
      "https://polygon.drpc.org",
      "https://polygon.lava.build",
    ],
  },
  {
    id: "monad",
    name: "Monad",
    chainId: 143,
    testnet: false,
    rpcs: [
      "https://rpc.monad.xyz",
      "https://rpc1.monad.xyz",
      "https://rpc3.monad.xyz",
      "https://rpc4.monad.xyz",
      "https://monad-mainnet.drpc.org",
      "https://143.rpc.thirdweb.com",
    ],
  },
  {
    id: "xlayer",
    name: "XLayer",
    chainId: 196,
    testnet: false,
    rpcs: ["https://rpc.xlayer.tech", "https://xlayerrpc.okx.com"],
  },
  {
    id: "hedera",
    name: "Hedera",
    chainId: 295,
    testnet: false,
    rpcs: ["https://mainnet.hashio.io/api", "https://hedera.linkpool.pro"],
  },
  {
    id: "shape",
    name: "Shape",
    chainId: 360,
    testnet: false,
    rpcs: ["https://mainnet.shape.network", "https://shape-mainnet.g.alchemy.com/public"],
  },
  {
    id: "metis",
    name: "Metis",
    chainId: 1088,
    testnet: false,
    rpcs: ["https://andromeda.metis.io/?owner=1088", "https://metis.drpc.org"],
  },
  {
    id: "injective",
    name: "Injective",
    chainId: 1776,
    testnet: false,
    rpcs: ["https://sentry.evm-rpc.injective.network", "https://injectiveevm-rpc.polkachu.com"],
  },
  {
    id: "soneium",
    name: "Soneium",
    chainId: 1868,
    testnet: false,
    rpcs: ["https://rpc.soneium.org"],
  },
  {
    id: "goat",
    name: "GOAT",
    chainId: 2345,
    testnet: false,
    rpcs: ["https://rpc.goat.network"],
  },
  {
    id: "abstract",
    name: "Abstract",
    chainId: 2741,
    testnet: false,
    rpcs: ["https://api.mainnet.abs.xyz", "https://abstract.api.onfinality.io/public"],
  },
  {
    id: "megaeth",
    name: "MegaETH",
    chainId: 4326,
    testnet: false,
    rpcs: ["https://mainnet.megaeth.com/rpc", "https://alpha.megaeth.com/rpc"],
  },
  {
    id: "robinhood",
    name: "Robinhood Chain",
    chainId: 4663,
    testnet: false,
    rpcs: ["https://rpc.mainnet.chain.robinhood.com", "https://robinhood-rpc.publicnode.com"],
  },
  {
    id: "mantle",
    name: "Mantle",
    chainId: 5000,
    testnet: false,
    rpcs: [
      "https://rpc.mantle.xyz",
      "https://mantle-rpc.publicnode.com",
      "https://1rpc.io/mantle",
      "https://mantle.drpc.org",
    ],
  },
  {
    id: "base",
    name: "Base",
    chainId: 8453,
    testnet: false,
    rpcs: [
      "https://mainnet.base.org",
      "https://base.gateway.tenderly.co",
      "https://base-rpc.publicnode.com",
      "https://1rpc.io/base",
      "https://base.drpc.org",
      "https://base.llamarpc.com",
    ],
  },
  {
    id: "zero-g",
    name: "0G",
    chainId: 16661,
    testnet: false,
    rpcs: ["https://evmrpc.0g.ai"],
  },
  {
    id: "celo",
    name: "Celo",
    chainId: 42220,
    testnet: false,
    rpcs: [
      "https://forno.celo.org",
      "https://celo-json-rpc.stakely.io",
      "https://1rpc.io/celo",
      "https://celo.drpc.org",
    ],
  },
  {
    id: "arbitrum",
    name: "Arbitrum",
    chainId: 42161,
    testnet: false,
    rpcs: [
      "https://arb1.arbitrum.io/rpc",
      "https://arbitrum.gateway.tenderly.co",
      "https://arbitrum-one-rpc.publicnode.com",
      "https://1rpc.io/arb",
      "https://arb1.lava.build",
    ],
  },
  {
    id: "avalanche",
    name: "Avalanche",
    chainId: 43114,
    testnet: false,
    rpcs: [
      "https://api.avax.network/ext/bc/C/rpc",
      "https://avalanche-c-chain-rpc.publicnode.com",
      "https://1rpc.io/avax/c",
      "https://avalanche.drpc.org",
    ],
  },
  {
    id: "billions",
    name: "Billions",
    chainId: 45056,
    testnet: false,
    rpcs: ["https://billions-rpc.eu-north-2.gateway.fm"],
  },
  {
    id: "linea",
    name: "Linea",
    chainId: 59144,
    testnet: false,
    rpcs: [
      "https://rpc.linea.build",
      "https://linea-rpc.publicnode.com",
      "https://1rpc.io/linea",
      "https://linea.drpc.org",
    ],
  },
  {
    id: "taiko",
    name: "Taiko",
    chainId: 167000,
    testnet: false,
    rpcs: [
      "https://rpc.mainnet.taiko.xyz",
      "https://rpc.taiko.xyz",
      "https://taiko-rpc.publicnode.com",
      "https://taiko.drpc.org",
    ],
  },
  {
    id: "scroll",
    name: "Scroll",
    chainId: 534352,
    testnet: false,
    rpcs: [
      "https://rpc.scroll.io",
      "https://scroll-rpc.publicnode.com",
      "https://1rpc.io/scroll",
      "https://scroll.drpc.org",
    ],
  },
  {
    id: "skale-base",
    name: "SKALE Base",
    chainId: 1187947933,
    testnet: false,
    rpcs: ["https://skale-base.skalenodes.com/v1/base"],
  },
  {
    id: "bsc-testnet",
    name: "BSC Testnet",
    chainId: 97,
    testnet: true,
    rpcs: [
      "https://bsc-testnet-rpc.publicnode.com",
      "https://bsc-testnet.drpc.org",
      "https://data-seed-prebsc-1-s1.binance.org:8545",
    ],
  },
  {
    id: "hedera-testnet",
    name: "Hedera Testnet",
    chainId: 296,
    testnet: true,
    rpcs: ["https://testnet.hashio.io/api"],
  },
  {
    id: "injective-testnet",
    name: "Injective Testnet",
    chainId: 1439,
    testnet: true,
    rpcs: [
      "https://k8s.testnet.json-rpc.injective.network",
      "https://injectiveevm-testnet-rpc.polkachu.com",
    ],
  },
  {
    id: "soneium-minato",
    name: "Soneium Minato",
    chainId: 1946,
    testnet: true,
    rpcs: ["https://rpc.minato.soneium.org"],
  },
  {
    id: "xlayer-testnet",
    name: "XLayer Testnet",
    chainId: 1952,
    testnet: true,
    rpcs: ["https://testrpc.xlayer.tech", "https://xlayertestrpc.okx.com"],
  },
  {
    id: "mantle-sepolia",
    name: "Mantle Sepolia",
    chainId: 5003,
    testnet: true,
    rpcs: ["https://rpc.sepolia.mantle.xyz"],
  },
  {
    id: "megaeth-testnet",
    name: "MegaETH Testnet",
    chainId: 6343,
    testnet: true,
    rpcs: ["https://timothy.megaeth.com/rpc", "https://carrot.megaeth.com/rpc"],
  },
  {
    id: "billions-testnet",
    name: "Billions Testnet",
    chainId: 6913,
    testnet: true,
    rpcs: ["https://billions-testnet-rpc.eu-north-2.gateway.fm"],
  },
  {
    id: "gnosis-chiado",
    name: "Gnosis Chiado",
    chainId: 10200,
    testnet: true,
    rpcs: ["https://rpc.chiadochain.net", "https://rpc.chiado.gnosis.gateway.fm"],
  },
  {
    id: "monad-testnet",
    name: "Monad Testnet",
    chainId: 10143,
    testnet: true,
    rpcs: ["https://testnet-rpc.monad.xyz"],
  },
  {
    id: "shape-sepolia",
    name: "Shape Sepolia",
    chainId: 11011,
    testnet: true,
    rpcs: ["https://sepolia.shape.network"],
  },
  {
    id: "abstract-sepolia",
    name: "Abstract Sepolia",
    chainId: 11124,
    testnet: true,
    rpcs: ["https://api.testnet.abs.xyz"],
  },
  {
    id: "celo-sepolia",
    name: "Celo Sepolia",
    chainId: 11142220,
    testnet: true,
    rpcs: ["https://forno.celo-sepolia.celo-testnet.org"],
  },
  {
    id: "zero-g-galileo",
    name: "0G Galileo Testnet",
    chainId: 16602,
    testnet: true,
    rpcs: ["https://evmrpc-testnet.0g.ai"],
  },
  {
    id: "robinhood-testnet",
    name: "Robinhood Chain Testnet",
    chainId: 46630,
    testnet: true,
    rpcs: [
      "https://rpc.testnet.chain.robinhood.com/rpc",
      "https://robinhood-sepolia-rpc.publicnode.com",
    ],
  },
  {
    id: "goat-testnet",
    name: "GOAT Testnet",
    chainId: 48816,
    testnet: true,
    rpcs: ["https://rpc.testnet3.goat.network"],
  },
  {
    id: "linea-sepolia",
    name: "Linea Sepolia",
    chainId: 59141,
    testnet: true,
    rpcs: ["https://rpc.sepolia.linea.build"],
  },
  {
    id: "metis-sepolia",
    name: "Metis Sepolia",
    chainId: 59902,
    testnet: true,
    rpcs: ["https://sepolia.metisdevops.link", "https://metis-sepolia-rpc.publicnode.com"],
  },
  {
    id: "polygon-amoy",
    name: "Polygon Amoy",
    chainId: 80002,
    testnet: true,
    rpcs: ["https://rpc-amoy.polygon.technology", "https://polygon-amoy-bor-rpc.publicnode.com"],
  },
  {
    id: "avalanche-fuji",
    name: "Avalanche Fuji",
    chainId: 43113,
    testnet: true,
    rpcs: [
      "https://api.avax-test.network/ext/bc/C/rpc",
      "https://avalanche-fuji-c-chain-rpc.publicnode.com",
    ],
  },
  {
    id: "base-sepolia",
    name: "Base Sepolia",
    chainId: 84532,
    testnet: true,
    rpcs: [
      "https://sepolia.base.org",
      "https://base-sepolia-rpc.publicnode.com",
      "https://base-sepolia.drpc.org",
    ],
  },
  {
    id: "taiko-hoodi",
    name: "Taiko Hoodi",
    chainId: 167013,
    testnet: true,
    rpcs: ["https://rpc.hoodi.taiko.xyz"],
  },
  {
    id: "arbitrum-sepolia",
    name: "Arbitrum Sepolia",
    chainId: 421614,
    testnet: true,
    rpcs: ["https://sepolia-rollup.arbitrum.io/rpc", "https://arbitrum-sepolia-rpc.publicnode.com"],
  },
  {
    id: "scroll-sepolia",
    name: "Scroll Sepolia",
    chainId: 534351,
    testnet: true,
    rpcs: [
      "https://sepolia-rpc.scroll.io",
      "https://scroll-sepolia-rpc.publicnode.com",
      "https://scroll-testnet-public.unifra.io",
      "https://scroll-sepolia.chainstacklabs.com",
      "https://scroll-sepolia.drpc.org",
    ],
  },
  {
    id: "arc-testnet",
    name: "Arc Testnet",
    chainId: 5042002,
    testnet: true,
    rpcs: [
      "https://rpc.testnet.arc.network",
      "https://rpc.testnet.arc.io",
      "https://rpc.blockdaemon.testnet.arc.io",
    ],
  },
  {
    id: "optimism-sepolia",
    name: "Optimism Sepolia",
    chainId: 11155420,
    testnet: true,
    rpcs: ["https://sepolia.optimism.io", "https://optimism-sepolia-rpc.publicnode.com"],
  },
  {
    id: "ethereum-sepolia",
    name: "Ethereum Sepolia",
    chainId: 11155111,
    testnet: true,
    rpcs: [
      "https://ethereum-sepolia-rpc.publicnode.com",
      "https://sepolia.drpc.org",
      "https://rpc.sepolia.org",
      "https://1rpc.io/sepolia",
      "https://0xrpc.io/sep",
    ],
  },
  {
    id: "skale-base-sepolia",
    name: "SKALE Base Sepolia",
    chainId: 324705682,
    testnet: true,
    rpcs: ["https://base-sepolia-testnet.skalenodes.com/v1/jubilant-horrible-ancha"],
  },
];

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

async function rpc(url: string, method: string, params: unknown[]): Promise<unknown> {
  const res = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
    signal: AbortSignal.timeout(TIMEOUT_MS),
  });
  const json: unknown = await res.json();
  if (!isRecord(json)) {
    throw new Error("malformed JSON-RPC response");
  }
  const { error } = json;
  if (isRecord(error)) {
    const { message } = error;
    throw new Error(typeof message === "string" ? message : "JSON-RPC error");
  }
  return json["result"];
}

function err(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  const [first = message] = message.split("\n");
  return first.slice(0, 120);
}

async function call(url: string, to: string, data: Hex): Promise<Hex> {
  const result = await rpc(url, "eth_call", [{ to, data }, "latest"]);
  if (!isHex(result)) {
    throw new Error("malformed eth_call result");
  }
  return result;
}

type Outcome = { ok: true; rpc: string } | { ok: false; reason: string };

// Probes the three registries on an endpoint whose eth_chainId already
// matched — its answers are authoritative for the chain.
async function probeContracts(url: string, addresses: typeof MAINNET_ADDRESSES): Promise<Outcome> {
  const registries = [
    ["identity", addresses.identityRegistry],
    ["reputation", addresses.reputationRegistry],
    ["validation", addresses.validationRegistry],
  ] as const;
  for (const [label, address] of registries) {
    let data: Hex;
    try {
      data = await call(url, address, GET_VERSION);
    } catch (error) {
      return { ok: false, reason: `${label} getVersion() reverted: ${err(error)}` };
    }
    if (data === "0x") {
      return { ok: false, reason: `${label} registry has no code at ${address}` };
    }
    let version: string;
    try {
      [version] = decodeAbiParameters([{ type: "string" }], data);
    } catch {
      return { ok: false, reason: `${label} getVersion() returned undecodable data` };
    }
    if (version !== EXPECTED_VERSION) {
      return { ok: false, reason: `${label} getVersion() = "${version}"` };
    }
  }
  for (const [label, address] of registries.slice(1)) {
    let data: Hex;
    try {
      data = await call(url, address, GET_IDENTITY_REGISTRY);
    } catch (error) {
      return { ok: false, reason: `${label} getIdentityRegistry() reverted: ${err(error)}` };
    }
    try {
      const [identity] = decodeAbiParameters([{ type: "address" }], data);
      if (getAddress(identity) !== addresses.identityRegistry) {
        return {
          ok: false,
          reason: `${label} getIdentityRegistry() = ${getAddress(identity)}`,
        };
      }
    } catch {
      return { ok: false, reason: `${label} getIdentityRegistry() returned undecodable data` };
    }
  }
  return { ok: true, rpc: url };
}

async function verify(candidate: Candidate): Promise<Outcome> {
  const addresses = candidate.testnet ? TESTNET_ADDRESSES : MAINNET_ADDRESSES;
  const attempts: string[] = [];
  for (const url of candidate.rpcs) {
    try {
      const reported = await rpc(url, "eth_chainId", []);
      if (typeof reported !== "string" || BigInt(reported) !== BigInt(candidate.chainId)) {
        attempts.push(`${url}: eth_chainId ${String(reported)}`);
        continue;
      }
    } catch (error) {
      attempts.push(`${url}: ${err(error)}`);
      continue;
    }
    return probeContracts(url, addresses);
  }
  const [first = "no endpoint answered", ...rest] = attempts;
  return {
    ok: false,
    reason: `no usable RPC (${first}${rest.length > 0 ? `, +${rest.length} more` : ""})`,
  };
}

type Deployment = {
  id: string;
  name: string;
  chainId: number;
  testnet: boolean;
  identityRegistry: string;
  reputationRegistry: string;
  validationRegistry: string;
};

if (import.meta.main) {
  const included: Deployment[] = [];
  const results = await Promise.all(
    CANDIDATES.map(async (candidate) => ({ candidate, outcome: await verify(candidate) })),
  );
  for (const { candidate, outcome } of results) {
    const addresses = candidate.testnet ? TESTNET_ADDRESSES : MAINNET_ADDRESSES;
    if (outcome.ok) {
      included.push({
        id: candidate.id,
        name: candidate.name,
        chainId: candidate.chainId,
        testnet: candidate.testnet,
        ...addresses,
      });
      console.log(`  ✓ ${candidate.id} (${candidate.chainId}) via ${outcome.rpc}`);
    } else {
      console.log(`  ✗ ${candidate.id} (${candidate.chainId}): ${outcome.reason}`);
    }
  }
  const deployments = included.toSorted(
    (a, b) => Number(a.testnet) - Number(b.testnet) || a.chainId - b.chainId,
  );
  const out = {
    source: {
      generatedBy: "scripts/deployments/verify.ts",
      generatedAt: new Date().toISOString(),
      rule: 'eth_chainId matches + getVersion() == "2.0.0" on all three registries + reputation/validation getIdentityRegistry() == identity address',
      upstream: {
        repo: "https://github.com/erc-8004/erc-8004-contracts",
        commit: "b9e466c250744a7e06b13dff9d3c2844ed64f825",
      },
    },
    deployments,
  };
  const path = join(import.meta.dirname, "../../data/deployments.json");
  writeFileSync(path, `${JSON.stringify(out, null, 2)}\n`);
  console.log(`\n${included.length}/${CANDIDATES.length} chains included → data/deployments.json`);
}
