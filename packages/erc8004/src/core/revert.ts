import { decodeErrorResult, isAddress } from "viem";
import type { Address, Hex } from "viem";

import { identityRegistryAbi } from "../abi/index.ts";

/**
 * One shared reason enum for `Error(string)` reasons, the ABI custom errors, `Panic(uint256)`, and
 * everything else. Decoding never fails — unrecognised data stays available as `Unknown { data }`.
 */
export type RevertReason =
  | Readonly<{ reason: "NotAuthorized" }>
  | Readonly<{ reason: "ReservedMetadataKey" }>
  | Readonly<{ reason: "InvalidWallet" }>
  | Readonly<{ reason: "DeadlineExpired" }>
  | Readonly<{ reason: "DeadlineTooFar" }>
  | Readonly<{ reason: "InvalidWalletSignature" }>
  | Readonly<{ reason: "TooManyDecimals" }>
  | Readonly<{ reason: "ValueOutOfRange" }>
  | Readonly<{ reason: "SelfFeedback" }>
  | Readonly<{ reason: "InvalidFeedbackIndex" }>
  | Readonly<{ reason: "FeedbackIndexOutOfBounds" }>
  | Readonly<{ reason: "AlreadyRevoked" }>
  | Readonly<{ reason: "EmptyResponseUri" }>
  | Readonly<{ reason: "ClientAddressesRequired" }>
  | Readonly<{ reason: "InvalidValidator" }>
  | Readonly<{ reason: "RequestExists" }>
  | Readonly<{ reason: "UnknownRequest" }>
  | Readonly<{ reason: "NotValidator" }>
  | Readonly<{ reason: "ResponseOutOfRange" }>
  | Readonly<{ reason: "Message"; message: string }>
  | Readonly<{ reason: "AgentNotFound"; agentId: bigint }>
  | Readonly<{ reason: "IncorrectOwner"; sender: Address; agentId: bigint; owner: Address }>
  | Readonly<{ reason: "InsufficientApproval"; operator: Address; agentId: bigint }>
  | Readonly<{ reason: "InvalidReceiver"; receiver: Address }>
  | Readonly<{ reason: "InvalidSender"; sender: Address }>
  | Readonly<{ reason: "InvalidOwner"; owner: Address }>
  | Readonly<{ reason: "InvalidApprover"; approver: Address }>
  | Readonly<{ reason: "InvalidOperator"; operator: Address }>
  | Readonly<{ reason: "Panic"; code: bigint }>
  | Readonly<{ reason: "Unknown"; data: Hex }>;

const ERROR_STRINGS: Record<string, RevertReason> = {
  "Not authorized": { reason: "NotAuthorized" },
  "reserved key": { reason: "ReservedMetadataKey" },
  "bad wallet": { reason: "InvalidWallet" },
  expired: { reason: "DeadlineExpired" },
  "deadline too far": { reason: "DeadlineTooFar" },
  "invalid wallet sig": { reason: "InvalidWalletSignature" },
  "too many decimals": { reason: "TooManyDecimals" },
  "value too large": { reason: "ValueOutOfRange" },
  "Self-feedback not allowed": { reason: "SelfFeedback" },
  "index must be > 0": { reason: "InvalidFeedbackIndex" },
  "index out of bounds": { reason: "FeedbackIndexOutOfBounds" },
  "Already revoked": { reason: "AlreadyRevoked" },
  "Empty URI": { reason: "EmptyResponseUri" },
  "clientAddresses required": { reason: "ClientAddressesRequired" },
  "bad validator": { reason: "InvalidValidator" },
  exists: { reason: "RequestExists" },
  unknown: { reason: "UnknownRequest" },
  "not validator": { reason: "NotValidator" },
  "resp>100": { reason: "ResponseOutOfRange" },
};

function argsByName(decoded: {
  errorName: string;
  args?: unknown;
  abiItem?: { inputs?: ReadonlyArray<{ name?: string | undefined }> } | undefined;
}): Record<string, unknown> {
  const inputs = decoded.abiItem?.inputs ?? [];
  const args = Array.isArray(decoded.args) ? decoded.args : [];
  const out: Record<string, unknown> = {};
  for (const [i, input] of inputs.entries()) {
    if (input.name !== undefined) {
      out[input.name] = args[i];
    }
  }
  return out;
}

function arg(
  decoded: {
    errorName: string;
    args?: unknown;
    abiItem?: { inputs?: ReadonlyArray<{ name?: string | undefined }> } | undefined;
  },
  name: string,
): unknown {
  return argsByName(decoded)[name];
}

/** Decodes revert data into one reason; never throws — undecodable payloads become `Unknown`. */
export function decodeRevert(data: Hex): RevertReason {
  let decoded: { errorName: string; args?: unknown };
  try {
    decoded = decodeErrorResult({ abi: identityRegistryAbi, data });
  } catch {
    return { reason: "Unknown", data };
  }
  switch (decoded.errorName) {
    case "Error": {
      const positional: unknown = Array.isArray(decoded.args)
        ? decoded.args[0]
        : arg(decoded, "message");
      const message = typeof positional === "string" ? positional : "";
      return ERROR_STRINGS[message] ?? { reason: "Message", message };
    }
    case "Panic": {
      const positional: unknown = Array.isArray(decoded.args)
        ? decoded.args[0]
        : arg(decoded, "code");
      return { reason: "Panic", code: typeof positional === "bigint" ? positional : 0n };
    }
    case "ERC721NonexistentToken": {
      const agentId = arg(decoded, "tokenId");
      return { reason: "AgentNotFound", agentId: typeof agentId === "bigint" ? agentId : 0n };
    }
    case "ERC721IncorrectOwner": {
      const sender = arg(decoded, "sender");
      const tokenId = arg(decoded, "tokenId");
      const owner = arg(decoded, "owner");
      if (
        typeof sender === "string" &&
        isAddress(sender) &&
        typeof tokenId === "bigint" &&
        typeof owner === "string" &&
        isAddress(owner)
      ) {
        return { reason: "IncorrectOwner", sender, agentId: tokenId, owner };
      }
      return { reason: "Unknown", data };
    }
    case "ERC721InsufficientApproval": {
      const operator = arg(decoded, "operator");
      const tokenId = arg(decoded, "tokenId");
      if (typeof operator === "string" && isAddress(operator) && typeof tokenId === "bigint") {
        return { reason: "InsufficientApproval", operator, agentId: tokenId };
      }
      return { reason: "Unknown", data };
    }
    case "ERC721InvalidReceiver": {
      const receiver = arg(decoded, "receiver");
      return typeof receiver === "string" && isAddress(receiver)
        ? { reason: "InvalidReceiver", receiver }
        : { reason: "Unknown", data };
    }
    case "ERC721InvalidSender": {
      const sender = arg(decoded, "sender");
      return typeof sender === "string" && isAddress(sender)
        ? { reason: "InvalidSender", sender }
        : { reason: "Unknown", data };
    }
    case "ERC721InvalidOwner": {
      const owner = arg(decoded, "owner");
      return typeof owner === "string" && isAddress(owner)
        ? { reason: "InvalidOwner", owner }
        : { reason: "Unknown", data };
    }
    case "ERC721InvalidApprover": {
      const approver = arg(decoded, "approver");
      return typeof approver === "string" && isAddress(approver)
        ? { reason: "InvalidApprover", approver }
        : { reason: "Unknown", data };
    }
    case "ERC721InvalidOperator": {
      const operator = arg(decoded, "operator");
      return typeof operator === "string" && isAddress(operator)
        ? { reason: "InvalidOperator", operator }
        : { reason: "Unknown", data };
    }
    default:
      return { reason: "Unknown", data };
  }
}
