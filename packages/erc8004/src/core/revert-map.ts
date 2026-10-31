import { ContractFunctionRevertedError, RawContractError } from "viem";
import type { Hex } from "viem";

import { ContractRevertError } from "./errors.ts";
import { decodeRevert } from "./revert.ts";

function revertData(error: Error): Hex | undefined {
  if (error instanceof ContractFunctionRevertedError) {
    return error.raw;
  }
  if (error instanceof RawContractError) {
    const { data } = error;
    return typeof data === "string" ? data : data?.data;
  }
  return undefined;
}

/**
 * Rethrows an error whose cause chain reaches a `ContractFunctionRevertedError` or
 * `RawContractError` as {@link ContractRevertError}; everything else passes through.
 */
export function mapContractRevert(error: unknown): never {
  let current: unknown = error;
  while (current instanceof Error) {
    const data = revertData(current);
    if (data !== undefined) {
      throw new ContractRevertError(decodeRevert(data), { cause: error });
    }
    current = current.cause;
  }
  throw error;
}

/** Awaits `promise`, mapping contract reverts to {@link ContractRevertError}. */
export async function withContractRevert<T>(promise: Promise<T>): Promise<T> {
  try {
    return await promise;
  } catch (error) {
    return mapContractRevert(error);
  }
}
