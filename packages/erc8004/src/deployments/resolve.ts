import type { Address, Client } from "viem";

import { UnsupportedChainError } from "../core/errors.ts";
import type { Deployment } from "./deployment.ts";
import { getDeployment } from "./deployment.ts";

/** Which registry an action targets. */
export type RegistryKind = "identity" | "reputation" | "validation";

const FIELD: Record<
  RegistryKind,
  keyof Pick<Deployment, "identityRegistry" | "reputationRegistry" | "validationRegistry">
> = {
  identity: "identityRegistry",
  reputation: "reputationRegistry",
  validation: "validationRegistry",
};

/**
 * Registry address for an action: `args.address`, else `deployment`'s, else the `chain.id`
 * deployment table entry, else {@link UnsupportedChainError}.
 */
export function resolveRegistryAddress(
  client: Pick<Client, "chain">,
  kind: RegistryKind,
  address?: Address,
  deployment?: Deployment,
): Address {
  if (address !== undefined) {
    return address;
  }
  const found = deployment ?? getDeployment(client.chain?.id ?? -1);
  if (found === undefined) {
    throw new UnsupportedChainError(client.chain?.id ?? -1);
  }
  return found[FIELD[kind]];
}
