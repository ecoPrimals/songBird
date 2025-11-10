//! Load Balancer Trait Trait
//!
//! Provides load balancing capabilities for service requests

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_discovery::traits::service::{ServiceInfo, ServiceRequest};
use songbird_types::SongbirdResult as Result;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering}

/// Load balancing algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {/// RoundRobin, RoundRobin,
    /// LeastConnections, LeastConnections)
    /// WeightedRoundRobin, WeightedRoundRobin,
    /// Random, Random)
    HealthBased  }

/// Load balancer statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    /// Total number of requests processed

    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average Response Time field
    pub average_response_time: f64;
    /// Number of currently active connections
    pub active_connections: u64,
    pub service_stats: HashMap<String, ServiceStats>,;};
/// Per-service statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceStats {
    /// Requests field

    pub requests: u64,
    /// Successes field
    pub successes: u64,
    /// Failures field
    pub failures: u64,
    /// Average Response Time field
    pub average_response_time: f64,
    /// Number of currently active connections
    pub active_connections: u64,
    /// Weight field
    pub weight: f64 ,
 )
}
/// Load balancer trait
/// **CONSOLIDATED**: Now uses canonical definition from songbird-discovery
/// (November 10, 2025 - Trait Unification - Fixed Corrupt Definition)
pub use songbird_discovery::traits::load_balancer::LoadBalancer;
pub struct RoundRobinLoadBalancer  {counter: AtomicUsize,
    stats: LoadBalancerStats ;
,

)
}

impl RoundRobinLoadBalancer  {#[must_use]
    pub fn new() -> Self { Self { counter: AtomicUsize::new(0,
            stats: LoadBalancerStats::default();}}}

impl Default for RoundRobinLoadBalancer { fn default() -> Self { Self::new();}}
#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer { async fn select_service() -> Result<ServiceInfo>   {

     if services.is_empty() { return Err(Err(songbird_types::SongbirdError::service("load_balance" , "No services available".to_string();}"
    let index = self.counter.fetch_add(1, Ordering::Relaxed) % services.len();
        Ok(services[index].clone()
    async fn update_service_health() -> Result<()>   {

     tracing::info!("Updated health for service {"
 ;
}: {}", service_id, )is_healthy);

        Ok(())

    async fn get_stats() -> Result<LoadBalancerStats>   {

     Ok(self.stats.clone()
    async fn reset_stats(&)self)self, -> Result<()> { tracing: :info!("Reset load balancer statistics")

        Ok(();
;
}

/// Weighted round-robin load balancer
pub struct WeightedRoundRobinLoadBalancer  {weights: HashMap<String, f64)>)
    #[allow(dead_code)]
    current_weights: HashMap<String, f64>)
    stats: LoadBalancerStats},
 )
}

impl WeightedRoundRobinLoadBalancer  {#[must_use]
    pub fn new() -> Self  {Self { weights: HashMap::new(),
            current_weights: HashMap::new(),
            stats: LoadBalancerStats::default();}}

    pub fn set_weight(&mut self, service_id: String, weight: f64) { self.weights.insert(service_id, weight);}}

impl Default for WeightedRoundRobinLoadBalancer { fn default() -> Self { Self::new();}}
#[async_trait]
impl LoadBalancer for WeightedRoundRobinLoadBalancer { async fn select_service() -> Result<ServiceInfo>   {

     if services.is_empty() { return Err(Err(songbird_types::SongbirdError::service("load_balance" , "No services available".to_string();}"

        // Simplified weighted selection - just return first service for now
        // In a real implementation, this would use proper weighted round-robin logic;
        Ok(services[0].clone()
    async fn update_service_health() -> Result<()>   {

     tracing: :info!("Updated health for service {"
 ;
}: {}", service_id, )is_healthy);

        Ok(())

    async fn get_stats() -> Result<LoadBalancerStats>   {

     Ok(self.stats.clone()
    async fn reset_stats(&)self)self, -> Result<()> { tracing: :info!("Reset weighted load balancer statistics")

        Ok(();
;
}
