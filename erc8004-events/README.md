# ERC-8004 Events

[![crates.io][crate-badge]][crate-url]
[![docs.rs][doc-badge]][doc-url]

[crate-badge]: https://img.shields.io/crates/v/erc8004-events.svg
[crate-url]: https://crates.io/crates/erc8004-events
[doc-badge]: https://img.shields.io/docsrs/erc8004-events.svg
[doc-url]: https://docs.rs/erc8004-events

Raw on-chain event archiver for the [ERC-8004](https://eips.ethereum.org/EIPS/eip-8004) Trustless Agents protocol.

Fetches all event logs from the **Identity Registry** and **Reputation Registry** contracts across every known ERC-8004 deployment and stores them as [Apache Parquet](https://parquet.apache.org/) files. The archived dataset is published to [HuggingFace](https://huggingface.co/datasets/qntx/erc8004-events) and updated automatically via GitHub Actions.

## Supported Networks

28 chains are currently tracked (16 mainnets + 12 testnets):

| Network | Chain ID | Type |
| --- | --- | --- |
| Base | 8453 | mainnet |
| Ethereum | 1 | mainnet |
| Polygon | 137 | mainnet |
| Arbitrum One | 42161 | mainnet |
| Celo | 42220 | mainnet |
| Gnosis | 100 | mainnet |
| Scroll | 534352 | mainnet |
| Taiko (Alethia) | 167000 | mainnet |
| BNB Smart Chain | 56 | mainnet |
| Monad | 143 | mainnet |
| Abstract | 2741 | mainnet |
| Avalanche C-Chain | 43114 | mainnet |
| Linea | 59144 | mainnet |
| Mantle | 5000 | mainnet |
| MegaETH | 4326 | mainnet |
| Optimism | 10 | mainnet |
| Base Sepolia | 84532 | testnet |
| Ethereum Sepolia | 11155111 | testnet |
| Polygon Amoy | 80002 | testnet |
| Arbitrum Sepolia | 421614 | testnet |
| Celo Alfajores | 44787 | testnet |
| Scroll Sepolia | 534351 | testnet |
| BNB Smart Chain Testnet | 97 | testnet |
| Monad Testnet | 10143 | testnet |
| Linea Sepolia | 59141 | testnet |
| Mantle Sepolia | 5003 | testnet |
| MegaETH Testnet | 6342 | testnet |
| Optimism Sepolia | 11155420 | testnet |

## Data Format

Each chain produces two Parquet files under `data/<chain_id>/`:

- **`identity.parquet`** — events from the Identity Registry (ERC-721 agent NFTs)
- **`reputation.parquet`** — events from the Reputation Registry (feedback signals)

Both files use the raw `eth_getLogs` schema:

| Column | Type | Description |
| --- | --- | --- |
| `block_number` | `UInt64` | Block in which the event was emitted |
| `tx_hash` | `Utf8` | Transaction hash (`0x`-prefixed hex) |
| `tx_index` | `UInt32` | Transaction position in the block |
| `log_index` | `UInt32` | Log position in the transaction |
| `address` | `Utf8` | Emitting contract address (`0x`-prefixed hex) |
| `topic0` | `Utf8` | Event signature hash |
| `topic1` | `Utf8?` | First indexed parameter (nullable) |
| `topic2` | `Utf8?` | Second indexed parameter (nullable) |
| `topic3` | `Utf8?` | Third indexed parameter (nullable) |
| `data` | `Utf8` | ABI-encoded non-indexed parameters (`0x`-prefixed hex) |
| `removed` | `Boolean` | Whether the log was removed due to a chain reorg |

This is the **universal EVM log format** — any EVM library in any language can decode these fields directly.

### Key Event Signatures

**Identity Registry:**

| Event | `topic0` |
| --- | --- |
| `Transfer(address,address,uint256)` | `0xddf252ad…f523b3ef` |
| Agent registered (custom) | `0xca52e62c…9bc4a` |
| Agent URI updated (custom) | `0x2c149ed5…1468b` |

**Reputation Registry:**

| Event | `topic0` |
| --- | --- |
| Feedback given (custom) | `0x6a4a6174…58febc` |
| Response appended (custom) | `0xb1c6be0b…6051d4` |

> **Tip:** Filter mint events with `topic0 = Transfer AND topic1 = 0x000…000` to count unique agent registrations.

## Usage

```bash
# Sync all mainnet chains
cargo run --release -- sync --data-dir ./data

# Sync a specific chain by chain ID
cargo run --release -- sync --data-dir ./data --chain 8453

# Use a custom RPC endpoint
cargo run --release -- sync --data-dir ./data --chain 8453 --rpc https://my-rpc.example.com

# Include testnets
cargo run --release -- sync --data-dir ./data --include-testnets

# List all supported chains
cargo run --release -- list
```

Sync is **incremental** — a `cursor.json` file tracks the last synced block per chain. Re-running sync only fetches new events.

## Consuming the Data

### Python

```python
import pandas as pd

# Load identity events for Base mainnet
df = pd.read_parquet("data/8453/identity.parquet")

# Count unique registered agents (ERC-721 mints: Transfer from 0x0)
TRANSFER = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
ZERO = "0x" + "0" * 64
mints = df[(df["topic0"] == TRANSFER) & (df["topic1"] == ZERO)]
print(f"Registered agents: {mints['topic3'].nunique()}")
```

### Rust

```rust
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

let file = std::fs::File::open("data/8453/identity.parquet")?;
let reader = ParquetRecordBatchReaderBuilder::try_new(file)?.build()?;
for batch in reader {
    println!("{:?}", batch?);
}
```

### DuckDB

```sql
-- Count registered agents on Base
SELECT count(DISTINCT topic3) AS agents
FROM read_parquet('data/8453/identity.parquet')
WHERE topic0 = '0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef'
  AND topic1 = '0x0000000000000000000000000000000000000000000000000000000000000000';
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.
