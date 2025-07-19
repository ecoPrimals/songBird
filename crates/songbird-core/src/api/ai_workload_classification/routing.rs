//! Routing strategies and algorithms for workload distribution

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Optimal routing strategy for workloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStrategy {
    /// Primary routing algorithm
    pub algorithm: RoutingAlgorithm,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,

    /// Failover targets
    pub failover_targets: Vec<String>,

    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Available routing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingAlgorithm {
    RoundRobin,
    WeightedRoundRobin { weights: Vec<f64> },
    LeastConnections,
    WeightedLeastConnections { weights: Vec<f64> },
    ResourceBased,
    LatencyBased,
    Hash { hash_key: String },
    GeographicProximity,
    CostOptimized,
    AIOptimized { model_confidence: f64 },
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,

    /// Timeout duration when circuit is open
    pub timeout_duration_ms: u64,

    /// Number of requests to test in half-open state
    pub half_open_test_requests: u32,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    None,
    Random,
    RoundRobin,
    WeightedRandom { weights: Vec<f64> },
    LeastConnections,
    ResourceAware,
    AdaptiveLoadBalancing,
    PredictiveLoadBalancing { prediction_window_ms: u64 },
}

/// Retry configuration for failed requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Base delay between retries
    pub base_delay_ms: u64,

    /// Maximum delay between retries
    pub max_delay_ms: u64,

    /// Backoff strategy
    pub backoff_strategy: BackoffStrategy,

    /// Jitter to add to delays
    pub jitter_ms: u64,
}

/// Backoff strategies for retries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed,
    Linear,
    Exponential { multiplier: f64 },
    ExponentialWithJitter { multiplier: f64, jitter_factor: f64 },
}
