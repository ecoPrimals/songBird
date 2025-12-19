//! Error Recovery & Resilience
//!
//! Implements:
//! - Retry policies with exponential backoff
//! - Circuit breakers
//! - Graceful degradation
//! - Partial success handling
//!
//! Modern Rust, no unsafe code, production-ready patterns.

use serde::{Deserialize, Serialize};

mod circuit_breaker;
mod degradation;
mod partial_success;
mod retry;

pub use circuit_breaker::*;
pub use degradation::*;
pub use partial_success::*;
pub use retry::*;

/// Error classification for recovery decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorClass {
    /// Transient error (retry likely to succeed)
    Transient,

    /// Permanent error (retry will not help)
    Permanent,

    /// Rate limit error (need backoff)
    RateLimit,

    /// Timeout error
    Timeout,

    /// Resource exhaustion
    ResourceExhausted,
}

/// Classify an error for recovery strategy
pub fn classify_error(error: &anyhow::Error) -> ErrorClass {
    let error_string = format!("{:?}", error);

    // Simple classification based on error messages
    // In production, would use structured error types
    if error_string.contains("timeout") || error_string.contains("Timeout") {
        ErrorClass::Timeout
    } else if error_string.contains("rate limit") || error_string.contains("429") {
        ErrorClass::RateLimit
    } else if error_string.contains("not found") || error_string.contains("invalid") {
        ErrorClass::Permanent
    } else if error_string.contains("exhausted") || error_string.contains("quota") {
        ErrorClass::ResourceExhausted
    } else {
        // Default to transient
        ErrorClass::Transient
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let timeout_err = anyhow::anyhow!("Connection timeout");
        assert_eq!(classify_error(&timeout_err), ErrorClass::Timeout);

        let not_found_err = anyhow::anyhow!("Resource not found");
        assert_eq!(classify_error(&not_found_err), ErrorClass::Permanent);

        let generic_err = anyhow::anyhow!("Unknown error");
        assert_eq!(classify_error(&generic_err), ErrorClass::Transient);
    }
}
