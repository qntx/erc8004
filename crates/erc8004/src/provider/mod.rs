//! Provider actions — `Erc8004<P>` borrows any alloy `Provider` and exposes the
//! same operations as the TypeScript `identity`/`reputation`/`validation` layers.

#![allow(
    unreachable_pub,
    reason = "private module; public items reach callers through the crate-root re-export"
)]

mod client;
mod error;
mod identity;
mod reputation;
mod validation;

pub use error::ProviderError;

use alloy::providers::Provider;
use alloy::rpc::types::TransactionReceipt;
use alloy::sol_types::SolCall;

use crate::deployments::Deployment;

pub use identity::{Agent, Identity, MetadataEntry};
pub use reputation::{Feedback, FeedbackEntry, Reputation, ReputationSummary};
pub use validation::{Validation, ValidationStatus, ValidationSummary};

/// A mined write: the transaction hash plus the value parsed from its receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TxOutcome<T> {
    /// The transaction hash.
    pub tx_hash: alloy::primitives::B256,
    /// The value decoded from the receipt (agentId, feedbackIndex, or `()`).
    pub value: T,
}

/// On-chain ERC-8004 access over any alloy `Provider`.
#[derive(Debug, Clone, Copy)]
pub struct Erc8004<P> {
    provider: P,
    deployment: Deployment,
}

impl<P: Provider> Erc8004<P> {
    /// Binds `provider` to `deployment` (registry addresses).
    #[must_use]
    pub const fn new(provider: P, deployment: Deployment) -> Self {
        Self {
            provider,
            deployment,
        }
    }

    /// The underlying provider.
    #[must_use]
    pub const fn provider(&self) -> &P {
        &self.provider
    }

    /// The bound deployment.
    #[must_use]
    pub const fn deployment(&self) -> Deployment {
        self.deployment
    }

    /// Identity-registry handle.
    #[must_use]
    pub const fn identity(&self) -> Identity<'_, P> {
        Identity::new(self)
    }

    /// Reputation-registry handle.
    #[must_use]
    pub const fn reputation(&self) -> Reputation<'_, P> {
        Reputation::new(self)
    }

    /// Validation-registry handle.
    #[must_use]
    pub const fn validation(&self) -> Validation<'_, P> {
        Validation::new(self)
    }

    pub(crate) async fn call<C: SolCall>(
        &self,
        to: alloy::primitives::Address,
        call: C,
    ) -> Result<C::Return, ProviderError> {
        client::call(&self.provider, to, call).await
    }

    pub(crate) async fn send<C: SolCall>(
        &self,
        to: alloy::primitives::Address,
        call: C,
    ) -> Result<TransactionReceipt, ProviderError> {
        client::send(&self.provider, to, call).await
    }

    pub(crate) async fn latest_timestamp(&self) -> Result<u64, ProviderError> {
        client::latest_timestamp(&self.provider).await
    }
}
