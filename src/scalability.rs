use crate::errors::SongbirdError;
use crate::traits::service::ServiceInfo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub service_info: ServiceInfo,
    pub instance_id: String,
    pub weight: u32,
    pub current_connections: u32,
    pub is_healthy: bool,
    pub last_health_check: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceScalingConfig {
    pub min_instances: usize,
    pub max_instances: usize,
    pub target_cpu_utilization: f64,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub cooldown_period: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct ScalabilityStats {
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub avg_cpu_utilization: f64,
    pub avg_memory_utilization: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ResourcePool {
    pub max_cpu_cores: u32,
    pub max_memory_mb: u64,
    pub available_cpu_cores: u32,
    pub available_memory_mb: u64,
    pub allocated_instances: HashMap<String, ResourceUsage>,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub cpu_percentage: f64,
    pub memory_usage_mb: u64,
    pub network_bytes_per_sec: u64,
    pub disk_io_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu_request: f64,
    pub cpu_limit: f64,
    pub memory_request_mb: u64,
    pub memory_limit_mb: u64,
    pub disk_limit_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub response_time_threshold_ms: u64,
    pub throughput_threshold_rps: u64,
    pub error_rate_threshold: f64,
    pub monitoring_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_response_time_ms: u64,
    pub throughput_rps: u64,
    pub error_rate: f64,
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_response_time_ms: u64,
    pub min_throughput_rps: u64,
    pub max_error_rate: f64,
    pub max_cpu_utilization: f64,
    pub max_memory_utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    Random,
    HealthAware,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub algorithm: LoadBalancingAlgorithm,
    pub health_check_enabled: bool,
    pub health_check_interval: Duration,
    pub session_affinity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingStrategy {
    Manual,
    Automatic,
    Predictive,
    Reactive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAction {
    ScaleUp,
    ScaleDown,
    NoAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingDecision {
    pub action: ScalingAction,
    pub target_instances: usize,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceHealth {
    Healthy,
    Unhealthy,
    Unknown,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityConfig {
    pub enabled: bool,
    pub strategy: ScalingStrategy,
    pub performance_config: PerformanceConfig,
    pub resource_config: ResourceConfig,
    pub load_balancing_config: LoadBalancingConfig,
    pub thresholds: PerformanceThresholds,
}

#[derive(Debug, Clone, Default)]
pub struct ScalingGroup {
    pub service_id: String,
    pub instances: Vec<ServiceInstance>,
    pub config: ServiceScalingConfig,
    pub resource_pool: ResourcePool,
    pub metrics: ScalabilityStats,
}

impl ScalingGroup {
    pub fn new(service_id: String, config: ServiceScalingConfig) -> Self {
        Self {
            service_id,
            instances: Vec::new(),
            config,
            resource_pool: ResourcePool::default(),
            metrics: ScalabilityStats {
                total_instances: 0,
                healthy_instances: 0,
                avg_cpu_utilization: 0.0,
                avg_memory_utilization: 0.0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn push(&mut self, instance: ServiceInstance) {
        self.instances.push(instance);
    }

    pub fn pop(&mut self) -> Option<ServiceInstance> {
        self.instances.pop()
    }

    pub async fn add_instance(&mut self, instance: ServiceInstance) -> Result<(), SongbirdError> {
        if self.instances.len() >= self.config.max_instances {
            return Err(SongbirdError::Configuration {
                field: "max_instances".to_string(),
                message: "Maximum instances reached".to_string(),
            });
        }
        self.instances.push(instance);
        self.metrics.total_instances = self.instances.len();
        Ok(())
    }

    pub async fn remove_instance(&mut self, instance_id: &str) -> Result<(), SongbirdError> {
        self.instances.retain(|i| i.instance_id != instance_id);
        self.metrics.total_instances = self.instances.len();
        Ok(())
    }

    pub fn get_healthy_instances(&self) -> Vec<&ServiceInstance> {
        self.instances.iter().filter(|i| i.is_healthy).collect()
    }
}

pub struct ScalabilityManager {
    pub config: ScalabilityConfig,
    pub scaling_groups: HashMap<String, ScalingGroup>,
    pub stats: ScalabilityStats,
}

impl ScalabilityManager {
    pub fn new(config: ScalabilityConfig) -> Self {
        Self {
            config,
            scaling_groups: HashMap::new(),
            stats: ScalabilityStats {
                total_instances: 0,
                healthy_instances: 0,
                avg_cpu_utilization: 0.0,
                avg_memory_utilization: 0.0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
            },
        }
    }

    pub async fn add_scaling_group(
        &mut self,
        service_id: String,
        group: ScalingGroup,
    ) -> Result<(), SongbirdError> {
        self.scaling_groups.insert(service_id, group);
        Ok(())
    }

    pub async fn make_scaling_decision(
        &self,
        service_id: &str,
        metrics: &PerformanceMetrics,
    ) -> Result<ScalingDecision, SongbirdError> {
        let group = self.scaling_groups.get(service_id).ok_or_else(|| {
            SongbirdError::Configuration {
                field: "service_id".to_string(),
                message: format!("Service {} not found", service_id),
            }
        })?;

        let current_instances = group.instances.len();
        let healthy_instances = group.get_healthy_instances().len();

        let action = if metrics.cpu_utilization > self.config.thresholds.max_cpu_utilization
            && current_instances < group.config.max_instances
        {
            ScalingAction::ScaleUp
        } else if metrics.cpu_utilization
            < self.config.performance_config.response_time_threshold_ms as f64 * 0.5
            && current_instances > group.config.min_instances
        {
            ScalingAction::ScaleDown
        } else {
            ScalingAction::NoAction
        };

        let target_instances = match action {
            ScalingAction::ScaleUp => (current_instances + 1).min(group.config.max_instances),
            ScalingAction::ScaleDown => (current_instances - 1).max(group.config.min_instances),
            ScalingAction::NoAction => current_instances,
        };

        Ok(ScalingDecision {
            action,
            target_instances,
            reason: format!(
                "CPU: {:.2}%, Instances: {}/{}",
                metrics.cpu_utilization, healthy_instances, current_instances
            ),
            timestamp: Utc::now(),
        })
    }

    pub async fn get_stats(&self) -> Result<ScalabilityStats, SongbirdError> {
        Ok(self.stats.clone())
    }

    pub async fn scale_up(&mut self, service_id: &str) -> Result<(), SongbirdError> {
        let mut instances = self.scaling_groups.get(service_id).cloned().unwrap_or_default();

        if instances.is_empty() {
            return Err(SongbirdError::Configuration {
                field: "service_id".to_string(),
                message: format!("Service {} not found", service_id),
            });
        }

        // Create new instance
        let instance_id = format!("{}_{}", service_id, instances.len() + 1);
        let instance = ServiceInstance {
            service_info: ServiceInfo {
                id: service_id.to_string(),
                name: service_id.to_string(),
                version: "1.0.0".to_string(),
                service_type: "hpc-service".to_string(),
                description: format!("HPC service instance for {}", service_id),
                endpoints: vec![],
                capabilities: vec!["compute".to_string(), "storage".to_string()],
                tags: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
            },
            instance_id: instance_id.clone(),
            weight: 1,
            current_connections: 0,
            is_healthy: true,
            last_health_check: Some(chrono::Utc::now()),
        };

        instances.push(instance);
        self.scaling_groups.insert(service_id.to_string(), instances);

        tracing::info!("Scaled up service {}: new instance {}", service_id, instance_id);
        Ok(())
    }

    /// Scale down a service by removing an instance
    pub async fn scale_down(&mut self, service_id: &str) -> Result<(), SongbirdError> {
        let mut instances = self.scaling_groups.get(service_id).cloned().unwrap_or_default();

        if instances.is_empty() {
            return Err(SongbirdError::Configuration {
                field: "service_id".to_string(),
                message: format!("Service {} not found", service_id),
            });
        }

        // Remove the last instance
        instances.pop();
        self.scaling_groups.insert(service_id.to_string(), instances);

        tracing::info!("Scaled down service {}: removed last instance", service_id);
        Ok(())
    }
}
