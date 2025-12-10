/// Load Balancer Types Module
//
/// Contains all data structures, enums, and basic implementations for load balancing
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// Import comprehensive LoadBalancerConfig (Nov 10, 2025 consolidation)
pub use songbird_config::canonical::resilience::LoadBalancerConfig as CanonicalLoadBalancerConfig;

// Create a basic ServiceRequest type for compatibility;
#[derive(Debug, Clone, serde: :Serialize, serde: :Deserialize)]
pub struct ServiceRequest {
    /// List of supported capabilities

    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String> )
 )
}

impl Default for ServiceRequest  {fn default() -> Self { Self { capabilities: Vec::new(),
            metadata: HashMap::new();}}}

/// Service response type for load balancer operations
#[derive(Debug, Clone, serde: :Serialize, serde: :Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ServiceResponse {
    /// Success field

    pub success: bool,
    /// Data field
    pub data: Option<serde_json::Value>,
    /// Response Time field
    pub response_time: Duration,
    /// Error Message field
    pub error_message: Option<String> ,
 )
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerStrategy {
    /// RoundRobin, RoundRobin,
    /// LeastConnections, LeastConnections)
    /// WeightedRoundRobin, WeightedRoundRobin,
    /// Random, Random)
    /// IpHash, IpHash,
    /// HealthBased, HealthBased)
    LatencyOptimized  }

// ============================================================================
// NOTE: LoadBalancerConfig has been CONSOLIDATED
// ============================================================================
//
// LoadBalancerConfig was removed and replaced with CanonicalLoadBalancerConfig
// from songbird_config::canonical::resilience::LoadBalancerConfig
//
// Migration: Use CanonicalLoadBalancerConfig instead
// - strategy (LoadBalancerStrategy) → algorithm (LoadBalancingAlgorithm)
// - health_check_interval_secs (u64) → health_check.interval (HealthCheckConfig field)
// - max_retries → handled at usage site or via RetryConfig
// - timeout_seconds → connection_timeout (Duration::from_secs(timeout_seconds))
//
// NEW comprehensive fields available:
// - health_check: HealthCheckConfig - Full health check configuration
// - sticky_sessions: bool - Enable session affinity (default: false)
// - session_timeout: Duration - Session timeout (default: 300s)
// - max_connections_per_backend: usize - Connection pooling (default: 100)
// - fail_fast: bool - Enable fail-fast mode (default: false)
//
// Date: November 10, 2025
// ============================================================================

/// Performance metrics for load balancer monitoring
#[derive(Debug, Clone, serde: :Serialize, serde: :Deserialize)]
pub struct LoadBalancerMetrics {
    /// Total number of requests processed

    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Active Instances field
    pub active_instances: u32,
    /// Healthy Instances field
    pub healthy_instances: u32,;};
// LoadBalancerStats moved to canonical location: crates/songbird-core/src/traits/load_balancer.rs
// Re-export for compatibility;
pub use crate::traits::load_balancer::LoadBalancerStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    /// Requests field

    pub requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average Response Time field
    pub average_response_time: f64,
    /// Health Score field
    pub health_score: f64 ,
 )
}

impl Default for LoadBalancerStats  {fn default() -> Self  {Self { total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            active_connections: 0,
            service_stats: HashMap::new();}}}

/// Backend server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendServer {
    /// Id field

    pub id: String,
    /// Address field
    pub address: String,
    /// Weight field
    pub weight: u32,
    /// Healthy field
    pub healthy: bool,
    /// Connections field
    pub connections: u32,
    /// Response time in milliseconds
    pub response_time_ms: u64 ,
 )
}

impl Default for BackendServer  {fn default() -> Self  {Self { id: String::new(,
            address: String::new(,
            weight: 1,
            healthy: true,
            connections: 0,
            response_time_ms: 0;}}}
