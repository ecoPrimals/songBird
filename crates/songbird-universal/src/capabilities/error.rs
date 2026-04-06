// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability error types

/// Errors that can occur during capability operations
#[derive(Debug)]
pub enum CapabilityError {
    /// Network communication error
    NetworkError(String),
    /// Invalid capability format
    ParseError(String),
    /// Primal not found
    PrimalNotFound(String),
    /// Capability not available
    CapabilityUnavailable(String),
    /// No providers found for requested capability
    NoProvidersFound {
        /// Capability type that was requested
        capability_type: String,
    },
    /// Feature not yet implemented
    NotImplemented(String),
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::PrimalNotFound(name) => write!(f, "Primal not found: {name}"),
            Self::CapabilityUnavailable(cap) => {
                write!(f, "Capability unavailable: {cap}")
            }
            Self::NoProvidersFound {
                capability_type,
            } => {
                write!(f, "No providers found for capability: {capability_type}")
            }
            Self::NotImplemented(feature) => {
                write!(f, "Not yet implemented: {feature}")
            }
        }
    }
}

impl std::error::Error for CapabilityError {}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::CapabilityError;

    #[test]
    fn network_error_display() {
        let e = CapabilityError::NetworkError("reset".to_string());
        assert!(e.to_string().contains("reset"));
    }

    #[test]
    fn parse_error_display() {
        let e = CapabilityError::ParseError("x".to_string());
        assert!(e.to_string().contains("Parse"));
    }

    #[test]
    fn primal_not_found_display() {
        let e = CapabilityError::PrimalNotFound("p".to_string());
        assert!(e.to_string().contains('p'));
    }

    #[test]
    fn capability_unavailable_display() {
        let e = CapabilityError::CapabilityUnavailable("cap".to_string());
        assert!(e.to_string().contains("cap"));
    }

    #[test]
    fn no_providers_display() {
        let e = CapabilityError::NoProvidersFound {
            capability_type: "t".to_string(),
        };
        assert!(e.to_string().contains('t'));
    }

    #[test]
    fn not_implemented_display() {
        let e = CapabilityError::NotImplemented("x".to_string());
        assert!(e.to_string().contains('x'));
    }

    #[test]
    fn error_source_is_none() {
        let e = CapabilityError::NetworkError("n".to_string());
        assert!(std::error::Error::source(&e).is_none());
    }
}
