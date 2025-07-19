//! Error types and state definitions for robustness patterns

use serde::{Deserialize, Serialize};

/// Retryable error types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RetryableError {
    NetworkTimeout,
    ServiceUnavailable,
    InternalServerError,
    BadGateway,
    GatewayTimeout,
    TooManyRequests,
    ConnectionError,
    Custom(String),
}

/// Circuit breaker state
#[derive(Debug, Clone)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
} 