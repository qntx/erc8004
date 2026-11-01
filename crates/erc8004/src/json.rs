//! Internal `serde_json::Value` extraction helpers shared by the registration
//! and feedback files, plus the shared document error.

use thiserror::Error;

use alloy::primitives::U256;
use serde_json::{Map, Value};

/// Lenient JSON-document (registration file, feedback file) failure.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum JsonDocumentError {
    /// The document is not valid JSON text (slice parsers only).
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
    /// The document is not a JSON object.
    #[error("document must be a JSON object")]
    NotAnObject,
    /// A field has the wrong JSON type or is missing.
    #[error("invalid field {field}: {message}")]
    InvalidField {
        /// Field path like `services[0].endpoint`.
        field: String,
        /// Human-readable detail.
        message: String,
    },
}

impl JsonDocumentError {
    /// `InvalidField` constructor.
    #[must_use]
    pub fn invalid_field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidField {
            field: field.into(),
            message: message.into(),
        }
    }

    /// The shared error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidJson(_) => "InvalidJson",
            Self::NotAnObject => "NotAnObject",
            Self::InvalidField { .. } => "InvalidField",
        }
    }

    /// The failing field path, when set.
    #[must_use]
    pub const fn field(&self) -> Option<&str> {
        match self {
            Self::InvalidField { field, .. } => Some(field.as_str()),
            _ => None,
        }
    }
}

fn invalid(field: &str, what: &str) -> JsonDocumentError {
    JsonDocumentError::invalid_field(field, format!("must be {what}"))
}

/// Field path for `key` under `parent` (`""` → `key`; `"services[0]"` + `"name"` → `services[0].name`).
pub(crate) fn field_path(parent: &str, key: &str) -> String {
    if parent.is_empty() {
        key.to_owned()
    } else {
        format!("{parent}.{key}")
    }
}

/// Errors with `NotAnObject` when the document is not a JSON object.
pub(crate) fn as_object(value: &Value) -> Result<&Map<String, Value>, JsonDocumentError> {
    value.as_object().ok_or(JsonDocumentError::NotAnObject)
}

/// Required `obj[key]` as `&str`.
pub(crate) fn required_string<'a>(
    obj: &'a Map<String, Value>,
    parent: &str,
    key: &str,
) -> Result<&'a str, JsonDocumentError> {
    let field = field_path(parent, key);
    obj.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(&field, "be present and a string"))
}

/// Optional `obj[key]` as `&str`.
pub(crate) fn optional_string<'a>(
    obj: &'a Map<String, Value>,
    parent: &str,
    key: &str,
) -> Result<Option<&'a str>, JsonDocumentError> {
    let field = field_path(parent, key);
    obj.get(key)
        .map(|v| v.as_str().ok_or_else(|| invalid(&field, "be a string")))
        .transpose()
}

/// Optional `obj[key]` as `bool`.
pub(crate) fn optional_bool(
    obj: &Map<String, Value>,
    parent: &str,
    key: &str,
) -> Result<Option<bool>, JsonDocumentError> {
    let field = field_path(parent, key);
    obj.get(key)
        .map(|v| v.as_bool().ok_or_else(|| invalid(&field, "be a boolean")))
        .transpose()
}

/// Optional `obj[key]` as a JSON array of strings.
pub(crate) fn optional_string_array<'a>(
    obj: &'a Map<String, Value>,
    parent: &str,
    key: &str,
) -> Result<Option<Vec<&'a str>>, JsonDocumentError> {
    let field = field_path(parent, key);
    let Some(value) = obj.get(key) else {
        return Ok(None);
    };
    let arr = value
        .as_array()
        .ok_or_else(|| invalid(&field, "be an array of strings"))?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        out.push(
            item.as_str()
                .ok_or_else(|| invalid(&field, "be an array of strings"))?,
        );
    }
    Ok(Some(out))
}

/// Optional `obj[key]` as a JSON object; `null` and arrays are not objects.
pub(crate) fn optional_object<'a>(
    obj: &'a Map<String, Value>,
    parent: &str,
    key: &str,
) -> Result<Option<&'a Map<String, Value>>, JsonDocumentError> {
    let field = field_path(parent, key);
    obj.get(key)
        .map(|v| v.as_object().ok_or_else(|| invalid(&field, "be an object")))
        .transpose()
}

const MAX_SAFE: u64 = 9_007_199_254_740_991; // 2^53 - 1
const MAX_SAFE_F64: f64 = 9_007_199_254_740_991.0;

fn is_canonical_decimal(s: &str) -> bool {
    s == "0" || (!s.is_empty() && !s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit()))
}

fn decimal_to_u256(s: &str) -> Option<U256> {
    U256::from_str_radix(s, 10).ok()
}

/// Required `obj[key]` as uint: JSON number integral in `0..=2^53-1` (any
/// spelling) or canonical decimal string ≤ uint256 max.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "integral f64 range-checked"
)]
pub(crate) fn required_uint(
    obj: &Map<String, Value>,
    parent: &str,
    key: &str,
) -> Result<U256, JsonDocumentError> {
    let field = field_path(parent, key);
    let Some(value) = obj.get(key) else {
        return Err(invalid(&field, "be present"));
    };
    let parsed = value
        .as_u64()
        .filter(|n| *n <= MAX_SAFE)
        .map(U256::from)
        .or_else(|| {
            value
                .as_f64()
                .filter(|f| {
                    value.is_number() && f.fract() == 0.0 && (0.0..=MAX_SAFE_F64).contains(f)
                })
                .map(|f| U256::from(f as u64))
        })
        .or_else(|| {
            value
                .as_str()
                .filter(|s| is_canonical_decimal(s))
                .and_then(decimal_to_u256)
        });
    parsed.ok_or_else(|| {
        invalid(
            &field,
            "be a JSON integer in 0..=2^53-1 or a canonical decimal string",
        )
    })
}

/// Required `obj[key]` as signed int: JSON number integral within ±(2^53-1) or
/// canonical signed decimal string.
#[allow(
    clippy::cast_possible_truncation,
    reason = "integral f64 range-checked"
)]
pub(crate) fn required_int(
    obj: &Map<String, Value>,
    parent: &str,
    key: &str,
) -> Result<i128, JsonDocumentError> {
    let field = field_path(parent, key);
    let Some(value) = obj.get(key) else {
        return Err(invalid(&field, "be present"));
    };
    let parsed: Option<i128> = value
        .as_f64()
        .filter(|f| {
            value.is_number() && f.fract() == 0.0 && (-MAX_SAFE_F64..=MAX_SAFE_F64).contains(f)
        })
        .map(|f| f as i128)
        .or_else(|| {
            value.as_str().and_then(|s| {
                let body = s.strip_prefix('-').unwrap_or(s);
                (is_canonical_decimal(body) && !body.is_empty()).then(|| s.parse::<i128>().ok())?
            })
        });
    parsed.ok_or_else(|| {
        invalid(
            &field,
            "be a JSON integer within ±(2^53-1) or a canonical signed decimal string",
        )
    })
}

/// Required `obj[key]` as a JSON integer in `0..=18` — for `valueDecimals`.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "integral f64 range-checked"
)]
pub(crate) fn required_decimals(
    obj: &Map<String, Value>,
    parent: &str,
    key: &str,
) -> Result<u8, JsonDocumentError> {
    let field = field_path(parent, key);
    let Some(value) = obj.get(key) else {
        return Err(invalid(&field, "be present"));
    };
    value
        .as_f64()
        .filter(|f| value.is_number() && f.fract() == 0.0 && *f >= 0.0 && *f <= 18.0)
        .map(|f| f as u8)
        .ok_or_else(|| invalid(&field, "be a JSON integer in 0..=18"))
}

/// Bigints serialize as JSON numbers up to 2^53-1, decimal strings above.
pub(crate) fn uint_to_json(value: U256) -> Value {
    if value <= U256::from(MAX_SAFE) {
        Value::from(value.to::<u64>())
    } else {
        Value::from(value.to_string())
    }
}

/// Signed bigints serialize as JSON numbers within ±(2^53-1), strings above.
#[allow(clippy::cast_possible_truncation, reason = "range-checked i128")]
pub(crate) fn signed_to_json(value: i128) -> Value {
    if value >= -i128::from(MAX_SAFE) && value <= i128::from(MAX_SAFE) {
        Value::from(value as i64)
    } else {
        Value::from(value.to_string())
    }
}

/// Keys of `obj` not in `known`, preserving their values.
pub(crate) fn extras(obj: &Map<String, Value>, known: &[&str]) -> Map<String, Value> {
    let mut out = Map::new();
    for (key, value) in obj {
        if !known.contains(&key.as_str()) {
            out.insert(key.clone(), value.clone());
        }
    }
    out
}
