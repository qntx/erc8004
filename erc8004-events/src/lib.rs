//! ERC-8004 raw on-chain event archiver library.
//!
//! Fetches event logs from Identity and Reputation registries across all
//! known ERC-8004 deployments and stores them as Parquet files.

pub mod chains;
pub mod config;
pub mod cursor;
pub mod fetcher;
pub mod parquet;
