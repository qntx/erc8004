"""Find the first event block for each ERC-8004 chain deployment.

Strategy:
  1. If local parquet data exists, read min(block_number) directly.
  2. Otherwise, exponential-probe the RPC with growing windows
     (2K → 4K → … → 1M blocks), then binary-refine within the
     hit window.
  3. Per-chain wall-clock timeout of 90 seconds to avoid hanging.
"""

import os
import sys
import time
import concurrent.futures

import requests

try:
    import pyarrow.parquet as pq
    HAS_PARQUET = True
except ImportError:
    HAS_PARQUET = False

# Contract addresses (CREATE2 deterministic)
MAINNET_IDENTITY = "0x8004A169FB4a3325136EB29fA0ceB6D2e539a432"
TESTNET_IDENTITY = "0x8004A818BFB912233c491871b3d84c89A494BD9e"

# (name, chain_id, deployment_block, rpc_url, is_testnet)
CHAINS = [
    ("BaseMainnet",       8453,      41_663_783, "https://mainnet.base.org",                   False),
    ("EthereumMainnet",   1,         24_339_871, "https://ethereum-rpc.publicnode.com",         False),
    ("PolygonMainnet",    137,       73_019_847, "https://polygon-rpc.com",                    False),
    ("ArbitrumMainnet",   42161,    327_832_400, "https://arb1.arbitrum.io/rpc",               False),
    ("CeloMainnet",       42220,     32_479_428, "https://forno.celo.org",                     False),
    ("GnosisMainnet",     100,       39_025_823, "https://rpc.gnosischain.com",                False),
    ("ScrollMainnet",     534352,    15_577_120, "https://rpc.scroll.io",                      False),
    ("TaikoMainnet",      167000,       871_920, "https://rpc.mainnet.taiko.xyz",              False),
    ("BscMainnet",        56,        49_143_533, "https://bsc-rpc.publicnode.com",             False),
    ("MonadMainnet",      143,       56_017_606, "https://rpc.monad.xyz",                      False),
    ("AbstractMainnet",   2741,      41_233_800, "https://api.mainnet.abs.xyz",                False),
    ("AvalancheMainnet",  43114,     77_893_000, "https://api.avax.network/ext/bc/C/rpc",     False),
    ("LineaMainnet",      59144,     28_949_707, "https://rpc.linea.build",                    False),
    ("MantleMainnet",     5000,      91_520_634, "https://rpc.mantle.xyz",                     False),
    ("MegaEthMainnet",    4326,       7_833_805, "https://rpc.megaeth.com",                    False),
    ("OptimismMainnet",   10,       147_956_461, "https://mainnet.optimism.io",                False),
    ("BaseSepolia",       84532,     24_899_933, "https://sepolia.base.org",                   True),
    ("EthereumSepolia",   11155111,   8_067_632, "https://ethereum-sepolia-rpc.publicnode.com",True),
    ("PolygonAmoy",       80002,     20_965_364, "https://rpc-amoy.polygon.technology",        True),
    ("ArbitrumSepolia",   421614,   159_589_032, "https://sepolia-rollup.arbitrum.io/rpc",     True),
    ("CeloAlfajores",     44787,     31_382_416, "https://alfajores-forno.celo-testnet.org",   True),
    ("ScrollSepolia",     534351,    14_050_456, "https://sepolia-rpc.scroll.io",              True),
    ("BscTestnet",        97,        51_893_896, "https://bsc-testnet-rpc.publicnode.com",     True),
    ("MonadTestnet",      10143,     10_400_000, "https://testnet-rpc.monad.xyz",              True),
    ("LineaSepolia",      59141,     24_323_547, "https://rpc.sepolia.linea.build",            True),
    ("MantleSepolia",     5003,      34_586_937, "https://rpc.sepolia.mantle.xyz",             True),
    ("MegaEthTestnet",    6342,      11_668_749, "https://carrot.megaeth.com/rpc",             True),
    ("OptimismSepolia",   11155420,  39_855_448, "https://sepolia.optimism.io",                True),
]

DATA_DIR = "data"
INIT_WINDOW = 2_000
MAX_WINDOW = 2_000_000
PER_CHAIN_TIMEOUT = 90  # seconds


def rpc_call(url: str, method: str, params: list, timeout: int = 15) -> dict:
    payload = {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
    try:
        r = requests.post(url, json=payload, timeout=timeout)
        r.raise_for_status()
        data = r.json()
        if "error" in data:
            return {"error": data["error"]}
        return data
    except Exception as e:
        return {"error": str(e)}


def get_latest_block(rpc: str) -> int | None:
    resp = rpc_call(rpc, "eth_blockNumber", [])
    return int(resp["result"], 16) if "result" in resp else None


def get_logs(rpc: str, address: str, fr: int, to: int) -> list | None:
    resp = rpc_call(rpc, "eth_getLogs", [{"address": address, "fromBlock": hex(fr), "toBlock": hex(to)}])
    return resp.get("result") if "result" in resp else None


def from_local_parquet(chain_id: int) -> int | None:
    if not HAS_PARQUET:
        return None
    path = os.path.join(DATA_DIR, str(chain_id), "identity.parquet")
    if not os.path.exists(path):
        return None
    try:
        t = pq.read_table(path, columns=["block_number"])
        return int(t.column("block_number")[0].as_py()) if t.num_rows > 0 else None
    except Exception:
        return None


def find_first_event(rpc: str, address: str, deploy: int, latest: int) -> int | None:
    """Exponential probe + binary refinement, with wall-clock timeout."""
    deadline = time.monotonic() + PER_CHAIN_TIMEOUT
    cursor = deploy
    window = INIT_WINDOW

    while cursor <= latest and time.monotonic() < deadline:
        end = min(cursor + window - 1, latest)
        logs = get_logs(rpc, address, cursor, end)

        if logs is None:
            # RPC error — shrink and retry once
            window = max(500, window // 4)
            logs = get_logs(rpc, address, cursor, min(cursor + window - 1, latest))
            if logs is None:
                return None

        if logs:
            first = int(logs[0]["blockNumber"], 16)
            # Refine: binary search within [cursor, first]
            lo, hi = cursor, first
            while hi - lo > INIT_WINDOW and time.monotonic() < deadline:
                mid = (lo + hi) // 2
                r = get_logs(rpc, address, lo, mid)
                if r is None:
                    break
                if r:
                    hi = int(r[0]["blockNumber"], 16)
                else:
                    lo = mid + 1
            # Final precise fetch
            r = get_logs(rpc, address, lo, hi)
            return int(r[0]["blockNumber"], 16) if r else hi

        cursor = end + 1
        window = min(window * 2, MAX_WINDOW)
        time.sleep(0.02)

    return None  # Timeout


def scan_chain(entry):
    """Process a single chain entry. Returns (name, chain_id, deploy, first, source)."""
    name, chain_id, deploy, rpc, is_testnet = entry
    identity = TESTNET_IDENTITY if is_testnet else MAINNET_IDENTITY

    # Strategy 1: local parquet
    first = from_local_parquet(chain_id)
    if first is not None:
        return (name, chain_id, deploy, first, "parquet")

    # Strategy 2: RPC scan
    latest = get_latest_block(rpc)
    if latest is None:
        return (name, chain_id, deploy, None, "RPC down")

    first = find_first_event(rpc, identity, deploy, latest)
    source = "RPC scan" if first is not None else "timeout/no events"
    return (name, chain_id, deploy, first, source)


def main():
    print(f"{'Chain':<22} {'ID':>8}  {'Deploy':>12}  {'1st Event':>12}  {'Skip':>12}  Source")
    print("-" * 90)

    results = {}

    # Process chains sequentially (RPCs don't like parallel hammering)
    for entry in CHAINS:
        name, chain_id, deploy, first, source = scan_chain(entry)
        results[name] = first

        if first is not None:
            skip = first - deploy
            print(f"{name:<22} {chain_id:>8}  {deploy:>12,}  {first:>12,}  {skip:>12,}  {source}")
        else:
            print(f"{name:<22} {chain_id:>8}  {deploy:>12,}  {'N/A':>12}  {'N/A':>12}  {source}")
        sys.stdout.flush()

    # Output Rust code
    print("\n" + "=" * 60)
    print(" Rust first_event_block values for chains.rs")
    print("=" * 60)
    for name, chain_id, deploy, rpc, is_testnet in CHAINS:
        first = results.get(name)
        if first is not None and first > deploy + 100:
            print(f"  // {name} (chain {chain_id}): skip {first - deploy:,} empty blocks")
            print(f"  first_event_block: Some({first:_}),")
        else:
            note = "at deploy block" if first is not None else "unknown"
            print(f"  // {name} (chain {chain_id}): {note}")
            print(f"  first_event_block: None,")


if __name__ == "__main__":
    main()
