//! Provider-layer error: RPC failures, contract reverts, and client-side guards.

use thiserror::Error;

use crate::revert::RevertReason;
use crate::wallet::AgentWalletDeadlineError;

/// Any failure from an on-chain action.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProviderError {
    /// The transport or node returned an error (non-revert payloads, timeouts).
    #[error("rpc error: {0}")]
    Rpc(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// The contract reverted; `0` is the decoded reason.
    #[error("contract reverted: {0}")]
    Revert(RevertReason),
    /// The transaction mined with `status = false`.
    #[error("transaction {tx_hash} failed")]
    TransactionFailed {
        /// The failing transaction hash.
        tx_hash: alloy::primitives::B256,
    },
    /// An expected event was absent from a receipt's logs.
    #[error("expected a {event} event in the receipt logs")]
    MissingEvent {
        /// The expected event name.
        event: &'static str,
    },
    /// An on-chain value violated the SDK's value domain.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    /// `getSummary`/`readAllFeedback` with an empty client list.
    #[error("clientAddresses required")]
    ClientAddressesRequired,
    /// `validationResponse` outside `0..=100`.
    #[error("response out of range")]
    ResponseOutOfRange,
    /// Metadata key `agentWallet` is reserved by the contract.
    #[error("reserved metadata key")]
    ReservedMetadataKey,
    /// `appendResponse` with an empty `responseURI`.
    #[error("empty response uri")]
    EmptyResponseUri,
    /// `setAgentWallet` deadline outside the contract's ±5 minute window.
    #[error(transparent)]
    Deadline(#[from] AgentWalletDeadlineError),
}

impl ProviderError {
    /// The shared error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Rpc(_) => "Rpc",
            Self::Revert(_) => "ContractRevert",
            Self::TransactionFailed { .. } => "TransactionFailed",
            Self::MissingEvent { .. } => "MissingEvent",
            Self::InvalidResponse(_) => "InvalidResponse",
            Self::ClientAddressesRequired => "ClientAddressesRequired",
            Self::ResponseOutOfRange => "ResponseOutOfRange",
            Self::ReservedMetadataKey => "ReservedMetadataKey",
            Self::EmptyResponseUri => "EmptyResponseUri",
            Self::Deadline(e) => e.code(),
        }
    }
}
