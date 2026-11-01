import { decodeErrorResult, parseAbi } from "viem";
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

const errorAbi = [
  ...identityRegistryAbi,
  ...parseAbi(["error Error(string message)", "error Panic(uint256 code)"]),
];

/** Decodes revert data into one reason; never throws — undecodable payloads become `Unknown`. */
export function decodeRevert(data: Hex): RevertReason {
  let decoded;
  try {
    decoded = decodeErrorResult({ abi: errorAbi, data });
  } catch {
    return { reason: "Unknown", data };
  }
  switch (decoded.errorName) {
    case "Error": {
      const [message] = decoded.args;
      return ERROR_STRINGS[message] ?? { reason: "Message", message };
    }
    case "Panic":
      return { reason: "Panic", code: decoded.args[0] };

    case "ERC721NonexistentToken":
      return { reason: "AgentNotFound", agentId: decoded.args[0] };

    case "ERC721IncorrectOwner":
      return {
        reason: "IncorrectOwner",
        sender: decoded.args[0],
        agentId: decoded.args[1],
        owner: decoded.args[2],
      };

    case "ERC721InsufficientApproval":
      return {
        reason: "InsufficientApproval",
        operator: decoded.args[0],
        agentId: decoded.args[1],
      };

    case "ERC721InvalidReceiver":
      return { reason: "InvalidReceiver", receiver: decoded.args[0] };

    case "ERC721InvalidSender":
      return { reason: "InvalidSender", sender: decoded.args[0] };

    case "ERC721InvalidOwner":
      return { reason: "InvalidOwner", owner: decoded.args[0] };

    case "ERC721InvalidApprover":
      return { reason: "InvalidApprover", approver: decoded.args[0] };

    case "ERC721InvalidOperator":
      return { reason: "InvalidOperator", operator: decoded.args[0] };

    default:
      return { reason: "Unknown", data };
  }
}
