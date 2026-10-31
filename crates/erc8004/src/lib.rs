//! ERC-8004 Trustless Agents protocol SDK.
//!
//! The base crate is the platform-neutral, provider-free (sans-IO) core: the
//! vendored-ABI event/error decoders, the `eip155:{chainId}:{address}` agent
//! registry, feedback values, the lenient JSON registration/feedback file
//! models, data-URI and agentURI handling, domain verification, the
//! `AgentWalletSet` EIP-712 message, the verified deployment table, and the
//! shared error codes whose spellings match `@qntx/erc8004`.
//!
//! The `provider` feature adds [`Erc8004<P>`](crate::Erc8004), an on-chain
//! client over an alloy `Provider` with identity/reputation/
//! validation handles; `fetch` adds [`Resolver`] for agentURI resolution.
//!
//! ```no_run
//! # #[cfg(feature = "provider")]
//! # async fn example() -> Result<(), erc8004::ProviderError> {
//! use alloy::primitives::U256;
//! use alloy::providers::ProviderBuilder;
//! use erc8004::{deployments, Erc8004};
//!
//! let provider = ProviderBuilder::new()
//!     .connect_http("https://mainnet.base.org".parse().unwrap());
//! let erc = Erc8004::new(provider, deployments::BASE);
//! let agent = erc.identity().get_agent(U256::from(1u64)).await?;
//! # Ok(())
//! # }
//! ```

#![allow(
    unused_crate_dependencies,
    reason = "dev-dependencies (tokio, extra alloy features) are used only by test targets"
)]
#[allow(
    clippy::too_many_arguments,
    reason = "contract ABI signatures match upstream"
)]
mod abi;
mod agent_registry;
pub mod deployments;
mod events;
mod feedback;
mod feedback_value;
mod json;
#[cfg(feature = "provider")]
mod provider;
mod registration;
#[cfg(feature = "fetch")]
mod resolve;
mod revert;
mod wallet;

pub use agent_registry::{AgentRegistry, AgentRegistryError};
pub use deployments::{Deployment, by_chain_id};
pub use events::{Event, EventDecodeError};
pub use feedback::FeedbackFile;
pub use feedback_value::{FeedbackValue, FeedbackValueError};
pub use json::JsonDocumentError;
#[cfg(feature = "provider")]
pub use provider::{
    Agent, Erc8004, Feedback, FeedbackEntry, FeedbackFilter, Identity, MetadataEntry, NewFeedback,
    ProviderError, Reputation, ReputationSummary, TxOutcome, Validation, ValidationResponse,
    ValidationStatus, ValidationSummary,
};
pub use registration::agent_uri::{AgentUri, AgentUriError, DEFAULT_IPFS_GATEWAY};
pub use registration::conformance::ConformanceIssue;
pub use registration::data_uri::{DataUriError, decode_data_uri, encode_data_uri};
pub use registration::domain::well_known_registration_url;
pub use registration::file::{REGISTRATION_TYPE_V1, Registration, RegistrationFile, Service};
#[cfg(feature = "fetch")]
pub use resolve::{ResolveError, Resolver, ResolverOptions};
pub use revert::RevertReason;
pub use wallet::{AgentWalletDeadlineError, AgentWalletSet, check_agent_wallet_deadline};
