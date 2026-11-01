//! `eip155:{chainId}:{address}` — the on-chain identity of an agent.

/// `eip155:{chainId}:{address}` parse failure.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AgentRegistryError {
    /// Not exactly three `:`-separated non-empty parts.
    #[error("malformed agent registry string")]
    Malformed,
    /// Namespace is not `eip155` (case-sensitive).
    #[error("unsupported namespace: {0}")]
    UnsupportedNamespace(String),
    /// Not `[1-9][0-9]*`, or exceeds 2^53-1.
    #[error("invalid chain id: {0}")]
    InvalidChainId(String),
    /// Not `0x` + 40 hex digits.
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    /// Mixed-case address that fails EIP-55.
    #[error("bad EIP-55 checksum: {0}")]
    BadChecksum(String),
}

impl AgentRegistryError {
    /// The shared error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Malformed => "Malformed",
            Self::UnsupportedNamespace(_) => "UnsupportedNamespace",
            Self::InvalidChainId(_) => "InvalidChainId",
            Self::InvalidAddress(_) => "InvalidAddress",
            Self::BadChecksum(_) => "BadChecksum",
        }
    }
}

use std::fmt;
use std::str::FromStr;

use alloy::primitives::Address;

use thiserror::Error;

/// An `eip155:{chainId}:{address}` agent registry reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentRegistry {
    /// EIP-155 chain id (1..=2^53-1).
    pub chain_id: u64,
    /// Identity registry address.
    pub address: Address,
}

const MAX_CHAIN_ID: u64 = 9_007_199_254_740_991; // 2^53 - 1

impl AgentRegistry {
    /// Parses `eip155:{chainId}:{address}`; the address must be all-lowercase,
    /// all-uppercase, or valid EIP-55.
    ///
    /// # Errors
    ///
    /// Returns [`AgentRegistryError`] with the failing rule's code.
    pub fn parse(text: &str) -> Result<Self, AgentRegistryError> {
        let parts: Vec<&str> = text.split(':').collect();
        let [namespace, chain_id_text, address_text] = parts.as_slice() else {
            return Err(AgentRegistryError::Malformed);
        };
        let (namespace, chain_id_text, address_text) = (*namespace, *chain_id_text, *address_text);
        if namespace.is_empty() || chain_id_text.is_empty() || address_text.is_empty() {
            return Err(AgentRegistryError::Malformed);
        }
        if namespace != "eip155" {
            return Err(AgentRegistryError::UnsupportedNamespace(
                namespace.to_owned(),
            ));
        }
        let valid = !chain_id_text.is_empty()
            && chain_id_text.bytes().all(|b| b.is_ascii_digit())
            && !chain_id_text.starts_with('0');
        if !valid {
            return Err(AgentRegistryError::InvalidChainId(chain_id_text.to_owned()));
        }
        let chain_id: u64 = chain_id_text
            .parse()
            .map_err(|_| AgentRegistryError::InvalidChainId(chain_id_text.to_owned()))?;
        if chain_id > MAX_CHAIN_ID {
            return Err(AgentRegistryError::InvalidChainId(chain_id_text.to_owned()));
        }
        let Some(hex) = address_text.strip_prefix("0x") else {
            return Err(AgentRegistryError::InvalidAddress(address_text.to_owned()));
        };
        let shape_ok = hex.len() == 40 && hex.bytes().all(|b| b.is_ascii_hexdigit());
        if !shape_ok {
            return Err(AgentRegistryError::InvalidAddress(address_text.to_owned()));
        }
        let is_lower = hex == hex.to_ascii_lowercase();
        let is_upper = hex == hex.to_ascii_uppercase();
        let address = if !is_lower && !is_upper {
            Address::parse_checksummed(address_text, None)
                .map_err(|_| AgentRegistryError::BadChecksum(address_text.to_owned()))?
        } else {
            address_text
                .parse::<Address>()
                .map_err(|_| AgentRegistryError::InvalidAddress(address_text.to_owned()))?
        };
        Ok(Self { chain_id, address })
    }
}

impl FromStr for AgentRegistry {
    type Err = AgentRegistryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Formats with an EIP-55 checksummed address.
impl fmt::Display for AgentRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "eip155:{}:{}",
            self.chain_id,
            self.address.to_checksum(None)
        )
    }
}
