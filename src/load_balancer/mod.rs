//! Load Balancer Module
//!
//! Provides load balancing algorithms for service selection

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use rand::Rng;

use crate::errors::Result;
use crate::traits::service::ServiceInfo;

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub strategy: LoadBalancerStrategy,
    pub health_check_interval: std::time::Duration,
    pub max_retries: u32,
    pub timeout: std::time::Duration,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancerStrategy::RoundRobin,
            health_check_interval: std::time::Duration::from_secs(30),
            max_retries: 3,
            timeout: std::time::Duration::from_secs(10),
        }
    }
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    Random,
}

/// Service instance for load balancing
#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub service_info: ServiceInfo,
    pub weight: u32,
    pub current_connections: u32,
    pub is_healthy: bool,
}

/// Load balancer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub healthy_instances: u64,
    pub unhealthy_instances: u64,
}

impl Default for LoadBalancerStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            healthy_instances: 0,
            unhealthy_instances: 0,
        }
    }
}

/// Load balancer trait
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    /// Select a service instance from the available instances
    async fn select_service(&self, instances: &[ServiceInstance]) -> Result<Option<ServiceInstance>>;

    /// Update the health status of a service
    async fn update_service_health(&self, service_id: &str, is_healthy: bool) -> Result<()>;

    /// Get load balancer statistics
    async fn get_stats(&self) -> Result<LoadBalancerStats>;
}

/// Default load balancer implementation
pub struct DefaultLoadBalancer {
    config: LoadBalancerConfig,
    round_robin_counter: Arc<AtomicU64>,
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
    service_health: Arc<parking_lot::RwLock<HashMap<String, bool>>>,
    services: Arc<parking_lot::RwLock<HashMap<String, f64>>>, // service_id -> weight
}

impl DefaultLoadBalancer {
    pub fn new(config: LoadBalancerConfig) -> Self {
        Self {
            config,
            round_robin_counter: Arc::new(AtomicU64::new(0)),
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            service_health: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            services: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Add a service to the load balancer
    pub async fn add_service(&self, service_id: String, weight: f64) -> Result<()> {
        self.services.write().insert(service_id.clone(), weight);
        self.service_health.write().insert(service_id, true); // Default to healthy
        Ok(())
    }

    /// Remove a service from the load balancer
    pub async fn remove_service(&self, service_id: &str) -> Result<()> {
        self.services.write().remove(service_id);
        self.service_health.write().remove(service_id);
        Ok(())
    }

    /// Select a service from registered services (no parameters)
    pub async fn select_service(&self) -> Option<String> {
        let services = self.services.read();
        let health = self.service_health.read();
        
        // Get healthy services
        let healthy_services: Vec<_> = services
            .iter()
            .filter(|(id, _)| *health.get(*id).unwrap_or(&false))
            .collect();

        if healthy_services.is_empty() {
            return None;
        }

        // Increment total requests counter
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        let selected = match self.config.strategy {
            LoadBalancerStrategy::RoundRobin => {
                let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) as usize;
                healthy_services[index % healthy_services.len()].0.clone()
            }
            LoadBalancerStrategy::Random => {
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..healthy_services.len());
                healthy_services[index].0.clone()
            }
            LoadBalancerStrategy::LeastConnections => {
                // For simplicity, use round robin for now
                let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) as usize;
                healthy_services[index % healthy_services.len()].0.clone()
            }
            LoadBalancerStrategy::WeightedRoundRobin => {
                // Simple weighted selection
                let total_weight: f64 = healthy_services.iter().map(|(_, weight)| *weight).sum();
                if total_weight == 0.0 {
                    healthy_services[0].0.clone()
                } else {
                    let mut rng = rand::thread_rng();
                    let mut random_weight = rng.gen::<f64>() * total_weight;
                    
                    for (service_id, weight) in &healthy_services {
                        if random_weight < **weight {
                            return Some((*service_id).clone());
                        } else {
                            random_weight -= **weight;
                        }
                    }
                    healthy_services[0].0.clone()
                }
            }
        };

        // Increment successful requests counter
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        Some(selected)
    }
}

#[async_trait]
impl LoadBalancer for DefaultLoadBalancer {
    async fn select_service(&self, instances: &[ServiceInstance]) -> Result<Option<ServiceInstance>> {
        // Increment total requests counter
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        if instances.is_empty() {
            // Increment failed requests counter for empty list
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
            return Ok(None);
        }

        // Filter healthy instances
        let healthy_instances: Vec<&ServiceInstance> = instances
            .iter()
            .filter(|instance| instance.is_healthy)
            .collect();

        if healthy_instances.is_empty() {
            // Increment failed requests counter for no healthy instances
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
            return Ok(None);
        }

        let selected = match self.config.strategy {
            LoadBalancerStrategy::RoundRobin => {
                let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) as usize;
                healthy_instances[index % healthy_instances.len()]
            }
            LoadBalancerStrategy::Random => {
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..healthy_instances.len());
                healthy_instances[index]
            }
            LoadBalancerStrategy::LeastConnections => {
                healthy_instances
                    .iter()
                    .min_by_key(|instance| instance.current_connections)
                    .unwrap()
            }
            LoadBalancerStrategy::WeightedRoundRobin => {
                // Simple weighted selection - pick by weight
                let total_weight: u32 = healthy_instances.iter().map(|i| i.weight).sum();
                if total_weight == 0 {
                    healthy_instances[0]
                } else {
                    let mut rng = rand::thread_rng();
                    let mut random_weight = rng.gen_range(0..total_weight);
                    
                    let mut selected = healthy_instances[0]; // Default fallback
                    for instance in &healthy_instances {
                        if random_weight < instance.weight {
                            selected = instance;
                            break;
                        } else {
                            random_weight -= instance.weight;
                        }
                    }
                    selected
                }
            }
        };

        // Increment successful requests counter
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        Ok(Some(selected.clone()))
    }

    async fn update_service_health(&self, service_id: &str, is_healthy: bool) -> Result<()> {
        self.service_health.write().insert(service_id.to_string(), is_healthy);
        Ok(())
    }

    async fn get_stats(&self) -> Result<LoadBalancerStats> {
        let health_map = self.service_health.read();
        let healthy_count = health_map.values().filter(|&&h| h).count() as u64;
        let unhealthy_count = health_map.values().filter(|&&h| !h).count() as u64;

        Ok(LoadBalancerStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            average_response_time_ms: 0.0,
            healthy_instances: healthy_count,
            unhealthy_instances: unhealthy_count,
        })
    }
} 