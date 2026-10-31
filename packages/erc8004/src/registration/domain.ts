import type { AgentRegistry } from "../core/agent-registry.ts";
import { parseAgentRegistry } from "../core/agent-registry.ts";
import type { RegistrationFile } from "./file.ts";

const WELL_KNOWN_PATH = "/.well-known/agent-registration.json";

/**
 * `https://{authority}/.well-known/agent-registration.json` — deterministic string rules, no URL
 * library. `null` when the endpoint is not usable.
 */
export function wellKnownRegistrationUrl(
  endpoint: string,
  // oxlint-disable-next-line typescript/no-restricted-types -- spec: unusable endpoints return null.
): string | null {
  if (!endpoint.startsWith("https://")) {
    return null;
  }
  const rest = endpoint.slice("https://".length);
  const end = rest.search(/[/?#]/);
  let authority = end === -1 ? rest : rest.slice(0, end);
  if (authority === "" || authority.includes("@")) {
    return null;
  }
  authority = authority.toLowerCase();
  if (authority.endsWith(":443")) {
    authority = authority.slice(0, -":443".length);
  }
  return `https://${authority}${WELL_KNOWN_PATH}`;
}

/**
 * True iff some registration's `agentRegistry` parses and equals `{chainId, address}` and `agentId`
 * equals. Unparseable entries are skipped.
 */
export function verifyEndpointDomain(
  file: RegistrationFile,
  expected: { agentRegistry: AgentRegistry; agentId: bigint },
): boolean {
  for (const registration of file.registrations ?? []) {
    if (registration.agentId !== expected.agentId) {
      continue;
    }
    let parsed: AgentRegistry;
    try {
      parsed = parseAgentRegistry(registration.agentRegistry);
    } catch {
      continue;
    }
    if (
      parsed.chainId === expected.agentRegistry.chainId &&
      parsed.address === expected.agentRegistry.address
    ) {
      return true;
    }
  }
  return false;
}
