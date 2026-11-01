import type {
  Abi,
  Account,
  Address,
  Client,
  ContractFunctionArgs,
  ContractFunctionName,
  ContractFunctionReturnType,
} from "viem";
import { parseAccount } from "viem/accounts";
import { readContract } from "viem/actions";

import { ClientGuardError, UnsupportedChainError } from "../core/errors.ts";
import { withContractRevert } from "../core/revert-map.ts";
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

/** Shared optional args every action accepts. */
export type ActionArgs = {
  /** Registry address override; resolves by deployment/chain otherwise. */
  address?: Address | undefined;
  /** Pins the deployment for address resolution. */
  deployment?: Deployment | undefined;
  /** Signing account override; falls back to `client.account`. */
  account?: Account | Address | undefined;
};

/**
 * The signing account for a write: `args.account`, else `client.account` — throws
 * {@link ClientGuardError} `MissingAccount` when neither exists (viem's `AccountNotFoundError` is
 * not a public export to reuse here).
 */
export function sender(client: Client, account?: Account | Address): Account {
  const found = account ?? client.account;
  if (found === undefined) {
    throw new ClientGuardError(
      "MissingAccount",
      "no account on the client; pass args.account or use a wallet client",
    );
  }
  return parseAccount(found);
}

/** The signing account's address; same resolution as {@link sender}. */
export function senderAddress(client: Client, account?: Account | Address): Address {
  return sender(client, account).address;
}

/** Registry address for `kind` resolved from `args.address`/`args.deployment`/`client.chain`. */
export function registry(
  client: Pick<Client, "chain">,
  kind: RegistryKind,
  args: ActionArgs,
): Address {
  return resolveRegistryAddress(client, kind, args.address, args.deployment);
}

/**
 * Typed `readContract` for view/pure calls — wraps the call in {@link withContractRevert} so
 * contract reverts surface as `ContractRevertError`.
 */
export async function read<
  const abi extends Abi,
  const name extends ContractFunctionName<abi, "pure" | "view">,
>(
  client: Client,
  address: Address,
  abi: abi,
  functionName: name,
  args: ContractFunctionArgs<abi, "pure" | "view", name>,
): Promise<ContractFunctionReturnType<abi, "pure" | "view", name>> {
  return withContractRevert(readContract(client, { abi, address, functionName, args }));
}
