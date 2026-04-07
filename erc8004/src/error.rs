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

    /// A registration transaction succeeded but emitted no `Registered` event.
    #[error("transaction receipt contained no Registered event")]
    MissingRegisteredEvent,

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_not_configured_display() {
        let err = Erc8004Error::RegistryNotConfigured {
            registry: "identity",
        };
        assert_eq!(err.to_string(), "registry not configured: identity");
    }

    #[test]
    fn test_agent_not_found_display() {
        let err = Erc8004Error::AgentNotFound {
            agent_id: alloy::primitives::U256::from(42),
        };
        assert_eq!(err.to_string(), "agent 42 does not exist");
    }

    #[test]
    fn test_missing_registered_event_display() {
        let err = Erc8004Error::MissingRegisteredEvent;
        assert_eq!(
            err.to_string(),
            "transaction receipt contained no Registered event"
        );
    }

    #[test]
    fn test_identity_registry_mismatch_display() {
        let expected = Address::ZERO;
        let actual = Address::repeat_byte(0x01);
        let err = Erc8004Error::IdentityRegistryMismatch { expected, actual };
        assert_eq!(
            err.to_string(),
            format!("identity registry mismatch: expected {expected}, got {actual}")
        );
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Erc8004Error>();
    }
}
