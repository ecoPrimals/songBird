// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit breakers, retry policies, and fault tolerance types

#![allow(missing_docs, reason = "fault-tolerance structs mirror operational runbooks")]

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// **CANONICAL**: Circuit breaker configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting to close the circuit
    pub timeout: Duration,
    /// Number of successful requests needed to close the circuit
    pub success_threshold: u32,
    /// Maximum number of requests allowed in half-open state
    pub half_open_max_requests: u32,
    /// Whether the circuit breaker is enabled
    pub enabled: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 3,
            half_open_max_requests: 10,
            enabled: true,
        }
    }
}

/// **CANONICAL**: Retry configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to add jitter to retry delays
    pub jitter: bool,
    /// Whether retries are enabled
    pub enabled: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
            enabled: true,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CircuitBreakerState {
    /// Circuit is closed, requests are allowed
    #[default]
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, limited requests are allowed
    HalfOpen,
}

/// Retry strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Exponential backoff with optional jitter
    ExponentialBackoff {
        jitter: bool,
    },
    /// Linear backoff
    LinearBackoff,
    /// Custom retry strategy
    Custom {
        name: String,
    },
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::ExponentialBackoff {
            jitter: true,
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new circuit breaker configuration with custom settings
    #[must_use]
    pub const fn new(
        failure_threshold: u32,
        timeout: Duration,
        success_threshold: u32,
        half_open_max_requests: u32,
    ) -> Self {
        Self {
            failure_threshold,
            timeout,
            success_threshold,
            half_open_max_requests,
            enabled: true,
        }
    }

    /// Create a disabled circuit breaker configuration
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Check if the configuration is valid
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.failure_threshold > 0
            && self.success_threshold > 0
            && self.half_open_max_requests > 0
            && !self.timeout.is_zero()
    }
}

impl RetryConfig {
    /// Create a new retry configuration with custom settings
    #[must_use]
    pub const fn new(max_attempts: u32, initial_delay: Duration, backoff_multiplier: f64) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay: Duration::from_secs(30),
            backoff_multiplier,
            jitter: true,
            enabled: true,
        }
    }

    /// Create a disabled retry configuration
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Calculate the delay for a given attempt number
    #[must_use]
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 || !self.enabled {
            return Duration::from_millis(0);
        }

        #[allow(
            clippy::cast_precision_loss,
            reason = "intentional pattern; clippy false positive for this API"
        )]
        let base_delay = self.initial_delay.as_millis() as f64;
        #[allow(
            clippy::cast_possible_wrap,
            reason = "intentional pattern; clippy false positive for this API"
        )]
        let multiplied_delay = base_delay * self.backoff_multiplier.powi(attempt as i32 - 1);
        #[allow(
            clippy::cast_precision_loss,
            reason = "intentional pattern; clippy false positive for this API"
        )]
        let capped_delay = multiplied_delay.min(self.max_delay.as_millis() as f64);

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "intentional pattern; clippy false positive for this API"
        )]
        Duration::from_millis(capped_delay as u64)
    }

    /// Check if the configuration is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.max_attempts > 0
            && !self.initial_delay.is_zero()
            && !self.max_delay.is_zero()
            && self.backoff_multiplier > 0.0
            && self.max_delay >= self.initial_delay
    }
}

// ============================================================================
// ROBUSTNESS TYPES - Consolidated from unified/robustness.rs
// ============================================================================

/// Unified robustness configuration wrapper
///
/// **Merged from**: `unified/robustness.rs`\
/// **Purpose**: Comprehensive fault tolerance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RobustnessConfig {
    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limiting: RateLimitingConfig,

    /// Bulkhead configuration
    #[serde(default)]
    pub bulkhead: BulkheadConfig,

    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Load balancer configuration
    #[serde(default)]
    pub load_balancer: LoadBalancerConfig,

    /// Health check configuration
    #[serde(default)]
    pub health_check: HealthCheckConfig,

    /// Zero-cost router configuration
    #[serde(default)]
    pub zero_cost_router: ZeroCostRouterConfig,
}

/// Rate limiting configuration
///
/// **Merged from**: `unified/robustness.rs`\
/// **Purpose**: API rate limiting and throttling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,

    /// Maximum requests per second
    pub max_requests_per_second: u32,

    /// Burst capacity
    pub burst_capacity: u32,

    /// Window size for rate limiting (in seconds)
    pub window_size_secs: u64,

    /// Rate limit algorithm
    pub algorithm: RateLimitAlgorithm,

    /// Per-client rate limiting
    pub per_client_enabled: bool,

    /// Per-client max requests
    pub per_client_max_requests: u32,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests_per_second: 1000,
            burst_capacity: 2000,
            window_size_secs: 60,
            algorithm: RateLimitAlgorithm::TokenBucket,
            per_client_enabled: true,
            per_client_max_requests: 100,
        }
    }
}

/// Rate limiting algorithms
///
/// **Merged from**: `unified/robustness.rs`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RateLimitAlgorithm {
    /// Token bucket algorithm
    TokenBucket,
    /// Leaky bucket algorithm
    LeakyBucket,
    /// Fixed window algorithm
    FixedWindow,
    /// Sliding window algorithm
    SlidingWindow,
}

/// Bulkhead configuration for resource isolation
///
/// **Merged from**: `unified/robustness.rs`\
/// **Purpose**: Isolate resources to prevent cascade failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkheadConfig {
    /// Enable bulkhead pattern
    pub enabled: bool,

    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,

    /// Queue size for pending operations
    pub queue_size: usize,

    /// Operation timeout in milliseconds
    pub operation_timeout_ms: u64,

    /// Thread pool size
    pub thread_pool_size: usize,

    /// Isolation strategy
    pub isolation_strategy: IsolationStrategy,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_operations: 100,
            queue_size: 1000,
            operation_timeout_ms: 30000,
            thread_pool_size: 10,
            isolation_strategy: IsolationStrategy::ThreadPool,
        }
    }
}

/// Isolation strategies for bulkhead pattern
///
/// **Merged from**: `unified/robustness.rs`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IsolationStrategy {
    /// Isolate using thread pools
    ThreadPool,
    /// Isolate using semaphores
    Semaphore,
    /// Isolate using queues
    Queue,
}

/// Backoff strategies for retry mechanism
///
/// **Merged from**: `unified/robustness.rs`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Linear backoff
    Linear,
    /// Exponential backoff
    Exponential,
    /// Exponential backoff with jitter
    ExponentialWithJitter,
}

/// Load balancer configuration
///
/// **Merged from**: `unified/robustness.rs`\
/// **Purpose**: Load balancing and traffic distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,

    /// Sticky sessions enabled
    pub sticky_sessions: bool,

    /// Session timeout in seconds
    pub session_timeout_secs: u64,

    /// Maximum connections per backend
    pub max_connections_per_backend: usize,

    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,

    /// Enable fail-fast
    pub fail_fast: bool,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            algorithm: LoadBalancingAlgorithm::RoundRobin,
            sticky_sessions: false,
            session_timeout_secs: 300,
            max_connections_per_backend: 100,
            connection_timeout_ms: 30000,
            fail_fast: true,
        }
    }
}

/// Load balancing algorithms
///
/// **Merged from**: `unified/robustness.rs`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoadBalancingAlgorithm {
    /// Round-robin distribution
    RoundRobin,
    /// Least connections algorithm
    LeastConnections,
    /// Weighted round-robin
    WeightedRoundRobin,
    /// Random selection
    Random,
    /// IP hash-based routing
    IpHash,
    /// Capability-based routing
    CapabilityBased,
}

/// Health check configuration
///
/// **Canonical location**: `songbird_config::canonical::resilience`
/// **Merged from**: `unified/robustness.rs`, `config/mod.rs`, and 10+ other locations
/// **Purpose**: Universal health check configuration for all services
/// **Migration**: Replaces 12-15 duplicate definitions (Week 2, Nov 10 2025)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Health check interval in seconds
    pub interval_secs: u64,

    /// Health check timeout in seconds
    pub timeout_secs: u64,

    /// Failure threshold before marking unhealthy
    pub failure_threshold: u32,

    /// Recovery threshold before marking healthy
    pub recovery_threshold: u32,

    /// Health check path/endpoint
    pub path: String,

    /// Maximum retry attempts (0 = no retries, added for compatibility)
    #[serde(default)]
    pub max_retries: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 10,
            failure_threshold: 3,
            recovery_threshold: 2,
            path: "/health".to_string(),
            max_retries: 0,
        }
    }
}

/// Zero-cost router configuration
///
/// **Merged from**: `unified/robustness.rs`\
/// **Purpose**: Compile-time route optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroCostRouterConfig {
    /// Enable zero-cost routing
    pub enabled: bool,

    /// Route cache size
    pub route_cache_size: usize,

    /// Route cache TTL in seconds
    pub route_cache_ttl_secs: u64,

    /// Enable route optimization
    pub optimize_routes: bool,

    /// Maximum route depth
    pub max_route_depth: usize,

    /// Route discovery timeout in milliseconds
    pub discovery_timeout_ms: u64,
}

impl Default for ZeroCostRouterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            route_cache_size: 10000,
            route_cache_ttl_secs: 300,
            optimize_routes: true,
            max_route_depth: 10,
            discovery_timeout_ms: 30000,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
#[path = "resilience_tests.rs"]
mod tests;
