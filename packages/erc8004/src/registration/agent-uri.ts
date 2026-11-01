import { AgentUriError } from "../core/errors.ts";
import { decodeDataUri } from "./data-uri.ts";

/** Default IPFS gateway. */
export const DEFAULT_IPFS_GATEWAY = "https://ipfs.io/ipfs/";

/**
 * AgentURI classification: `""` NotSet; `data:` decoded inline; `ipfs://` via a configurable
 * gateway; `https://` fetched; everything else UnsupportedScheme.
 */
export type AgentUri =
  | Readonly<{ kind: "NotSet" }>
  | Readonly<{ kind: "Data"; bytes: Uint8Array }>
  | Readonly<{ kind: "Ipfs"; url: string }>
  | Readonly<{ kind: "Https"; url: string }>
  | Readonly<{ kind: "UnsupportedScheme" }>;

function startsWith(text: string, prefix: string): boolean {
  return text.toLowerCase().startsWith(prefix);
}

/** Classifies an agentURI; bad data URIs surface {@link DataUriError}. */
export function classifyAgentUri(
  uri: string,
  ipfsGateway: string = DEFAULT_IPFS_GATEWAY,
): AgentUri {
  if (uri === "") {
    return { kind: "NotSet" };
  }
  if (startsWith(uri, "data:")) {
    return { kind: "Data", bytes: decodeDataUri(uri) };
  }
  if (startsWith(uri, "ipfs://")) {
    const rest = uri.slice("ipfs://".length);
    if (rest === "") {
      return { kind: "UnsupportedScheme" };
    }
    if (!ipfsGateway.endsWith("/")) {
      throw new AgentUriError(
        "InvalidGateway",
        `IPFS gateway must end with "/", got ${JSON.stringify(ipfsGateway)}`,
      );
    }
    return { kind: "Ipfs", url: ipfsGateway + rest };
  }
  if (startsWith(uri, "https://")) {
    return { kind: "Https", url: uri };
  }
  return { kind: "UnsupportedScheme" };
}
