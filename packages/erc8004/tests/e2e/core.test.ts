import { readFileSync } from "node:fs";

import type { Address } from "viem";
import { createPublicClient, createWalletClient, http, keccak256, toBytes } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { getBlock, waitForTransactionReceipt } from "viem/actions";
import { foundry } from "viem/chains";
import { describe, expect, it } from "vite-plus/test";

import {
  getAgent,
  getAgentOwner,
  getMetadata,
  parseRegisteredAgentId,
  registerAgent,
  setAgentUri,
  setAgentWallet,
  setMetadata,
  signAgentWallet,
  transferAgent,
  unsetAgentWallet,
} from "../../src/identity/index.ts";
import {
  ClientGuardError,
  ContractRevertError,
  decodeErc8004Log,
  parseFeedbackValue,
} from "../../src/index.ts";
import {
  appendResponse,
  getReputationSummary,
  getResponseCount,
  giveFeedback,
  parseFeedbackIndex,
  readAllFeedback,
  readFeedback,
  revokeFeedback,
} from "../../src/reputation/index.ts";
import {
  getValidationStatus,
  getValidationSummary,
  requestValidation,
  respondToValidation,
} from "../../src/validation/index.ts";

const DEPLOYMENT_PATH = process.env["ERC8004_E2E_DEPLOYMENT"];
const RPC = process.env["ERC8004_E2E_RPC"] ?? "http://127.0.0.1:8545";

// anvil default accounts: #0 owner, #1 client, #2 validator, #3 new wallet
const KEYS = [
  "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
  "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
  "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
  "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
] as const;

type DeploymentFile = {
  chainId: number;
  identityRegistry: Address;
  reputationRegistry: Address;
  validationRegistry: Address;
};

describe("e2e", { timeout: 30_000 }, () => {
  if (DEPLOYMENT_PATH === undefined) {
    // no anvil deployment — e2e/run.sh sets ERC8004_E2E_DEPLOYMENT
    it.skip("skipped (no deployment.json)", () => {
      expect.unreachable("anvil deployment.json required");
    });
    return;
  }
  const deployment = JSON.parse(readFileSync(DEPLOYMENT_PATH, "utf8")) as DeploymentFile;
  const owner = privateKeyToAccount(KEYS[0]);
  const client = privateKeyToAccount(KEYS[1]);
  const validator = privateKeyToAccount(KEYS[2]);
  const newWallet = privateKeyToAccount(KEYS[3]);
  const chain = { ...foundry, id: deployment.chainId };
  const publicClient = createPublicClient({ chain, transport: http(RPC) });
  const ownerW = createWalletClient({ account: owner, chain, transport: http(RPC) });
  const clientW = createWalletClient({ account: client, chain, transport: http(RPC) });
  const validatorW = createWalletClient({ account: validator, chain, transport: http(RPC) });
  const newWalletW = createWalletClient({ account: newWallet, chain, transport: http(RPC) });
  const idArgs = { address: deployment.identityRegistry } as const;
  const repArgs = { address: deployment.reputationRegistry } as const;
  const valArgs = { address: deployment.validationRegistry } as const;

  it("registers an agent with uri + metadata and reads it back", async () => {
    const hash = await registerAgent(ownerW, {
      ...idArgs,
      agentUri: "ipfs://bafkrei/test-agent",
      metadata: [{ key: "endpoint", value: "0x68656c6c6f" }],
    });
    const receipt = await waitForTransactionReceipt(publicClient, { hash });
    const events = receipt.logs.map(decodeErc8004Log).filter((e) => e !== null);
    expect(events.some((e) => e.eventName === "Registered")).toBe(true);
    const agentId = parseRegisteredAgentId(receipt.logs);

    const agent = await getAgent(publicClient, { ...idArgs, agentId });
    expect(agent.owner).toBe(owner.address);
    expect(agent.agentUri).toBe("ipfs://bafkrei/test-agent");
    // register() defaults the agent wallet to the owner
    expect(agent.agentWallet).toBe(owner.address);
  });

  it("updates agentURI and metadata; agentWallet key is reserved", async () => {
    const hash = await registerAgent(ownerW, { ...idArgs, agentUri: "" });
    const receipt = await waitForTransactionReceipt(publicClient, { hash });
    const agentId = parseRegisteredAgentId(receipt.logs);

    await waitForTransactionReceipt(publicClient, {
      hash: await setAgentUri(ownerW, {
        ...idArgs,
        agentId,
        agentUri: "https://a.test/agent.json",
      }),
    });
    const updated = await getAgent(publicClient, { ...idArgs, agentId });
    expect(updated.agentUri).toBe("https://a.test/agent.json");

    const metaHash = await setMetadata(ownerW, {
      ...idArgs,
      agentId,
      key: "version",
      value: "0x01",
    });
    await waitForTransactionReceipt(publicClient, { hash: metaHash });
    expect(await getMetadata(publicClient, { ...idArgs, agentId, key: "version" })).toBe("0x01");

    await expect(
      setMetadata(ownerW, { ...idArgs, agentId, key: "agentWallet", value: "0x01" }),
    ).rejects.toMatchObject({ code: "ReservedMetadataKey" });
    await expect(
      setMetadata(ownerW, { ...idArgs, agentId, key: "agentWallet", value: "0x01" }),
    ).rejects.toThrow(ClientGuardError);
  });

  it("sets and unsets the agent wallet with an EIP-712 signature", async () => {
    const hash = await registerAgent(ownerW, { ...idArgs, agentUri: "" });
    const receipt = await waitForTransactionReceipt(publicClient, { hash });
    const agentId = parseRegisteredAgentId(receipt.logs);

    const block = await getBlock(publicClient);
    const deadline = block.timestamp + 60n;
    const signature = await signAgentWallet(newWalletW, {
      chainId: deployment.chainId,
      identityRegistry: deployment.identityRegistry,
      agentId,
      owner: owner.address,
      deadline,
    });
    await waitForTransactionReceipt(publicClient, {
      hash: await setAgentWallet(ownerW, {
        ...idArgs,
        agentId,
        newWallet: newWallet.address,
        deadline,
        signature,
      }),
    });
    const withWallet = await getAgent(publicClient, { ...idArgs, agentId });
    expect(withWallet.agentWallet).toBe(newWallet.address);

    // deadline too far → client guard
    const far = block.timestamp + 600n;
    const sig2 = await signAgentWallet(newWalletW, {
      chainId: deployment.chainId,
      identityRegistry: deployment.identityRegistry,
      agentId,
      owner: owner.address,
      deadline: far,
    });
    await expect(
      setAgentWallet(ownerW, {
        ...idArgs,
        agentId,
        newWallet: newWallet.address,
        deadline: far,
        signature: sig2,
      }),
    ).rejects.toMatchObject({ code: "DeadlineTooFar" });

    await waitForTransactionReceipt(publicClient, {
      hash: await unsetAgentWallet(ownerW, { ...idArgs, agentId }),
    });
    const unset = await getAgent(publicClient, { ...idArgs, agentId });
    expect(unset.agentWallet).toBeNull();
  });

  it("feedback lifecycle: give, read, summarize, revoke, respond", async () => {
    const hash = await registerAgent(ownerW, { ...idArgs, agentUri: "" });
    const receipt = await waitForTransactionReceipt(publicClient, { hash });
    const agentId = parseRegisteredAgentId(receipt.logs);

    const fbHash = await giveFeedback(clientW, {
      ...repArgs,
      agentId,
      value: parseFeedbackValue("87.50"),
      tag1: "quality",
      tag2: "speed",
    });
    const fbReceipt = await waitForTransactionReceipt(publicClient, { hash: fbHash });
    expect(fbReceipt.logs.map(decodeErc8004Log).some((e) => e?.eventName === "NewFeedback")).toBe(
      true,
    );
    const index = parseFeedbackIndex(fbReceipt.logs);
    expect(index).toBe(1n);

    // self-feedback reverts
    await expect(
      giveFeedback(ownerW, { ...repArgs, agentId, value: parseFeedbackValue("1") }),
    ).rejects.toMatchObject({ code: "ContractRevert" });
    await expect(
      giveFeedback(ownerW, { ...repArgs, agentId, value: parseFeedbackValue("1") }),
    ).rejects.toThrow(ContractRevertError);

    const fb = await readFeedback(publicClient, {
      ...repArgs,
      agentId,
      client: client.address,
      feedbackIndex: index,
    });
    expect(fb.value.value).toBe(8750n);
    expect(fb.value.decimals).toBe(2);
    expect(fb.tag1).toBe("quality");
    expect(fb.isRevoked).toBe(false);

    const all = await readAllFeedback(publicClient, {
      ...repArgs,
      agentId,
      clients: [client.address],
    });
    expect(all).toHaveLength(1);
    expect(all[0]?.feedbackIndex).toBe(index);

    const summary = await getReputationSummary(publicClient, {
      ...repArgs,
      agentId,
      clients: [client.address],
    });
    expect(summary.count).toBe(1n);
    expect(summary.value.value).toBe(8750n);
    expect(summary.value.decimals).toBe(2);

    // empty clients → client guard
    await expect(
      getReputationSummary(publicClient, { ...repArgs, agentId, clients: [] }),
    ).rejects.toMatchObject({ code: "ClientAddressesRequired" });

    // append a response then read the count
    await waitForTransactionReceipt(publicClient, {
      hash: await appendResponse(clientW, {
        ...repArgs,
        agentId,
        client: client.address,
        feedbackIndex: index,
        responseUri: "ipfs://response",
      }),
    });
    const count = await getResponseCount(publicClient, {
      ...repArgs,
      agentId,
      client: client.address,
      feedbackIndex: index,
      responders: [client.address],
    });
    expect(count).toBe(1n);

    // revoke, then revoke again → AlreadyRevoked
    await waitForTransactionReceipt(publicClient, {
      hash: await revokeFeedback(clientW, { ...repArgs, agentId, feedbackIndex: index }),
    });
    await expect(
      revokeFeedback(clientW, { ...repArgs, agentId, feedbackIndex: index }),
    ).rejects.toMatchObject({ code: "ContractRevert" });

    // empty response URI → client guard
    await expect(
      appendResponse(clientW, {
        ...repArgs,
        agentId,
        client: client.address,
        feedbackIndex: index,
        responseUri: "",
      }),
    ).rejects.toMatchObject({ code: "EmptyResponseUri" });
  });

  it("validation lifecycle: request, respond, status, summary", async () => {
    const hash = await registerAgent(ownerW, { ...idArgs, agentUri: "" });
    const receipt = await waitForTransactionReceipt(publicClient, { hash });
    const agentId = parseRegisteredAgentId(receipt.logs);
    const requestHash = keccak256(toBytes(`req-${agentId}-${Date.now()}`));

    await waitForTransactionReceipt(publicClient, {
      hash: await requestValidation(ownerW, {
        ...valArgs,
        validator: validator.address,
        agentId,
        requestUri: "ipfs://req",
        requestHash,
      }),
    });

    // out-of-range response → client guard
    await expect(
      respondToValidation(validatorW, { ...valArgs, requestHash, response: 101 }),
    ).rejects.toMatchObject({ code: "ResponseOutOfRange" });

    // wrong responder → contract revert
    await expect(
      respondToValidation(clientW, { ...valArgs, requestHash, response: 50 }),
    ).rejects.toMatchObject({ code: "ContractRevert" });

    await waitForTransactionReceipt(publicClient, {
      hash: await respondToValidation(validatorW, {
        ...valArgs,
        requestHash,
        response: 100,
        tag: "hard",
      }),
    });
    const status = await getValidationStatus(publicClient, { ...valArgs, requestHash });
    expect(status).not.toBeNull();
    expect(status?.validator).toBe(validator.address);
    expect(status?.response).toBe(100);
    expect(status?.tag).toBe("hard");

    const summary = await getValidationSummary(publicClient, {
      ...valArgs,
      agentId,
      validators: [validator.address],
    });
    expect(summary.count).toBe(1n);
    expect(summary.averageResponse).toBe(100);

    // unknown request → null
    const unknown = await getValidationStatus(publicClient, {
      ...valArgs,
      requestHash: keccak256(toBytes("nope")),
    });
    expect(unknown).toBeNull();
  });

  it("transfers an agent and clears the wallet", async () => {
    const hash = await registerAgent(ownerW, { ...idArgs, agentUri: "" });
    const receipt = await waitForTransactionReceipt(publicClient, { hash });
    const agentId = parseRegisteredAgentId(receipt.logs);
    const block = await getBlock(publicClient);
    const now = block.timestamp;
    const signature = await signAgentWallet(newWalletW, {
      chainId: deployment.chainId,
      identityRegistry: deployment.identityRegistry,
      agentId,
      owner: owner.address,
      deadline: now + 60n,
    });
    await waitForTransactionReceipt(publicClient, {
      hash: await setAgentWallet(ownerW, {
        ...idArgs,
        agentId,
        newWallet: newWallet.address,
        deadline: now + 60n,
        signature,
      }),
    });

    await waitForTransactionReceipt(publicClient, {
      hash: await transferAgent(ownerW, { ...idArgs, to: client.address, agentId }),
    });
    const agent = await getAgent(publicClient, { ...idArgs, agentId });
    expect(agent.owner).toBe(client.address);
    expect(agent.agentWallet).toBeNull();

    // nonexistent agent → AgentNotFound revert
    await expect(
      getAgentOwner(publicClient, { ...idArgs, agentId: 999999n }),
    ).rejects.toMatchObject({ code: "ContractRevert" });
  });
});
