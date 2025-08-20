/// Load Balancer Types Module
///
/// Contains all data structures, enums, and basic implementations for load balancing
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// Create a basic ServiceRequest type for compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceRequest {
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Default for ServiceRequest {
    fn default() -> Self {
        Self {
            capabilities: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Service response type for load balancer operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub response_time: Duration,
    pub error_message: Option<String>,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    Random,
    IpHash,
    HealthBased,
    LatencyOptimized,
}

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub strategy: LoadBalancerStrategy,
    pub health_check_interval_secs: u64,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancerStrategy::RoundRobin,
            health_check_interval_secs: 30,
            max_retries: 3,
            timeout_seconds: 30,
        }
    }
}

/// Performance metrics for load balancer monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoadBalancerMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub active_instances: u32,
    pub healthy_instances: u32,
}

// LoadBalancerStats moved to canonical location: crates/songbird-core/src/traits/load_balancer.rs
// Re-export for compatibility
pub use crate::traits::load_balancer::LoadBalancerStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    pub requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub health_score: f64,
}

impl Default for LoadBalancerStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            active_connections: 0,
            service_stats: HashMap::new(),
        }
    }
}

/// Backend server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendServer {
    pub id: String,
    pub address: String,
    pub weight: u32,
    pub healthy: bool,
    pub connections: u32,
    pub response_time_ms: u64,
}

impl Default for BackendServer {
    fn default() -> Self {
        Self {
            id: String::new(),
            address: String::new(),
            weight: 1,
            healthy: true,
            connections: 0,
            response_time_ms: 0,
        }
    }
}
