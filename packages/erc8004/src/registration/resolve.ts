import type { AgentRegistry } from "../core/agent-registry.ts";
import { ResolveError } from "../core/errors.ts";
import { isRecord } from "../core/json.ts";
import { classifyAgentUri, DEFAULT_IPFS_GATEWAY } from "./agent-uri.ts";
import { verifyEndpointDomain, wellKnownRegistrationUrl } from "./domain.ts";
import { parseRegistrationFile } from "./file.ts";
import type { RegistrationFile } from "./file.ts";

const MAX_BYTES = 1024 * 1024;

type Fetch = (
  url: string,
  init?: { signal?: AbortSignal | undefined },
) => Promise<{
  ok: boolean;
  status: number;
  headers: {
    // oxlint-disable-next-line typescript/no-restricted-types -- fetch API returns null for missing headers.
    get: (name: string) => string | null;
  };
  text: () => Promise<string>;
  arrayBuffer: () => Promise<ArrayBuffer>;
}>;

/** Options for agentURI resolution — every I/O edge is injected. */
export type ResolveOptions = {
  fetch?: Fetch | undefined;
  ipfsGateway?: string | undefined;
  maxBytes?: number | undefined;
  signal?: AbortSignal | undefined;
};

async function fetchText(url: string, options: ResolveOptions): Promise<string> {
  const fetchFn = options.fetch ?? globalThis.fetch;
  if (fetchFn === undefined) {
    throw new ResolveError("UnsupportedScheme", "no fetch available to resolve the agentURI");
  }
  const init = options.signal === undefined ? undefined : { signal: options.signal };
  const response = await fetchFn(url, init);
  const maxBytes = options.maxBytes ?? MAX_BYTES;
  if (!response.ok) {
    throw new ResolveError("HttpStatus", `agentURI fetch failed: HTTP ${response.status}`, {
      status: response.status,
    });
  }
  const length = response.headers.get("content-length");
  if (length !== null && Math.trunc(Number(length)) > maxBytes) {
    throw new ResolveError(
      "TooLarge",
      `agentURI body declares ${length} bytes (limit ${maxBytes})`,
    );
  }
  const text = await response.text();
  if (new TextEncoder().encode(text).length > maxBytes) {
    throw new ResolveError("TooLarge", `agentURI body exceeds ${maxBytes} bytes`);
  }
  return text;
}

function parseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    throw new ResolveError("InvalidJson", "agentURI body is not valid JSON");
  }
}

/**
 * Resolves an agentURI to a parsed {@link RegistrationFile}: inline `data:` bodies decode directly;
 * `ipfs://` goes through `ipfsGateway`; `https://` is fetched; everything else
 * (`NotSet`/`UnsupportedScheme`) throws. JSON document errors propagate unchanged.
 */
export async function resolveAgentUri(
  uri: string,
  options: ResolveOptions = {},
): Promise<RegistrationFile> {
  const classified = classifyAgentUri(uri, options.ipfsGateway ?? DEFAULT_IPFS_GATEWAY);
  switch (classified.kind) {
    case "NotSet":
      throw new ResolveError("NotSet", "agentURI is not set");
    case "Data": {
      let json: unknown;
      try {
        json = JSON.parse(new TextDecoder().decode(classified.bytes));
      } catch {
        throw new ResolveError("InvalidJson", "data URI payload is not valid JSON");
      }
      return parseRegistrationFile(json);
    }
    case "Ipfs":
    case "Https": {
      const text = await fetchText(classified.url, options);
      return parseRegistrationFile(parseJson(text));
    }
    default:
      throw new ResolveError("UnsupportedScheme", `unsupported agentURI scheme`);
  }
}

/**
 * Fetches `{endpoint}/.well-known/agent-registration.json` and checks whether it registers `{
 * agentRegistry, agentId }` — `false` when the endpoint cannot host a well-known document, the
 * fetch fails, or the document is invalid.
 */
export async function checkEndpointDomain(
  endpoint: string,
  identity: { agentRegistry: AgentRegistry; agentId: bigint },
  options: ResolveOptions = {},
): Promise<boolean> {
  const url = wellKnownRegistrationUrl(endpoint);
  if (url === null) {
    return false;
  }
  let text: string;
  try {
    text = await fetchText(url, options);
  } catch {
    return false;
  }
  let json: unknown;
  try {
    json = JSON.parse(text);
  } catch {
    return false;
  }
  if (!isRecord(json)) {
    return false;
  }
  let file: RegistrationFile;
  try {
    file = parseRegistrationFile(json);
  } catch {
    return false;
  }
  return verifyEndpointDomain(file, {
    agentRegistry: identity.agentRegistry,
    agentId: identity.agentId,
  });
}
