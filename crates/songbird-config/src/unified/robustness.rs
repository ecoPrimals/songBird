//! Robustness Configuration Module
//!
//! Consolidates all robustness-related configuration structs including circuit breakers,
//! rate limiting, bulkheads, and retry policies

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Unified robustness configuration
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

    /// Zero-cost router configuration
    #[serde(default)]
    pub zero_cost_router: ZeroCostRouterConfig,
}

/// Circuit breaker configuration (consolidated from multiple `CircuitBreakerConfig` structs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    pub enabled: bool,

    /// Failure threshold to open circuit
    pub failure_threshold: u32,

    /// Timeout before attempting recovery
    pub timeout: Duration,

    /// Recovery timeout
    pub recovery_timeout: Duration,

    /// Success threshold to close circuit
    pub success_threshold: u32,

    /// Half-open max calls
    pub half_open_max_calls: u32,

    /// Minimum throughput threshold
    pub min_throughput_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
            half_open_max_calls: 10,
            min_throughput_threshold: 20,
        }
    }
}

/// Rate limiting configuration (consolidated from `RateLimitingConfig` structs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
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
pub struct BulkheadConfig {
    /// Enable bulkhead pattern
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

/// Retry configuration (consolidated from `RetryConfig` structs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Enable retry mechanism
    pub enabled: bool,

    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Initial retry delay
    pub initial_delay: Duration,

    /// Maximum retry delay
    pub max_delay: Duration,

    /// Backoff multiplier
    pub backoff_multiplier: f64,

    /// Backoff strategy
    pub backoff_strategy: BackoffStrategy,

    /// Jitter enabled
    pub jitter_enabled: bool,

    /// Retryable errors
    pub retryable_errors: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            backoff_strategy: BackoffStrategy::Exponential,
            jitter_enabled: true,
            retryable_errors: vec![
                "network_error".to_string(),
                "timeout".to_string(),
                "service_unavailable".to_string(),
            ],
        }
    }
}

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
pub struct LoadBalancerConfig {
    /// Load balancing algorithm
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    Random,
    IpHash,
    CapabilityBased,
}

/// Health check configuration for load balancer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Health check interval
    pub interval: Duration,

    /// Health check timeout
    pub timeout: Duration,

    /// Failure threshold
    pub failure_threshold: u32,

    /// Recovery threshold
    pub recovery_threshold: u32,

    /// Health check path
    pub path: String,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_threshold: 2,
            path: "/health".to_string(),
        }
    }
}

/// Zero-cost router configuration (consolidated from `ZeroCostRouterConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroCostRouterConfig {
    /// Enable zero-cost routing
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
            discovery_timeout: Duration::from_secs(30),
        }
    }
}
