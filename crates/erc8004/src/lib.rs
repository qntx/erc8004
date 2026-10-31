//! ERC-8004 Trustless Agents protocol SDK — pure (sans-IO) core.
//!
//! This crate holds the platform-neutral half of the dual-language SDK: the
//! vendored-ABI event/error decoders, the `eip155:{chainId}:{address}` agent
//! registry, feedback values, the lenient JSON registration/feedback file
//! models, data-URI and agentURI handling, domain verification, the
//! `AgentWalletSet` EIP-712 message, the verified deployment table, and the
//! shared error codes whose spellings match `@qntx/erc8004`.

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
mod registration;
mod revert;
mod wallet;

pub use agent_registry::{AgentRegistry, AgentRegistryError};
pub use deployments::{Deployment, by_chain_id};
pub use events::{Event, EventDecodeError};
pub use feedback::FeedbackFile;
pub use feedback_value::{FeedbackValue, FeedbackValueError};
pub use json::JsonDocumentError;
pub use registration::agent_uri::{AgentUri, AgentUriError, DEFAULT_IPFS_GATEWAY};
pub use registration::conformance::ConformanceIssue;
pub use registration::data_uri::{DataUriError, decode_data_uri, encode_data_uri};
pub use registration::domain::well_known_registration_url;
pub use registration::file::{REGISTRATION_TYPE_V1, Registration, RegistrationFile, Service};
pub use revert::RevertReason;
pub use wallet::{AgentWalletDeadlineError, AgentWalletSet, check_agent_wallet_deadline};
