import { describe, expect, it } from "vite-plus/test";

import { checkEndpointDomain, resolveAgentUri } from "../src/index.ts";
import { createRegistrationFile, registrationFileToJson } from "../src/registration/index.ts";

const FILE_JSON = JSON.stringify(
  registrationFileToJson(
    createRegistrationFile({ name: "t", description: "d", image: "https://i.test/i.png" }),
  ),
);

function fakeFetch(body: string | Uint8Array, init?: { status?: number; length?: string }) {
  return async () =>
    Promise.resolve({
      ok: (init?.status ?? 200) >= 200 && (init?.status ?? 200) < 300,
      status: init?.status ?? 200,
      headers: {
        get: (name: string) => (name === "content-length" ? (init?.length ?? null) : null),
      },
      text: async () =>
        Promise.resolve(typeof body === "string" ? body : new TextDecoder().decode(body)),
      arrayBuffer: async () =>
        Promise.resolve(
          typeof body === "string"
            ? new TextEncoder().encode(body).buffer
            : (body.buffer as ArrayBuffer),
        ),
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
