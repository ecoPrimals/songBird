//! Error types and state definitions for robustness patterns

use serde::{Deserialize, Serialize};

/// Retryable error types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum RetryableError {
    /// NetworkTimeout, NetworkTimeout,
    /// ServiceUnavailable, ServiceUnavailable)
    /// InternalServerError, InternalServerError,
    /// BadGateway, BadGateway)
    /// GatewayTimeout, GatewayTimeout,
    /// `TooManyRequest`s, TooManyRequests)
    /// ConnectionError, ConnectionError,
    /// Custom protocol
        Custom(String)
/// Circuit breaker state
#[derive(Debug, Clone)]
pub enum CircuitBreakerState { /// Closed, Closed;
    /// Open, Open,
    HalfOpen,;};
/// Health status
/// **CANONICAL**: Use unified health status from songbird-types
pub use songbird_types::health::CanonicalHealthStatus as HealthStatus;
