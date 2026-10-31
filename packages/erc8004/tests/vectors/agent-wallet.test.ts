import type { Address } from "viem";
import { describe, expect, it } from "vite-plus/test";

import { assertAgentWalletDeadline, hashAgentWallet } from "../../src/identity/agent-wallet.ts";
import { expectError, loadVector, named } from "./helpers.ts";

const cases = loadVector("vectors/identity/agent-wallet.json", "identity.agent-wallet");

describe("identity.agent-wallet", () => {
  it.each(named(cases, "hash"))("hashes %s", (_title, vectorCase) => {
    const input = vectorCase["input"] as {
      chainId: number;
      identityRegistry: Address;
      message: { agentId: string; newWallet: Address; owner: Address; deadline: string };
    };
    const hash = hashAgentWallet({
      chainId: input.chainId,
      identityRegistry: input.identityRegistry,
      message: {
        agentId: BigInt(input.message.agentId),
        newWallet: input.message.newWallet,
        owner: input.message.owner,
        deadline: BigInt(input.message.deadline),
      },
    });
    expect(hash).toStrictEqual(vectorCase["output"]);
  });

  it.each(named(cases, "deadline"))("accepts %s", (_title, vectorCase) => {
    const input = vectorCase["input"] as { deadline: string; now: string };
    expect(() =>
      assertAgentWalletDeadline(BigInt(input.deadline), BigInt(input.now)),
    ).not.toThrow();
  });

  it.each(named(cases, "deadline", "err"))("rejects %s", (_title, vectorCase) => {
    expect.hasAssertions();
    const input = vectorCase["input"] as { deadline: string; now: string };
    expectError(
      () => assertAgentWalletDeadline(BigInt(input.deadline), BigInt(input.now)),
      vectorCase["error"],
    );
  });
});
