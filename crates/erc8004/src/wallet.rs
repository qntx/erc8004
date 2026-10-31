//! `AgentWalletSet` EIP-712 typed data for `setAgentWallet`/`unsetAgentWallet`.

use alloy::primitives::{Address, B256, U256};
use alloy::sol;
use alloy::sol_types::{Eip712Domain, SolStruct};
use thiserror::Error;

sol! {
    /// `AgentWalletSet(uint256 agentId,address newWallet,address owner,uint256 deadline)`.
    #[derive(Debug)]
    struct AgentWalletSet {
        uint256 agentId;
        address newWallet;
        address owner;
        uint256 deadline;
    }
}

/// `AgentWalletSet` deadline outside the contract's ±5 minute window.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AgentWalletDeadlineError {
    /// `deadline < now`.
    #[error("deadline {deadline} is in the past (now {now})")]
    DeadlineExpired {
        /// The supplied deadline.
        deadline: u64,
        /// The comparison timestamp.
        now: u64,
    },
    /// `deadline > now + 300`.
    #[error("deadline {deadline} is more than 5 minutes ahead (now {now})")]
    DeadlineTooFar {
        /// The supplied deadline.
        deadline: u64,
        /// The comparison timestamp.
        now: u64,
    },
}

impl AgentWalletDeadlineError {
    /// The shared error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::DeadlineExpired { .. } => "DeadlineExpired",
            Self::DeadlineTooFar { .. } => "DeadlineTooFar",
        }
    }
}

impl AgentWalletSet {
    /// The `ERC8004IdentityRegistry` EIP-712 domain for `chain_id` at
    /// `identity_registry`.
    #[must_use]
    pub fn domain(chain_id: u64, identity_registry: Address) -> Eip712Domain {
        Eip712Domain {
            name: Some("ERC8004IdentityRegistry".into()),
            version: Some("1".into()),
            chain_id: Some(U256::from(chain_id)),
            verifying_contract: Some(identity_registry),
            salt: None,
        }
    }

    /// The EIP-712 signing hash of this message under the domain for
    /// `chain_id` at `identity_registry`.
    #[must_use]
    pub fn signing_hash(&self, chain_id: u64, identity_registry: Address) -> B256 {
        self.eip712_signing_hash(&Self::domain(chain_id, identity_registry))
    }
}

/// Contract check: `block.timestamp <= deadline <= block.timestamp + 5 minutes`.
///
/// # Errors
///
/// `DeadlineExpired` when `deadline < now`, `DeadlineTooFar` when `deadline > now + 300`.
pub const fn check_agent_wallet_deadline(
    deadline: u64,
    now: u64,
) -> Result<(), AgentWalletDeadlineError> {
    if deadline < now {
        return Err(AgentWalletDeadlineError::DeadlineExpired { deadline, now });
    }
    if deadline > now.saturating_add(300) {
        return Err(AgentWalletDeadlineError::DeadlineTooFar { deadline, now });
    }
    Ok(())
}
