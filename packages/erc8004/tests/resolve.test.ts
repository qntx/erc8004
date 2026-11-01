import { describe, expect, it } from "vite-plus/test";

import { checkEndpointDomain, resolveAgentUri } from "../src/index.ts";
import { createRegistrationFile, registrationFileToJson } from "../src/registration/index.ts";

const FILE_JSON = JSON.stringify(
  registrationFileToJson(
    createRegistrationFile({ name: "t", description: "d", image: "https://i.test/i.png" }),
  ),
);

function fakeFetch(
  body: string | Uint8Array,
  init?: { status?: number; length?: string; url?: string },
) {
  return async () =>
    Promise.resolve({
      ok: (init?.status ?? 200) >= 200 && (init?.status ?? 200) < 300,
      status: init?.status ?? 200,
      url: init?.url ?? "",
      headers: {
        get: (name: string) => (name === "content-length" ? (init?.length ?? null) : null),
      },
      arrayBuffer: async () =>
        Promise.resolve(
          typeof body === "string"
            ? new TextEncoder().encode(body).buffer
            : (body.buffer as ArrayBuffer),
        ),
    });
}

/**
 * Streams `chunks` through a `body.getReader()` fake; `cancelled` flips when the reader is
 * cancelled.
 */
function streamingFetch(chunks: ReadonlyArray<Uint8Array>, cancelled: { value: boolean }) {
  return async () =>
    Promise.resolve({
      ok: true,
      status: 200,
      url: "",
      headers: { get: () => null },
      arrayBuffer: async () => Promise.resolve(new ArrayBuffer(0)),
      body: {
        getReader: () => {
          let i = 0;
          return {
            read: async () =>
              Promise.resolve(
                i < chunks.length
                  ? { done: false, value: chunks[i++] }
                  : { done: true, value: undefined },
              ),
            cancel: async () => {
              cancelled.value = true;
              return Promise.resolve();
            },
          };
        },
      },
    });
}

describe("resolveAgentUri", () => {
  it("rejects an unset URI", async () => {
    await expect(resolveAgentUri("", { fetch: fakeFetch(FILE_JSON) })).rejects.toMatchObject({
      code: "NotSet",
    });
  });

  it("decodes a data URI inline", async () => {
    const uri = `data:application/json;base64,${btoa(FILE_JSON)}`;
    const file = await resolveAgentUri(uri);
    expect(file.name).toBe("t");
  });

  it("rejects invalid JSON in a data URI", async () => {
    const uri = `data:application/json;base64,${btoa("{oops")}`;
    await expect(resolveAgentUri(uri)).rejects.toMatchObject({ code: "InvalidJson" });
  });

  it("fetches https URIs", async () => {
    const file = await resolveAgentUri("https://a.test/agent.json", {
      fetch: fakeFetch(FILE_JSON),
    });
    expect(file.name).toBe("t");
  });

  it("routes ipfs URIs through the gateway", async () => {
    let seen = "";
    const file = await resolveAgentUri("ipfs://bafkreitest", {
      fetch: async (url: string) => {
        seen = url;
        return fakeFetch(FILE_JSON)();
      },
    });
    expect(file.name).toBe("t");
    expect(seen).toBe("https://ipfs.io/ipfs/bafkreitest");
  });

  it("rejects unsupported schemes", async () => {
    await expect(resolveAgentUri("ftp://x", { fetch: fakeFetch(FILE_JSON) })).rejects.toMatchObject(
      {
        code: "UnsupportedScheme",
      },
    );
  });

  it("maps non-2xx to HttpStatus with the status", async () => {
    await expect(
      resolveAgentUri("https://a.test/x", { fetch: fakeFetch("", { status: 404 }) }),
    ).rejects.toMatchObject({ code: "HttpStatus", status: 404 });
  });

  it("enforces maxBytes via content-length", async () => {
    await expect(
      resolveAgentUri("https://a.test/x", {
        fetch: fakeFetch(FILE_JSON, { length: "99999999" }),
        maxBytes: 1024,
      }),
    ).rejects.toMatchObject({ code: "TooLarge" });
  });

  it("enforces maxBytes on the actual body", async () => {
    await expect(
      resolveAgentUri("https://a.test/x", { fetch: fakeFetch(FILE_JSON), maxBytes: 8 }),
    ).rejects.toMatchObject({ code: "TooLarge" });
  });

  it("streams the body and cancels the reader on overflow", async () => {
    const cancelled = { value: false };
    const chunks = [
      new TextEncoder().encode("a".repeat(16)),
      new TextEncoder().encode("b".repeat(16)),
    ];
    await expect(
      resolveAgentUri("https://a.test/x", {
        fetch: streamingFetch(chunks, cancelled),
        maxBytes: 8,
      }),
    ).rejects.toMatchObject({ code: "TooLarge" });
    expect(cancelled.value).toBe(true);
  });

  it("reads a streamed body under the cap", async () => {
    const cancelled = { value: false };
    const json = new TextEncoder().encode(FILE_JSON);
    const chunks = [json.slice(0, 8), json.slice(8)];
    const file = await resolveAgentUri("https://a.test/x", {
      fetch: streamingFetch(chunks, cancelled),
    });
    expect(file.name).toBe("t");
    expect(cancelled.value).toBe(false);
  });

  it("rejects a redirect downgraded to non-https", async () => {
    await expect(
      resolveAgentUri("https://a.test/x", {
        fetch: fakeFetch(FILE_JSON, { url: "http://evil.test/x" }),
      }),
    ).rejects.toMatchObject({ code: "UnsupportedScheme" });
  });

  it("maps a rejected fetch to Transport with the original cause", async () => {
    const cause = new Error("network down");
    await expect(
      resolveAgentUri("https://a.test/x", {
        fetch: async () => Promise.reject(cause),
      }),
    ).rejects.toMatchObject({ code: "Transport", cause });
  });

  it("rethrows aborts instead of wrapping them", async () => {
    const controller = new AbortController();
    controller.abort();
    const cause = new Error("aborted");
    await expect(
      resolveAgentUri("https://a.test/x", {
        fetch: async () => Promise.reject(cause),
        signal: controller.signal,
      }),
    ).rejects.toBe(cause);
  });
});

describe("checkEndpointDomain", () => {
  const file = createRegistrationFile({
    name: "t",
    description: "d",
    image: "x",
    registrations: [
      {
        agentId: 1n,
        agentRegistry: "eip155:1:0x0000000000000000000000000000000000000001",
        extra: {},
      },
    ],
  });
  const doc = JSON.stringify(registrationFileToJson(file));

  it("verifies a matching registration", async () => {
    const ok = await checkEndpointDomain(
      "https://a.test",
      {
        agentRegistry: { chainId: 1, address: "0x0000000000000000000000000000000000000001" },
        agentId: 1n,
      },
      { fetch: fakeFetch(doc) },
    );
    expect(ok).toBe(true);
  });

  it("is false for a non-http endpoint", async () => {
    const ok = await checkEndpointDomain(
      "ftp://a.test",
      {
        agentRegistry: { chainId: 1, address: "0x0000000000000000000000000000000000000001" },
        agentId: 1n,
      },
      { fetch: fakeFetch(doc) },
    );
    expect(ok).toBe(false);
  });

  it("is false when the fetch fails", async () => {
    const ok = await checkEndpointDomain(
      "https://a.test",
      {
        agentRegistry: { chainId: 1, address: "0x0000000000000000000000000000000000000001" },
        agentId: 1n,
      },
      { fetch: async () => Promise.reject(new Error("net down")) },
    );
    expect(ok).toBe(false);
  });
});
