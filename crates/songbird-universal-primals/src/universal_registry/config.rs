//! Configuration types for Universal Service Registry

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Resource specification for services
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceSpec {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub storage_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<f64>,
    pub gpu_count: Option<u32>,
    pub custom_resources: std::collections::HashMap<String, serde_json::Value>,
}

/// Service endpoints configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub name: String,
    pub url: String,
    pub health_check: Option<HealthCheckConfig>,
    pub authentication_required: bool,
    pub rate_limit: Option<RateLimitConfig>,
    pub circuit_breaker: Option<CircuitBreakerConfig>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub path: String,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

/// Integration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPreferences {
    pub preferred_protocols: Vec<String>,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub circuit_breaker: CircuitBreakerConfig,
    pub rate_limiting: RateLimitConfig,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub priority: ServicePriority,
}

/// Load balancing strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin {
        weights: std::collections::HashMap<String, u32>,
    },
    Random,
    ConsistentHash {
        hash_key: String,
    },
    HealthBased,
    ResponseTime,
    Custom {
        algorithm: String,
    },
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout_duration: Duration,
    pub half_open_max_calls: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub strategy: RateLimitStrategy,
    pub max_requests: u32,
    pub window_duration: Duration,
    pub burst_size: Option<u32>,
}

/// Rate limiting strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RateLimitStrategy {
    TokenBucket,
    LeakyBucket,
    FixedWindow,
    SlidingWindow,
    Adaptive { baseline_rps: f64 },
}

/// Service priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServicePriority {
    Critical,
    High,
    Normal,
    Low,
    BestEffort,
}
