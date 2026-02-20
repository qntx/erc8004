"""Count registered agents from local identity.parquet data.

Each agent registration mints an ERC-721 token, emitting a Transfer event
from the zero address. We count these mint events to determine the total
number of registered agents per chain.
"""

import os
import sys

try:
    import pyarrow.parquet as pq
    import pyarrow.compute as pc
except ImportError:
    print("Error: pyarrow is required. Install with: pip install pyarrow")
    sys.exit(1)

DATA_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "data")

# ERC-721 Transfer(address indexed from, address indexed to, uint256 indexed tokenId)
TRANSFER_TOPIC0 = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
# Zero address padded to 32 bytes (mint source)
ZERO_ADDR_PADDED = "0x0000000000000000000000000000000000000000000000000000000000000000"

# Chain name mapping
CHAIN_NAMES = {
    1: "Ethereum Mainnet",
    8453: "Base Mainnet",
    137: "Polygon Mainnet",
    42161: "Arbitrum Mainnet",
}


def count_agents_in_chain(chain_id: int) -> dict:
    """Count registered agents for a given chain from identity.parquet."""
    path = os.path.join(DATA_DIR, str(chain_id), "identity.parquet")
    if not os.path.exists(path):
        return {"error": f"File not found: {path}"}

    table = pq.read_table(path)
    total_events = table.num_rows

    # Filter: Transfer events (topic0) where from=0x0 (topic1) => mints
    is_transfer = pc.equal(table.column("topic0"), TRANSFER_TOPIC0)
    is_mint = pc.equal(table.column("topic1"), ZERO_ADDR_PADDED)
    mint_mask = pc.and_(is_transfer, is_mint)
    mint_table = table.filter(mint_mask)

    mint_count = mint_table.num_rows

    # Each mint's topic3 = tokenId = agentId, count unique ones
    if mint_count > 0:
        unique_ids = pc.unique(mint_table.column("topic3"))
        unique_agent_count = len(unique_ids)
    else:
        unique_agent_count = 0

    # Also count distinct event types for reference
    topic0_counts = {}
    for val, cnt in zip(
        *pc.value_counts(table.column("topic0")).flatten()
    ):
        topic0_counts[val.as_py()] = cnt.as_py()

    return {
        "total_events": total_events,
        "mint_count": mint_count,
        "unique_agents": unique_agent_count,
        "topic0_counts": topic0_counts,
    }


def main():
    # Discover available chain directories
    chain_dirs = []
    for name in os.listdir(DATA_DIR):
        chain_path = os.path.join(DATA_DIR, name, "identity.parquet")
        if os.path.isdir(os.path.join(DATA_DIR, name)) and os.path.exists(chain_path):
            chain_dirs.append(int(name))
    chain_dirs.sort()

    if not chain_dirs:
        print("No identity.parquet files found in data/")
        return

    total_agents_all = 0

    print("=" * 65)
    print(f"  ERC-8004 Agent Registration Statistics")
    print("=" * 65)

    for chain_id in chain_dirs:
        chain_name = CHAIN_NAMES.get(chain_id, f"Chain {chain_id}")
        result = count_agents_in_chain(chain_id)

        if "error" in result:
            print(f"\n[{chain_name} ({chain_id})] {result['error']}")
            continue

        print(f"\n  [{chain_name} (ID: {chain_id})]")
        print(f"    Total events in identity.parquet : {result['total_events']:>10,}")
        print(f"    Mint (Transfer from 0x0) events  : {result['mint_count']:>10,}")
        print(f"    Unique agent IDs (tokenIds)      : {result['unique_agents']:>10,}")

        total_agents_all += result["unique_agents"]

    print()
    print("=" * 65)
    print(f"  Total unique agents across all chains: {total_agents_all:,}")
    print("=" * 65)
    print()
    print("Note: Each chain has its own IdentityRegistry, so the same")
    print("entity may register on multiple chains with different agentIds.")


if __name__ == "__main__":
    main()
