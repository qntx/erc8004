//! Core domain types for the ERC-8004 SDK.
//!
//! These types model the off-chain JSON payloads defined by the ERC-8004
//! specification (agent registration files, feedback files, etc.) and provide
//! ergonomic wrappers around on-chain primitives.

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a `u64` from either a JSON number or a JSON string.
fn deserialize_u64_or_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNum {
        Num(u64),
        Str(String),
    }
    match StringOrNum::deserialize(deserializer)? {
        StringOrNum::Num(n) => Ok(n),
        StringOrNum::Str(s) => s.parse::<u64>().map_err(serde::de::Error::custom),
    }
}

/// The top-level agent registration file resolved by `agentURI`.
///
/// See: <https://eips.ethereum.org/EIPS/eip-8004#agent-uri-and-agent-registration-file>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationFile {
    /// Must be `"https://eips.ethereum.org/EIPS/eip-8004#registration-v1"`.
    #[serde(rename = "type")]
    pub type_field: String,

    /// Human-readable agent name.
    pub name: String,

    /// Natural-language description of the agent.
    pub description: String,

    /// URL to the agent's image / avatar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// List of service endpoints the agent advertises.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<ServiceEndpoint>,

    /// Whether the agent supports x402 payments.
    #[serde(default, rename = "x402Support")]
    pub x402_support: bool,

    /// Whether the agent is currently active.
    #[serde(default = "default_true")]
    pub active: bool,

    /// On-chain registrations of this agent across chains.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registrations: Vec<Registration>,

    /// Trust models the agent supports.
    #[serde(
        default,
        rename = "supportedTrust",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub supported_trust: Vec<String>,
}

const fn default_true() -> bool {
    true
}

/// A service endpoint advertised by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Protocol name (e.g. `"A2A"`, `"MCP"`, `"OASF"`, `"ENS"`, `"web"`).
    pub name: String,

    /// The endpoint URL or identifier.
    pub endpoint: String,

    /// Protocol version (RECOMMENDED).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Optional list of skills (for OASF endpoints).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,

    /// Optional list of domains (for OASF endpoints).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<String>>,
}

/// An on-chain registration entry referencing a specific chain deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    /// The ERC-721 token ID on the target chain.
    #[serde(rename = "agentId", deserialize_with = "deserialize_u64_or_string")]
    pub agent_id: u64,

    /// A colon-separated string `{namespace}:{chainId}:{identityRegistry}`.
    #[serde(rename = "agentRegistry")]
    pub agent_registry: String,
}

/// A single feedback entry as returned by `readFeedback`.
#[derive(Debug, Clone)]
pub struct Feedback {
    /// Signed feedback value (e.g. score).
    pub value: i128,

    /// Number of decimal places for `value`.
    pub value_decimals: u8,

    /// Primary categorization tag (e.g. `"a2a.task"`).
    pub tag1: String,

    /// Secondary categorization tag.
    pub tag2: String,

    /// Whether this feedback has been revoked.
    pub is_revoked: bool,
}

/// Aggregated reputation summary as returned by `getSummary`.
#[derive(Debug, Clone, Copy)]
pub struct ReputationSummary {
    /// Total number of (non-revoked) feedback entries.
    pub count: u64,

    /// Aggregated value across all matching feedback.
    pub summary_value: i128,

    /// Decimal places for `summary_value`.
    pub summary_value_decimals: u8,
}

/// The status of a validation request as returned by `getValidationStatus`.
#[derive(Debug, Clone)]
pub struct ValidationStatus {
    /// Address of the validator contract / EOA.
    pub validator_address: alloy::primitives::Address,

    /// The agent being validated.
    pub agent_id: alloy::primitives::U256,

    /// Response value (0-100). 0 = failed, 100 = passed.
    pub response: u8,

    /// Keccak-256 hash of the response payload.
    pub response_hash: alloy::primitives::FixedBytes<32>,

    /// Optional categorization tag.
    pub tag: String,

    /// Block timestamp of the last update.
    pub last_update: alloy::primitives::U256,
}

/// Aggregated validation summary as returned by `getSummary`.
#[derive(Debug, Clone, Copy)]
pub struct ValidationSummary {
    /// Number of validation responses.
    pub count: u64,

    /// Average response value (0-100).
    pub avg_response: u8,
}

impl RegistrationFile {
    /// Create a new registration file with the standard type field.
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            type_field: "https://eips.ethereum.org/EIPS/eip-8004#registration-v1".to_owned(),
            name: name.into(),
            description: description.into(),
            image: None,
            services: Vec::new(),
            x402_support: false,
            active: true,
            registrations: Vec::new(),
            supported_trust: Vec::new(),
        }
    }

    /// Serialize this registration file to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a registration file from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
