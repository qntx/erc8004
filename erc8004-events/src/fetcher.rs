//! RPC event fetching and chain sync orchestration.
//!
//! For each chain the fetcher:
//! 1. Reads the existing Parquet file (if any) to collect prior events.
//! 2. Queries `eth_getLogs` in adaptive batches from the cursor to the
//!    chain tip.
//! 3. Merges old + new events and rewrites the Parquet file atomically.
//! 4. Updates the chain cursor.

use std::path::Path;
use std::time::Duration;

use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Filter, Log};
use anyhow::{Context, Result, bail};
use arrow_array::RecordBatch;

use crate::chains::ChainConfig;
use crate::cursor::Cursor;
use crate::parquet;

/// Per-request timeout for RPC calls.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Delay between consecutive RPC calls to avoid rate-limiting.
const INTER_BATCH_DELAY: Duration = Duration::from_millis(100);

/// Tracks `eth_getLogs` batch size with an adaptive ceiling.
///
/// On success the size doubles toward the ceiling; on error the ceiling
/// is permanently lowered so the RPC's actual limit is learned once.
struct Batcher {
    size: u64,
    ceiling: u64,
}

impl Batcher {
    const DEFAULT: u64 = 2_000;
    const MIN: u64 = 10;

    const fn new() -> Self {
        Self {
            size: Self::DEFAULT,
            ceiling: Self::DEFAULT,
        }
    }

    /// Grow toward the learned ceiling after a successful request.
    fn grow(&mut self) {
        self.size = (self.size * 2).min(self.ceiling);
    }

    /// Shrink and lower the ceiling after a failed request.
    /// Returns `false` when already at the minimum (caller should bail).
    fn shrink(&mut self) -> bool {
        if self.size <= Self::MIN {
            return false;
        }
        self.ceiling = (self.size / 2).max(Self::MIN);
        self.size = self.ceiling;
        true
    }
}

/// Maximum consecutive RPC errors before giving up.
const MAX_CONSECUTIVE_ERRORS: u32 = 10;

/// Progress is logged every N batches.
const PROGRESS_INTERVAL: u64 = 50;

/// Fetch all logs from `address` in `[from, to]` using adaptive batches.
async fn fetch_logs<P: Provider>(
    provider: &P,
    address: Address,
    from: u64,
    to: u64,
    chain_id: u64,
) -> Result<Vec<Log>> {
    let mut logs = Vec::new();
    let mut block = from;
    let mut batch = Batcher::new();
    let mut count = 0u64;
    let mut errors = 0u32;

    while block <= to {
        let end = (block + batch.size - 1).min(to);
        let filter = Filter::new()
            .address(address)
            .from_block(block)
            .to_block(end);

        let result = tokio::time::timeout(REQUEST_TIMEOUT, provider.get_logs(&filter))
            .await
            .map_err(|_| anyhow::anyhow!("request timed out"))
            .and_then(|r| r.map_err(|e| anyhow::anyhow!("{e}")));

        match result {
            Ok(new) => {
                errors = 0;
                logs.extend(new);
                batch.grow();
                block = end + 1;
                count += 1;
                if count.is_multiple_of(PROGRESS_INTERVAL) {
                    tracing::info!(chain_id, batch = count, block, progress = %pct(block, from, to), "fetching");
                }
                tokio::time::sleep(INTER_BATCH_DELAY).await;
            }
            Err(e) => {
                errors += 1;
                if errors >= MAX_CONSECUTIVE_ERRORS {
                    bail!("chain {chain_id}: {errors} consecutive errors at block {block}: {e}");
                }
                if !batch.shrink() {
                    bail!("chain {chain_id}: failed at min batch size (block {block}): {e}");
                }
                tracing::warn!(chain_id, block, batch_size = batch.size, error = %e, "retrying");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Ok(logs)
}

/// Format progress as a percentage string.
fn pct(current: u64, from: u64, to: u64) -> String {
    if to <= from {
        return "100%".into();
    }
    #[allow(clippy::cast_precision_loss)]
    let ratio = (current - from) as f64 / (to - from) as f64 * 100.0;
    format!("{ratio:.0}%")
}

/// Fetch new events for a single contract and append to its Parquet file.
///
/// Uses `max(block_number)` from the existing file as its effective start,
/// preventing duplicate data when a prior run partially succeeded.
async fn sync_contract<P: Provider>(
    provider: &P,
    address: Address,
    name: &str,
    chain_dir: &Path,
    start: u64,
    latest: u64,
    chain_id: u64,
) -> Result<()> {
    let path = chain_dir.join(format!("{name}.parquet"));
    let mut batches = parquet::read(&path)?;

    let from = parquet::max_block_number(&batches).map_or(start, |b| b + 1);
    if from > latest {
        tracing::info!(chain_id, contract = name, "already up to date");
        return Ok(());
    }

    tracing::info!(chain_id, contract = name, %address, from, to = latest, "fetching logs");

    let logs = fetch_logs(provider, address, from, latest, chain_id).await?;
    if logs.is_empty() {
        tracing::info!(chain_id, contract = name, "no new events");
        return Ok(());
    }

    let (batch, count) = parquet::logs_to_batch(&logs)?;
    if count == 0 {
        return Ok(());
    }

    batches.push(batch);
    parquet::write(&path, &batches)?;

    let total: usize = batches.iter().map(RecordBatch::num_rows).sum();
    tracing::info!(
        chain_id,
        contract = name,
        new_events = count,
        total_events = total,
        "updated"
    );
    Ok(())
}

/// Synchronize a single chain with automatic RPC fallback.
///
/// Tries each RPC in `rpcs` in order. On failure the next endpoint is
/// attempted — cursor and parquet tracking ensure no data is duplicated.
///
/// The data directory layout is:
/// ```text
/// <data_dir>/<chain_id>/
///   ├── cursor.json
///   ├── identity.parquet
///   └── reputation.parquet
/// ```
///
/// # Errors
///
/// Returns an error only if *all* RPCs fail.
///
/// # Panics
///
/// Panics if `rpcs` is empty.
pub async fn sync_chain(chain: &ChainConfig, data_dir: &Path, rpcs: &[String]) -> Result<()> {
    let chain_id = chain.chain_id();
    let mut last_err = None;

    for (i, rpc_url) in rpcs.iter().enumerate() {
        match try_sync(chain, data_dir, rpc_url).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if i + 1 < rpcs.len() {
                    tracing::warn!(
                        chain_id,
                        rpc = %rpc_url,
                        next = %rpcs[i + 1],
                        error = %e,
                        "RPC failed, falling back"
                    );
                } else {
                    tracing::error!(chain_id, rpc = %rpc_url, error = %e, "last RPC failed");
                }
                last_err = Some(e);
            }
        }
    }

    Err(last_err.expect("rpcs is non-empty"))
}

/// Attempt a full sync using a single RPC endpoint.
async fn try_sync(chain: &ChainConfig, data_dir: &Path, rpc_url: &str) -> Result<()> {
    let chain_id = chain.chain_id();
    let chain_dir = data_dir.join(chain_id.to_string());
    std::fs::create_dir_all(&chain_dir)?;

    tracing::info!(chain_id, rpc = rpc_url, "connecting");

    let provider = ProviderBuilder::new().connect_http(
        rpc_url
            .parse()
            .with_context(|| format!("invalid RPC URL: {rpc_url}"))?,
    );

    let latest = tokio::time::timeout(REQUEST_TIMEOUT, provider.get_block_number())
        .await
        .context("get_block_number timed out")?
        .context("get_block_number failed")?;

    let start =
        Cursor::load(&chain_dir)?.map_or_else(|| chain.deployment_block, |c| c.last_block + 1);

    if start > latest {
        tracing::info!(chain_id, latest, "already up to date");
        return Ok(());
    }

    tracing::info!(
        chain_id,
        from = start,
        to = latest,
        blocks = latest - start,
        "syncing"
    );

    let addrs = chain.network.addresses();
    for (addr, name) in [
        (addrs.identity, "identity"),
        (addrs.reputation, "reputation"),
    ] {
        sync_contract(&provider, addr, name, &chain_dir, start, latest, chain_id).await?;
    }

    Cursor::now(latest).save(&chain_dir)?;
    tracing::info!(chain_id, last_block = latest, "cursor updated");
    Ok(())
}
