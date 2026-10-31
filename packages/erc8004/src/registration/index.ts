export {
  createRegistrationFile,
  parseRegistrationFile,
  registrationAgentRegistry,
  registrationFileToJson,
  REGISTRATION_TYPE_V1,
} from "./file.ts";
export type { Registration, RegistrationFile, Service } from "./file.ts";
export { checkRegistrationFile } from "./conformance.ts";
export type { ConformanceIssue } from "./conformance.ts";
export { decodeDataUri, encodeDataUri } from "./data-uri.ts";
export { classifyAgentUri, DEFAULT_IPFS_GATEWAY } from "./agent-uri.ts";
export type { AgentUri } from "./agent-uri.ts";
export { verifyEndpointDomain, wellKnownRegistrationUrl } from "./domain.ts";
