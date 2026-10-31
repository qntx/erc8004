//! agentURI resolution over `reqwest` — `data:` decodes inline, `ipfs://` via a
//! gateway, `https://` fetched with a running byte cap.

#![allow(
    unreachable_pub,
    reason = "private module; public items reach callers through the crate-root re-export"
)]

use thiserror::Error;

use alloy::primitives::U256;

use crate::JsonDocumentError;
use crate::agent_registry::AgentRegistry;
use crate::registration::agent_uri::{AgentUri, DEFAULT_IPFS_GATEWAY};
use crate::registration::data_uri::DataUriError;
use crate::registration::domain::well_known_registration_url;
use crate::registration::file::RegistrationFile;

/// agentURI resolution failure — codes mirror `ResolveError` in `@qntx/erc8004`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ResolveError {
    /// The URI is empty.
    #[error("agentURI is not set")]
    NotSet,
    /// The scheme is not `data:`/`ipfs:`/`https:`.
    #[error("unsupported agentURI scheme")]
    UnsupportedScheme,
    /// The HTTP response was not 2xx; `0` is the status code.
    #[error("agentURI fetch failed: HTTP {0}")]
    HttpStatus(u16),
    /// The body exceeded `max_bytes` (declared or actual).
    #[error("agentURI body exceeds the size limit")]
    TooLarge,
    /// The body is not valid JSON text.
    #[error("agentURI body is not valid JSON")]
    InvalidJson,
    /// The data URI failed to decode.
    #[error(transparent)]
    DataUri(#[from] DataUriError),
    /// The JSON document failed to parse.
    #[error(transparent)]
    Document(#[from] JsonDocumentError),
    /// The HTTP transport failed.
    #[error("transport error: {0}")]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl ResolveError {
    /// The shared error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::NotSet => "NotSet",
            Self::UnsupportedScheme => "UnsupportedScheme",
            Self::HttpStatus(_) => "HttpStatus",
            Self::TooLarge => "TooLarge",
            Self::InvalidJson => "InvalidJson",
            Self::DataUri(e) => e.code(),
            Self::Document(e) => e.code(),
            Self::Transport(_) => "Transport",
        }
    }
}

/// Construction options for [`Resolver`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ResolverOptions {
    /// IPFS gateway base (must end with `/`).
    pub ipfs_gateway: String,
    /// Response-body cap in bytes.
    pub max_bytes: usize,
    /// Per-request timeout.
    pub timeout: std::time::Duration,
}

impl Default for ResolverOptions {
    fn default() -> Self {
        Self {
            ipfs_gateway: DEFAULT_IPFS_GATEWAY.to_owned(),
            max_bytes: 1024 * 1024,
            timeout: std::time::Duration::from_secs(10),
        }
    }
}

/// agentURI resolver holding a private `reqwest::Client`.
#[derive(Debug, Clone)]
pub struct Resolver {
    client: reqwest::Client,
    options: ResolverOptions,
}

impl Resolver {
    /// Builds a resolver; `options` falls back to the defaults.
    ///
    /// # Errors
    ///
    /// Propagates `reqwest::ClientBuilder` failures as [`ResolveError::Transport`].
    pub fn new(options: ResolverOptions) -> Result<Self, ResolveError> {
        let client = reqwest::Client::builder()
            .timeout(options.timeout)
            .build()
            .map_err(|e| ResolveError::Transport(Box::new(e)))?;
        Ok(Self { client, options })
    }

    async fn fetch(&self, url: &str) -> Result<Vec<u8>, ResolveError> {
        let mut response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ResolveError::Transport(Box::new(e)))?;
        if !response.status().is_success() {
            return Err(ResolveError::HttpStatus(response.status().as_u16()));
        }
        if response
            .content_length()
            .is_some_and(|l| usize::try_from(l).is_ok_and(|n| n > self.options.max_bytes))
        {
            return Err(ResolveError::TooLarge);
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| ResolveError::Transport(Box::new(e)))?
        {
            if body.len() + chunk.len() > self.options.max_bytes {
                return Err(ResolveError::TooLarge);
            }
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }

    /// Resolves an agentURI to a parsed [`RegistrationFile`].
    ///
    /// # Errors
    ///
    /// [`ResolveError`] variants for scheme, HTTP, size, JSON, or document failures;
    /// data-URI and document errors propagate via `#[from]`.
    pub async fn resolve_agent_uri(&self, uri: &str) -> Result<RegistrationFile, ResolveError> {
        match AgentUri::classify(uri, &self.options.ipfs_gateway).map_err(|e| match e {
            crate::registration::agent_uri::AgentUriError::Data(d) => ResolveError::DataUri(d),
            crate::registration::agent_uri::AgentUriError::InvalidGateway(_) => {
                ResolveError::UnsupportedScheme
            }
        })? {
            AgentUri::NotSet => Err(ResolveError::NotSet),
            AgentUri::UnsupportedScheme => Err(ResolveError::UnsupportedScheme),
            AgentUri::Data { bytes } => {
                let text = std::str::from_utf8(&bytes).map_err(|_| ResolveError::InvalidJson)?;
                serde_json::from_str::<serde_json::Value>(text)
                    .map_err(|_| ResolveError::InvalidJson)
                    .and_then(|v| RegistrationFile::from_value(&v).map_err(Into::into))
            }
            AgentUri::Ipfs { url } | AgentUri::Https { url } => {
                let body = self.fetch(&url).await?;
                serde_json::from_slice::<serde_json::Value>(&body)
                    .map_err(|_| ResolveError::InvalidJson)
                    .and_then(|v| RegistrationFile::from_value(&v).map_err(Into::into))
            }
        }
    }

    /// Fetches `{endpoint}/.well-known/agent-registration.json` and verifies it registers
    /// `{agent_registry, agent_id}` — `Ok(false)` on every failure mode.
    ///
    /// # Errors
    ///
    /// Never — transport, parse, and scheme failures all yield `Ok(false)`.
    pub async fn check_endpoint_domain(
        &self,
        endpoint: &str,
        agent_registry: &AgentRegistry,
        agent_id: U256,
    ) -> Result<bool, ResolveError> {
        let Some(url) = well_known_registration_url(endpoint) else {
            return Ok(false);
        };
        let Ok(body) = self.fetch(&url).await else {
            return Ok(false);
        };
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&body) else {
            return Ok(false);
        };
        let Ok(file) = RegistrationFile::from_value(&value) else {
            return Ok(false);
        };
        Ok(file.has_registration(agent_registry, agent_id))
    }
}
