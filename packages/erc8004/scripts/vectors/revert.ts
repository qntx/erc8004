import { encodeAbiParameters, toFunctionSelector } from "viem";
import type { Hex } from "viem";

import { identityRegistryAbi } from "../../src/abi/index.ts";

const A = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed" as const;
const B = "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359" as const;
const C = "0x8004A169FB4a3325136EB29fA0ceB6D2e539a432" as const;

function errorString(message: string): Hex {
  return `0x08c379a0${encodeAbiParameters([{ type: "string" }], [message]).slice(2)}` as Hex;
}

function panic(code: bigint): Hex {
  return `0x4e487b71${encodeAbiParameters([{ type: "uint256" }], [code]).slice(2)}` as Hex;
}

function custom(name: string, args: ReadonlyArray<unknown>): Hex {
  for (const entry of identityRegistryAbi) {
    if (entry.type === "error" && entry.name === name) {
      const signature = `${name}(${entry.inputs.map((input) => input.type).join(",")})`;
      const params = entry.inputs.map((input) => ({ type: input.type }));
      return `0x${toFunctionSelector(signature).slice(2)}${encodeAbiParameters(params, args).slice(2)}`;
    }
  }
  throw new Error(`error ${name} not in Identity ABI`);
}

export function build(): { capability: string; cases: unknown[] } {
  const cases: Array<{ input: { data: Hex }; output: Record<string, unknown> }> = [];
  const push = (data: Hex, output: Record<string, unknown>) =>
    cases.push({ input: { data }, output });

  const strings: Array<[string, Record<string, unknown>]> = [
    ["Not authorized", { reason: "NotAuthorized" }],
    ["reserved key", { reason: "ReservedMetadataKey" }],
    ["bad wallet", { reason: "InvalidWallet" }],
    ["expired", { reason: "DeadlineExpired" }],
    ["deadline too far", { reason: "DeadlineTooFar" }],
    ["invalid wallet sig", { reason: "InvalidWalletSignature" }],
    ["too many decimals", { reason: "TooManyDecimals" }],
    ["value too large", { reason: "ValueOutOfRange" }],
    ["Self-feedback not allowed", { reason: "SelfFeedback" }],
    ["index must be > 0", { reason: "InvalidFeedbackIndex" }],
    ["index out of bounds", { reason: "FeedbackIndexOutOfBounds" }],
    ["Already revoked", { reason: "AlreadyRevoked" }],
    ["Empty URI", { reason: "EmptyResponseUri" }],
    ["clientAddresses required", { reason: "ClientAddressesRequired" }],
    ["bad validator", { reason: "InvalidValidator" }],
    ["exists", { reason: "RequestExists" }],
    ["unknown", { reason: "UnknownRequest" }],
    ["not validator", { reason: "NotValidator" }],
    ["resp>100", { reason: "ResponseOutOfRange" }],
    ["something else entirely", { reason: "Message", message: "something else entirely" }],
    ["", { reason: "Message", message: "" }],
    [
      "Error message with unicode 你好 🚀",
      { reason: "Message", message: "Error message with unicode 你好 🚀" },
    ],
  ];
  for (const [message, output] of strings) {
    push(errorString(message), output);
  }

  push(panic(0x11n), { reason: "Panic", code: "17" });
  push(panic(0x01n), { reason: "Panic", code: "1" });
  push(panic(0x21n), { reason: "Panic", code: "33" });

  push(custom("ERC721NonexistentToken", [5n]), { reason: "AgentNotFound", agentId: "5" });
  push(custom("ERC721NonexistentToken", [0n]), { reason: "AgentNotFound", agentId: "0" });
  push(custom("ERC721IncorrectOwner", [A, 7n, B]), {
    reason: "IncorrectOwner",
    sender: A,
    agentId: "7",
    owner: B,
  });
  push(custom("ERC721InsufficientApproval", [B, 42n]), {
    reason: "InsufficientApproval",
    operator: B,
    agentId: "42",
  });
  push(custom("ERC721InvalidReceiver", [C]), { reason: "InvalidReceiver", receiver: C });
  push(custom("ERC721InvalidSender", [A]), { reason: "InvalidSender", sender: A });
  push(custom("ERC721InvalidOwner", [B]), { reason: "InvalidOwner", owner: B });
  push(custom("ERC721InvalidApprover", [C]), { reason: "InvalidApprover", approver: C });
  push(custom("ERC721InvalidOperator", [A]), { reason: "InvalidOperator", operator: A });

  push(custom("OwnableUnauthorizedAccount", [A]), {
    reason: "Unknown",
    data: custom("OwnableUnauthorizedAccount", [A]),
  });
  push(custom("UUPSUnauthorizedCallContext", []), {
    reason: "Unknown",
    data: custom("UUPSUnauthorizedCallContext", []),
  });
  push("0x", { reason: "Unknown", data: "0x" });
  push("0xdeadbeef", { reason: "Unknown", data: "0xdeadbeef" });
  push("0x08c379a0", { reason: "Unknown", data: "0x08c379a0" }); // Error(string) selector, no payload
  push(`0x08c379a0${"00".repeat(10)}`, { reason: "Unknown", data: `0x08c379a0${"00".repeat(10)}` });
  push("0x4e487b71", { reason: "Unknown", data: "0x4e487b71" }); // Panic selector, no payload
  push(custom("ERC721NonexistentToken", [5n]).slice(0, 10) as Hex, {
    reason: "Unknown",
    data: custom("ERC721NonexistentToken", [5n]).slice(0, 10),
  });

  return { capability: "core.revert", cases };
}
