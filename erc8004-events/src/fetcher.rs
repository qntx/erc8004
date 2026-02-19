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

/// Maximum block range per `eth_getLogs` request (initial value).
const DEFAULT_BATCH_SIZE: u64 = 2_000;

/// Minimum batch size before giving up on retry.
const MIN_BATCH_SIZE: u64 = 50;

/// Delay between consecutive RPC calls to avoid rate-limiting.
const INTER_BATCH_DELAY: Duration = Duration::from_millis(100);

/// Progress log interval (every N batches).
const PROGRESS_LOG_INTERVAL: u64 = 50;

/// Per-request timeout for RPC calls.
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum consecutive RPC errors before giving up entirely.
const MAX_CONSECUTIVE_ERRORS: u32 = 10;

/// Fetch all logs from a contract address in the given block range.
///
/// Uses adaptive batch sizing: on timeout/error the batch is halved and
/// retried. A small delay between calls prevents RPC rate-limiting.
async fn fetch_logs<P: Provider>(
    provider: &P,
    address: Address,
    from: u64,
    to: u64,
    chain_id: u64,
) -> Result<Vec<Log>> {
    let mut all_logs = Vec::new();
    let mut cursor = from;
    let mut batch_size = DEFAULT_BATCH_SIZE;
    let mut batch_count = 0u64;
    let mut consecutive_errors = 0u32;

    while cursor <= to {
        let batch_end = (cursor + batch_size - 1).min(to);

        let filter = Filter::new()
            .address(address)
            .from_block(cursor)
            .to_block(batch_end);

        // Wrap RPC call with a timeout to prevent hanging on unresponsive nodes.
        let result = tokio::time::timeout(RPC_REQUEST_TIMEOUT, provider.get_logs(&filter))
            .await
            .map_err(|_| anyhow::anyhow!("request timed out"))
            .and_then(|r| r.map_err(|e| anyhow::anyhow!("{e}")));

        match result {
            Ok(logs) => {
                consecutive_errors = 0;
                let count = logs.len();
                all_logs.extend(logs);

                batch_size = DEFAULT_BATCH_SIZE;
                cursor = batch_end + 1;
                batch_count += 1;

                if batch_count.is_multiple_of(PROGRESS_LOG_INTERVAL) {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let pct = if to > from {
                        ((cursor - from) as f64 / (to - from) as f64 * 100.0) as u32
                    } else {
                        100
                    };
                    tracing::info!(
                        chain_id,
                        batch = batch_count,
                        block = cursor,
                        events = count,
                        progress = %format!("{pct}%"),
                        "fetching"
                    );
                }

                tokio::time::sleep(INTER_BATCH_DELAY).await;
            }
            Err(e) => {
                let msg = e.to_string();
                consecutive_errors += 1;

                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    bail!(
                        "chain {chain_id}: giving up after {MAX_CONSECUTIVE_ERRORS} \
                         consecutive errors at block {cursor}: {msg}"
                    );
                }
                if batch_size <= MIN_BATCH_SIZE {
                    bail!(
                        "chain {chain_id}: get_logs failed at block {cursor} \
                         with min batch size: {msg}"
                    );
                }
                batch_size = (batch_size / 2).max(MIN_BATCH_SIZE);
                tracing::warn!(
                    chain_id,
                    block = cursor,
                    new_batch_size = batch_size,
                    error = %msg,
                    "reducing batch size and retrying"
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Ok(all_logs)
}

/// Synchronize a single chain: fetch new events and write to Parquet.
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
/// Returns an error on RPC failures, I/O errors, or invalid log data.
pub async fn sync_chain(
    chain: &ChainConfig,
    data_dir: &Path,
    rpc_override: Option<&str>,
) -> Result<()> {
    let chain_id = chain.chain_id();
    let chain_dir = data_dir.join(chain_id.to_string());
    std::fs::create_dir_all(&chain_dir)?;

    let rpc_url = rpc_override.unwrap_or(chain.default_rpc);
    tracing::info!(chain_id, rpc = rpc_url, "connecting to RPC");

    let provider = ProviderBuilder::new().connect_http(
        rpc_url
            .parse()
            .with_context(|| format!("invalid RPC URL: {rpc_url}"))?,
    );

    // Determine the chain tip with a timeout.
    let latest = tokio::time::timeout(RPC_REQUEST_TIMEOUT, provider.get_block_number())
        .await
        .context("get_block_number timed out")?
        .context("get_block_number failed")?;

    let cursor = Cursor::load(&chain_dir)?;
    let start_block = cursor
        .as_ref()
        .map_or_else(|| chain.sync_start_block(), |c| c.last_block + 1);

    if start_block > latest {
        tracing::info!(chain_id, latest, "already up to date");
        return Ok(());
    }

    tracing::info!(
        chain_id,
        from = start_block,
        to = latest,
        blocks = latest - start_block,
        "starting sync"
    );

    // Sync each contract's events independently.
    // Each contract determines its effective start block from its own
    // Parquet file, preventing duplicate data when a prior run wrote
    // one contract successfully but failed on the other.
    let addrs = chain.network.addresses();
    for (address, name) in [
        (addrs.identity, "identity"),
        (addrs.reputation, "reputation"),
    ] {
        let parquet_path = chain_dir.join(format!("{name}.parquet"));
        let mut batches = parquet::read(&parquet_path)?;

        // Use per-file block tracking to prevent duplicates on re-runs.
        let effective_start = parquet::max_block_number(&batches).map_or(start_block, |b| b + 1);

        if effective_start > latest {
            tracing::info!(chain_id, contract = name, "already up to date");
            continue;
        }

        tracing::info!(
            chain_id,
            contract = name,
            address = %address,
            from = effective_start,
            to = latest,
            "fetching logs"
        );

        let logs = fetch_logs(&provider, address, effective_start, latest, chain_id).await?;

        if logs.is_empty() {
            tracing::info!(chain_id, contract = name, "no new events");
            continue;
        }

        let (new_batch, new_count) = parquet::logs_to_batch(&logs)?;
        if new_count == 0 {
            tracing::info!(chain_id, contract = name, "no valid events after filtering");
            continue;
        }

        // Append and write atomically.
        batches.push(new_batch);
        parquet::write(&parquet_path, &batches)?;

        let total: usize = batches.iter().map(RecordBatch::num_rows).sum();
        tracing::info!(
            chain_id,
            contract = name,
            new_events = new_count,
            total_events = total,
            "parquet updated"
        );
    }

    // Update cursor to the last fully synced block.
    Cursor::now(latest).save(&chain_dir)?;
    tracing::info!(chain_id, last_block = latest, "cursor updated");

    Ok(())
}
