"""Count ERC-8004 events stored in per-chain parquet files."""

import json
from datetime import datetime, timezone
from pathlib import Path

try:
    import pyarrow.compute as pc
    import pyarrow.parquet as pq
except ImportError:
    raise SystemExit("pyarrow is required: pip install pyarrow")

DATA_DIR = Path(__file__).resolve().parent.parent / "data"

# Chain ID -> human-readable name (from erc8004 SDK)
CHAIN_NAMES: dict[int, str] = {
    1: "Ethereum",
    10: "Optimism",
    56: "BNB Smart Chain",
    100: "Gnosis",
    137: "Polygon",
    143: "Monad",
    2741: "Abstract",
    4326: "MegaETH",
    5000: "Mantle",
    8453: "Base",
    42161: "Arbitrum One",
    42220: "Celo",
    43114: "Avalanche",
    59144: "Linea",
    167000: "Taiko",
    534352: "Scroll",
    # Testnets
    97: "BSC Testnet",
    6342: "MegaETH Testnet",
    10143: "Monad Testnet",
    11124: "Abstract Testnet",
    43113: "Avalanche Testnet",
    44787: "Celo Alfajores",
    59141: "Linea Sepolia",
    80002: "Polygon Amoy",
    84532: "Base Sepolia",
    534351: "Scroll Sepolia",
    421614: "Arbitrum Sepolia",
    5003: "Mantle Sepolia",
    11155111: "Ethereum Sepolia",
    11155420: "Optimism Sepolia",
}

# Keccak-256 hashes of ERC-8004 event signatures (topic0 selectors).
TOPIC_REGISTERED = "0xca52e62c367d81bb2e328eb795f7c7ba24afb478408a26c0e201d155c449bc4a"
TOPIC_NEW_FEEDBACK = "0x6a4a61743519c9d648a14e6493f47dbe3ff1aa29e7785c96c8326a205e58febc"
TOPIC_FEEDBACK_REVOKED = "0x25156fd3288212246d8b008d5921fde376c71ed14ac2e072a506eb06fde6d09d"
TOPIC_RESPONSE_APPENDED = "0xb1c6be0b5b8aef6539e2fac0fd131a2faa7b49edf8e505b5eb0ad487d56051d4"


def read_cursor(chain_dir: Path) -> dict | None:
    """Read sync cursor metadata if present."""
    path = chain_dir / "cursor.json"
    if path.exists():
        return json.loads(path.read_text())
    return None


def count_parquet_rows(path: Path) -> int:
    """Return the number of rows in a parquet file, 0 if missing."""
    if not path.exists() or path.stat().st_size == 0:
        return 0
    return pq.read_metadata(path).num_rows


def count_by_topic0(path: Path, topic0: str) -> int:
    """Count rows whose topic0 column matches the given event selector."""
    if not path.exists() or path.stat().st_size == 0:
        return 0
    table = pq.read_table(path, columns=["topic0"])
    mask = pc.equal(table.column("topic0"), topic0)
    return pc.sum(mask.cast("int64")).as_py() or 0


def main() -> None:
    chain_dirs = sorted(
        (d for d in DATA_DIR.iterdir() if d.is_dir()),
        key=lambda d: int(d.name) if d.name.isdigit() else 0,
    )

    rows: list[tuple] = []
    tot_id_ev = tot_rep_ev = tot_agents = tot_fb = tot_revoked = tot_resp = 0

    for d in chain_dirs:
        if not d.name.isdigit():
            continue
        cid = int(d.name)
        name = CHAIN_NAMES.get(cid, f"Unknown({cid})")

        id_path = d / "identity.parquet"
        rep_path = d / "reputation.parquet"

        id_events = count_parquet_rows(id_path)
        rep_events = count_parquet_rows(rep_path)

        # Semantic counts from topic0 filtering
        agents = count_by_topic0(id_path, TOPIC_REGISTERED)
        feedbacks = count_by_topic0(rep_path, TOPIC_NEW_FEEDBACK)
        revoked = count_by_topic0(rep_path, TOPIC_FEEDBACK_REVOKED)
        responses = count_by_topic0(rep_path, TOPIC_RESPONSE_APPENDED)

        tot_id_ev += id_events
        tot_rep_ev += rep_events
        tot_agents += agents
        tot_fb += feedbacks
        tot_revoked += revoked
        tot_resp += responses

        # Sync time from cursor
        cursor = read_cursor(d)
        if cursor and "synced_at" in cursor:
            ts = datetime.fromtimestamp(cursor["synced_at"], tz=timezone.utc)
            synced = ts.strftime("%Y-%m-%d %H:%M UTC")
        else:
            synced = "-"

        rows.append((cid, name, agents, feedbacks, revoked, responses, id_events, rep_events, synced))

    # Print table
    hdr = (
        f"{'Chain ID':>8}  {'Name':<18}"
        f"  {'Agents':>8}  {'Feedback':>8}  {'Revoked':>8}  {'Resp':>8}"
        f"  {'Id Evts':>8}  {'Rep Evts':>8}  {'Synced At'}"
    )
    sep = "-" * len(hdr)
    print(sep)
    print(f"  ERC-8004 Event Statistics  ({len(rows)} chains)")
    print(sep)
    print(hdr)
    print(sep)

    for cid, name, agents, fb, rev, resp, id_ev, rep_ev, synced in rows:
        print(
            f"{cid:>8}  {name:<18}"
            f"  {agents:>8,}  {fb:>8,}  {rev:>8,}  {resp:>8,}"
            f"  {id_ev:>8,}  {rep_ev:>8,}  {synced}"
        )

    grand = tot_id_ev + tot_rep_ev
    print(sep)
    print(
        f"{'':>8}  {'TOTAL':<18}"
        f"  {tot_agents:>8,}  {tot_fb:>8,}  {tot_revoked:>8,}  {tot_resp:>8,}"
        f"  {tot_id_ev:>8,}  {tot_rep_ev:>8,}"
    )
    print(sep)


if __name__ == "__main__":
    main()
