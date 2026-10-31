//! Verified on-chain deployment table — generated from `data/deployments.json`.

use alloy::primitives::Address;

use crate::agent_registry::AgentRegistry;

mod data;

pub use data::*;

/// A verified ERC-8004 deployment on one chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Deployment {
    /// Kebab-case deployment id (e.g. `base-sepolia`).
    pub id: &'static str,
    /// Human-readable chain name.
    pub name: &'static str,
    /// EIP-155 chain id.
    pub chain_id: u64,
    /// Whether this is a testnet.
    pub testnet: bool,
    /// Identity Registry address (EIP-55).
    pub identity_registry: Address,
    /// Reputation Registry address (EIP-55).
    pub reputation_registry: Address,
    /// Validation Registry address (EIP-55).
    pub validation_registry: Address,
}

impl Deployment {
    /// The `eip155:{chainId}:{identityRegistry}` of this deployment.
    #[must_use]
    pub const fn agent_registry(&self) -> AgentRegistry {
        AgentRegistry {
            chain_id: self.chain_id,
            address: self.identity_registry,
        }
    }
}

/// The deployment for `chain_id`, if any.
#[must_use]
pub fn by_chain_id(chain_id: u64) -> Option<&'static Deployment> {
    ALL.iter().find(|d| d.chain_id == chain_id)
}
