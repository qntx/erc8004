"""
Test free public RPC endpoints for all ERC-8004 mainnet chains.

For each chain × RPC combination, tests:
  1. Connectivity (eth_blockNumber)
  2. Historical eth_getLogs support (archive node check)
  3. Maximum block range per eth_getLogs call
  4. Latency

Outputs a ranked table per chain and a draft config.toml at the end.

Usage:
    pip install requests
    python scripts/test_rpcs.py
"""

import time
from dataclasses import dataclass, field

import requests

# ── Constants ────────────────────────────────────────────────────────────────

IDENTITY_MAINNET = "0x8004A169FB4a3325136EB29fA0ceB6D2e539a432"
TEST_RANGES = [100, 1_000, 5_000, 10_000, 50_000]
REQUEST_TIMEOUT = 15

# ── Chain definitions ────────────────────────────────────────────────────────
# Each chain: (chain_id, name, deployment_block, [(provider, url), ...])

CHAINS: list[tuple[int, str, int, list[tuple[str, str]]]] = [
    (1, "Ethereum", 24_339_871, [
        ("PublicNode",   "https://ethereum-rpc.publicnode.com"),
        ("dRPC",         "https://eth.drpc.org"),
        ("1RPC",         "https://1rpc.io/eth"),
        ("Tenderly",     "https://ethereum.gateway.tenderly.co"),
        ("Blast",        "https://eth-mainnet.public.blastapi.io"),
    ]),
    (8453, "Base", 41_663_783, [
        ("Base Official","https://mainnet.base.org"),
        ("PublicNode",   "https://base-rpc.publicnode.com"),
        ("dRPC",         "https://base.drpc.org"),
        ("1RPC",         "https://1rpc.io/base"),
        ("Tenderly",     "https://base.gateway.tenderly.co"),
        ("Blast",        "https://base-mainnet.public.blastapi.io"),
    ]),
    (137, "Polygon", 73_019_847, [
        ("Tenderly",     "https://polygon.gateway.tenderly.co"),
        ("dRPC",         "https://polygon.drpc.org"),
        ("1RPC",         "https://1rpc.io/matic"),
        ("PublicNode",   "https://polygon-bor-rpc.publicnode.com"),
        ("Blast",        "https://polygon-mainnet.public.blastapi.io"),
    ]),
    (42161, "Arbitrum", 327_832_400, [
        ("PublicNode",   "https://arbitrum-one-rpc.publicnode.com"),
        ("Arbitrum",     "https://arb1.arbitrum.io/rpc"),
        ("dRPC",         "https://arbitrum.drpc.org"),
        ("1RPC",         "https://1rpc.io/arb"),
        ("Tenderly",     "https://arbitrum.gateway.tenderly.co"),
        ("Blast",        "https://arbitrum-one.public.blastapi.io"),
    ]),
    (56, "BSC", 49_143_533, [
        ("dRPC",         "https://bsc.drpc.org"),
        ("Blast",        "https://bsc-mainnet.public.blastapi.io"),
        ("1RPC",         "https://1rpc.io/bnb"),
        ("PublicNode",   "https://bsc-rpc.publicnode.com"),
        ("Tenderly",     "https://bsc.gateway.tenderly.co"),
    ]),
    (10, "Optimism", 147_956_461, [
        ("Optimism",     "https://mainnet.optimism.io"),
        ("PublicNode",   "https://optimism-rpc.publicnode.com"),
        ("dRPC",         "https://optimism.drpc.org"),
        ("1RPC",         "https://1rpc.io/op"),
        ("Tenderly",     "https://optimism.gateway.tenderly.co"),
        ("Blast",        "https://optimism-mainnet.public.blastapi.io"),
    ]),
    (43114, "Avalanche", 77_893_000, [
        ("Avalanche",    "https://api.avax.network/ext/bc/C/rpc"),
        ("PublicNode",   "https://avalanche-c-chain-rpc.publicnode.com"),
        ("dRPC",         "https://avalanche.drpc.org"),
        ("1RPC",         "https://1rpc.io/avax/c"),
        ("Blast",        "https://ava-mainnet.public.blastapi.io/ext/bc/C/rpc"),
    ]),
    (42220, "Celo", 32_479_428, [
        ("Celo",         "https://forno.celo.org"),
        ("dRPC",         "https://celo.drpc.org"),
        ("1RPC",         "https://1rpc.io/celo"),
        ("PublicNode",   "https://celo-rpc.publicnode.com"),
    ]),
    (100, "Gnosis", 39_025_823, [
        ("Gnosis",       "https://rpc.gnosischain.com"),
        ("dRPC",         "https://gnosis.drpc.org"),
        ("1RPC",         "https://1rpc.io/gnosis"),
        ("PublicNode",   "https://gnosis-rpc.publicnode.com"),
        ("Blast",        "https://gnosis-mainnet.public.blastapi.io"),
    ]),
    (534352, "Scroll", 15_577_120, [
        ("Scroll",       "https://rpc.scroll.io"),
        ("dRPC",         "https://scroll.drpc.org"),
        ("1RPC",         "https://1rpc.io/scroll"),
        ("PublicNode",   "https://scroll-rpc.publicnode.com"),
    ]),
    (59144, "Linea", 28_949_707, [
        ("Linea",        "https://rpc.linea.build"),
        ("dRPC",         "https://linea.drpc.org"),
        ("1RPC",         "https://1rpc.io/linea"),
        ("PublicNode",   "https://linea-rpc.publicnode.com"),
    ]),
    (5000, "Mantle", 91_520_634, [
        ("Mantle",       "https://rpc.mantle.xyz"),
        ("dRPC",         "https://mantle.drpc.org"),
        ("1RPC",         "https://1rpc.io/mantle"),
        ("PublicNode",   "https://mantle-rpc.publicnode.com"),
    ]),
    (167000, "Taiko", 871_920, [
        ("Taiko",        "https://rpc.mainnet.taiko.xyz"),
        ("dRPC",         "https://taiko.drpc.org"),
        ("PublicNode",   "https://taiko-rpc.publicnode.com"),
    ]),
    (143, "Monad", 56_017_606, [
        ("Monad",        "https://rpc.monad.xyz"),
    ]),
    (2741, "Abstract", 41_233_800, [
        ("Abstract",     "https://api.mainnet.abs.xyz"),
    ]),
    (6342, "MegaETH", 7_833_805, [
        ("MegaETH",      "https://rpc.megaeth.com"),
    ]),
]


# ── RPC helpers ──────────────────────────────────────────────────────────────

def rpc_call(url: str, method: str, params: list, timeout: float = REQUEST_TIMEOUT) -> dict:
    """Send a JSON-RPC request and return the parsed response."""
    payload = {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
    resp = requests.post(
        url, json=payload, timeout=timeout,
        headers={"Content-Type": "application/json"},
    )
    resp.raise_for_status()
    return resp.json()


# ── Test result ──────────────────────────────────────────────────────────────

@dataclass
class Result:
    provider: str
    url: str
    reachable: bool = False
    latency_ms: float = 0.0
    archive: bool = False
    max_range: int = 0
    error: str = ""


# ── Individual tests ─────────────────────────────────────────────────────────

def check_connectivity(url: str) -> tuple[bool, float, str]:
    """Return (reachable, latency_ms, error)."""
    try:
        t0 = time.monotonic()
        data = rpc_call(url, "eth_blockNumber", [])
        ms = (time.monotonic() - t0) * 1000
        if "error" in data:
            return False, ms, data["error"].get("message", str(data["error"]))[:120]
        return True, ms, ""
    except Exception as e:
        return False, 0, str(e)[:120]


def check_archive(url: str, deploy_block: int) -> tuple[bool, str]:
    """Return (has_archive, detail)."""
    params = [{
        "address": IDENTITY_MAINNET,
        "fromBlock": hex(deploy_block),
        "toBlock": hex(deploy_block + 100),
    }]
    try:
        data = rpc_call(url, "eth_getLogs", params, timeout=20)
        if "error" in data:
            return False, data["error"].get("message", "")[:100]
        return True, f"{len(data.get('result', []))} logs"
    except Exception as e:
        return False, str(e)[:100]


def check_max_range(url: str, deploy_block: int) -> int:
    """Return the largest block range that succeeds."""
    best = 0
    for r in TEST_RANGES:
        params = [{
            "address": IDENTITY_MAINNET,
            "fromBlock": hex(deploy_block),
            "toBlock": hex(deploy_block + r),
        }]
        try:
            data = rpc_call(url, "eth_getLogs", params, timeout=20)
            if "error" not in data:
                best = r
            else:
                break
        except Exception:
            break
    return best


def test_rpc(provider: str, url: str, deploy_block: int) -> Result:
    """Run the full test suite on one RPC endpoint."""
    reachable, latency, err = check_connectivity(url)
    if not reachable:
        return Result(provider, url, error=err)

    archive, detail = check_archive(url, deploy_block)
    if not archive:
        return Result(provider, url, reachable=True, latency_ms=latency,
                      error=f"no archive: {detail}")

    max_range = check_max_range(url, deploy_block)
    return Result(provider, url, reachable=True, latency_ms=latency,
                  archive=True, max_range=max_range)


# ── Output formatting ────────────────────────────────────────────────────────

def print_table(chain_id: int, name: str, results: list[Result]) -> None:
    """Print a sorted results table for one chain."""
    results.sort(key=lambda r: (not r.archive, -r.max_range, r.latency_ms))
    hdr = f"{name} ({chain_id})"
    print(f"\n{'=' * 90}\n  {hdr}\n{'=' * 90}")
    print(f"{'Provider':<16} {'Latency':>8} {'Archive':>8} {'MaxRange':>10}  Status")
    print(f"{'-'*16} {'-'*8} {'-'*8} {'-'*10}  {'-'*40}")
    for r in results:
        lat = f"{r.latency_ms:.0f}ms" if r.latency_ms else "N/A"
        arc = "YES" if r.archive else "NO"
        rng = f"{r.max_range:,}" if r.max_range else "-"
        sts = "OK" if r.archive else (r.error[:40] or "unreachable")
        print(f"{r.provider:<16} {lat:>8} {arc:>8} {rng:>10}  {sts}")


def generate_config_toml(chain_results: dict[int, tuple[str, list[Result]]]) -> str:
    """Generate a config.toml string from test results."""
    lines = [
        "# ERC-8004 events sync configuration.",
        "# RPC endpoints per chain, ordered by priority (best first).",
        "# The sync engine tries each in order; on failure it falls back.",
        "",
    ]
    for chain_id, (name, results) in sorted(chain_results.items()):
        good = sorted(
            [r for r in results if r.archive],
            key=lambda r: (-r.max_range, r.latency_ms),
        )
        if not good:
            # Fall back to any reachable endpoint
            good = [r for r in results if r.reachable]
        if not good:
            continue
        urls = [r.url for r in good]
        lines.append(f"[chains.{chain_id}]  # {name}")
        lines.append("rpcs = [")
        for url in urls:
            lines.append(f'    "{url}",')
        lines.append("]")
        lines.append("")
    return "\n".join(lines)


# ── Main ─────────────────────────────────────────────────────────────────────

def main() -> None:
    print("ERC-8004 RPC Endpoint Tester — All Mainnet Chains")
    print(f"Testing {sum(len(c[3]) for c in CHAINS)} endpoints across {len(CHAINS)} chains\n")

    chain_results: dict[int, tuple[str, list[Result]]] = {}

    for chain_id, name, deploy_block, rpcs in CHAINS:
        results = []
        for provider, url in rpcs:
            print(f"  [{chain_id:>6}] {name:<12} {provider:<16}", end="", flush=True)
            r = test_rpc(provider, url, deploy_block)
            tag = "OK" if r.archive else ("NO ARCHIVE" if r.reachable else "FAIL")
            print(f" → {tag}")
            results.append(r)
            time.sleep(0.2)

        chain_results[chain_id] = (name, results)
        print_table(chain_id, name, results)

    # Generate and print config.toml
    toml = generate_config_toml(chain_results)
    print(f"\n{'=' * 90}")
    print("  GENERATED config.toml")
    print(f"{'=' * 90}\n")
    print(toml)

    # Also write to file
    with open("config.toml", "w", encoding="utf-8") as f:
        f.write(toml)
    print("  → Written to config.toml")


if __name__ == "__main__":
    main()
