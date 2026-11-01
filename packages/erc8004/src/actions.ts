import type { Client } from "viem";

import type { Deployment } from "./deployments/index.ts";
import {
  approveAgent,
  getAgent,
  getAgentOwner,
  getAgentUri,
  getAgentWallet,
  getApproved,
  getIdentityVersion,
  getMetadata,
  isApprovedForAll,
  isAuthorizedOrOwner,
  registerAgent,
  setAgentOperator,
  setAgentUri,
  setAgentWallet,
  setMetadata,
  signAgentWallet,
  transferAgent,
  unsetAgentWallet,
} from "./identity/actions.ts";
import {
  appendResponse,
  getClients,
  getLastIndex,
  getReputationSummary,
  getReputationVersion,
  getResponseCount,
  giveFeedback,
  readAllFeedback,
  readFeedback,
  revokeFeedback,
} from "./reputation/actions.ts";
import {
  getAgentValidations,
  getValidationStatus,
  getValidationSummary,
  getValidationVersion,
  getValidatorRequests,
  requestValidation,
  respondToValidation,
} from "./validation/actions.ts";

/** Bound registry addresses for every action in this client. */
export type Erc8004ActionsOptions = { deployment?: Deployment | undefined };

type Bound<F> = F extends (client: Client, args: infer A) => infer R ? (args: A) => R : never;

/** The methods {@link erc8004PublicActions} binds to a client. */
export type Erc8004PublicActionSet = {
  erc8004GetAgent: Bound<typeof getAgent>;
  erc8004GetAgentUri: Bound<typeof getAgentUri>;
  erc8004GetAgentOwner: Bound<typeof getAgentOwner>;
  erc8004GetAgentWallet: Bound<typeof getAgentWallet>;
  erc8004GetMetadata: Bound<typeof getMetadata>;
  erc8004IsAuthorizedOrOwner: Bound<typeof isAuthorizedOrOwner>;
  erc8004GetApproved: Bound<typeof getApproved>;
  erc8004IsApprovedForAll: Bound<typeof isApprovedForAll>;
  erc8004GetIdentityVersion: Bound<typeof getIdentityVersion>;
  erc8004ReadFeedback: Bound<typeof readFeedback>;
  erc8004ReadAllFeedback: Bound<typeof readAllFeedback>;
  erc8004GetReputationSummary: Bound<typeof getReputationSummary>;
  erc8004GetClients: Bound<typeof getClients>;
  erc8004GetLastIndex: Bound<typeof getLastIndex>;
  erc8004GetResponseCount: Bound<typeof getResponseCount>;
  erc8004GetReputationVersion: Bound<typeof getReputationVersion>;
  erc8004GetValidationStatus: Bound<typeof getValidationStatus>;
  erc8004GetValidationSummary: Bound<typeof getValidationSummary>;
  erc8004GetAgentValidations: Bound<typeof getAgentValidations>;
  erc8004GetValidatorRequests: Bound<typeof getValidatorRequests>;
  erc8004GetValidationVersion: Bound<typeof getValidationVersion>;
};

/** The methods {@link erc8004WalletActions} binds to a client. */
export type Erc8004WalletActionSet = Erc8004PublicActionSet & {
  erc8004RegisterAgent: Bound<typeof registerAgent>;
  erc8004SetAgentUri: Bound<typeof setAgentUri>;
  erc8004SetMetadata: Bound<typeof setMetadata>;
  erc8004SetAgentWallet: Bound<typeof setAgentWallet>;
  erc8004UnsetAgentWallet: Bound<typeof unsetAgentWallet>;
  erc8004TransferAgent: Bound<typeof transferAgent>;
  erc8004ApproveAgent: Bound<typeof approveAgent>;
  erc8004SetAgentOperator: Bound<typeof setAgentOperator>;
  erc8004SignAgentWallet: Bound<typeof signAgentWallet>;
  erc8004GiveFeedback: Bound<typeof giveFeedback>;
  erc8004RevokeFeedback: Bound<typeof revokeFeedback>;
  erc8004AppendResponse: Bound<typeof appendResponse>;
  erc8004RequestValidation: Bound<typeof requestValidation>;
  erc8004RespondToValidation: Bound<typeof respondToValidation>;
};

/**
 * Viem `extend` decorator binding every ERC-8004 read action:
 * `publicClient.extend(erc8004PublicActions())`. `options.deployment` pins the registry addresses;
 * otherwise each call resolves by `client.chain.id`.
 */
export function erc8004PublicActions(options: Erc8004ActionsOptions = {}) {
  return (client: Client): Erc8004PublicActionSet => ({
    erc8004GetAgent: async (args: Parameters<typeof getAgent>[1]) =>
      getAgent(client, { ...args, deployment: options.deployment }),
    erc8004GetAgentUri: async (args: Parameters<typeof getAgentUri>[1]) =>
      getAgentUri(client, { ...args, deployment: options.deployment }),
    erc8004GetAgentOwner: async (args: Parameters<typeof getAgentOwner>[1]) =>
      getAgentOwner(client, { ...args, deployment: options.deployment }),
    erc8004GetAgentWallet: async (args: Parameters<typeof getAgentWallet>[1]) =>
      getAgentWallet(client, { ...args, deployment: options.deployment }),
    erc8004GetMetadata: async (args: Parameters<typeof getMetadata>[1]) =>
      getMetadata(client, { ...args, deployment: options.deployment }),
    erc8004IsAuthorizedOrOwner: async (args: Parameters<typeof isAuthorizedOrOwner>[1]) =>
      isAuthorizedOrOwner(client, { ...args, deployment: options.deployment }),
    erc8004GetApproved: async (args: Parameters<typeof getApproved>[1]) =>
      getApproved(client, { ...args, deployment: options.deployment }),
    erc8004IsApprovedForAll: async (args: Parameters<typeof isApprovedForAll>[1]) =>
      isApprovedForAll(client, { ...args, deployment: options.deployment }),
    erc8004GetIdentityVersion: async (args: Parameters<typeof getIdentityVersion>[1] = {}) =>
      getIdentityVersion(client, { ...args, deployment: options.deployment }),
    erc8004ReadFeedback: async (args: Parameters<typeof readFeedback>[1]) =>
      readFeedback(client, { ...args, deployment: options.deployment }),
    erc8004ReadAllFeedback: async (args: Parameters<typeof readAllFeedback>[1]) =>
      readAllFeedback(client, { ...args, deployment: options.deployment }),
    erc8004GetReputationSummary: async (args: Parameters<typeof getReputationSummary>[1]) =>
      getReputationSummary(client, { ...args, deployment: options.deployment }),
    erc8004GetClients: async (args: Parameters<typeof getClients>[1]) =>
      getClients(client, { ...args, deployment: options.deployment }),
    erc8004GetLastIndex: async (args: Parameters<typeof getLastIndex>[1]) =>
      getLastIndex(client, { ...args, deployment: options.deployment }),
    erc8004GetResponseCount: async (args: Parameters<typeof getResponseCount>[1]) =>
      getResponseCount(client, { ...args, deployment: options.deployment }),
    erc8004GetReputationVersion: async (args: Parameters<typeof getReputationVersion>[1] = {}) =>
      getReputationVersion(client, { ...args, deployment: options.deployment }),
    erc8004GetValidationStatus: async (args: Parameters<typeof getValidationStatus>[1]) =>
      getValidationStatus(client, { ...args, deployment: options.deployment }),
    erc8004GetValidationSummary: async (args: Parameters<typeof getValidationSummary>[1]) =>
      getValidationSummary(client, { ...args, deployment: options.deployment }),
    erc8004GetAgentValidations: async (args: Parameters<typeof getAgentValidations>[1]) =>
      getAgentValidations(client, { ...args, deployment: options.deployment }),
    erc8004GetValidatorRequests: async (args: Parameters<typeof getValidatorRequests>[1]) =>
      getValidatorRequests(client, { ...args, deployment: options.deployment }),
    erc8004GetValidationVersion: async (args: Parameters<typeof getValidationVersion>[1] = {}) =>
      getValidationVersion(client, { ...args, deployment: options.deployment }),
  });
}

/**
 * Viem `extend` decorator binding every ERC-8004 write action:
 * `walletClient.extend(erc8004WalletActions())`.
 */
export function erc8004WalletActions(options: Erc8004ActionsOptions = {}) {
  return (client: Client): Erc8004WalletActionSet => ({
    ...erc8004PublicActions(options)(client),
    erc8004RegisterAgent: async (args: Parameters<typeof registerAgent>[1]) =>
      registerAgent(client, { ...args, deployment: options.deployment }),
    erc8004SetAgentUri: async (args: Parameters<typeof setAgentUri>[1]) =>
      setAgentUri(client, { ...args, deployment: options.deployment }),
    erc8004SetMetadata: async (args: Parameters<typeof setMetadata>[1]) =>
      setMetadata(client, { ...args, deployment: options.deployment }),
    erc8004SetAgentWallet: async (args: Parameters<typeof setAgentWallet>[1]) =>
      setAgentWallet(client, { ...args, deployment: options.deployment }),
    erc8004UnsetAgentWallet: async (args: Parameters<typeof unsetAgentWallet>[1]) =>
      unsetAgentWallet(client, { ...args, deployment: options.deployment }),
    erc8004TransferAgent: async (args: Parameters<typeof transferAgent>[1]) =>
      transferAgent(client, { ...args, deployment: options.deployment }),
    erc8004ApproveAgent: async (args: Parameters<typeof approveAgent>[1]) =>
      approveAgent(client, { ...args, deployment: options.deployment }),
    erc8004SetAgentOperator: async (args: Parameters<typeof setAgentOperator>[1]) =>
      setAgentOperator(client, { ...args, deployment: options.deployment }),
    erc8004SignAgentWallet: async (args: Parameters<typeof signAgentWallet>[1]) =>
      signAgentWallet(client, args),
    erc8004GiveFeedback: async (args: Parameters<typeof giveFeedback>[1]) =>
      giveFeedback(client, { ...args, deployment: options.deployment }),
    erc8004RevokeFeedback: async (args: Parameters<typeof revokeFeedback>[1]) =>
      revokeFeedback(client, { ...args, deployment: options.deployment }),
    erc8004AppendResponse: async (args: Parameters<typeof appendResponse>[1]) =>
      appendResponse(client, { ...args, deployment: options.deployment }),
    erc8004RequestValidation: async (args: Parameters<typeof requestValidation>[1]) =>
      requestValidation(client, { ...args, deployment: options.deployment }),
    erc8004RespondToValidation: async (args: Parameters<typeof respondToValidation>[1]) =>
      respondToValidation(client, { ...args, deployment: options.deployment }),
  });
}
