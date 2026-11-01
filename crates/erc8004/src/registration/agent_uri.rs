//! agentURI classification — pure; fetching lives in `resolve.rs` (`fetch` feature).

use thiserror::Error;

use crate::registration::data_uri::{DataUriError, decode_data_uri};

/// agentURI classification failure: bad data URIs surface as
/// [`AgentUriError::Data`]; a bad gateway is [`AgentUriError::InvalidGateway`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AgentUriError {
    /// IPFS gateway does not end with `/`.
    #[error("invalid IPFS gateway: {0}")]
    InvalidGateway(String),
    /// The data URI failed to decode.
    #[error(transparent)]
    Data(#[from] DataUriError),
}

impl AgentUriError {
    /// The shared error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidGateway(_) => "InvalidGateway",
            Self::Data(e) => e.code(),
        }
    }
}

/// Default IPFS gateway (`https://ipfs.io/ipfs/`).
pub const DEFAULT_IPFS_GATEWAY: &str = "https://ipfs.io/ipfs/";

/// agentURI classification: `""` `NotSet`; `data:` decoded inline; `ipfs://`
/// via a configurable gateway; `https://` fetched; else `UnsupportedScheme`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AgentUri {
    /// Empty string — no URI set.
    NotSet,
    /// Inline `data:` URI payload (must be `application/json` base64).
    Data {
        /// The decoded data-URI payload.
        bytes: Vec<u8>,
    },
    /// `ipfs://<rest>` resolved through a gateway.
    Ipfs {
        /// The gateway URL for the IPFS content.
        url: String,
    },
    /// `https://` URL.
    Https {
        /// The HTTPS URL.
        url: String,
    },
    /// Any other scheme (including `http://` and empty `ipfs://`).
    UnsupportedScheme,
}

fn has_prefix(text: &str, prefix: &str) -> bool {
    text.as_bytes()
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix.as_bytes()))
}

impl AgentUri {
    /// Classifies an agentURI against `ipfs_gateway` (must end with `/` —
    /// pass [`DEFAULT_IPFS_GATEWAY`] for the default).
    ///
    /// # Errors
    ///
    /// [`AgentUriError::Data`] when a `data:` URI fails to decode;
    /// [`AgentUriError::InvalidGateway`] when the gateway lacks a trailing `/`.
    pub fn classify(uri: &str, ipfs_gateway: &str) -> Result<Self, AgentUriError> {
        if uri.is_empty() {
            return Ok(Self::NotSet);
        }
        if has_prefix(uri, "data:") {
            return Ok(Self::Data {
                bytes: decode_data_uri(uri)?,
            });
        }
        if has_prefix(uri, "ipfs://") {
            let rest = &uri["ipfs://".len()..];
            if rest.is_empty() {
                return Ok(Self::UnsupportedScheme);
            }
            let gateway = ipfs_gateway.strip_suffix('/').unwrap_or("");
            if gateway.is_empty() || !ipfs_gateway.ends_with('/') {
                return Err(AgentUriError::InvalidGateway(ipfs_gateway.to_owned()));
            }
            return Ok(Self::Ipfs {
                url: format!("{ipfs_gateway}{rest}"),
            });
        }
        if has_prefix(uri, "https://") {
            return Ok(Self::Https {
                url: uri.to_owned(),
            });
        }
        Ok(Self::UnsupportedScheme)
    }
}
