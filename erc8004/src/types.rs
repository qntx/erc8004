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

/// Input parameters for submitting feedback via
/// [`Reputation::give_feedback`](crate::reputation::Reputation::give_feedback).
#[derive(Debug, Clone)]
pub struct FeedbackInput {
    /// The target agent's on-chain ID.
    pub agent_id: alloy::primitives::U256,

    /// Signed feedback value (e.g. a score).
    pub value: i128,

    /// Number of decimal places for `value`.
    pub value_decimals: u8,

    /// Primary categorization tag (e.g. `"a2a.task"`).
    pub tag1: String,

    /// Secondary categorization tag.
    pub tag2: String,

    /// The endpoint this feedback relates to.
    pub endpoint: String,

    /// URI pointing to off-chain feedback details.
    pub feedback_uri: String,

    /// Keccak-256 hash of the feedback URI content.
    pub feedback_hash: alloy::primitives::FixedBytes<32>,
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

impl ServiceEndpoint {
    /// Create a new service endpoint with the given protocol name and URL.
    #[must_use]
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            endpoint: endpoint.into(),
            version: None,
            skills: None,
            domains: None,
        }
    }

    /// Set the protocol version.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the skills list (for OASF endpoints).
    #[must_use]
    pub fn with_skills(mut self, skills: Vec<String>) -> Self {
        self.skills = Some(skills);
        self
    }

    /// Set the domains list (for OASF endpoints).
    #[must_use]
    pub fn with_domains(mut self, domains: Vec<String>) -> Self {
        self.domains = Some(domains);
        self
    }
}

impl FeedbackInput {
    /// Create a new feedback input builder with the required fields.
    #[must_use]
    pub const fn new(agent_id: alloy::primitives::U256, value: i128, value_decimals: u8) -> Self {
        Self {
            agent_id,
            value,
            value_decimals,
            tag1: String::new(),
            tag2: String::new(),
            endpoint: String::new(),
            feedback_uri: String::new(),
            feedback_hash: alloy::primitives::FixedBytes::ZERO,
        }
    }

    /// Set the primary categorization tag.
    #[must_use]
    pub fn with_tag1(mut self, tag1: impl Into<String>) -> Self {
        self.tag1 = tag1.into();
        self
    }

    /// Set the secondary categorization tag.
    #[must_use]
    pub fn with_tag2(mut self, tag2: impl Into<String>) -> Self {
        self.tag2 = tag2.into();
        self
    }

    /// Set the endpoint this feedback relates to.
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Set the feedback URI and its content hash.
    #[must_use]
    pub fn with_feedback_uri(
        mut self,
        uri: impl Into<String>,
        hash: alloy::primitives::FixedBytes<32>,
    ) -> Self {
        self.feedback_uri = uri.into();
        self.feedback_hash = hash;
        self
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_file_new_sets_type_field() {
        let reg = RegistrationFile::new("Bot", "A test bot");
        assert_eq!(
            reg.type_field,
            "https://eips.ethereum.org/EIPS/eip-8004#registration-v1"
        );
        assert_eq!(reg.name, "Bot");
        assert_eq!(reg.description, "A test bot");
        assert!(reg.active, "new registration should be active by default");
        assert!(reg.services.is_empty());
        assert!(reg.registrations.is_empty());
    }

    #[test]
    fn test_registration_file_json_round_trip() {
        let mut reg = RegistrationFile::new("WeatherBot", "Provides forecasts");
        reg.services
            .push(ServiceEndpoint::new("A2A", "https://example.com/a2a").with_version("0.2"));
        reg.x402_support = true;

        let json = reg.to_json().expect("serialization should succeed");
        let parsed = RegistrationFile::from_json(&json).expect("deserialization should succeed");

        assert_eq!(parsed.name, "WeatherBot");
        assert_eq!(parsed.services.len(), 1);
        let svc = parsed.services.first().expect("should have one service");
        assert_eq!(svc.name, "A2A");
        assert_eq!(svc.version.as_deref(), Some("0.2"));
        assert!(parsed.x402_support);
    }

    #[test]
    fn test_registration_file_deserialize_minimal() {
        let json = r#"{
            "type": "https://eips.ethereum.org/EIPS/eip-8004#registration-v1",
            "name": "Minimal",
            "description": "Bare minimum"
        }"#;
        let reg = RegistrationFile::from_json(json).expect("should parse minimal JSON");
        assert_eq!(reg.name, "Minimal");
        assert!(reg.active, "active should default to true");
        assert!(!reg.x402_support, "x402_support should default to false");
    }

    #[test]
    fn test_registration_deserialize_agent_id_as_string() {
        let json = r#"{"agentId": "42", "agentRegistry": "eip155:1:0xABC"}"#;
        let reg: Registration = serde_json::from_str(json).expect("should parse string agentId");
        assert_eq!(reg.agent_id, 42);
    }

    #[test]
    fn test_registration_deserialize_agent_id_as_number() {
        let json = r#"{"agentId": 42, "agentRegistry": "eip155:1:0xABC"}"#;
        let reg: Registration = serde_json::from_str(json).expect("should parse numeric agentId");
        assert_eq!(reg.agent_id, 42);
    }

    #[test]
    fn test_service_endpoint_builder() {
        let ep = ServiceEndpoint::new("MCP", "https://example.com/mcp")
            .with_version("2025-03-26")
            .with_skills(vec!["weather".to_owned()])
            .with_domains(vec!["forecast".to_owned()]);

        assert_eq!(ep.name, "MCP");
        assert_eq!(ep.endpoint, "https://example.com/mcp");
        assert_eq!(ep.version.as_deref(), Some("2025-03-26"));
        assert_eq!(ep.skills.as_ref().map(Vec::len), Some(1));
        assert_eq!(ep.domains.as_ref().map(Vec::len), Some(1));
    }

    #[test]
    fn test_feedback_input_builder() {
        let input = FeedbackInput::new(alloy::primitives::U256::from(1), 500, 2)
            .with_tag1("a2a.task")
            .with_tag2("quality")
            .with_endpoint("https://example.com");

        assert_eq!(input.agent_id, alloy::primitives::U256::from(1));
        assert_eq!(input.value, 500);
        assert_eq!(input.value_decimals, 2);
        assert_eq!(input.tag1, "a2a.task");
        assert_eq!(input.tag2, "quality");
        assert_eq!(input.endpoint, "https://example.com");
    }

    #[test]
    fn test_summary_types_are_copy_send_sync() {
        fn assert_traits<T: Copy + Send + Sync>() {}
        assert_traits::<ReputationSummary>();
        assert_traits::<ValidationSummary>();
    }
}
