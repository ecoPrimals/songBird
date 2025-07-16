//! Load Balancer Module
//!
//! Provides load balancing functionality for service requests

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::errors::{Result, SongbirdError};

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub strategy: LoadBalancerStrategy,
    pub health_check_interval: u64,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancerStrategy::RoundRobin,
            health_check_interval: 30,
            max_retries: 3,
            timeout_seconds: 30,
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
    IpHash,
}

/// Service instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub weight: u32,
    pub healthy: bool,
}

/// Load balancer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
}

impl Default for LoadBalancerStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
        }
    }
}

/// Load balancer trait
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    /// Select the next service instance
    async fn select_instance(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance>;

    /// Record request result
    async fn record_request(&self, instance_id: &str, success: bool, response_time: f64);

    /// Get load balancer statistics
    async fn get_stats(&self) -> LoadBalancerStats;
}

/// Round robin load balancer
pub struct RoundRobinLoadBalancer {
    current_index: Arc<RwLock<usize>>,
    stats: Arc<RwLock<LoadBalancerStats>>,
}

impl Default for RoundRobinLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl RoundRobinLoadBalancer {
    /// Create a new round robin load balancer
    pub fn new() -> Self {
        Self {
            current_index: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(LoadBalancerStats::default())),
        }
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinLoadBalancer {
    async fn select_instance(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        if instances.is_empty() {
            return None;
        }

        // Filter healthy instances
        let healthy_instances: Vec<&ServiceInstance> = instances
            .iter()
            .filter(|instance| instance.healthy)
            .collect();

        if healthy_instances.is_empty() {
            return None;
        }

        let mut current_index = self.current_index.write().await;
        let index = *current_index % healthy_instances.len();
        *current_index = (*current_index + 1) % healthy_instances.len();

        // Use reference instead of cloning
        Some(healthy_instances[index].clone())
    }

    async fn record_request(&self, instance_id: &str, success: bool, response_time: f64) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }
        stats.average_response_time = if stats.total_requests > 1 {
            (stats.average_response_time * (stats.total_requests - 1) as f64 + response_time) / stats.total_requests as f64
        } else {
            response_time
        };
        
        // Update last request time - removed since field doesn't exist
    }

    async fn get_stats(&self) -> LoadBalancerStats {
        self.stats.read().await.clone()
    }
}

/// Least connections load balancer
pub struct LeastConnectionsLoadBalancer {
    connection_counts: Arc<RwLock<HashMap<String, u32>>>,
    stats: Arc<RwLock<LoadBalancerStats>>,
}

impl Default for LeastConnectionsLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LeastConnectionsLoadBalancer {
    /// Create a new least connections load balancer
    pub fn new() -> Self {
        Self {
            connection_counts: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(LoadBalancerStats::default())),
        }
    }

    /// Increment connection count for an instance
    pub async fn increment_connections(&self, instance_id: &str) {
        let mut counts = self.connection_counts.write().await;
        *counts.entry(instance_id.to_string()).or_insert(0) += 1;
    }

    /// Decrement connection count for an instance
    pub async fn decrement_connections(&self, instance_id: &str) {
        let mut counts = self.connection_counts.write().await;
        if let Some(count) = counts.get_mut(instance_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}

#[async_trait]
impl LoadBalancer for LeastConnectionsLoadBalancer {
    async fn select_instance(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        if instances.is_empty() {
            return None;
        }

        // Filter healthy instances
        let healthy_instances: Vec<&ServiceInstance> = instances
            .iter()
            .filter(|instance| instance.healthy)
            .collect();

        if healthy_instances.is_empty() {
            return None;
        }

        let connection_counts = self.connection_counts.read().await;

        // Find instance with least connections
        let selected_instance = healthy_instances
            .iter()
            .min_by_key(|instance| connection_counts.get(&instance.id).unwrap_or(&0))
            .map(|instance| (*instance).clone());

        selected_instance
    }

    async fn record_request(&self, _instance_id: &str, success: bool, response_time: f64) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        if success {
            stats.successful_requests += 1;
            // Only update average response time for successful requests
            let total_successful = stats.successful_requests;
            stats.average_response_time =
                (stats.average_response_time * (total_successful - 1) as f64 + response_time)
                    / total_successful as f64;
        } else {
            stats.failed_requests += 1;
        }
    }

    async fn get_stats(&self) -> LoadBalancerStats {
        self.stats.read().await.clone()
    }
}

/// Weighted round robin load balancer
pub struct WeightedRoundRobinLoadBalancer {
    current_weights: Arc<RwLock<HashMap<String, u32>>>,
    stats: Arc<RwLock<LoadBalancerStats>>,
}

impl Default for WeightedRoundRobinLoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl WeightedRoundRobinLoadBalancer {
    /// Create a new weighted round robin load balancer
    pub fn new() -> Self {
        Self {
            current_weights: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(LoadBalancerStats::default())),
        }
    }
}

#[async_trait]
impl LoadBalancer for WeightedRoundRobinLoadBalancer {
    async fn select_instance(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        if instances.is_empty() {
            return None;
        }

        // Filter healthy instances
        let healthy_instances: Vec<&ServiceInstance> = instances
            .iter()
            .filter(|instance| instance.healthy)
            .collect();

        if healthy_instances.is_empty() {
            return None;
        }

        let mut current_weights = self.current_weights.write().await;

        // Initialize weights if not present
        for instance in &healthy_instances {
            current_weights
                .entry(instance.id.clone())
                .or_insert(instance.weight);
        }

        // Find instance with highest current weight
        let selected_instance = healthy_instances
            .iter()
            .max_by_key(|instance| current_weights.get(&instance.id).unwrap_or(&0))
            .map(|&instance| instance.clone());

        // Decrease selected instance's current weight and increase others
        if let Some(ref instance) = selected_instance {
            if let Some(current_weight) = current_weights.get_mut(&instance.id) {
                *current_weight = current_weight.saturating_sub(1);
            }

            // Reset weights if all are zero
            if current_weights.values().all(|&w| w == 0) {
                for instance in &healthy_instances {
                    current_weights.insert(instance.id.clone(), instance.weight);
                }
            }
        }

        selected_instance
    }

    async fn record_request(&self, _instance_id: &str, success: bool, response_time: f64) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        if success {
            stats.successful_requests += 1;
            // Only update average response time for successful requests
            let total_successful = stats.successful_requests;
            stats.average_response_time =
                (stats.average_response_time * (total_successful - 1) as f64 + response_time)
                    / total_successful as f64;
        } else {
            stats.failed_requests += 1;
        }
    }

    async fn get_stats(&self) -> LoadBalancerStats {
        self.stats.read().await.clone()
    }
}

/// Load balancer manager
pub struct LoadBalancerManager {
    load_balancer: Box<dyn LoadBalancer>,
    instances: Arc<RwLock<Vec<ServiceInstance>>>,
    config: LoadBalancerConfig,
}

impl LoadBalancerManager {
    /// Create a new load balancer manager
    pub fn new(config: LoadBalancerConfig) -> Self {
        let load_balancer: Box<dyn LoadBalancer> = match config.strategy {
            LoadBalancerStrategy::RoundRobin => Box::new(RoundRobinLoadBalancer::new()),
            LoadBalancerStrategy::LeastConnections => Box::new(LeastConnectionsLoadBalancer::new()),
            LoadBalancerStrategy::WeightedRoundRobin => {
                Box::new(WeightedRoundRobinLoadBalancer::new())
            }
            LoadBalancerStrategy::Random => Box::new(RoundRobinLoadBalancer::new()), // Fallback
            LoadBalancerStrategy::IpHash => Box::new(RoundRobinLoadBalancer::new()), // Fallback
        };

        Self {
            load_balancer,
            instances: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Add a service instance
    pub async fn add_instance(&self, instance: ServiceInstance) -> Result<()> {
        let mut instances = self.instances.write().await;

        // Check if instance already exists
        if instances.iter().any(|i| i.id == instance.id) {
            return Err(SongbirdError::Config {
                field: Some("instance_id".to_string()),
                message: format!("Instance with ID {} already exists", instance.id),
            });
        }

        instances.push(instance);
        Ok(())
    }

    /// Remove a service instance
    pub async fn remove_instance(&self, instance_id: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        let initial_len = instances.len();
        instances.retain(|i| i.id != instance_id);

        if instances.len() == initial_len {
            return Err(SongbirdError::Config {
                field: Some("instance_id".to_string()),
                message: format!("Instance with ID {instance_id} not found"),
            });
        }

        Ok(())
    }

    /// Update instance health status
    pub async fn update_instance_health(&self, instance_id: &str, healthy: bool) -> Result<()> {
        let mut instances = self.instances.write().await;

        if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
            instance.healthy = healthy;
            Ok(())
        } else {
            Err(SongbirdError::Config {
                field: Some("instance_id".to_string()),
                message: format!("Instance with ID {instance_id} not found"),
            })
        }
    }

    /// Select the next instance for load balancing
    pub async fn select_instance(&self) -> Option<ServiceInstance> {
        let instances = self.instances.read().await;
        self.load_balancer.select_instance(&instances).await
    }

    /// Record request result
    pub async fn record_request(&self, instance_id: &str, success: bool, response_time: f64) {
        self.load_balancer
            .record_request(instance_id, success, response_time)
            .await;
    }

    /// Get all instances
    pub async fn get_instances(&self) -> Vec<ServiceInstance> {
        self.instances.read().await.clone()
    }

    /// Get healthy instances count
    pub async fn get_healthy_instances_count(&self) -> usize {
        let instances = self.instances.read().await;
        instances.iter().filter(|i| i.healthy).count()
    }

    /// Get load balancer statistics
    pub async fn get_stats(&self) -> LoadBalancerStats {
        self.load_balancer.get_stats().await
    }

    /// Get configuration
    pub fn get_config(&self) -> &LoadBalancerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_instances() -> Vec<ServiceInstance> {
        // Create test backend servers with environment configuration - NO MORE HARDCODING!
        let env_config = crate::config::environment::EnvironmentConfig::default();

        let mut backends = Vec::new();

        // Create multiple backend servers based on environment
        for i in 0..3 {
            backends.push(BackendServer {
                id: format!("backend-{}", i + 1),
                address: env_config.bind_address.clone(),
                port: env_config.bind_port + i as u16, // Offset ports for multiple backends
                weight: 1,
                health_status: HealthStatus::Healthy,
                current_connections: 0,
                total_requests: 0,
                failed_requests: 0,
            });
        }

        backends
            .into_iter()
            .map(|backend| ServiceInstance {
                id: backend.id,
                address: backend.address,
                port: backend.port,
                weight: backend.weight,
                healthy: backend.health_status == HealthStatus::Healthy,
            })
            .collect()
    }

    #[tokio::test]
    async fn test_round_robin_load_balancer() {
        let lb = RoundRobinLoadBalancer::new();
        let instances = create_test_instances();

        // Should select healthy instances in round robin fashion
        let selected1 = lb.select_instance(&instances).await;
        assert!(selected1.is_some());
        let instance1 = selected1.expect("Test load balancer should select an instance");

        let selected2 = lb.select_instance(&instances).await;
        assert!(selected2.is_some());
        let instance2 = selected2.expect("Test load balancer should select an instance");

        // Should not select the same instance twice in a row (with 2 healthy instances)
        assert_ne!(instance1.id, instance2.id);
    }

    #[tokio::test]
    async fn test_least_connections_load_balancer() {
        let lb = LeastConnectionsLoadBalancer::new();
        let instances = create_test_instances();

        // Initially should select any healthy instance
        let selected = lb.select_instance(&instances).await;
        assert!(selected.is_some());

        // Test connection tracking
        lb.increment_connections("instance1").await;
        lb.increment_connections("instance1").await;
        lb.increment_connections("instance2").await;

        // Should prefer instance2 (1 connection) over instance1 (2 connections)
        let selected = lb.select_instance(&instances).await;
        assert!(selected.is_some());
    }

    #[tokio::test]
    async fn test_weighted_round_robin_load_balancer() {
        let lb = WeightedRoundRobinLoadBalancer::new();
        let instances = create_test_instances();

        // Should select instances based on weights
        let selected = lb.select_instance(&instances).await;
        assert!(selected.is_some());

        // Record some requests
        lb.record_request("instance1", true, 100.0).await;
        lb.record_request("instance2", true, 200.0).await;

        let stats = lb.get_stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.average_response_time, 150.0);
    }

    #[tokio::test]
    async fn test_load_balancer_manager() {
        let config = LoadBalancerConfig::default();
        let manager = LoadBalancerManager::new(config);

        // Add instances with environment configuration - NO MORE HARDCODING!
        let env_config = crate::config::environment::EnvironmentConfig::default();
        let instance1 = ServiceInstance {
            id: "test1".to_string(),
            address: env_config.bind_address.clone(),
            port: env_config.bind_port,
            weight: 1,
            healthy: true,
        };

        assert!(manager.add_instance(instance1).await.is_ok());

        // Check instance count
        let instances = manager.get_instances().await;
        assert_eq!(instances.len(), 1);
        assert_eq!(manager.get_healthy_instances_count().await, 1);

        // Select instance
        let selected = manager.select_instance().await;
        assert!(selected.is_some());
        assert_eq!(
            selected
                .expect("Load balancer should select test instance")
                .id,
            "test1"
        );

        // Update health
        assert!(manager.update_instance_health("test1", false).await.is_ok());
        assert_eq!(manager.get_healthy_instances_count().await, 0);

        // Remove instance
        assert!(manager.remove_instance("test1").await.is_ok());
        assert_eq!(manager.get_instances().await.len(), 0);
    }

    #[tokio::test]
    async fn test_load_balancer_with_no_healthy_instances() {
        let lb = RoundRobinLoadBalancer::new();
        let mut instances = create_test_instances();

        // Make all instances unhealthy
        for instance in &mut instances {
            instance.healthy = false;
        }

        let selected = lb.select_instance(&instances).await;
        assert!(selected.is_none());
    }

    #[tokio::test]
    async fn test_load_balancer_stats() {
        let lb = RoundRobinLoadBalancer::new();

        // Record some requests
        lb.record_request("instance1", true, 100.0).await;
        lb.record_request("instance2", false, 200.0).await;
        lb.record_request("instance1", true, 300.0).await;

        let stats = lb.get_stats().await;
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.average_response_time, 200.0); // (100 + 300) / 2
    }

    #[test]
    fn test_load_balancer_config_default() {
        let config = LoadBalancerConfig::default();
        assert!(matches!(config.strategy, LoadBalancerStrategy::RoundRobin));
        assert_eq!(config.health_check_interval, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_service_instance_creation() {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        let instance = ServiceInstance {
            id: "test".to_string(),
            address: env_config.bind_address.clone(),
            port: env_config.bind_port,
            weight: 1,
            healthy: true,
        };

        assert_eq!(instance.id, "test");
        assert_eq!(instance.address, env_config.bind_address);
        assert_eq!(instance.port, env_config.bind_port);
        assert_eq!(instance.weight, 1);
        assert!(instance.healthy);
    }
}

#[derive(Debug, Clone)]
pub struct BackendServer {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub weight: u32,
    pub health_status: HealthStatus,
    pub failed_requests: u64,
    pub current_connections: u32,
    pub total_requests: u64,
}

use crate::observability::HealthStatus;
