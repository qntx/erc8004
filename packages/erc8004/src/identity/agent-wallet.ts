import { hashTypedData } from "viem";
import type { Address, Hex, TypedDataDefinition } from "viem";

import { AgentWalletDeadlineError } from "../core/errors.ts";

/** `AgentWalletSet` EIP-712 typed-data message for `setAgentWallet` / `unsetAgentWallet`. */
export type AgentWalletMessage = {
  readonly agentId: bigint;
  readonly newWallet: Address;
  readonly owner: Address;
  readonly deadline: bigint;
};

export const AGENT_WALLET_TYPES = {
  AgentWalletSet: [
    { name: "agentId", type: "uint256" },
    { name: "newWallet", type: "address" },
    { name: "owner", type: "address" },
    { name: "deadline", type: "uint256" },
  ],
} as const;

/**
 * The `AgentWalletSet` typed-data definition, usable directly with viem
 * `signTypedData`/`hashTypedData`.
 */
export function agentWalletTypedData(input: {
  chainId: number;
  identityRegistry: Address;
  message: AgentWalletMessage;
}): TypedDataDefinition<typeof AGENT_WALLET_TYPES, "AgentWalletSet"> {
  return {
    domain: {
      name: "ERC8004IdentityRegistry",
      version: "1",
      chainId: input.chainId,
      verifyingContract: input.identityRegistry,
    },
    types: AGENT_WALLET_TYPES,
    primaryType: "AgentWalletSet",
    message: input.message,
  };
}

/** EIP-712 hash of the `AgentWalletSet` message. */
export function hashAgentWallet(input: {
  chainId: number;
  identityRegistry: Address;
  message: AgentWalletMessage;
}): Hex {
  return hashTypedData(agentWalletTypedData(input));
}

/**
 * Contract check: `block.timestamp <= deadline <= block.timestamp + 5 minutes`. Throws
 * {@link AgentWalletDeadlineError}.
 */
export function assertAgentWalletDeadline(deadline: bigint, now: bigint): void {
  if (deadline < now) {
    throw new AgentWalletDeadlineError(
      "DeadlineExpired",
      `deadline ${deadline} is in the past (now ${now})`,
    );
  }
  if (deadline > now + 300n) {
    throw new AgentWalletDeadlineError(
      "DeadlineTooFar",
      `deadline ${deadline} is more than 5 minutes ahead (now ${now})`,
    );
  }
}
