import { base64 } from "@scure/base";

import { DataUriError } from "../core/errors.ts";

const PREFIX = "data:";
const MEDIA_TYPE = "application/json";
const BASE64_FLAG = ";base64";

/**
 * Decodes `data:<mediatype>[;params];base64,<payload>` — media type `application/json`
 * case-insensitive, extra params allowed.
 */
export function decodeDataUri(uri: string): Uint8Array {
  if (!uri.toLowerCase().startsWith(PREFIX)) {
    throw new DataUriError("NotDataUri", `not a data URI: ${JSON.stringify(uri)}`);
  }
  const comma = uri.indexOf(",");
  if (comma === -1) {
    throw new DataUriError(
      "NotDataUri",
      `data URI missing payload separator: ${JSON.stringify(uri)}`,
    );
  }
  const header = uri.slice(PREFIX.length, comma).toLowerCase();
  if (!header.endsWith(BASE64_FLAG)) {
    throw new DataUriError("NotBase64", `data URI is not base64-encoded: ${JSON.stringify(uri)}`);
  }
  const [mediaType] = header.slice(0, -BASE64_FLAG.length).split(";");
  if (mediaType !== MEDIA_TYPE) {
    throw new DataUriError(
      "UnsupportedMediaType",
      `unsupported media type ${JSON.stringify(mediaType)}`,
    );
  }
  try {
    return base64.decode(uri.slice(comma + 1));
  } catch {
    throw new DataUriError("InvalidBase64", `invalid base64 payload in ${JSON.stringify(uri)}`);
  }
}

/** Encodes JSON bytes as `data:application/json;base64,<padded>`. */
export function encodeDataUri(bytes: Uint8Array): string {
  return `${PREFIX}${MEDIA_TYPE}${BASE64_FLAG},${base64.encode(bytes)}`;
}
