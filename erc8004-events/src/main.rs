//! ERC-8004 event archiver CLI.
//!
//! Fetches raw on-chain event logs from Identity and Reputation registries
//! across all known ERC-8004 deployments and stores them as Parquet files.
//!
//! # Usage
//!
//! ```bash
//! # Sync all mainnet chains using default public RPCs
//! erc8004-events sync --data-dir ./data
//!
//! # Sync a specific chain
//! erc8004-events sync --data-dir ./data --chain 8453
//!
//! # Sync with a custom RPC endpoint
//! erc8004-events sync --data-dir ./data --chain 8453 --rpc https://my-rpc.example.com
//!
//! # Include testnets
//! erc8004-events sync --data-dir ./data --include-testnets
//! ```

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use erc8004_events::{chains, fetcher};

/// ERC-8004 raw on-chain event archiver.
#[derive(Debug, Parser)]
#[command(name = "erc8004-events", version, about)]
struct Cli {
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
        /// If omitted, all mainnet chains are synced.
        #[arg(long)]
        chain: Option<u64>,

        /// Override the default RPC endpoint for the target chain.
        /// Only valid when `--chain` is also specified.
        #[arg(long)]
        rpc: Option<String>,

        /// Include testnet chains in the sync.
        #[arg(long)]
        include_testnets: bool,
    },

    /// List all known chain configurations.
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Sync {
            data_dir,
            chain,
            rpc,
            include_testnets,
        } => cmd_sync(data_dir, chain, rpc, include_testnets).await,
        Command::List => {
            cmd_list();
            Ok(())
        }
    }
}

/// Execute the `sync` subcommand.
async fn cmd_sync(
    data_dir: PathBuf,
    chain_filter: Option<u64>,
    rpc_override: Option<String>,
    include_testnets: bool,
) -> Result<()> {
    // Validate args: --rpc requires --chain.
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

    tracing::info!(
        chains = targets.len(),
        data_dir = %data_dir.display(),
        "starting sync"
    );

    let mut success = 0u32;
    let mut failed = 0u32;

    for chain in &targets {
        let chain_id = chain.chain_id();
        match fetcher::sync_chain(chain, &data_dir, rpc_override.as_deref()).await {
            Ok(()) => {
                success += 1;
                tracing::info!(chain_id, "sync complete");
            }
            Err(e) => {
                failed += 1;
                tracing::error!(chain_id, error = %e, "sync failed");
            }
        }
    }

    tracing::info!(success, failed, "sync finished");

    if failed > 0 {
        bail!("{failed} chain(s) failed to sync");
    }

    Ok(())
}

/// Execute the `list` subcommand.
#[allow(clippy::print_stdout)]
fn cmd_list() {
    println!(
        "{:<12} {:<20} {:<8} {:<15} RPC",
        "Chain ID", "Name", "Type", "Deploy Block"
    );
    println!("{}", "-".repeat(90));

    for chain in chains::ALL {
        let net_type = if chain.is_testnet { "test" } else { "main" };
        println!(
            "{:<12} {:<20} {:<8} {:<15} {}",
            chain.chain_id(),
            format!("{:?}", chain.network),
            net_type,
            chain.deployment_block,
            chain.default_rpc,
        );
    }
}
