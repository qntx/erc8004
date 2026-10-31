import { parseAgentRegistry } from "../core/agent-registry.ts";
import { REGISTRATION_TYPE_V1 } from "./file.ts";
import type { RegistrationFile } from "./file.ts";

/** A spec-conformance issue; separate from parse failures — parsing is lenient. */
export type ConformanceIssue =
  | Readonly<{ code: "MissingType" }>
  | Readonly<{ code: "UnknownType" }>
  | Readonly<{ code: "MissingName" }>
  | Readonly<{ code: "MissingDescription" }>
  | Readonly<{ code: "MissingRegistrations" }>
  | Readonly<{ code: "InvalidAgentRegistry"; index: number }>;

/** Returns every conformance issue; never throws. */
export function checkRegistrationFile(file: RegistrationFile): ConformanceIssue[] {
  const issues: ConformanceIssue[] = [];
  if (file.type === undefined) {
    issues.push({ code: "MissingType" });
  } else if (file.type !== REGISTRATION_TYPE_V1) {
    issues.push({ code: "UnknownType" });
  }
  if (file.name === undefined) {
    issues.push({ code: "MissingName" });
  }
  if (file.description === undefined) {
    issues.push({ code: "MissingDescription" });
  }
  if (file.registrations === undefined || file.registrations.length === 0) {
    issues.push({ code: "MissingRegistrations" });
  } else {
    for (const [index, registration] of file.registrations.entries()) {
      try {
        parseAgentRegistry(registration.agentRegistry);
      } catch {
        issues.push({ code: "InvalidAgentRegistry", index });
      }
    }
  }
  return issues;
}
