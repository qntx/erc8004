//! `data:` URI decode/encode for inline registration files.

/// `data:` URI decode failure.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DataUriError {
    /// Does not start with `data:` or lacks a payload separator.
    #[error("not a data URI")]
    NotDataUri,
    /// Media type is not `application/json` (case-insensitive).
    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),
    /// No `;base64` marker.
    #[error("data URI is not base64-encoded")]
    NotBase64,
    /// Payload is not standard padded base64.
    #[error("invalid base64 payload")]
    InvalidBase64,
}

impl DataUriError {
    /// The shared error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::NotDataUri => "NotDataUri",
            Self::UnsupportedMediaType(_) => "UnsupportedMediaType",
            Self::NotBase64 => "NotBase64",
            Self::InvalidBase64 => "InvalidBase64",
        }
    }
}

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use thiserror::Error;

const PREFIX: &str = "data:";
const MEDIA_TYPE: &str = "application/json";
const BASE64_FLAG: &str = ";base64";

/// Decodes `data:<mediatype>[;params];base64,<payload>` — media type
/// `application/json` case-insensitive, extra params allowed.
///
/// # Errors
///
/// Returns [`DataUriError`] (`NotDataUri`/`UnsupportedMediaType`/`NotBase64`/`InvalidBase64`).
pub fn decode_data_uri(uri: &str) -> Result<Vec<u8>, DataUriError> {
    if !uri.to_ascii_lowercase().starts_with(PREFIX) {
        return Err(DataUriError::NotDataUri);
    }
    let comma = uri.find(',').ok_or(DataUriError::NotDataUri)?;
    let header = uri[PREFIX.len()..comma].to_ascii_lowercase();
    if !header.ends_with(BASE64_FLAG) {
        return Err(DataUriError::NotBase64);
    }
    let media_type = header[..header.len() - BASE64_FLAG.len()]
        .split(';')
        .next()
        .unwrap_or("");
    if media_type != MEDIA_TYPE {
        return Err(DataUriError::UnsupportedMediaType(media_type.into()));
    }
    STANDARD
        .decode(&uri[comma + 1..])
        .map_err(|_| DataUriError::InvalidBase64)
}

/// Encodes bytes as `data:application/json;base64,<padded>`.
#[must_use]
pub fn encode_data_uri(bytes: &[u8]) -> String {
    format!(
        "{}{}{},{}",
        PREFIX,
        MEDIA_TYPE,
        BASE64_FLAG,
        STANDARD.encode(bytes)
    )
}
