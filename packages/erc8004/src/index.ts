export {
  identityRegistryAbi,
  protocolEventsAbi,
  reputationRegistryAbi,
  validationRegistryAbi,
} from "./abi/index.ts";
export {
  AgentRegistryError,
  AgentUriError,
  AgentWalletDeadlineError,
  DataUriError,
  Erc8004Error,
  EventDecodeError,
  FeedbackValueError,
  ClientGuardError,
  ContractRevertError,
  JsonDocumentError,
  InvalidResponseError,
  MissingEventError,
  ResolveError,
  UnsupportedChainError,
  formatAgentRegistry,
  parseAgentRegistry,
  feedbackValue,
  formatFeedbackValue,
  parseFeedbackValue,
  decodeRevert,
} from "./core/index.ts";
export type {
  AgentRegistry,
  AgentRegistryErrorCode,
  AgentUriErrorCode,
  AgentWalletDeadlineErrorCode,
  DataUriErrorCode,
  FeedbackValueErrorCode,
  ClientGuardErrorCode,
  JsonObject,
  ResolveErrorCode,
  RevertReason,
} from "./core/index.ts";
export { deployments, getDeployment } from "./deployments/index.ts";
export type { Deployment, DeploymentId } from "./deployments/index.ts";
export {
  checkRegistrationFile,
  classifyAgentUri,
  DEFAULT_IPFS_GATEWAY,
  createRegistrationFile,
  decodeDataUri,
  encodeDataUri,
  parseRegistrationFile,
  registrationAgentRegistry,
  registrationFileToJson,
  verifyEndpointDomain,
  wellKnownRegistrationUrl,
  REGISTRATION_TYPE_V1,
} from "./registration/index.ts";
export type {
  AgentUri,
  ConformanceIssue,
  Registration,
  RegistrationFile,
  Service,
} from "./registration/index.ts";
export { decodeErc8004Log } from "./events/index.ts";
export type { Erc8004Event } from "./events/index.ts";
export {
  agentWalletTypedData,
  assertAgentWalletDeadline,
  hashAgentWallet,
  AGENT_WALLET_TYPES,
} from "./identity/index.ts";
export type { AgentWalletMessage } from "./identity/index.ts";
export { createFeedbackFile, feedbackFileToJson, parseFeedbackFile } from "./reputation/index.ts";
export type { FeedbackFile } from "./reputation/index.ts";
export type { Agent } from "./identity/index.ts";
export type { Feedback, FeedbackEntry, ReputationSummary } from "./reputation/index.ts";
export type { ValidationStatus, ValidationSummary } from "./validation/index.ts";
export { erc8004PublicActions, erc8004WalletActions } from "./actions.ts";
export type { Erc8004ActionsOptions } from "./actions.ts";
export {
  getAgent,
  getAgentOwner,
  getAgentUri,
  getAgentWallet,
  getApproved,
  getIdentityVersion,
  getMetadata,
  isApprovedForAll,
  isAuthorizedOrOwner,
  parseRegisteredAgentId,
  registerAgent,
  setAgentOperator,
  setAgentUri,
  setAgentWallet,
  setMetadata,
  signAgentWallet,
  transferAgent,
  unsetAgentWallet,
  approveAgent,
} from "./identity/index.ts";
export {
  appendResponse,
  getClients,
  getLastIndex,
  getReputationSummary,
  getReputationVersion,
  getResponseCount,
  giveFeedback,
  parseFeedbackIndex,
  readAllFeedback,
  readFeedback,
  revokeFeedback,
} from "./reputation/index.ts";
export {
  getAgentValidations,
  getValidationStatus,
  getValidationSummary,
  getValidationVersion,
  getValidatorRequests,
  requestValidation,
  respondToValidation,
} from "./validation/index.ts";
export { checkEndpointDomain, resolveAgentUri } from "./registration/index.ts";
export type { ResolveOptions } from "./registration/index.ts";
