//! Typed error definitions for the ERC-8004 SDK.

use alloy::primitives::Address;

/// The primary error type for all ERC-8004 SDK operations.
#[derive(Debug, thiserror::Error)]
pub enum Erc8004Error {
    /// A contract interaction failed.
    #[error("contract error: {0}")]
    Contract(#[from] alloy::contract::Error),

    /// An RPC transport error occurred.
    #[error("transport error: {0}")]
    Transport(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),

    /// The requested registry address was not configured.
    #[error("registry not configured: {registry}")]
    RegistryNotConfigured {
        /// Which registry is missing.
        registry: &'static str,
    },

    /// The agent with the given ID does not exist on-chain.
    #[error("agent {agent_id} does not exist")]
    AgentNotFound {
        /// The queried agent ID.
        agent_id: alloy::primitives::U256,
    },

    /// A pending transaction was dropped or failed to confirm.
    #[error("pending transaction error: {0}")]
    PendingTransaction(#[from] alloy::providers::PendingTransactionError),

    /// JSON serialization / deserialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// An address string could not be parsed.
    #[error("invalid address: {address}")]
    InvalidAddress {
        /// The raw address string that failed to parse.
        address: String,
        /// The underlying parse error.
        #[source]
        source: alloy::hex::FromHexError,
    },

    /// The identity registry address returned by Reputation/Validation
    /// does not match the configured identity registry.
    #[error("identity registry mismatch: expected {expected}, got {actual}")]
    IdentityRegistryMismatch {
        /// The expected address.
        expected: Address,
        /// The actual address returned by the contract.
        actual: Address,
    },
}

/// A convenience type alias used throughout the SDK.
pub type Result<T> = core::result::Result<T, Erc8004Error>;
