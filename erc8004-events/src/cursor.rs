//! Per-chain sync cursor persistence.
//!
//! Each chain directory contains a `cursor.json` that records the last
//! fully-synced block number so that subsequent runs only fetch the delta.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Sync progress for a single chain.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cursor {
    /// The last block whose events have been fully written to Parquet.
    pub last_block: u64,
    /// Unix timestamp (seconds) of the last successful sync.
    pub synced_at: u64,
}

impl Cursor {
    /// Create a new cursor at the given block with the current timestamp.
    #[must_use]
    pub fn now(last_block: u64) -> Self {
        let synced_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            last_block,
            synced_at,
        }
    }

    /// Read cursor from `<dir>/cursor.json`.
    ///
    /// Returns `None` if the file does not exist (first sync) or contains
    /// invalid JSON (logs a warning and triggers fresh sync).
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read (I/O error).
    pub fn load(dir: &Path) -> Result<Option<Self>> {
        let path = dir.join("cursor.json");
        if !path.exists() {
            return Ok(None);
        }
        let data = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        match serde_json::from_str::<Self>(&data) {
            Ok(cursor) => Ok(Some(cursor)),
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "corrupted cursor, starting fresh");
                Ok(None)
            }
        }
    }

    /// Persist cursor to `<dir>/cursor.json` atomically.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file
    /// cannot be written.
    pub fn save(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;

        let path = dir.join("cursor.json");
        let tmp = dir.join("cursor.json.tmp");

        std::fs::write(&tmp, serde_json::to_string_pretty(self)?.as_bytes())
            .with_context(|| format!("writing {}", tmp.display()))?;
        std::fs::rename(&tmp, &path)
            .with_context(|| format!("renaming {} → {}", tmp.display(), path.display()))?;

        Ok(())
    }
}
