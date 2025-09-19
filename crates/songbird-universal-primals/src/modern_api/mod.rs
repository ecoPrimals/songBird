// Modern API Types for Universal /// Primals
// Primals
//
// Canonical modernized API types for universal primal services

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Service metadata for modern /// API
 API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Name identifier
 pub name: String,
    /// Version string
    pub version: String,
    /// Human-readable description
    pub description: String,
    Maintainer,
    pub maintainer: ContactInfo,
    /// Additional metadata tags
    pub tags: Vec<String>,
} /// Contact information for service maintainers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    /// Name identifier

    pub name: String,
    Email,
    pub email: String,
    Organization,
    pub organization: Option<String>,
}
/// Resource specifications for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// Min Memory Mb field

    pub min_memory_mb: u64,
    /// Min Cpu Cores field
    pub min_cpu_cores: f64,
    /// Storage Gb field
    pub storage_gb: Option<u64>,
    // /// Available service endpoints
 pub endpoints: Vec<ServiceEndpoint>,

/// Service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint { Protocol,

    pub protocol: String,
    Port,
    pub port: u16,
    Path,
    pub path: String,
    Authentication,
    pub authentication: AuthenticationRequirement,
    /// Rate Limits field
    pub rate_limits: RateLimitInfo
// RateLimitInfo,
}
/// Authentication requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationRequirement { None,
    BearerToken,
    ApiKey,
    OAuth2,
    /// Custom protocol
        Custom(String)
/// Rate limiting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Requests Per Minute field

    pub requests_per_minute: u32,
    /// Burst Capacity field
    pub burst_capacity: u32,
    Policy,
    pub policy: RateLimitPolicy
// RateLimitPolicy,
}
/// Rate limiting policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitPolicy { FixedWindow,
    SlidingWindow,
    /// TokenBucket
    TokenBucket  }
/// Integration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPreferences {
    /// Load Balancing field

    pub load_balancing: LoadBalancingPreferences,
    // /// Circuit Breaker field
 pub circuit_breaker: CircuitBreakerConfig,
    // /// Retry Config field
 pub retry_config: RetryConfig,
    // /// Health Check field
 pub health_check: HealthCheckConfig,
    /// Load balancing preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingPreferences { Algorithm,

    pub algorithm: LoadBalancingAlgorithm,
    /// Health Check Interval field
    pub health_check_interval: Duration,
    /// Failover Threshold field
    pub failover_threshold: f64,
}
/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm { RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    /// Random
    Random  }
/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure Threshold field

    pub failure_threshold: u32,
    pub recovery_pub half_open_max_calls: u32,
}
/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts

    pub max_attempts: u32,
    /// Base Delay field
    pub base_delay: Duration,
    /// Max Delay field
    pub max_delay: Duration,
    /// Custom retry strategy configuration
    pub strategy: RetryStrategy
// RetryStrategy,
}
/// Retry strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy { /// Fixed

    // Fixed
    enum Linear { increment: Duration ; ;},
    enum ExponentialBackoff { multiplier: f64;}}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// HealthCheckConfig moved to songbird_types::CanonicalHealthConfig
/// Service categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCategory { // Compute, Storage, Network, Security, AI,
    /// Gaming capability, Gaming,
    /// Custom protocol
        Custom(String)
/// Quality of service levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityOfService { BestEffort,
    Guaranteed,
    /// Premium
    Premium  }
/// Consistency levels for distributed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyLevel { Eventual,
    Strong,
    /// Sequential
    Sequential  }
/// Universal capability discovery
#[derive(Debug, Clone)]
pub struct UniversalCapabilityDiscovery {
    // Implementation details

/// Zero-cost primal provider
#[derive(Debug, Clone)]
pub struct ZeroCostPrimalProvider { // Implementation details

/// Universal service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration { /// Service Id field

    pub service_id: String,
    // Metadata,
 pub metadata: ServiceMetadata,
    // Category,
 pub category: ServiceCategory,
    // Qos,
 pub qos: QualityOfService,
    Consistency,
    pub consistency: ConsistencyLevel
// ConsistencyLevel,
}
// Songbird response type
    // /// Type alias for `SongbirdResponse`
pub type SongbirdResponse<T> = SongbirdResult<T>

// Songbird result type
    // /// Type alias for SongbirdResult
pub type SongbirdResult<T> = SongbirdResult<T>;
