//! ERC-8004 event archiver CLI.
//!
//! Fetches raw on-chain event logs from Identity and Reputation registries
//! across all known ERC-8004 deployments and stores them as Parquet files.
//!
//! # Usage
//!
//! ```bash
//! # Sync all mainnet chains (uses config.toml RPC pool if present)
//! erc8004-events sync --data-dir ./data
//!
//! # Sync a specific chain with a custom RPC
//! erc8004-events sync --data-dir ./data --chain 8453 --rpc https://my-rpc.example.com
//!
//! # Include testnets
//! erc8004-events sync --data-dir ./data --include-testnets
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use erc8004_events::{chains, config::Config, fetcher};
use tokio::task::JoinSet;

/// ERC-8004 raw on-chain event archiver.
#[derive(Debug, Parser)]
#[command(name = "erc8004-events", version, about)]
struct Cli {
    /// Path to config.toml (RPC pool configuration).
    #[arg(long, default_value = "config.toml", global = true)]
    config: PathBuf,

    #[command(subcommand)]
    command: Command,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Fetch new events from on-chain registries and write to Parquet.
    Sync {
        /// Output directory for chain data (e.g. `./data`).
        #[arg(long, default_value = "data")]
        data_dir: PathBuf,

        /// Sync only a specific chain by its EIP-155 chain ID.
        #[arg(long)]
        chain: Option<u64>,

        /// Override all configured RPCs with a single endpoint.
        /// Only valid when `--chain` is also specified.
        #[arg(long)]
        rpc: Option<String>,

        /// Include testnet chains in the sync.
        #[arg(long)]
        include_testnets: bool,

        /// Number of chains to sync in parallel.
        #[arg(long, default_value = "16")]
        parallel: usize,
    },

    /// List all known chain configurations.
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let config = Config::load(&cli.config)?;

    match cli.command {
        Command::Sync {
            data_dir,
            chain,
            rpc,
            include_testnets,
            parallel,
        } => cmd_sync(&config, data_dir, chain, rpc, include_testnets, parallel).await,
        Command::List => {
            cmd_list(&config);
            Ok(())
        }
    }
}

/// Execute the `sync` subcommand.
async fn cmd_sync(
    config: &Config,
    data_dir: PathBuf,
    chain_filter: Option<u64>,
    rpc_override: Option<String>,
    include_testnets: bool,
    parallel: usize,
) -> Result<()> {
    if rpc_override.is_some() && chain_filter.is_none() {
        bail!("--rpc requires --chain to be specified");
    }

    let targets: Vec<&chains::ChainConfig> = if let Some(id) = chain_filter {
        let cfg = chains::by_chain_id(id).with_context(|| format!("unknown chain ID {id}"))?;
        vec![cfg]
    } else {
        chains::ALL
            .iter()
            .filter(|c| include_testnets || !c.is_testnet)
            .collect()
    };

    // Build per-chain RPC lists: CLI override > config.toml > built-in default.
    let chain_rpcs: Vec<(chains::ChainConfig, Vec<String>)> = targets
        .iter()
        .map(|chain| {
            let rpcs = if let Some(ref url) = rpc_override {
                vec![url.clone()]
            } else {
                config.rpcs_for(chain.chain_id(), chain.default_rpc)
            };
            (**chain, rpcs)
        })
        .collect();

    let concurrency = parallel.min(chain_rpcs.len()).max(1);
    tracing::info!(
        chains = chain_rpcs.len(),
        concurrency,
        data_dir = %data_dir.display(),
        "starting sync"
    );

    let data_dir = Arc::new(data_dir);
    let success = Arc::new(AtomicU32::new(0));
    let failed = Arc::new(AtomicU32::new(0));
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
    let mut set = JoinSet::new();

    for (chain, rpcs) in chain_rpcs {
        let data_dir = Arc::clone(&data_dir);
        let success = Arc::clone(&success);
        let failed = Arc::clone(&failed);
        let semaphore = Arc::clone(&semaphore);

        set.spawn(async move {
            let _permit = semaphore.acquire().await.expect("semaphore closed");
            let chain_id = chain.chain_id();
            match fetcher::sync_chain(&chain, &data_dir, &rpcs).await {
                Ok(()) => {
                    success.fetch_add(1, Ordering::Relaxed);
                    tracing::info!(chain_id, "sync complete");
                }
                Err(e) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    tracing::error!(chain_id, error = %e, "sync failed");
                }
            }
        });
    }

    while set.join_next().await.is_some() {}

    let s = success.load(Ordering::Relaxed);
    let f = failed.load(Ordering::Relaxed);
    tracing::info!(success = s, failed = f, "sync finished");

    if f > 0 && s == 0 {
        bail!("all {f} chain(s) failed to sync");
    }
    if f > 0 {
        tracing::warn!(failed = f, success = s, "some chains failed");
    }

    Ok(())
}

/// Execute the `list` subcommand.
#[allow(clippy::print_stdout)]
fn cmd_list(config: &Config) {
    println!(
        "{:<12} {:<20} {:<8} {:<15} {:<6} RPCs",
        "Chain ID", "Name", "Type", "Deploy Block", "Pool"
    );
    println!("{}", "-".repeat(100));

    for chain in chains::ALL {
        let net_type = if chain.is_testnet { "test" } else { "main" };
        let rpcs = config.rpcs_for(chain.chain_id(), chain.default_rpc);
        println!(
            "{:<12} {:<20} {:<8} {:<15} {:<6} {}",
            chain.chain_id(),
            format!("{:?}", chain.network),
            net_type,
            chain.deployment_block,
            rpcs.len(),
            rpcs[0],
        );
    }
}
