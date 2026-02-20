//! ERC-8004 event archiver CLI.
//!
//! ```bash
//! erc8004-events sync --data-dir ./data
//! erc8004-events sync --chain 8453 --rpc https://my-rpc.example.com
//! erc8004-events sync --include-testnets
//! erc8004-events list
//! ```

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use erc8004_events::{chains, config::Config, fetcher};

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

#[derive(Debug, Subcommand)]
enum Command {
    /// Fetch new events from on-chain registries and write to Parquet.
    Sync {
        /// Output directory for chain data.
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

        /// Delay in milliseconds between consecutive RPC requests.
        #[arg(long, default_value = "100")]
        batch_delay: u64,
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
            batch_delay,
        } => {
            if rpc.is_some() && chain.is_none() {
                bail!("--rpc requires --chain to be specified");
            }

            // Resolve target chains.
            let chains: Vec<_> = if let Some(id) = chain {
                vec![chains::by_chain_id(id).with_context(|| format!("unknown chain ID {id}"))?]
            } else {
                chains::ALL
                    .iter()
                    .filter(|c| include_testnets || !c.is_testnet)
                    .collect()
            };

            // Build per-chain RPC lists: CLI override > config.toml > built-in default.
            let targets: Vec<_> = chains
                .iter()
                .map(|c| {
                    let rpcs = rpc.as_ref().map_or_else(
                        || config.rpcs_for(c.chain_id(), c.default_rpc),
                        |url| vec![url.clone()],
                    );
                    (**c, rpcs)
                })
                .collect();

            let opts = fetcher::SyncOptions {
                batch_delay: Duration::from_millis(batch_delay),
                concurrency: parallel,
                ..Default::default()
            };

            fetcher::sync_all(targets, &data_dir, opts).await
        }
        Command::List => {
            cmd_list(&config);
            Ok(())
        }
    }
}

/// Print all known chain configurations.
#[allow(clippy::print_stdout)]
fn cmd_list(config: &Config) {
    println!(
        "{:<12} {:<20} {:<8} {:<15} {:<6} RPCs",
        "Chain ID", "Name", "Type", "Deploy Block", "Pool",
    );
    println!("{}", "-".repeat(100));
    for c in chains::ALL {
        let kind = if c.is_testnet { "test" } else { "main" };
        let rpcs = config.rpcs_for(c.chain_id(), c.default_rpc);
        println!(
            "{:<12} {:<20} {:<8} {:<15} {:<6} {}",
            c.chain_id(),
            c.name,
            kind,
            c.deployment_block,
            rpcs.len(),
            rpcs[0],
        );
    }
}
