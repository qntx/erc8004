# ERC-8004 Events

Raw on-chain event archiver for the [ERC-8004](https://eips.ethereum.org/EIPS/eip-8004) protocol.

Fetches all event logs from the **Identity Registry** and **Reputation Registry** contracts across every known ERC-8004 deployment and stores them as [Apache Parquet](https://parquet.apache.org/) files.

## Data Format

Each Parquet file contains raw EVM event logs in the standard `eth_getLogs` structure:

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

## Usage

```bash
# Sync all mainnet chains
cargo run -- sync --data-dir ./data

# Sync a specific chain
cargo run -- sync --data-dir ./data --chain 8453

# Use a custom RPC endpoint
cargo run -- sync --data-dir ./data --chain 8453 --rpc https://my-rpc.example.com

# Include testnets
cargo run -- sync --data-dir ./data --include-testnets

# List all supported chains
cargo run -- list
```

## Consuming the Data

### Python (pandas / polars)

```python
import pandas as pd

df = pd.read_parquet("data/8453/identity.parquet")
print(f"Total events: {len(df)}")
print(df.head())
```

### Rust (arrow / parquet)

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
SELECT * FROM read_parquet('data/8453/identity.parquet')
WHERE topic0 = '0xca52e62c367d81bb2e328eb795f7c7ba24afb478408a26c0e201d155c449bc4a'
LIMIT 10;
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.
