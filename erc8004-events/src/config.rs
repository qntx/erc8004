//! Runtime configuration loaded from `config.toml`.
//!
//! Provides per-chain RPC endpoint lists that the sync engine uses with
//! automatic fallback: if the primary RPC fails, the next one is tried.
//!
//! When no config file is present the built-in defaults from
//! [`crate::chains::ChainConfig::default_rpc`] are used.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Top-level configuration.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// Per-chain RPC overrides, keyed by chain ID.
    #[serde(default)]
    pub chains: HashMap<u64, ChainRpcs>,
}

/// RPC endpoint list for a single chain.
#[derive(Debug, Clone, Deserialize)]
pub struct ChainRpcs {
    /// Ordered list of RPC URLs (best first).
    pub rpcs: Vec<String>,
}

impl Config {
    /// Load configuration from a TOML file.
    ///
    /// Returns [`Config::default`] if the file does not exist,
    /// allowing the binary to work without any config.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    /// Return the RPC URL list for a chain, falling back to the built-in
    /// default if the config has no entry for this chain.
    #[must_use]
    pub fn rpcs_for(&self, chain_id: u64, default_rpc: &str) -> Vec<String> {
        match self.chains.get(&chain_id) {
            Some(c) if !c.rpcs.is_empty() => c.rpcs.clone(),
            _ => vec![default_rpc.to_owned()],
        }
    }
}
