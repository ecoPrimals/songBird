//! Robustness Configuration Module
//!
//! Consolidates all robustness-related configuration structs including circuit breakers)
//! rate limiting, bulkheads, and retry policies

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Unified robustness configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RobustnessConfig  {/// Circuit breaker configuration
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

    /// Zero-cost router configuration
    #[serde(default)]
    pub zero_cost_router: ZeroCostRouterConfig,
}

/// Circuit breaker configuration
///
/// **CONSOLIDATED**: Re-export of canonical version (Week 2, Nov 10 2025).
/// Note: Original had `min_throughput_threshold` which is not in canonical
pub use crate::canonical::resilience::CircuitBreakerConfig;

// Default implementation now provided by canonical

/// Rate limiting configuration (consolidated from `RateLimitingConfig` structs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig  {/// Enable rate limiting
    pub enabled: bool,

    /// Maximum requests per second
    pub max_requests_per_second: u32,

    /// Burst capacity
    pub burst_capacity: u32,

    /// Window size for rate limiting
    pub window_size: Duration,

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
            window_size: Duration::from_secs(60),
            algorithm: RateLimitAlgorithm::TokenBucket,
            per_client_enabled: true,
            per_client_max_requests: 100,
        }
    }
}

/// Rate limiting algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitAlgorithm {
    TokenBucket,
    LeakyBucket,
    FixedWindow,
    SlidingWindow,
}

/// Bulkhead configuration (consolidated from `BulkheadConfig` structs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkheadConfig  {/// Enable bulkhead pattern
    pub enabled: bool,

    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,

    /// Queue size for pending operations
    pub queue_size: usize,

    /// Operation timeout
    pub operation_timeout: Duration,

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
            operation_timeout: Duration::from_secs(30),
            thread_pool_size: 10,
            isolation_strategy: IsolationStrategy::ThreadPool,
        }
    }
}

/// Isolation strategies for bulkhead pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationStrategy {
    ThreadPool,
    Semaphore,
    Queue,
}

/// **CONSOLIDATED**: Re-export of canonical `RetryConfig` (Nov 10, 2025)
/// 
/// Note: `enabled`, `backoff_strategy`, `jitter_enabled`, `retryable_errors` were unified-specific
///       These are now handled at usage site or via builder patterns
/// Default implementation provided by `canonical::resilience::RetryConfig`
pub use crate::canonical::resilience::RetryConfig;

/// Backoff strategies for retry mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed,
    Linear,
    Exponential,
    ExponentialWithJitter,
}

/// Load balancer configuration (consolidated from `LoadBalancerConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig  {/// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Sticky sessions enabled
    pub sticky_sessions: bool,

    /// Session timeout
    pub session_timeout: Duration,

    /// Maximum connections per backend
    pub max_connections_per_backend: usize,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Enable fail-fast
    pub fail_fast: bool,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            algorithm: LoadBalancingAlgorithm::RoundRobin,
            health_check: HealthCheckConfig::default(),
            sticky_sessions: false,
            session_timeout: Duration::from_secs(300),
            max_connections_per_backend: 100,
            connection_timeout: Duration::from_secs(30),
            fail_fast: true,
        }
    }
}

/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    Random,
    IpHash,
    CapabilityBased,
    HealthBased,
}

/// Health check configuration for load balancer
///
/// **CONSOLIDATED**: This is now a re-export of the canonical version.
/// Use `songbird_config::canonical::resilience::HealthCheckConfig` directly.
/// 
/// **Migration Note** (Week 2, Nov 10 2025):
/// This duplicate definition has been replaced with canonical version.
/// The canonical uses `u64` for intervals instead of `Duration` for consistency.
pub use crate::canonical::resilience::HealthCheckConfig;

/// Zero-cost router configuration (consolidated from `ZeroCostRouterConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroCostRouterConfig  {/// Enable zero-cost routing
    pub enabled: bool,

    /// Route cache size
    pub route_cache_size: usize,

    /// Route cache TTL
    pub route_cache_ttl: Duration,

    /// Enable route optimization
    pub optimize_routes: bool,

    /// Maximum route depth
    pub max_route_depth: usize,

    /// Route discovery timeout
    pub discovery_timeout: Duration,
}

impl Default for ZeroCostRouterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            route_cache_size: 10000,
            route_cache_ttl: Duration::from_secs(300),
            optimize_routes: true,
            max_route_depth: 10,
            discovery_timeout: Duration::from_secs(30)
        }
    }
}
