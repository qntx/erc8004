# Security Policy

## Supported versions

| Version | Supported |
| ------- | --------- |
| 0.5.x   | Yes       |
| < 0.5   | No        |

## Reporting a vulnerability

Report security issues privately via GitHub Security Advisories on
[qntx/erc8004](https://github.com/qntx/erc8004/security/advisories/new), or
email `security@qntx.org` if that channel is unavailable.

Please include:

- Affected version and runtime (Node/Bun/browser, or Rust toolchain)
- Minimal reproduction (transaction data, registration file, or agentURI)
- Impact assessment

Do not open public issues for unfixed vulnerabilities.

## Threat model (summary)

This SDK implements the ERC-8004 protocol layer: registry contract bindings,
registration/feedback file handling, agentURI resolution, and EIP-712
message construction. It does not custody keys and does not pin, index, or
serve content — signing is delegated to the caller's viem wallet or alloy
signer.

| Threat                                 | Severity | Mitigation                                                                                          |
| -------------------------------------- | -------- | --------------------------------------------------------------------------------------------------- |
| Malformed contract/event input         | Medium   | Strict ABI decode; unknown event topics are ignored, malformed data under a known topic is an error |
| Malicious agentURI / registration file | Medium   | Scheme allowlist, 1 MiB body cap, lenient-but-typed parsing                                         |
| Wrong-registry interaction             | Medium   | Chain-bound deployment table; `UnsupportedChainError` elsewhere                                     |
