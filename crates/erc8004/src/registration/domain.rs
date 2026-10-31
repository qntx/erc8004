//! Endpoint-domain verification: deterministic string rules, no URL library.

use alloy::primitives::U256;

use crate::agent_registry::AgentRegistry;
use crate::registration::file::RegistrationFile;

const WELL_KNOWN_PATH: &str = "/.well-known/agent-registration.json";

/// `https://{authority}/.well-known/agent-registration.json` — authority lowercased,
/// trailing `:443` stripped; `None` when the endpoint is not usable.
#[must_use]
pub fn well_known_registration_url(endpoint: &str) -> Option<String> {
    let rest = endpoint.strip_prefix("https://")?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let mut authority = rest[..end].to_ascii_lowercase();
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    if let Some(stripped) = authority.strip_suffix(":443") {
        authority = stripped.to_owned();
    }
    Some(format!("https://{authority}{WELL_KNOWN_PATH}"))
}

impl RegistrationFile {
    /// True iff some registration's `agentRegistry` parses and equals
    /// `{chain_id, address}` and `agentId` equals. Unparseable entries skipped.
    #[must_use]
    pub fn has_registration(&self, agent_registry: &AgentRegistry, agent_id: U256) -> bool {
        let Some(registrations) = &self.registrations else {
            return false;
        };
        registrations.iter().any(|registration| {
            registration.agent_id == agent_id
                && AgentRegistry::parse(&registration.agent_registry)
                    .is_ok_and(|parsed| parsed == *agent_registry)
        })
    }
}
