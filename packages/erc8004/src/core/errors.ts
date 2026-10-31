/**
 * Base error for all @qntx/erc8004 failures; `code` is the stable discriminator shared with the
 * Rust error variants.
 */
export class Erc8004Error extends Error {
  override name = "Erc8004Error";
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
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
