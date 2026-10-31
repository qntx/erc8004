import type { RevertReason } from "./revert.ts";

/**
 * Base error for all @qntx/erc8004 failures; `code` is the stable discriminator shared with the
 * Rust error variants.
 */
export class Erc8004Error extends Error {
  override name = "Erc8004Error";
  readonly code: string;

  constructor(code: string, message: string, options?: { cause?: unknown }) {
    super(message, options);
    this.code = code;
  }
}

/** The chain has no known ERC-8004 deployment in the deployment table. */
export class UnsupportedChainError extends Erc8004Error {
  override name = "UnsupportedChainError";
  override readonly code = "UnsupportedChain";
  readonly chainId: number;

  constructor(chainId: number) {
    super("UnsupportedChain", `no ERC-8004 deployment for chain ${chainId}`);
    this.chainId = chainId;
  }
}

export type AgentRegistryErrorCode =
  | "Malformed"
  | "UnsupportedNamespace"
  | "InvalidChainId"
  | "InvalidAddress"
  | "BadChecksum";

/** `eip155:{chainId}:{address}` parse failure. */
export class AgentRegistryError extends Erc8004Error {
  override name = "AgentRegistryError";
  override readonly code: AgentRegistryErrorCode;

  constructor(code: AgentRegistryErrorCode, message: string) {
    super(code, message);
    this.code = code;
  }
}

export type FeedbackValueErrorCode =
  | "InvalidDecimalString"
  | "TooManyDecimals"
  | "ValueOutOfRange"
  | "InvalidDecimals";

/** Decimal-string or `(value, decimals)` pair validation failure. */
export class FeedbackValueError extends Erc8004Error {
  override name = "FeedbackValueError";
  override readonly code: FeedbackValueErrorCode;

  constructor(code: FeedbackValueErrorCode, message: string) {
    super(code, message);
    this.code = code;
  }
}

export type JsonDocumentErrorCode = "NotAnObject" | "InvalidField";

/**
 * Lenient JSON-document (registration file, feedback file) field failure; `field` is the failing
 * path like `"services[0].endpoint"`.
 */
export class JsonDocumentError extends Erc8004Error {
  override name = "JsonDocumentError";
  override readonly code: JsonDocumentErrorCode;
  readonly field?: string | undefined;

  constructor(code: JsonDocumentErrorCode, message: string, field?: string) {
    super(code, message);
    this.code = code;
    this.field = field;
  }
}

export type DataUriErrorCode =
  | "NotDataUri"
  | "UnsupportedMediaType"
  | "NotBase64"
  | "InvalidBase64";

/** `data:` URI decode failure. */
export class DataUriError extends Erc8004Error {
  override name = "DataUriError";
  override readonly code: DataUriErrorCode;

  constructor(code: DataUriErrorCode, message: string) {
    super(code, message);
    this.code = code;
  }
}

export type AgentUriErrorCode = "InvalidGateway";

/** AgentURI classification failure (bad data URIs surface as {@link DataUriError}). */
export class AgentUriError extends Erc8004Error {
  override name = "AgentUriError";
  override readonly code: AgentUriErrorCode;

  constructor(code: AgentUriErrorCode, message: string) {
    super(code, message);
    this.code = code;
  }
}

/** Contract log payload under a known protocol event topic is malformed. */
export class EventDecodeError extends Erc8004Error {
  override name = "EventDecodeError";
  override readonly code = "MalformedLog";

  constructor(message: string) {
    super("MalformedLog", message);
  }
}

export type AgentWalletDeadlineErrorCode = "DeadlineExpired" | "DeadlineTooFar";

/** `AgentWalletSet` deadline outside the contract's ±5 minute window. */
export class AgentWalletDeadlineError extends Erc8004Error {
  override name = "AgentWalletDeadlineError";
  override readonly code: AgentWalletDeadlineErrorCode;

  constructor(code: AgentWalletDeadlineErrorCode, message: string) {
    super(code, message);
    this.code = code;
  }
}

/** A contract call reverted; `reason` is the decoded {@link RevertReason}. */
export class ContractRevertError extends Erc8004Error {
  override name = "ContractRevertError";
  override readonly code = "ContractRevert";
  readonly reason: RevertReason;

  constructor(reason: RevertReason, options?: { cause?: unknown }) {
    super("ContractRevert", `contract reverted: ${reason.reason}`, { cause: options?.cause });
    this.reason = reason;
  }
}

export type ClientGuardErrorCode =
  | "ClientAddressesRequired"
  | "MissingAccount"
  | "ResponseOutOfRange"
  | "ReservedMetadataKey"
  | "EmptyResponseUri";

/** A pre-flight guard tripped before any RPC call. */
export class ClientGuardError extends Erc8004Error {
  override name = "ClientGuardError";
  override readonly code: ClientGuardErrorCode;

  constructor(code: ClientGuardErrorCode, message: string) {
    super(code, message);
    this.code = code;
  }
}

/** An expected event was absent from a transaction receipt's logs. */
export class MissingEventError extends Erc8004Error {
  override name = "MissingEventError";
  override readonly code = "MissingEvent";

  constructor(event: string) {
    super("MissingEvent", `expected a ${event} event in the receipt logs`);
  }
}

export type ResolveErrorCode =
  | "NotSet"
  | "UnsupportedScheme"
  | "HttpStatus"
  | "TooLarge"
  | "InvalidJson";

/** AgentURI resolution failure. */
export class ResolveError extends Erc8004Error {
  override name = "ResolveError";
  override readonly code: ResolveErrorCode;
  readonly status: number | undefined;

  constructor(code: ResolveErrorCode, message: string, options?: { status?: number }) {
    super(code, message);
    this.code = code;
    this.status = options?.status;
  }
}
