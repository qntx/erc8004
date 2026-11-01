import type { Address, Client, Hash, Hex } from "viem";
import { zeroHash } from "viem";
import { writeContract } from "viem/actions";

import { validationRegistryAbi } from "../abi/index.ts";
import { ClientGuardError, ContractRevertError } from "../core/errors.ts";
import { withContractRevert } from "../core/revert-map.ts";
import { read, registry, sender } from "../deployments/resolve.ts";
import type { ActionArgs } from "../deployments/resolve.ts";

type Args = ActionArgs;

/** `getValidationStatus` row. */
export type ValidationStatus = {
  validator: Address;
  agentId: bigint;
  response: number;
  responseHash: Hex;
  tag: string;
  lastUpdate: bigint;
};

/** `getSummary` aggregate for validation requests. */
export type ValidationSummary = {
  count: bigint;
  averageResponse: number;
};

/**
 * `getValidationStatus` for one request hash — `null` exactly when the contract reverts with
 * `UnknownRequest`; other reverts surface as `ContractRevertError`.
 */
export async function getValidationStatus(
  client: Client,
  args: Args & { requestHash: Hex },
  // oxlint-disable-next-line typescript/no-restricted-types -- spec: UnknownRequest maps to null.
): Promise<ValidationStatus | null> {
  try {
    const [validator, agentId, response, responseHash, tag, lastUpdate] = await read(
      client,
      registry(client, "validation", args),
      validationRegistryAbi,
      "getValidationStatus",
      [args.requestHash],
    );
    return { validator, agentId, response, responseHash, tag, lastUpdate };
  } catch (error) {
    if (error instanceof ContractRevertError && error.reason.reason === "UnknownRequest") {
      return null;
    }
    throw error;
  }
}

/** `getSummary` — count + average response over `validators` (empty = all). */
export async function getValidationSummary(
  client: Client,
  args: Args & { agentId: bigint; validators?: ReadonlyArray<Address>; tag?: string },
): Promise<ValidationSummary> {
  const [count, averageResponse] = await read(
    client,
    registry(client, "validation", args),
    validationRegistryAbi,
    "getSummary",
    [args.agentId, [...(args.validators ?? [])], args.tag ?? ""],
  );
  return { count, averageResponse };
}

/** `getAgentValidations` — every request hash for `agentId`. */
export async function getAgentValidations(
  client: Client,
  args: Args & { agentId: bigint },
): Promise<ReadonlyArray<Hex>> {
  return read(
    client,
    registry(client, "validation", args),
    validationRegistryAbi,
    "getAgentValidations",
    [args.agentId],
  );
}

/** `getValidatorRequests` — every request hash assigned to `validator`. */
export async function getValidatorRequests(
  client: Client,
  args: Args & { validator: Address },
): Promise<ReadonlyArray<Hex>> {
  return read(
    client,
    registry(client, "validation", args),
    validationRegistryAbi,
    "getValidatorRequests",
    [args.validator],
  );
}

/** `getVersion` of the validation registry. */
export async function getValidationVersion(client: Client, args: Args = {}): Promise<string> {
  return read(
    client,
    registry(client, "validation", args),
    validationRegistryAbi,
    "getVersion",
    [],
  );
}

/** `validationRequest` write. */
export async function requestValidation(
  client: Client,
  args: Args & { validator: Address; agentId: bigint; requestUri: string; requestHash: Hex },
): Promise<Hash> {
  return withContractRevert(
    writeContract(client, {
      abi: validationRegistryAbi,
      address: registry(client, "validation", args),
      functionName: "validationRequest",
      args: [args.validator, args.agentId, args.requestUri, args.requestHash],
      account: sender(client, args.account),
      chain: client.chain,
    }),
  );
}

/**
 * `validationResponse` write; `ClientGuardError` `ResponseOutOfRange` for a non-integer or
 * out-of-`0..=100` response.
 */
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
      address: registry(client, "validation", args),
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
