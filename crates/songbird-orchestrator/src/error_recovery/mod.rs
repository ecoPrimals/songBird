// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
#[must_use]
pub fn classify_error(error: &anyhow::Error) -> ErrorClass {
    let error_string = format!("{error:?}");

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
    fn test_error_classification_timeout() {
        let timeout_err = anyhow::anyhow!("Connection timeout");
        assert_eq!(classify_error(&timeout_err), ErrorClass::Timeout);

        let timeout_err2 = anyhow::anyhow!("Request Timeout reached");
        assert_eq!(classify_error(&timeout_err2), ErrorClass::Timeout);
    }

    #[test]
    fn test_error_classification_permanent() {
        let not_found_err = anyhow::anyhow!("Resource not found");
        assert_eq!(classify_error(&not_found_err), ErrorClass::Permanent);

        // Uses lowercase "invalid" to match the classify_error function
        let invalid_err = anyhow::anyhow!("invalid input data");
        assert_eq!(classify_error(&invalid_err), ErrorClass::Permanent);
    }

    #[test]
    fn test_error_classification_rate_limit() {
        let rate_err = anyhow::anyhow!("rate limit exceeded");
        assert_eq!(classify_error(&rate_err), ErrorClass::RateLimit);

        let http_429 = anyhow::anyhow!("HTTP 429 Too Many Requests");
        assert_eq!(classify_error(&http_429), ErrorClass::RateLimit);
    }

    #[test]
    fn test_error_classification_resource_exhausted() {
        let exhausted_err = anyhow::anyhow!("Connection pool exhausted");
        assert_eq!(classify_error(&exhausted_err), ErrorClass::ResourceExhausted);

        let quota_err = anyhow::anyhow!("API quota exceeded");
        assert_eq!(classify_error(&quota_err), ErrorClass::ResourceExhausted);
    }

    #[test]
    fn test_error_classification_transient() {
        let generic_err = anyhow::anyhow!("Unknown error");
        assert_eq!(classify_error(&generic_err), ErrorClass::Transient);

        let connection_err = anyhow::anyhow!("Connection refused");
        assert_eq!(classify_error(&connection_err), ErrorClass::Transient);
    }

    #[test]
    fn test_error_class_equality() {
        assert_eq!(ErrorClass::Transient, ErrorClass::Transient);
        assert_ne!(ErrorClass::Transient, ErrorClass::Permanent);
        assert_ne!(ErrorClass::RateLimit, ErrorClass::Timeout);
    }

    #[test]
    fn test_error_class_clone() {
        let class = ErrorClass::Transient;
        let cloned = class;
        assert_eq!(class, cloned);
    }

    #[test]
    fn test_error_class_debug() {
        let class = ErrorClass::Timeout;
        let debug_str = format!("{class:?}");
        assert!(debug_str.contains("Timeout"));
    }
}
