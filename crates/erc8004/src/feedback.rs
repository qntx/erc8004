//! Off-chain feedback file — lenient, lossless; `value`/`valueDecimals` must
//! form a valid [`FeedbackValue`].

use serde_json::{Map, Value};

use crate::feedback_value::{FeedbackValue, FeedbackValueError};
use crate::json::{
    JsonDocumentError, as_object, extras, optional_object, optional_string, required_decimals,
    required_int, required_string, required_uint, signed_to_json, uint_to_json,
};

/// A parsed off-chain feedback file.
#[derive(Debug, Clone, PartialEq)]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "serde_json::Value fields preclude Eq"
)]
pub struct FeedbackFile {
    /// `eip155:{chainId}:{address}` kept raw.
    pub agent_registry: String,
    /// The agent id (uint256 semantics; JSON number ≤ 2^53-1, string above).
    pub agent_id: U256,
    /// Client address (`eip155:1:0x…` in spec files) kept raw.
    pub client_address: String,
    /// Creation timestamp kept raw.
    pub created_at: String,
    /// The `(value, decimals)` pair — validated at parse time.
    pub value: FeedbackValue,
    /// Optional primary tag.
    pub tag1: Option<String>,
    /// Optional secondary tag.
    pub tag2: Option<String>,
    /// Optional endpoint.
    pub endpoint: Option<String>,
    /// Optional MCP metadata object.
    pub mcp: Option<Map<String, Value>>,
    /// Optional A2A metadata object.
    pub a2a: Option<Map<String, Value>>,
    /// Optional OASF metadata object.
    pub oasf: Option<Map<String, Value>>,
    /// Optional proof-of-payment object.
    pub proof_of_payment: Option<Map<String, Value>>,
    /// Unknown keys preserved verbatim.
    pub extra: Map<String, Value>,
}

use alloy::primitives::U256;

const FILE_KEYS: &[&str] = &[
    "agentRegistry",
    "agentId",
    "clientAddress",
    "createdAt",
    "value",
    "valueDecimals",
    "tag1",
    "tag2",
    "endpoint",
    "mcp",
    "a2a",
    "oasf",
    "proofOfPayment",
];

impl FeedbackFile {
    /// Parses a feedback file leniently from a JSON value; `value`/`valueDecimals`
    /// must together form a valid [`FeedbackValue`].
    ///
    /// # Errors
    ///
    /// Returns [`JsonDocumentError`] on type violations or an invalid value pair.
    pub fn from_value(value: &Value) -> Result<Self, JsonDocumentError> {
        let obj = as_object(value)?;
        let agent_registry = required_string(obj, "", "agentRegistry")?.to_owned();
        let agent_id = required_uint(obj, "", "agentId")?;
        let client_address = required_string(obj, "", "clientAddress")?.to_owned();
        let created_at = required_string(obj, "", "createdAt")?.to_owned();
        let raw_value = required_int(obj, "", "value")?;
        let value_decimals = required_decimals(obj, "", "valueDecimals")?;
        let pair = FeedbackValue::new(raw_value, value_decimals).map_err(|e| {
            let field = match e {
                FeedbackValueError::TooManyDecimals => "valueDecimals",
                _ => "value",
            };
            JsonDocumentError::invalid_field(field, e.to_string())
        })?;
        Ok(Self {
            agent_registry,
            agent_id,
            client_address,
            created_at,
            value: pair,
            tag1: optional_string(obj, "", "tag1")?.map(Into::into),
            tag2: optional_string(obj, "", "tag2")?.map(Into::into),
            endpoint: optional_string(obj, "", "endpoint")?.map(Into::into),
            mcp: optional_object(obj, "", "mcp")?.cloned(),
            a2a: optional_object(obj, "", "a2a")?.cloned(),
            oasf: optional_object(obj, "", "oasf")?.cloned(),
            proof_of_payment: optional_object(obj, "", "proofOfPayment")?.cloned(),
            extra: extras(obj, FILE_KEYS),
        })
    }

    /// Parses a feedback file from raw JSON text.
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

    /// Serializes back to a JSON value; `agentId`/`value` are numbers within
    /// ±(2^53-1), decimal strings above.
    #[must_use]
    pub fn to_value(&self) -> Value {
        let mut out = Map::new();
        out.insert("agentRegistry".into(), self.agent_registry.clone().into());
        out.insert("agentId".into(), uint_to_json(self.agent_id));
        out.insert("clientAddress".into(), self.client_address.clone().into());
        out.insert("createdAt".into(), self.created_at.clone().into());
        out.insert("value".into(), signed_to_json(self.value.value()));
        out.insert("valueDecimals".into(), self.value.decimals().into());
        if let Some(v) = &self.tag1 {
            out.insert("tag1".into(), v.clone().into());
        }
        if let Some(v) = &self.tag2 {
            out.insert("tag2".into(), v.clone().into());
        }
        if let Some(v) = &self.endpoint {
            out.insert("endpoint".into(), v.clone().into());
        }
        if let Some(v) = &self.mcp {
            out.insert("mcp".into(), Value::Object(v.clone()));
        }
        if let Some(v) = &self.a2a {
            out.insert("a2a".into(), Value::Object(v.clone()));
        }
        if let Some(v) = &self.oasf {
            out.insert("oasf".into(), Value::Object(v.clone()));
        }
        if let Some(v) = &self.proof_of_payment {
            out.insert("proofOfPayment".into(), Value::Object(v.clone()));
        }
        for (k, v) in &self.extra {
            out.insert(k.clone(), v.clone());
        }
        Value::Object(out)
    }
}
