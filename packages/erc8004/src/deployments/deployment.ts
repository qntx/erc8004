import type { Address } from "viem";

import { deployments } from "./data.ts";

/** One verified ERC-8004 deployment: the three registries on one chain. */
export type Deployment = {
  /** Kebab-case identifier, e.g. `"base-sepolia"`. */
  readonly id: string;
  /** Human-readable chain name, e.g. `"Base Sepolia"`. */
  readonly name: string;
  readonly chainId: number;
  readonly testnet: boolean;
  readonly identityRegistry: Address;
  readonly reputationRegistry: Address;
  readonly validationRegistry: Address;
};

/** Deployment ids, e.g. `"ethereum"`, `"base-sepolia"`. */
export type DeploymentId = keyof typeof deployments;

/** The deployment for `chainId`, or `undefined` when the chain is not in the table. */
export function getDeployment(chainId: number): Deployment | undefined {
  return Object.values(deployments).find((deployment) => deployment.chainId === chainId);
}
