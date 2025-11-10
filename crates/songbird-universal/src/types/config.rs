//! Configuration and reliability pattern types
//!
//! This module provides configuration types for security, load balancing,
//! retry policies, circuit breakers, health checks, and feature flags.

use super::capability::SecurityLevel;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether security features are enabled
    pub enabled: bool,
    /// Security level to enforce
    pub level: SecurityLevel,
    /// Whether authentication is required for all operations
    pub authentication_required: bool,
    /// Whether TLS encryption is enabled
    pub tls_enabled: bool,
    /// Path to TLS certificate file
    pub certificate_path: Option<String>,
}

/// Load balancing strategy enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    #[default]
    RoundRobin,
    /// Least connections algorithm
    LeastConnections,
    /// Random selection
    Random,
    /// Weighted round-robin
    WeightedRoundRobin,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Strategy to use for load balancing
    pub strategy: LoadBalancingStrategy,
    /// Enable health checks before routing
    pub health_check_enabled: bool,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::RoundRobin,
            health_check_enabled: true,
            connection_timeout_ms: 5000,
            max_retries: 3,
        }
    }
}

/// Retry configuration for resilient operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries (for exponential backoff)
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

/// Circuit breaker configuration for fault tolerance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Time window for counting failures
    pub failure_window: Duration,
    /// Duration to wait before attempting to close circuit
    pub recovery_timeout: Duration,
    /// Number of successful requests needed to close circuit
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 2,
        }
    }
}

/// Health check configuration
///
/// **CONSOLIDATED**: Re-export of canonical version (Week 2, Nov 10 2025).
/// Field mappings: healthy_threshold→recovery_threshold, unhealthy_threshold→failure_threshold
pub use songbird_config::canonical::resilience::HealthCheckConfig;

/// Feature flags for runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // Feature flags are naturally boolean
pub struct FeatureFlags {
    /// Enable experimental features
    pub experimental_features: bool,
    /// Enable verbose logging
    pub verbose_logging: bool,
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Enable distributed tracing
    pub tracing_enabled: bool,
    /// Enable automatic capability discovery
    pub auto_discovery: bool,
}
