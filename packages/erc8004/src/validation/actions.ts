import type { Account, Address, Client, Hash, Hex } from "viem";
import { zeroHash } from "viem";
import { parseAccount } from "viem/accounts";
import { readContract, writeContract } from "viem/actions";

import { validationRegistryAbi } from "../abi/index.ts";
import { ClientGuardError, ContractRevertError } from "../core/errors.ts";
import { withContractRevert } from "../core/revert-map.ts";
import type { Deployment } from "../deployments/deployment.ts";
import { resolveRegistryAddress } from "../deployments/resolve.ts";

type Args = {
  address?: Address | undefined;
  deployment?: Deployment | undefined;
  account?: Account | Address | undefined;
};

function registry(client: Pick<Client, "chain">, args: Args): Address {
  return resolveRegistryAddress(client, "validation", args.address, args.deployment);
}

function sender(client: Client, account?: Account | Address): Account {
  const found = account ?? client.account;
  if (found === undefined) {
    throw new ClientGuardError(
      "MissingAccount",
      "no account on the client; pass args.account or use a wallet client",
    );
  }
  return parseAccount(found);
}

/** `getValidationStatus` row. */
export type ValidationStatus = {
  validator: Address;
  agentId: bigint;
  response: number;
  responseHash: Hex;
  tag: string;
  lastUpdate: bigint;
};

/** `null` exactly when the contract reverts with `UnknownRequest`. */
export async function getValidationStatus(
  client: Client,
  args: Args & { requestHash: Hex },
  // oxlint-disable-next-line typescript/no-restricted-types -- spec: UnknownRequest maps to null.
): Promise<ValidationStatus | null> {
  try {
    const [validator, agentId, response, responseHash, tag, lastUpdate] = await withContractRevert(
      readContract(client, {
        abi: validationRegistryAbi,
        address: registry(client, args),
        functionName: "getValidationStatus",
        args: [args.requestHash],
      }),
    );
    return { validator, agentId, response, responseHash, tag, lastUpdate };
  } catch (error) {
    if (error instanceof ContractRevertError && error.reason.reason === "UnknownRequest") {
      return null;
    }
    throw error;
  }
}

export async function getValidationSummary(
  client: Client,
  args: Args & { agentId: bigint; validators?: ReadonlyArray<Address>; tag?: string },
): Promise<{ count: bigint; averageResponse: number }> {
  const [count, avgResponse] = await withContractRevert(
    readContract(client, {
      abi: validationRegistryAbi,
      address: registry(client, args),
      functionName: "getSummary",
      args: [args.agentId, args.validators ? [...args.validators] : [], args.tag ?? ""],
    }),
  );
  return { count, averageResponse: avgResponse };
}

export async function getAgentValidations(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<ReadonlyArray<Hex>> {
  return withContractRevert(
    readContract(client, {
      abi: validationRegistryAbi,
      address: registry(client, args),
      functionName: "getAgentValidations",
      args: [args.agentId],
    }),
  );
}

export async function getValidatorRequests(
  client: Client,
  args: Args & { validator: Address },
): Promise<ReadonlyArray<Hex>> {
  return withContractRevert(
    readContract(client, {
      abi: validationRegistryAbi,
      address: registry(client, args),
      functionName: "getValidatorRequests",
      args: [args.validator],
    }),
  );
}

export async function getValidationVersion(client: Client, args: Args = {}): Promise<string> {
  return withContractRevert(
    readContract(client, {
      abi: validationRegistryAbi,
      address: registry(client, args),
      functionName: "getVersion",
    }),
  );
}

export async function requestValidation(
  client: Client,
  args: Args & { validator: Address; agentId: bigint; requestUri: string; requestHash: Hex },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: validationRegistryAbi,
      address: registry(client, args),
      functionName: "validationRequest",
      args: [args.validator, args.agentId, args.requestUri, args.requestHash],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/** `response` must be an integer in 0..=100 (`ResponseOutOfRange`). */
export async function respondToValidation(
  client: Client,
  args: Args & {
    requestHash: Hex;
    response: number;
    responseUri?: string;
    responseHash?: Hex;
    tag?: string;
  },
): Promise<Hash> {
  if (!Number.isInteger(args.response) || args.response < 0 || args.response > 100) {
    throw new ClientGuardError(
      "ResponseOutOfRange",
      "respondToValidation: response must be an integer in 0..=100",
    );
  }
  return withContractRevert(
    writeContract(client, {
      abi: validationRegistryAbi,
      address: registry(client, args),
      functionName: "validationResponse",
      args: [
        args.requestHash,
        args.response,
        args.responseUri ?? "",
        args.responseHash ?? zeroHash,
        args.tag ?? "",
      ],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}
