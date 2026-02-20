//! Chain sync orchestration and adaptive RPC event fetching.
//!
//! - [`sync_all`] — parallel sync of multiple chains (main entry point).
//! - [`sync_chain`] — single-chain sync with automatic RPC fallback.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Filter, Log};
use anyhow::{Context, Result, bail};
use arrow_array::RecordBatch;
use tokio::task::JoinSet;

use crate::chains::ChainConfig;
use crate::cursor::Cursor;
use crate::parquet;

/// Tunable parameters for a sync run.
#[derive(Debug, Clone, Copy)]
pub struct SyncOptions {
    /// Delay between consecutive `eth_getLogs` calls.
    pub batch_delay: Duration,
    /// Per-request timeout.
    pub request_timeout: Duration,
    /// Consecutive RPC errors before abandoning an endpoint.
    pub max_errors: u32,
    /// Chains synced in parallel.
    pub concurrency: usize,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            batch_delay: Duration::from_millis(100),
            request_timeout: Duration::from_secs(30),
            max_errors: 10,
            concurrency: 16,
        }
    }
}

/// Adaptive block-range window (TCP slow-start style).
///
/// Grows on success, shrinks on errors.  Only "range too large" errors
/// lower the ceiling; transient errors leave it intact.
struct Batcher {
    size: u64,
    ceiling: u64,
}

impl Batcher {
    const INITIAL: u64 = 500;
    const MAX_CEILING: u64 = 50_000;
    const MIN: u64 = 10;

    const fn new() -> Self {
        Self {
            size: Self::INITIAL,
            ceiling: Self::MAX_CEILING,
        }
    }

    fn grow(&mut self) {
        self.size = (self.size * 2).min(self.ceiling);
    }

    /// Permanently lower ceiling. Returns `false` at minimum.
    fn shrink_for_range(&mut self) -> bool {
        if self.size <= Self::MIN {
            return false;
        }
        self.ceiling = (self.size / 2).max(Self::MIN);
        self.size = self.ceiling;
        true
    }

    /// Halve without touching ceiling (transient recovery).
    fn shrink_transient(&mut self) {
        self.size = (self.size / 2).max(Self::MIN);
    }
}

// ── Error classification ─────────────────────────────────────────────

/// Broad classification of RPC errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RpcErrorKind {
    /// `eth_getLogs` block range exceeds the node's limit.
    RangeTooLarge,
    /// HTTP 429 or explicit rate-limit response.
    RateLimited,
    /// Timeout, connection reset, or other transient issue.
    Transient,
}

/// Heuristic classification covering major RPC providers.
fn classify_error(err: &anyhow::Error) -> RpcErrorKind {
    let msg = err.to_string().to_lowercase();

    // Range / block-limit errors.
    if msg.contains("block range")
        || msg.contains("range too large")
        || (msg.contains("exceed") && msg.contains("block"))
        || msg.contains("max range")
        || msg.contains("query returned more than")
        || msg.contains("log response size exceeded")
        || (msg.contains("eth_getlogs") && msg.contains("limit"))
    {
        return RpcErrorKind::RangeTooLarge;
    }

    // Rate-limit errors.
    if msg.contains("429")
        || msg.contains("rate limit")
        || msg.contains("too many request")
        || msg.contains("throttl")
        || msg.contains("backoff")
        || msg.contains("capacity")
    {
        return RpcErrorKind::RateLimited;
    }

    RpcErrorKind::Transient
}

/// Exponential back-off with clock-based jitter, capped at 30 s.
fn backoff_duration(attempt: u32) -> Duration {
    const BASE: u64 = 1_000;
    const CAP: u64 = 30_000;
    let ms = BASE.saturating_mul(1u64 << attempt.min(5)).min(CAP);
    let half = ms / 2;
    let jitter = u64::from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos(),
    ) % (half + 1);
    Duration::from_millis(half + jitter)
}

/// Flush pending logs to Parquet every N events.
const FLUSH_THRESHOLD: usize = 5_000;

/// Log progress every N RPC requests.
const PROGRESS_INTERVAL: u64 = 50;

/// Synchronise multiple chains in parallel.
///
/// # Errors
///
/// Returns an error only if **all** chains fail.
///
/// # Panics
///
/// Panics if the internal semaphore is closed (should never happen).
pub async fn sync_all(
    targets: Vec<(ChainConfig, Vec<String>)>,
    data_dir: &Path,
    opts: SyncOptions,
) -> Result<()> {
    let n = opts.concurrency.min(targets.len()).max(1);
    tracing::info!(
        chains = targets.len(),
        concurrency = n,
        data_dir = %data_dir.display(),
        "starting sync",
    );

    let data_dir = Arc::new(data_dir.to_path_buf());
    let opts = Arc::new(opts);
    let ok = Arc::new(AtomicU32::new(0));
    let fail = Arc::new(AtomicU32::new(0));
    let sem = Arc::new(tokio::sync::Semaphore::new(n));
    let mut set = JoinSet::new();

    for (chain, rpcs) in targets {
        let (dir, opts, ok, fail, sem) = (
            Arc::clone(&data_dir),
            Arc::clone(&opts),
            Arc::clone(&ok),
            Arc::clone(&fail),
            Arc::clone(&sem),
        );
        set.spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let cid = chain.chain_id();
            match sync_chain(&chain, &dir, &rpcs, &opts).await {
                Ok(()) => {
                    ok.fetch_add(1, Ordering::Relaxed);
                    tracing::info!(chain_id = cid, "sync complete");
                }
                Err(e) => {
                    fail.fetch_add(1, Ordering::Relaxed);
                    tracing::error!(chain_id = cid, error = %e, "sync failed");
                }
            }
        });
    }

    while set.join_next().await.is_some() {}

    let (s, f) = (ok.load(Ordering::Relaxed), fail.load(Ordering::Relaxed));
    tracing::info!(success = s, failed = f, "sync finished");
    if f > 0 && s == 0 {
        bail!("all {f} chain(s) failed to sync");
    }
    if f > 0 {
        tracing::warn!(failed = f, success = s, "some chains failed");
    }
    Ok(())
}

/// Synchronise a single chain, trying each RPC in order.
///
/// # Errors
///
/// Returns an error only if *all* RPCs fail.
///
/// # Panics
///
/// Panics if `rpcs` is empty.
pub async fn sync_chain(
    chain: &ChainConfig,
    data_dir: &Path,
    rpcs: &[String],
    opts: &SyncOptions,
) -> Result<()> {
    let cid = chain.chain_id();
    let mut last_err = None;
    for (i, url) in rpcs.iter().enumerate() {
        match try_sync(chain, data_dir, url, opts).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if i + 1 < rpcs.len() {
                    tracing::warn!(chain_id = cid, rpc = %url, next = %rpcs[i + 1], error = %e, "falling back");
                } else {
                    tracing::error!(chain_id = cid, rpc = %url, error = %e, "last RPC failed");
                }
                last_err = Some(e);
            }
        }
    }
    Err(last_err.expect("rpcs is non-empty"))
}

/// Binds a provider + chain context so that method signatures stay short.
struct Session<'a, P> {
    provider: &'a P,
    chain_id: u64,
    dir: &'a Path,
    opts: &'a SyncOptions,
}

/// Connect to a single RPC and sync both contracts.
async fn try_sync(
    chain: &ChainConfig,
    data_dir: &Path,
    rpc_url: &str,
    opts: &SyncOptions,
) -> Result<()> {
    let cid = chain.chain_id();
    let dir = data_dir.join(cid.to_string());
    std::fs::create_dir_all(&dir)?;

    tracing::info!(chain_id = cid, rpc = rpc_url, "connecting");
    let provider = ProviderBuilder::new().connect_http(
        rpc_url
            .parse()
            .with_context(|| format!("invalid RPC URL: {rpc_url}"))?,
    );

    let latest = tokio::time::timeout(opts.request_timeout, provider.get_block_number())
        .await
        .context("get_block_number timed out")?
        .context("get_block_number failed")?;

    let start = Cursor::load(&dir)?.map_or_else(|| chain.deployment_block, |c| c.last_block + 1);

    if start > latest {
        tracing::info!(chain_id = cid, latest, "already up to date");
        return Ok(());
    }

    tracing::info!(
        chain_id = cid,
        from = start,
        to = latest,
        blocks = latest - start,
        "syncing"
    );

    let s = Session {
        provider: &provider,
        chain_id: cid,
        dir: &dir,
        opts,
    };
    let addrs = chain.network.addresses();
    for (addr, name) in [
        (addrs.identity, "identity"),
        (addrs.reputation, "reputation"),
    ] {
        s.sync_contract(addr, name, start, latest).await?;
    }

    Cursor::now(latest).save(&dir)?;
    tracing::info!(chain_id = cid, last_block = latest, "cursor updated");
    Ok(())
}

impl<P: Provider> Session<'_, P> {
    /// Sync a single contract: read existing Parquet, fetch new logs, flush.
    async fn sync_contract(
        &self,
        address: Address,
        name: &str,
        start: u64,
        latest: u64,
    ) -> Result<()> {
        let path = self.dir.join(format!("{name}.parquet"));
        let mut batches = parquet::read(&path)?;

        let from = parquet::max_block_number(&batches).map_or(start, |b| b + 1);
        if from > latest {
            tracing::info!(
                chain_id = self.chain_id,
                contract = name,
                "already up to date"
            );
            return Ok(());
        }

        tracing::info!(chain_id = self.chain_id, contract = name, %address, from, to = latest, "fetching logs");

        let new = self
            .fetch_logs(address, &path, &mut batches, from, latest)
            .await?;
        if new == 0 {
            tracing::info!(chain_id = self.chain_id, contract = name, "no new events");
        } else {
            let total: usize = batches.iter().map(RecordBatch::num_rows).sum();
            tracing::info!(
                chain_id = self.chain_id,
                contract = name,
                new_events = new,
                total_events = total,
                "updated"
            );
        }
        Ok(())
    }

    /// Adaptive fetch loop with periodic flushing.
    async fn fetch_logs(
        &self,
        address: Address,
        path: &Path,
        batches: &mut Vec<RecordBatch>,
        from: u64,
        to: u64,
    ) -> Result<usize> {
        let cid = self.chain_id;
        let mut pending: Vec<Log> = Vec::new();
        let mut block = from;
        let mut batcher = Batcher::new();
        let mut reqs = 0u64;
        let mut errors = 0u32;
        let mut total = 0usize;

        while block <= to {
            let end = (block + batcher.size - 1).min(to);
            let filter = Filter::new()
                .address(address)
                .from_block(block)
                .to_block(end);

            let res =
                tokio::time::timeout(self.opts.request_timeout, self.provider.get_logs(&filter))
                    .await
                    .map_err(|_| anyhow::anyhow!("request timed out"))
                    .and_then(|r| r.map_err(|e| anyhow::anyhow!("{e}")));

            match res {
                Ok(logs) => {
                    errors = 0;
                    pending.extend(logs);
                    batcher.grow();
                    block = end + 1;
                    reqs += 1;

                    if pending.len() >= FLUSH_THRESHOLD {
                        total += flush(&mut pending, path, batches)?;
                    }
                    if reqs.is_multiple_of(PROGRESS_INTERVAL) {
                        #[allow(clippy::cast_precision_loss)]
                        let pct = if to > from {
                            (block - from) as f64 / (to - from) as f64 * 100.0
                        } else {
                            100.0
                        };
                        tracing::info!(
                            chain_id = cid, reqs, block,
                            batch_size = batcher.size,
                            progress = %format_args!("{pct:.0}%"),
                            "fetching",
                        );
                    }
                    tokio::time::sleep(self.opts.batch_delay).await;
                }
                Err(e) => {
                    let kind = classify_error(&e);
                    errors += 1;

                    if errors >= self.opts.max_errors {
                        let _ = flush(&mut pending, path, batches);
                        bail!("chain {cid}: {errors} consecutive errors at block {block}: {e}");
                    }

                    match kind {
                        RpcErrorKind::RangeTooLarge => {
                            if !batcher.shrink_for_range() {
                                let _ = flush(&mut pending, path, batches);
                                bail!("chain {cid}: range error at min batch (block {block}): {e}");
                            }
                            tracing::warn!(
                                chain_id = cid,
                                block,
                                batch_size = batcher.size,
                                "range too large, shrinking"
                            );
                            tokio::time::sleep(Duration::from_millis(200)).await;
                        }
                        RpcErrorKind::RateLimited => {
                            let d = backoff_duration(errors);
                            tracing::warn!(
                                chain_id = cid,
                                block,
                                delay_ms = d.as_millis(),
                                "rate limited"
                            );
                            tokio::time::sleep(d).await;
                        }
                        RpcErrorKind::Transient => {
                            batcher.shrink_transient();
                            let d = backoff_duration(errors);
                            tracing::warn!(chain_id = cid, block, batch_size = batcher.size, delay_ms = d.as_millis(), error = %e, "transient error");
                            tokio::time::sleep(d).await;
                        }
                    }
                }
            }
        }

        total += flush(&mut pending, path, batches)?;
        Ok(total)
    }
}

/// Write pending logs to Parquet and clear the buffer.
fn flush(pending: &mut Vec<Log>, path: &Path, batches: &mut Vec<RecordBatch>) -> Result<usize> {
    if pending.is_empty() {
        return Ok(0);
    }
    let (batch, n) = parquet::logs_to_batch(pending)?;
    if n > 0 {
        batches.push(batch);
        parquet::write(path, batches)?;
    }
    pending.clear();
    Ok(n)
}
