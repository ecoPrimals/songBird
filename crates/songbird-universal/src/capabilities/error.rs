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
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            CapabilityError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            CapabilityError::PrimalNotFound(name) => write!(f, "Primal not found: {name}"),
            CapabilityError::CapabilityUnavailable(cap) => {
                write!(f, "Capability unavailable: {cap}")
            }
        }
    }
}

impl std::error::Error for CapabilityError {}
