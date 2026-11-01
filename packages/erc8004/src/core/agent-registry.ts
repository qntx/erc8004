import { getAddress, isAddress } from "viem";
import type { Address } from "viem";

import { AgentRegistryError } from "./errors.ts";

/** `eip155:{chainId}:{address}` — the on-chain identity of an agent. */
export type AgentRegistry = {
  readonly chainId: number;
  readonly address: Address;
};

const CHAIN_ID = /^[1-9][0-9]*$/;
const MAX_CHAIN_ID = 9007199254740991; // 2^53 - 1

/**
 * Parses `eip155:{chainId}:{address}`; the address must be all-lowercase, all-uppercase, or valid
 * EIP-55.
 */
export function parseAgentRegistry(text: string): AgentRegistry {
  const parts = text.split(":");
  if (parts.length !== 3 || parts.some((part) => part === "")) {
    throw new AgentRegistryError(
      "Malformed",
      `agent registry must be "eip155:{chainId}:{address}", got ${JSON.stringify(text)}`,
    );
  }
  const [namespace, chainIdText, addressText] = parts;
  if (
    parts.length !== 3 ||
    namespace === undefined ||
    namespace === "" ||
    chainIdText === undefined ||
    chainIdText === "" ||
    addressText === undefined ||
    addressText === ""
  ) {
    throw new AgentRegistryError(
      "Malformed",
      `agent registry must be "eip155:{chainId}:{address}", got ${JSON.stringify(text)}`,
    );
  }
  if (namespace !== "eip155") {
    throw new AgentRegistryError(
      "UnsupportedNamespace",
      `unsupported namespace ${JSON.stringify(namespace)}`,
    );
  }
  if (!CHAIN_ID.test(chainIdText)) {
    throw new AgentRegistryError(
      "InvalidChainId",
      `invalid chain id ${JSON.stringify(chainIdText)}`,
    );
  }
  const chainId = Number(chainIdText);
  if (chainId > MAX_CHAIN_ID) {
    throw new AgentRegistryError("InvalidChainId", `chain id ${chainIdText} exceeds 2^53-1`);
  }
  if (!isAddress(addressText, { strict: false })) {
    throw new AgentRegistryError(
      "InvalidAddress",
      `invalid address ${JSON.stringify(addressText)}`,
    );
  }
  const hex = addressText.slice(2);
  const isLowerOrUpper = hex === hex.toLowerCase() || hex === hex.toUpperCase();
  if (!isLowerOrUpper && !isAddress(addressText)) {
    throw new AgentRegistryError(
      "BadChecksum",
      `bad EIP-55 checksum in ${JSON.stringify(addressText)}`,
    );
  }
  return { chainId, address: getAddress(addressText) };
}

/** Always formats the address EIP-55. */
export function formatAgentRegistry(registry: AgentRegistry): string {
  return `eip155:${registry.chainId}:${getAddress(registry.address)}`;
}
