# erc8004

ERC-8004 Trustless Agents Rust SDK — the pure (sans-IO) protocol core:
the Identity, Reputation and Validation registry ABIs (vendored at contract
version `2.0.0`), the `eip155:{chainId}:{address}` agent registry, feedback
values, the off-chain registration and feedback files, `data:` URIs, agentURI
classification, endpoint-domain verification, the `AgentWalletSet` EIP-712
message, the revert-reason decoder, the protocol event decoder, and the
verified deployment table — all with no RPC, provider or fetching code.

This is the Rust half of the dual-language SDK described in the repository
`AGENTS.md`, sharing error codes and test vectors with `@qntx/erc8004`.
RPC/provider functionality arrives in a later phase.

## License

Licensed under either of [Apache-2.0](../LICENSE-APACHE) or
[MIT](../LICENSE-MIT) at your option.
