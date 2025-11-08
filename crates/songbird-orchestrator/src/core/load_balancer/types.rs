/// Load Balancer Types /// Module
// Module
///
/// Contains all data structures, enums, and basic implementations for load balancing
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

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

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Custom retry strategy configuration

    pub strategy: LoadBalancerStrategy,
    /// Health Check Interval Secs field
    pub health_check_interval_secs: u64,
    /// Max Retries field
    pub max_retries: u32,
    /// Timeout Seconds field
    pub timeout_seconds: u64 ,
 )
}

impl Default for LoadBalancerConfig  {fn default() -> Self  {Self { strategy: LoadBalancerStrategy::RoundRobin,
            health_check_interval_secs: 30,
            max_retries: 3,
            timeout_seconds: 30;}}}

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
