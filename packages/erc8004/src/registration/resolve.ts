import type { AgentRegistry } from "../core/agent-registry.ts";
import { ResolveError } from "../core/errors.ts";
import { classifyAgentUri, DEFAULT_IPFS_GATEWAY } from "./agent-uri.ts";
import { verifyEndpointDomain, wellKnownRegistrationUrl } from "./domain.ts";
import { parseRegistrationFile } from "./file.ts";
import type { RegistrationFile } from "./file.ts";

const MAX_BYTES = 1024 * 1024;

type FetchBody = {
  getReader: () => {
    read: () => Promise<{ done: boolean; value?: Uint8Array | undefined }>;
    cancel: () => Promise<void>;
  };
};

type FetchResponse = {
  ok: boolean;
  status: number;
  /** Final URL after redirects; may be empty for synthetic responses. */
  url: string;
  headers: {
    // oxlint-disable-next-line typescript/no-restricted-types -- fetch API returns null for missing headers.
    get: (name: string) => string | null;
  };
  arrayBuffer: () => Promise<ArrayBuffer>;
  // oxlint-disable-next-line typescript/no-restricted-types -- fetch API uses `null` for missing bodies.
  body?: FetchBody | null | undefined;
};

type Fetch = (url: string, init?: { signal?: AbortSignal | undefined }) => Promise<FetchResponse>;

/** Options for agentURI resolution — every I/O edge is injected. */
export type ResolveOptions = {
  fetch?: Fetch | undefined;
  ipfsGateway?: string | undefined;
  maxBytes?: number | undefined;
  signal?: AbortSignal | undefined;
};

async function fetchBody(url: string, options: ResolveOptions): Promise<Uint8Array> {
  const fetchFn = options.fetch ?? globalThis.fetch;
  if (fetchFn === undefined) {
    throw new ResolveError("Transport", "no fetch available to resolve the agentURI");
  }
  const init = options.signal === undefined ? undefined : { signal: options.signal };
  let response: FetchResponse;
  try {
    response = await fetchFn(url, init);
  } catch (error) {
    if (options.signal?.aborted === true) {
      throw error;
    }
    throw new ResolveError("Transport", "agentURI fetch failed", { cause: error });
  }
  const maxBytes = options.maxBytes ?? MAX_BYTES;
  if (response.url !== "" && !response.url.startsWith("https://")) {
    throw new ResolveError(
      "UnsupportedScheme",
      `agentURI redirect downgraded to non-HTTPS: ${response.url}`,
    );
  }
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
  if (response.body?.getReader !== undefined) {
    const reader = response.body.getReader();
    const chunks: Uint8Array[] = [];
    let total = 0;
    try {
      for (;;) {
        // oxlint-disable-next-line eslint/no-await-in-loop -- stream reads are inherently sequential.
        const { done, value } = await reader.read();
        if (done) {
          break;
        }
        if (value !== undefined) {
          total += value.byteLength;
          if (total > maxBytes) {
            // oxlint-disable-next-line eslint/no-await-in-loop -- cancelling mid-stream is sequential.
            await reader.cancel();
            throw new ResolveError("TooLarge", `agentURI body exceeds ${maxBytes} bytes`);
          }
          chunks.push(value);
        }
      }
    } catch (error) {
      if (error instanceof ResolveError || options.signal?.aborted === true) {
        throw error;
      }
      throw new ResolveError("Transport", "agentURI body read failed", { cause: error });
    }
    const out = new Uint8Array(total);
    let offset = 0;
    for (const chunk of chunks) {
      out.set(chunk, offset);
      offset += chunk.byteLength;
    }
    return out;
  }
  const buffer = await response.arrayBuffer();
  if (buffer.byteLength > maxBytes) {
    throw new ResolveError("TooLarge", `agentURI body exceeds ${maxBytes} bytes`);
  }
  return new Uint8Array(buffer);
}

function parseJsonBytes(bytes: Uint8Array): unknown {
  try {
    return JSON.parse(new TextDecoder().decode(bytes));
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
    case "Data":
      return parseRegistrationFile(parseJsonBytes(classified.bytes));
    case "Ipfs":
    case "Https":
      return parseRegistrationFile(parseJsonBytes(await fetchBody(classified.url, options)));
    default:
      throw new ResolveError("UnsupportedScheme", `unsupported agentURI scheme`);
  }
}

/**
 * Fetches `{endpoint}/.well-known/agent-registration.json` and checks whether it registers `{
 * agentRegistry, agentId }` — `false` when the endpoint cannot host a well-known document, the
 * fetch fails, or the document is invalid. Aborted signals rethrow.
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
  let bytes: Uint8Array;
  try {
    bytes = await fetchBody(url, options);
  } catch (error) {
    if (options.signal?.aborted === true) {
      throw error;
    }
    return false;
  }
  let file: RegistrationFile;
  try {
    file = parseRegistrationFile(parseJsonBytes(bytes));
  } catch {
    return false;
  }
  return verifyEndpointDomain(file, {
    agentRegistry: identity.agentRegistry,
    agentId: identity.agentId,
  });
}
