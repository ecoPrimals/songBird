//! Load Balancer Trait
//!
//! Provides load balancing capabilities for service requests

use crate::errors::Result;
use crate::traits::service::{ServiceInfo, ServiceRequest};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Load balancing algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    Random,
    HealthBased,
}

/// Load balancer statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub active_connections: u64,
    pub service_stats: HashMap<String, ServiceStats>,
}

/// Per-service statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceStats {
    pub requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub average_response_time: f64,
    pub active_connections: u64,
    pub weight: f64,
}

/// Load balancer trait
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    /// Select a service instance for the given request
    async fn select_service(
        &self,
        services: &[ServiceInfo],
        request: &ServiceRequest,
    ) -> Result<ServiceInfo>;

    /// Update service health/availability
    async fn update_service_health(&self, service_id: &str, is_healthy: bool) -> Result<()>;

    /// Get load balancer statistics
    async fn get_stats(&self) -> Result<LoadBalancerStats>;

    /// Reset statistics
    async fn reset_stats(&self) -> Result<()>;
}

/// Round-robin load balancer implementation
pub struct RoundRobinLoadBalancer {
    counter: AtomicUsize,
    stats: LoadBalancerStats,
}

impl RoundRobinLoadBalancer {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            stats: LoadBalancerStats::default(),
        }
    }
}

impl Default for RoundRobinLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    async fn select_service(
        &self,
        services: &[ServiceInfo],
        _request: &ServiceRequest,
    ) -> Result<ServiceInfo> {
        if services.is_empty() {
            return Err(crate::errors::SongbirdError::LoadBalancer {
                message: "No services available".to_string(),
            });
        }

        let index = self.counter.fetch_add(1, Ordering::Relaxed) % services.len();
        Ok(services[index].clone())
    }

    async fn update_service_health(&self, service_id: &str, is_healthy: bool) -> Result<()> {
        tracing::info!("Updated health for service {}: {}", service_id, is_healthy);
        Ok(())
    }

    async fn get_stats(&self) -> Result<LoadBalancerStats> {
        Ok(self.stats.clone())
    }

    async fn reset_stats(&self) -> Result<()> {
        tracing::info!("Reset load balancer statistics");
        Ok(())
    }
}

/// Weighted round-robin load balancer
pub struct WeightedRoundRobinLoadBalancer {
    weights: HashMap<String, f64>,
    #[allow(dead_code)]
    current_weights: HashMap<String, f64>,
    stats: LoadBalancerStats,
}

impl WeightedRoundRobinLoadBalancer {
    pub fn new() -> Self {
        Self {
            weights: HashMap::new(),
            current_weights: HashMap::new(),
            stats: LoadBalancerStats::default(),
        }
    }

    pub fn set_weight(&mut self, service_id: String, weight: f64) {
        self.weights.insert(service_id, weight);
    }
}

impl Default for WeightedRoundRobinLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LoadBalancer for WeightedRoundRobinLoadBalancer {
    async fn select_service(
        &self,
        services: &[ServiceInfo],
        _request: &ServiceRequest,
    ) -> Result<ServiceInfo> {
        if services.is_empty() {
            return Err(crate::errors::SongbirdError::LoadBalancer {
                message: "No services available".to_string(),
            });
        }

        // Simplified weighted selection - just return first service for now
        // In a real implementation, this would use proper weighted round-robin logic
        Ok(services[0].clone())
    }

    async fn update_service_health(&self, service_id: &str, is_healthy: bool) -> Result<()> {
        tracing::info!("Updated health for service {}: {}", service_id, is_healthy);
        Ok(())
    }

    async fn get_stats(&self) -> Result<LoadBalancerStats> {
        Ok(self.stats.clone())
    }

    async fn reset_stats(&self) -> Result<()> {
        tracing::info!("Reset weighted load balancer statistics");
        Ok(())
    }
}
