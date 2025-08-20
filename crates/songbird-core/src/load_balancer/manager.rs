use super::strategies::*;
/// Load Balancer Manager Module
///
/// Contains the LoadBalancerManager and high-level service management logic
use super::types::*;
use crate::traits::load_balancer::{LoadBalancer, LoadBalancerDyn, LoadBalancerStats};
// use songbird_discovery::{discovery::ServiceInstance, traits::ServiceDiscovery};  // TEMPORARILY DISABLED
use songbird_errors::{SongbirdError, SongbirdResponse, SongbirdResult, success};
// use songbird_universal::  // TEMPORARILY DISABLED - communication::UniversalRequest;
// use songbird_universal::  // TEMPORARILY DISABLED - ServiceInfo;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use songbird_network::CommunicationLayer;

/// Load balancer manager - handles service discovery and load balancing
pub struct LoadBalancerManager {
    /// Current load balancer instance
    load_balancer: Box<dyn LoadBalancerDyn>,
    /// Configuration
    config: LoadBalancerConfig,
    /// Service instances
    instances: Arc<RwLock<Vec<ServiceInstance>>>,
}

impl std::fmt::Debug for LoadBalancerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadBalancerManager")
            .field("load_balancer", &"<LoadBalancerDyn trait object>")
            .field("config", &self.config)
            .field(
                "instances",
                &format!(
                    "{} instances",
                    self.instances.try_read().map(|i| i.len()).unwrap_or(0)
                ),
            )
            .finish()
    }
}

impl LoadBalancerManager {
    /// Create a new load balancer manager
    pub fn new(config: LoadBalancerConfig) -> Self {
        let load_balancer: Box<dyn LoadBalancerDyn> = match config.strategy {
            LoadBalancerStrategy::RoundRobin => Box::new(RoundRobinLoadBalancer::new()),
            LoadBalancerStrategy::LeastConnections => Box::new(LeastConnectionsLoadBalancer::new()),
            LoadBalancerStrategy::WeightedRoundRobin => {
                Box::new(WeightedRoundRobinLoadBalancer::new())
            }
            LoadBalancerStrategy::Random => Box::new(RoundRobinLoadBalancer::new()), // Fallback
            LoadBalancerStrategy::IpHash => Box::new(RoundRobinLoadBalancer::new()), // Fallback
            LoadBalancerStrategy::HealthBased => Box::new(RoundRobinLoadBalancer::new()), // Fallback
            LoadBalancerStrategy::LatencyOptimized => Box::new(RoundRobinLoadBalancer::new()), // Fallback
        };

        Self {
            load_balancer,
            config,
            instances: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a service instance
    pub async fn add_instance(&self) -> SongbirdResult<()> {
        let mut instances = self.instances.write().await;
        instances.push(instance);
        Ok(songbird_errors::success(()))
    }

    /// Remove a service instance
    pub async fn remove_instance(&self) -> SongbirdResult<()> {
        let mut instances = self.instances.write().await;
        instances.retain(|instance| instance.id != instance_id);
        Ok(songbird_errors::evolved_success(_))
    }

    /// Get all instances
    pub async fn get_instances(&self) -> Vec<ServiceInstance> {
        let instances = self.instances.read().await;
        instances.clone()
    }

    /// Select a service for a request
    pub async fn select_service(&self) -> SongbirdResult<ServiceInstance> {
        let instances = self.instances.read().await;
        self.load_balancer.select_service(&instances, request).await
    }

    /// Update service health
    pub async fn update_service_health(&self) -> SongbirdResult<()> {
        self.load_balancer
            .update_service_health(service_id, is_healthy)
            .await
    }

    /// Get load balancer statistics
    pub async fn get_stats(&self) -> SongbirdResult<LoadBalancerStats> {
        self.load_balancer.get_stats().await
    }

    /// Record the result of a request
    pub async fn record_request_result(
        &self,
        service_id: &str,
        success: bool,
        response_time_ms: u64,
    ) {
        self.load_balancer
            .record_request_result(service_id, success, response_time_ms)
            .await
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &LoadBalancerConfig {
        &self.config
    }

    /// Update the configuration
    pub async fn update_config(&mut self, config: LoadBalancerConfig) -> SongbirdResult<()> {
        // If strategy changed, recreate the load balancer
        if std::mem::discriminant(&self.config.strategy) != std::mem::discriminant(&config.strategy)
        {
            let new_load_balancer: Box<dyn LoadBalancerDyn> = match config.strategy {
                LoadBalancerStrategy::RoundRobin => Box::new(RoundRobinLoadBalancer::new()),
                LoadBalancerStrategy::LeastConnections => {
                    Box::new(LeastConnectionsLoadBalancer::new())
                }
                LoadBalancerStrategy::WeightedRoundRobin => {
                    Box::new(WeightedRoundRobinLoadBalancer::new())
                }
                LoadBalancerStrategy::Random => Box::new(RoundRobinLoadBalancer::new()),
                LoadBalancerStrategy::IpHash => Box::new(RoundRobinLoadBalancer::new()),
                LoadBalancerStrategy::HealthBased => Box::new(RoundRobinLoadBalancer::new()),
                LoadBalancerStrategy::LatencyOptimized => Box::new(RoundRobinLoadBalancer::new()),
            };
            self.load_balancer = new_load_balancer;
        }

        self.config = config;
        Ok(())
    }

    /// Check if the manager has healthy instances
    pub async fn has_healthy_instances(&self) -> bool {
        let instances = self.instances.read().await;
        instances
            .iter()
            .any(|instance| instance.health.is_available())
    }

    /// Get healthy instances count
    pub async fn healthy_instances_count(&self) -> usize {
        let instances = self.instances.read().await;
        instances
            .data
            .iter()
            .filter(|instance| instance.health.is_available())
            .count()
    }

    /// Get total instances count
    pub async fn total_instances_count(&self) -> usize {
        let instances = self.instances.read().await;
        instances.len()
    }

    /// Perform health check on all instances
    pub fn health_check(SongbirdResult<Vec<(String, bool)>>) -> SongbirdResult<()> {
        let instances = self.instances.read().await;
        let mut health_results = Vec::new();

        for instance in instances.iter() {
            let is_healthy = instance.health.is_available();
            health_results.push((instance.id.clone(), is_healthy));

            // Update the load balancer's health tracking
            let _ = self
                .load_balancer
                .update_service_health(&instance.id, is_healthy)
                .await;
        }

        Ok(songbird_errors::evolved_success(health_results))
    }

    /// Get instance by ID
    pub async fn get_instance(&self) -> Option<ServiceInstance> {
        let instances = self.instances.read().await;
        instances
            .data
            .iter()
            .find(|instance| instance.id == instance_id)
            .cloned()
    }

    /// Update instance metadata
    pub async fn update_instance_metadata(&self) -> SongbirdResult<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances
            .iter_mut()
            .find(|instance| instance.id == instance_id)
        {
            instance.metadata.extend(metadata);
            Ok(songbird_errors::evolved_success(_))
        } else {
            Err(SongbirdError::internal_error(Service {
                service: "load_balancer".to_string(),
                message: format!(
                    "Instance {} not found. Verify the instance ID is correct",
                    instance_id
                ),
                suggested_alternatives: vec![],
                recovery_actions: vec!["check_instance_registry".to_string()],
            })
        }
    }

    /// Add service to load balancer
    pub fn add_service(SongbirdResult<SongbirdResponse<()>>) -> SongbirdResult<()> {
        // ... existing implementation ...

        info!("Service added to load balancer: {}", service_info.name);
        Ok(())
    }
}
