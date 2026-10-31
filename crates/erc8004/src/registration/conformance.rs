//! Spec-conformance checks — a separate pass from lenient parsing; never fails.

use crate::agent_registry::AgentRegistry;
use crate::registration::file::{REGISTRATION_TYPE_V1, RegistrationFile};

/// A spec-conformance issue.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConformanceIssue {
    /// `type` absent.
    MissingType,
    /// `type` present but not `registration-v1`.
    UnknownType,
    /// `name` absent.
    MissingName,
    /// `description` absent.
    MissingDescription,
    /// `registrations` absent or empty.
    MissingRegistrations,
    /// `registrations[i].agentRegistry` does not parse.
    InvalidAgentRegistry {
        /// Registration index.
        index: usize,
    },
}

impl ConformanceIssue {
    /// The shared issue code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::MissingType => "MissingType",
            Self::UnknownType => "UnknownType",
            Self::MissingName => "MissingName",
            Self::MissingDescription => "MissingDescription",
            Self::MissingRegistrations => "MissingRegistrations",
            Self::InvalidAgentRegistry { .. } => "InvalidAgentRegistry",
        }
    }
}

impl RegistrationFile {
    /// Every spec-conformance issue; never fails.
    #[must_use]
    pub fn conformance(&self) -> Vec<ConformanceIssue> {
        let mut issues = Vec::new();
        match &self.r#type {
            None => issues.push(ConformanceIssue::MissingType),
            Some(t) if t != REGISTRATION_TYPE_V1 => issues.push(ConformanceIssue::UnknownType),
            _ => {}
        }
        if self.name.is_none() {
            issues.push(ConformanceIssue::MissingName);
        }
        if self.description.is_none() {
            issues.push(ConformanceIssue::MissingDescription);
        }
        match &self.registrations {
            None => issues.push(ConformanceIssue::MissingRegistrations),
            Some(registrations) if registrations.is_empty() => {
                issues.push(ConformanceIssue::MissingRegistrations);
            }
            Some(registrations) => issues.extend(
                registrations
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| AgentRegistry::parse(&r.agent_registry).is_err())
                    .map(|(index, _)| ConformanceIssue::InvalidAgentRegistry { index }),
            ),
        }
        issues
    }
}
