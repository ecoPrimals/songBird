//! Load Balancer Traits

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::errors::{Result, SongbirdError};
use crate::traits::service::{ServiceInfo, ServiceRequest, ServiceResponse};

/// Load balancer trait
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    /// Select a service instance for a request
    async fn select_service(
        &self,
        services: &[ServiceInfo],
        request: &ServiceRequest,
    ) -> Result<ServiceInfo>;

    /// Record the response for learning
    async fn record_response(
        &self,
        service: &ServiceInfo,
        response: &ServiceResponse,
    ) -> Result<()>;

    /// Update service weights
    async fn update_weights(&self, weights: HashMap<String, f64>) -> Result<()>;

    /// Get load balancer statistics
    async fn get_stats(&self) -> Result<LoadBalancerStats>;

    /// Get algorithm name
    fn algorithm(&self) -> &'static str;
}

/// Load balancing algorithms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    Random,
    WeightedRandom,
    HealthAware,
}

/// Load balancer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub service_stats: HashMap<String, ServiceStats>,
    pub algorithm: String,
    pub health_aware: bool,
}

/// Statistics for individual services
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServiceStats {
    pub requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub current_load: f64,
}

impl Default for ServiceStats {
    fn default() -> Self {
        Self {
            requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            current_load: 0.0,
        }
    }
}

impl Default for LoadBalancerStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            service_stats: HashMap::new(),
            algorithm: "round_robin".to_string(),
            health_aware: false,
        }
    }
}

/// Round-robin load balancer
pub struct RoundRobinLoadBalancer {
    counter: Arc<std::sync::atomic::AtomicUsize>,
    stats: Arc<parking_lot::RwLock<LoadBalancerStats>>,
}

impl RoundRobinLoadBalancer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            stats: Arc::new(parking_lot::RwLock::new(LoadBalancerStats {
                algorithm: "round_robin".to_string(),
                ..LoadBalancerStats::default()
            })),
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
            return Err(SongbirdError::LoadBalancer {
                message: "No services available".to_string(),
            });
        }

        let index = self
            .counter
            .fetch_add(1, Ordering::Relaxed)
            % services.len();
        Ok(services[index].clone())
    }

    async fn record_response(
        &self,
        service: &ServiceInfo,
        _response: &ServiceResponse,
    ) -> Result<()> {
        let mut stats = self.stats.write();
        stats.total_requests += 1;
        stats
            .service_stats
            .entry(service.id.clone())
            .or_default()
            .requests += 1;
        Ok(())
    }

    async fn update_weights(&self, _weights: HashMap<String, f64>) -> Result<()> {
        Err(SongbirdError::Internal {
            message: "Round robin does not support weights".to_string(),
        })
    }

    async fn get_stats(&self) -> Result<LoadBalancerStats> {
        Ok(self.stats.read().clone())
    }

    fn algorithm(&self) -> &'static str {
        "round_robin"
    }
}

/// Weighted load balancer that doesn't support weights
pub struct WeightedLoadBalancer;

impl WeightedLoadBalancer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WeightedLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LoadBalancer for WeightedLoadBalancer {
    async fn select_service(
        &self,
        _services: &[ServiceInfo],
        _request: &ServiceRequest,
    ) -> Result<ServiceInfo> {
        Err(SongbirdError::Internal {
            message: "Round robin does not support weights".to_string(),
        })
    }

    async fn record_response(
        &self,
        _service: &ServiceInfo,
        _response: &ServiceResponse,
    ) -> Result<()> {
        Err(SongbirdError::Internal {
            message: "Round robin does not support weights".to_string(),
        })
    }

    async fn update_weights(&self, _weights: HashMap<String, f64>) -> Result<()> {
        Err(SongbirdError::Internal {
            message: "Round robin does not support weights".to_string(),
        })
    }

    async fn get_stats(&self) -> Result<LoadBalancerStats> {
        Err(SongbirdError::Internal {
            message: "Round robin does not support weights".to_string(),
        })
    }

    fn algorithm(&self) -> &'static str {
        "round_robin"
    }
}
