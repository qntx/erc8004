//! Lenient, lossless registration-file model: spec fields typed when present,
//! unknown keys preserved in `extra`, no defaults invented.

use alloy::primitives::U256;
use serde_json::{Map, Value};

use crate::agent_registry::{AgentRegistry, AgentRegistryError};
use crate::json::{
    JsonDocumentError, as_object, extras, optional_bool, optional_string, optional_string_array,
    required_string, required_uint, uint_to_json,
};

/// `type` for the v1 registration file.
pub const REGISTRATION_TYPE_V1: &str = "https://eips.ethereum.org/EIPS/eip-8004#registration-v1";

/// One `services[]` entry; `extra` preserves non-spec keys.
#[derive(Debug, Clone, PartialEq)]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "serde_json::Value fields preclude Eq"
)]
pub struct Service {
    /// Required service name.
    pub name: String,
    /// Required endpoint URL.
    pub endpoint: String,
    /// Optional version.
    pub version: Option<String>,
    /// Optional skill tags.
    pub skills: Option<Vec<String>>,
    /// Optional domains.
    pub domains: Option<Vec<String>>,
    /// Non-spec keys preserved verbatim.
    pub extra: Map<String, Value>,
}

/// One `registrations[]` entry; `agent_registry` stays a raw string.
#[derive(Debug, Clone, PartialEq)]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "serde_json::Value fields preclude Eq"
)]
pub struct Registration {
    /// The agent id (uint256; serialized as JSON number ≤ 2^53-1, string above).
    pub agent_id: U256,
    /// `eip155:{chainId}:{address}` kept raw; see [`Registration::agent_registry`].
    pub agent_registry: String,
    /// Non-spec keys preserved verbatim.
    pub extra: Map<String, Value>,
}

impl Registration {
    /// Parses the raw `agent_registry` string.
    ///
    /// # Errors
    ///
    /// Returns [`AgentRegistryError`] when the stored string is malformed.
    pub fn agent_registry(&self) -> Result<AgentRegistry, AgentRegistryError> {
        AgentRegistry::parse(&self.agent_registry)
    }
}

/// Lenient registration file; spec fields typed when present.
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "serde_json::Value fields preclude Eq"
)]
pub struct RegistrationFile {
    /// Optional `type` (spec: [`REGISTRATION_TYPE_V1`]).
    pub r#type: Option<String>,
    /// Optional name.
    pub name: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Optional image URL.
    pub image: Option<String>,
    /// Optional services.
    pub services: Option<Vec<Service>>,
    /// Optional x402 support flag.
    pub x402_support: Option<bool>,
    /// Optional active flag.
    pub active: Option<bool>,
    /// Optional registrations.
    pub registrations: Option<Vec<Registration>>,
    /// Optional trust models.
    pub supported_trust: Option<Vec<String>>,
    /// Unknown top-level keys preserved verbatim (e.g. real-world misspellings).
    pub extra: Map<String, Value>,
}

const SERVICE_KEYS: &[&str] = &["name", "endpoint", "version", "skills", "domains"];
const REGISTRATION_KEYS: &[&str] = &["agentId", "agentRegistry"];
const FILE_KEYS: &[&str] = &[
    "type",
    "name",
    "description",
    "image",
    "services",
    "x402Support",
    "active",
    "registrations",
    "supportedTrust",
];

fn parse_service(value: &Value, field: &str) -> Result<Service, JsonDocumentError> {
    let obj = value
        .as_object()
        .ok_or_else(|| JsonDocumentError::invalid_field(field, "must be an object"))?;
    Ok(Service {
        name: required_string(obj, field, "name")?.to_owned(),
        endpoint: required_string(obj, field, "endpoint")?.to_owned(),
        version: optional_string(obj, field, "version")?.map(Into::into),
        skills: optional_string_array(obj, field, "skills")?
            .map(|v| v.into_iter().map(Into::into).collect()),
        domains: optional_string_array(obj, field, "domains")?
            .map(|v| v.into_iter().map(Into::into).collect()),
        extra: extras(obj, SERVICE_KEYS),
    })
}

fn parse_registration(value: &Value, field: &str) -> Result<Registration, JsonDocumentError> {
    let obj = value
        .as_object()
        .ok_or_else(|| JsonDocumentError::invalid_field(field, "must be an object"))?;
    Ok(Registration {
        agent_id: required_uint(obj, field, "agentId")?,
        agent_registry: required_string(obj, field, "agentRegistry")?.to_owned(),
        extra: extras(obj, REGISTRATION_KEYS),
    })
}

fn service_to_value(s: &Service) -> Value {
    let mut o = Map::new();
    o.insert("name".into(), s.name.clone().into());
    o.insert("endpoint".into(), s.endpoint.clone().into());
    if let Some(v) = &s.version {
        o.insert("version".into(), v.clone().into());
    }
    if let Some(v) = &s.skills {
        o.insert("skills".into(), v.clone().into());
    }
    if let Some(v) = &s.domains {
        o.insert("domains".into(), v.clone().into());
    }
    for (k, v) in &s.extra {
        o.insert(k.clone(), v.clone());
    }
    Value::Object(o)
}

fn registration_to_value(r: &Registration) -> Value {
    let mut o = Map::new();
    o.insert("agentId".into(), uint_to_json(r.agent_id));
    o.insert("agentRegistry".into(), r.agent_registry.clone().into());
    for (k, v) in &r.extra {
        o.insert(k.clone(), v.clone());
    }
    Value::Object(o)
}

impl RegistrationFile {
    /// Parses a registration file leniently from a JSON value.
    ///
    /// # Errors
    ///
    /// Returns [`JsonDocumentError`] (`NotAnObject`/`InvalidField`) on type violations.
    pub fn from_value(value: &Value) -> Result<Self, JsonDocumentError> {
        let obj = as_object(value)?;
        let mut file = Self {
            r#type: optional_string(obj, "", "type")?.map(Into::into),
            name: optional_string(obj, "", "name")?.map(Into::into),
            description: optional_string(obj, "", "description")?.map(Into::into),
            image: optional_string(obj, "", "image")?.map(Into::into),
            x402_support: optional_bool(obj, "", "x402Support")?,
            active: optional_bool(obj, "", "active")?,
            supported_trust: optional_string_array(obj, "", "supportedTrust")?
                .map(|v| v.into_iter().map(Into::into).collect()),
            extra: extras(obj, FILE_KEYS),
            ..Self::default()
        };
        if let Some(services) = obj.get("services") {
            let arr = services
                .as_array()
                .ok_or_else(|| JsonDocumentError::invalid_field("services", "must be an array"))?;
            let mut parsed = Vec::with_capacity(arr.len());
            for (i, service) in arr.iter().enumerate() {
                parsed.push(parse_service(service, &format!("services[{i}]"))?);
            }
            file.services = Some(parsed);
        }
        if let Some(registrations) = obj.get("registrations") {
            let arr = registrations.as_array().ok_or_else(|| {
                JsonDocumentError::invalid_field("registrations", "must be an array")
            })?;
            let mut parsed = Vec::with_capacity(arr.len());
            for (i, registration) in arr.iter().enumerate() {
                parsed.push(parse_registration(
                    registration,
                    &format!("registrations[{i}]"),
                )?);
            }
            file.registrations = Some(parsed);
        }
        Ok(file)
    }

    /// Parses a registration file from raw JSON text.
    ///
    /// # Errors
    ///
    /// Returns [`JsonDocumentError::InvalidJson`] on syntax errors,
    /// [`JsonDocumentError`] variants on type violations.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, JsonDocumentError> {
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|e| JsonDocumentError::InvalidJson(e.to_string()))?;
        Self::from_value(&value)
    }

    /// Serializes back to a JSON value; `agentId` is a JSON number ≤ 2^53-1,
    /// a decimal string above.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut out = Map::new();
        if let Some(t) = &self.r#type {
            out.insert("type".into(), t.clone().into());
        }
        if let Some(v) = &self.name {
            out.insert("name".into(), v.clone().into());
        }
        if let Some(v) = &self.description {
            out.insert("description".into(), v.clone().into());
        }
        if let Some(v) = &self.image {
            out.insert("image".into(), v.clone().into());
        }
        if let Some(services) = &self.services {
            out.insert(
                "services".into(),
                services
                    .iter()
                    .map(service_to_value)
                    .collect::<Vec<_>>()
                    .into(),
            );
        }
        if let Some(v) = self.x402_support {
            out.insert("x402Support".into(), v.into());
        }
        if let Some(v) = self.active {
            out.insert("active".into(), v.into());
        }
        if let Some(registrations) = &self.registrations {
            out.insert(
                "registrations".into(),
                registrations
                    .iter()
                    .map(registration_to_value)
                    .collect::<Vec<_>>()
                    .into(),
            );
        }
        if let Some(v) = &self.supported_trust {
            out.insert("supportedTrust".into(), v.clone().into());
        }
        for (k, v) in &self.extra {
            out.insert(k.clone(), v.clone());
        }
        Value::Object(out)
    }

    /// Builder: sets `type` to [`REGISTRATION_TYPE_V1`].
    #[must_use]
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            r#type: Some(REGISTRATION_TYPE_V1.into()),
            name: Some(name.into()),
            description: Some(description.into()),
            ..Self::default()
        }
    }
}
