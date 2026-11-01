import type { AgentRegistry } from "../core/agent-registry.ts";
import { parseAgentRegistry } from "../core/agent-registry.ts";
import { JsonDocumentError } from "../core/errors.ts";
import {
  extras,
  indexPath,
  isRecord,
  notAnObject,
  optionalBool,
  optionalString,
  optionalStringArray,
  requiredString,
  requiredUint,
  uintToJson,
} from "../core/json.ts";
import type { JsonObject } from "../core/json.ts";

export const REGISTRATION_TYPE_V1 = "https://eips.ethereum.org/EIPS/eip-8004#registration-v1";

/** One `services[]` entry; `extra` preserves non-spec keys. */
export type Service = {
  name: string;
  endpoint: string;
  version?: string | undefined;
  skills?: string[] | undefined;
  domains?: string[] | undefined;
  extra: JsonObject;
};

/**
 * One `registrations[]` entry; `agentRegistry` stays a raw string (see
 * {@link registrationAgentRegistry}).
 */
export type Registration = {
  agentId: bigint;
  agentRegistry: string;
  extra: JsonObject;
};

/**
 * Lenient, lossless registration file: spec fields typed when present, unknown keys preserved in
 * `extra`, no defaults invented.
 */
export type RegistrationFile = {
  type?: string | undefined;
  name?: string | undefined;
  description?: string | undefined;
  image?: string | undefined;
  services?: Service[] | undefined;
  x402Support?: boolean | undefined;
  active?: boolean | undefined;
  registrations?: Registration[] | undefined;
  supportedTrust?: string[] | undefined;
  extra: JsonObject;
};

const SERVICE_KEYS = new Set(["name", "endpoint", "version", "skills", "domains"]);
const REGISTRATION_KEYS = new Set(["agentId", "agentRegistry"]);
const FILE_KEYS = new Set([
  "type",
  "name",
  "description",
  "image",
  "services",
  "x402Support",
  "active",
  "registrations",
  "supportedTrust",
]);

function parseService(value: unknown, field: string): Service {
  if (!isRecord(value)) {
    throw new JsonDocumentError("InvalidField", `${field} must be an object`, field);
  }
  return {
    name: requiredString(value, field, "name"),
    endpoint: requiredString(value, field, "endpoint"),
    version: optionalString(value, field, "version"),
    skills: optionalStringArray(value, field, "skills"),
    domains: optionalStringArray(value, field, "domains"),
    extra: extras(value, SERVICE_KEYS),
  };
}

function parseRegistration(value: unknown, field: string): Registration {
  if (!isRecord(value)) {
    throw new JsonDocumentError("InvalidField", `${field} must be an object`, field);
  }
  return {
    agentId: requiredUint(value, field, "agentId"),
    agentRegistry: requiredString(value, field, "agentRegistry"),
    extra: extras(value, REGISTRATION_KEYS),
  };
}

/**
 * Parses a registration file leniently (any JSON value) — `NotAnObject`/`InvalidField` errors carry
 * the failing path.
 */
export function parseRegistrationFile(json: unknown): RegistrationFile {
  if (!isRecord(json)) {
    throw notAnObject(json);
  }
  const servicesValue = json["services"];
  let services: Service[] | undefined;
  if (servicesValue !== undefined) {
    if (!Array.isArray(servicesValue)) {
      throw new JsonDocumentError("InvalidField", "services must be an array", "services");
    }
    services = servicesValue.map((service, i) => parseService(service, indexPath("services", i)));
  }
  const registrationsValue = json["registrations"];
  let registrations: Registration[] | undefined;
  if (registrationsValue !== undefined) {
    if (!Array.isArray(registrationsValue)) {
      throw new JsonDocumentError(
        "InvalidField",
        "registrations must be an array",
        "registrations",
      );
    }
    registrations = registrationsValue.map((registration, i) =>
      parseRegistration(registration, indexPath("registrations", i)),
    );
  }
  return {
    type: optionalString(json, "", "type"),
    name: optionalString(json, "", "name"),
    description: optionalString(json, "", "description"),
    image: optionalString(json, "", "image"),
    services,
    x402Support: optionalBool(json, "", "x402Support"),
    active: optionalBool(json, "", "active"),
    registrations,
    supportedTrust: optionalStringArray(json, "", "supportedTrust"),
    extra: extras(json, FILE_KEYS),
  };
}

function serviceToJson(service: Service): JsonObject {
  return {
    name: service.name,
    endpoint: service.endpoint,
    ...(service.version === undefined ? {} : { version: service.version }),
    ...(service.skills === undefined ? {} : { skills: service.skills }),
    ...(service.domains === undefined ? {} : { domains: service.domains }),
    ...service.extra,
  };
}

function registrationToJson(registration: Registration): JsonObject {
  return {
    agentId: uintToJson(registration.agentId),
    agentRegistry: registration.agentRegistry,
    ...registration.extra,
  };
}

/**
 * Serializes a registration file back to a JSON value; `agentId` is a number ≤ 2^53-1, a decimal
 * string above.
 */
export function registrationFileToJson(file: RegistrationFile): JsonObject {
  return {
    ...(file.type === undefined ? {} : { type: file.type }),
    ...(file.name === undefined ? {} : { name: file.name }),
    ...(file.description === undefined ? {} : { description: file.description }),
    ...(file.image === undefined ? {} : { image: file.image }),
    ...(file.services === undefined ? {} : { services: file.services.map(serviceToJson) }),
    ...(file.x402Support === undefined ? {} : { x402Support: file.x402Support }),
    ...(file.active === undefined ? {} : { active: file.active }),
    ...(file.registrations === undefined
      ? {}
      : { registrations: file.registrations.map(registrationToJson) }),
    ...(file.supportedTrust === undefined ? {} : { supportedTrust: file.supportedTrust }),
    ...file.extra,
  };
}

/** `registrations[].agentRegistry` parsed — the raw string stays untouched in the model. */
export function registrationAgentRegistry(registration: Registration): AgentRegistry {
  return parseAgentRegistry(registration.agentRegistry);
}

/** Builder: sets `type` to {@link REGISTRATION_TYPE_V1}. */
export function createRegistrationFile(input: {
  name: string;
  description: string;
  image?: string | undefined;
  services?: Service[] | undefined;
  x402Support?: boolean | undefined;
  active?: boolean | undefined;
  registrations?: Registration[] | undefined;
  supportedTrust?: string[] | undefined;
}): RegistrationFile {
  return {
    type: REGISTRATION_TYPE_V1,
    name: input.name,
    description: input.description,
    ...(input.image === undefined ? {} : { image: input.image }),
    ...(input.services === undefined ? {} : { services: input.services }),
    ...(input.x402Support === undefined ? {} : { x402Support: input.x402Support }),
    ...(input.active === undefined ? {} : { active: input.active }),
    ...(input.registrations === undefined ? {} : { registrations: input.registrations }),
    ...(input.supportedTrust === undefined ? {} : { supportedTrust: input.supportedTrust }),
    extra: {},
  };
}
