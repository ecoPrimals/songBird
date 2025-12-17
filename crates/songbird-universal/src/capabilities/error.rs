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
