//! Scalability Module
//!
//! Provides auto-scaling capabilities for services


use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::errors::{Result, SongbirdError};

/// Service scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceScalingConfig {
    pub min_instances: u32,
    pub max_instances: u32,
    pub target_cpu_percent: f64,
    pub target_memory_percent: f64,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
}

/// Scalability statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityStats {
    pub total_scale_events: u64,
    pub scale_up_events: u64,
    pub scale_down_events: u64,
    pub average_response_time: f64,
    pub current_load: f64,
    pub resource_utilization: ResourceUsage,
}

/// Resource pool for managing compute resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub total_cpu_cores: u32,
    pub total_memory_mb: u32,
    pub available_cpu_cores: u32,
    pub available_memory_mb: u32,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub network_io_mbps: f64,
    pub disk_io_mbps: f64,
}

/// Resource configuration for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu_request: f64,
    pub memory_request_mb: u32,
    pub cpu_limit: f64,
    pub memory_limit_mb: u32,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_requests: u32,
    pub request_timeout_ms: u64,
    pub connection_pool_size: u32,
    pub cache_size_mb: u32,
}

impl Default for ServiceScalingConfig {
    fn default() -> Self {
        Self {
            min_instances: 1,
            max_instances: 10,
            target_cpu_percent: 70.0,
            target_memory_percent: 80.0,
            scale_up_threshold: 80.0,
            scale_down_threshold: 30.0,
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_percent: 0.0,
            network_io_mbps: 0.0,
            disk_io_mbps: 0.0,
        }
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            cpu_request: 0.5,
            memory_request_mb: 512,
            cpu_limit: 1.0,
            memory_limit_mb: 1024,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            request_timeout_ms: 30000,
            connection_pool_size: 10,
            cache_size_mb: 128,
        }
    }
}

/// Auto-scaler for managing service instances
pub struct AutoScaler {
    config: ServiceScalingConfig,
    stats: ScalabilityStats,
    resource_pool: ResourcePool,
    scaling_history: Vec<ScalingEvent>,
    last_scaling_time: Option<Instant>,
    cooldown_period: Duration,
}

/// Scaling decision
#[derive(Debug, Clone)]
pub enum ScalingDecision {
    ScaleUp(u32),
    ScaleDown(u32),
    NoAction,
}

/// Scaling event for history tracking
#[derive(Debug, Clone)]
pub struct ScalingEvent {
    pub timestamp: Instant,
    pub service_id: String,
    pub decision: ScalingDecision,
    pub reason: String,
    pub current_instances: u32,
    pub target_instances: u32,
}

impl AutoScaler {
    /// Create a new auto-scaler
    pub fn new(config: ServiceScalingConfig, resource_pool: ResourcePool) -> Self {
        Self {
            config,
            stats: ScalabilityStats {
                total_scale_events: 0,
                scale_up_events: 0,
                scale_down_events: 0,
                average_response_time: 0.0,
                current_load: 0.0,
                resource_utilization: ResourceUsage::default(),
            },
            resource_pool,
            scaling_history: Vec::new(),
            last_scaling_time: None,
            cooldown_period: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Evaluate scaling decision based on current metrics
    pub fn evaluate_scaling(
        &mut self,
        service_id: &str,
        current_instances: u32,
        resource_usage: &ResourceUsage,
        request_rate: f64,
    ) -> Result<ScalingDecision> {
        // Check cooldown period
        if let Some(last_time) = self.last_scaling_time {
            if last_time.elapsed() < self.cooldown_period {
                return Ok(ScalingDecision::NoAction);
            }
        }

        // Determine scaling decision based on resource usage
        let decision = if resource_usage.cpu_percent > self.config.scale_up_threshold
            || resource_usage.memory_percent > self.config.scale_up_threshold
        {
            if current_instances < self.config.max_instances {
                let scale_factor = self.calculate_scale_factor(resource_usage, request_rate);
                let target_instances = (current_instances + scale_factor).min(self.config.max_instances);
                ScalingDecision::ScaleUp(target_instances - current_instances)
            } else {
                ScalingDecision::NoAction
            }
        } else if resource_usage.cpu_percent < self.config.scale_down_threshold
            && resource_usage.memory_percent < self.config.scale_down_threshold
        {
            if current_instances > self.config.min_instances {
                ScalingDecision::ScaleDown(1)
            } else {
                ScalingDecision::NoAction
            }
        } else {
            ScalingDecision::NoAction
        };

        // Record decision in history for evaluation tracking (stats updated on execution)
        let _event = ScalingEvent {
            timestamp: Instant::now(),
            service_id: service_id.to_string(),
            decision: decision.clone(),
            reason: self.generate_scaling_reason(&decision, resource_usage),
            current_instances,
            target_instances: match decision {
                ScalingDecision::ScaleUp(instances) => current_instances + instances,
                ScalingDecision::ScaleDown(instances) => current_instances.saturating_sub(instances),
                ScalingDecision::NoAction => current_instances,
            },
        };
        // Don't add to history until actually executed
        // self.scaling_history.push(_event);
        self.last_scaling_time = Some(Instant::now());

        // Update current metrics
        self.stats.resource_utilization = resource_usage.clone();
        self.stats.current_load = request_rate;

        Ok(decision)
    }

    /// Calculate scale factor based on resource usage
    fn calculate_scale_factor(&self, resource_usage: &ResourceUsage, request_rate: f64) -> u32 {
        // Simple scaling algorithm - can be made more sophisticated
        let cpu_factor = if resource_usage.cpu_percent > 90.0 { 2 } else { 1 };
        let memory_factor = if resource_usage.memory_percent > 90.0 { 2 } else { 1 };
        let load_factor = if request_rate > 1000.0 { 2 } else { 1 };

        cpu_factor.max(memory_factor).max(load_factor)
    }

    /// Generate reason for scaling decision
    fn generate_scaling_reason(&self, decision: &ScalingDecision, resource_usage: &ResourceUsage) -> String {
        match decision {
            ScalingDecision::ScaleUp(_) => {
                format!(
                    "High resource usage: CPU {}%, Memory {}%",
                    resource_usage.cpu_percent,
                    resource_usage.memory_percent
                )
            }
            ScalingDecision::ScaleDown(_) => {
                format!(
                    "Low resource usage: CPU {}%, Memory {}%",
                    resource_usage.cpu_percent,
                    resource_usage.memory_percent
                )
            }
            ScalingDecision::NoAction => "No scaling required".to_string(),
        }
    }

    /// Execute scaling action
    pub async fn execute_scaling(
        &mut self,
        service_id: &str,
        decision: &ScalingDecision,
    ) -> Result<()> {
        match decision {
            ScalingDecision::ScaleUp(instances) => {
                info!("Scaling up service {}: adding {} instances", service_id, instances);
                self.scale_up_service(service_id, *instances).await?;
                
                // Update statistics
                self.stats.scale_up_events += 1;
                self.stats.total_scale_events += 1;
                
                // Record scaling event
                let event = ScalingEvent {
                    timestamp: Instant::now(),
                    service_id: service_id.to_string(),
                    decision: decision.clone(),
                    reason: self.generate_scaling_reason(decision, &ResourceUsage::default()),
                    current_instances: 0, // This would come from service registry in real implementation
                    target_instances: *instances,
                };
                self.scaling_history.push(event);
                self.last_scaling_time = Some(Instant::now());
            }
            ScalingDecision::ScaleDown(instances) => {
                info!("Scaling down service {}: removing {} instances", service_id, instances);
                self.scale_down_service(service_id, *instances).await?;
                
                // Update statistics
                self.stats.scale_down_events += 1;
                self.stats.total_scale_events += 1;
                
                // Record scaling event
                let event = ScalingEvent {
                    timestamp: Instant::now(),
                    service_id: service_id.to_string(),
                    decision: decision.clone(),
                    reason: self.generate_scaling_reason(decision, &ResourceUsage::default()),
                    current_instances: 0, // This would come from service registry in real implementation
                    target_instances: 0_u32.saturating_sub(*instances),
                };
                self.scaling_history.push(event);
                self.last_scaling_time = Some(Instant::now());
            }
            ScalingDecision::NoAction => {
                // Do nothing
            }
        }

        Ok(())
    }

    /// Scale up service instances
    async fn scale_up_service(&mut self, service_id: &str, instances: u32) -> Result<()> {
        // Check if we have enough resources
        let required_cpu = self.config.min_instances as f64 * 0.5; // Assume 0.5 CPU per instance
        let required_memory = self.config.min_instances * 512; // Assume 512MB per instance

        if (self.resource_pool.available_cpu_cores as f64) < required_cpu {
            return Err(SongbirdError::Service {
                service: service_id.to_string(),
                message: format!("Insufficient CPU resources: need {:.1} cores, have {}", 
                    required_cpu, self.resource_pool.available_cpu_cores),
            });
        }

        if self.resource_pool.available_memory_mb < required_memory {
            return Err(SongbirdError::Service {
                service: service_id.to_string(),
                message: format!("Insufficient memory resources: need {}MB, have {}MB", 
                    required_memory, self.resource_pool.available_memory_mb),
            });
        }

        // Reserve resources
        self.resource_pool.available_cpu_cores -= (required_cpu as u32).min(self.resource_pool.available_cpu_cores);
        self.resource_pool.available_memory_mb -= required_memory.min(self.resource_pool.available_memory_mb);

        info!("Scaled up service {}: added {} instances", service_id, instances);
        Ok(())
    }

    /// Scale down service instances
    async fn scale_down_service(&mut self, service_id: &str, instances: u32) -> Result<()> {
        // Free up resources
        let freed_cpu = instances as f64 * 0.5; // Assume 0.5 CPU per instance
        let freed_memory = instances * 512; // Assume 512MB per instance

        self.resource_pool.available_cpu_cores += freed_cpu as u32;
        self.resource_pool.available_memory_mb += freed_memory;

        // Ensure we don't exceed total resources
        self.resource_pool.available_cpu_cores = self.resource_pool.available_cpu_cores
            .min(self.resource_pool.total_cpu_cores);
        self.resource_pool.available_memory_mb = self.resource_pool.available_memory_mb
            .min(self.resource_pool.total_memory_mb);

        info!("Scaled down service {}: removed {} instances", service_id, instances);
        Ok(())
    }

    /// Get scaling statistics
    pub fn get_stats(&self) -> &ScalabilityStats {
        &self.stats
    }

    /// Get scaling history
    pub fn get_scaling_history(&self) -> &[ScalingEvent] {
        &self.scaling_history
    }

    /// Get resource pool status
    pub fn get_resource_pool(&self) -> &ResourcePool {
        &self.resource_pool
    }

    /// Update resource pool
    pub fn update_resource_pool(&mut self, resource_pool: ResourcePool) {
        self.resource_pool = resource_pool;
    }

    /// Set cooldown period
    pub fn set_cooldown_period(&mut self, duration: Duration) {
        self.cooldown_period = duration;
    }

    /// Set last scaling time (for testing purposes)
    pub fn set_last_scaling_time(&mut self, time: Option<Instant>) {
        self.last_scaling_time = time;
    }
}

/// Performance optimizer for service configurations
pub struct PerformanceOptimizer {
    performance_config: PerformanceConfig,
    optimization_history: Vec<OptimizationEvent>,
}

/// Optimization event
#[derive(Debug, Clone)]
pub struct OptimizationEvent {
    pub timestamp: Instant,
    pub service_id: String,
    pub optimization_type: OptimizationType,
    pub old_value: f64,
    pub new_value: f64,
    pub improvement_percent: f64,
}

/// Types of optimizations
#[derive(Debug, Clone)]
pub enum OptimizationType {
    ConnectionPoolSize,
    CacheSize,
    RequestTimeout,
    ConcurrentRequests,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(performance_config: PerformanceConfig) -> Self {
        Self {
            performance_config,
            optimization_history: Vec::new(),
        }
    }

    /// Optimize performance based on metrics
    pub fn optimize_performance(
        &mut self,
        service_id: &str,
        metrics: &PerformanceMetrics,
    ) -> Result<Vec<OptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Optimize connection pool size
        if let Some(recommendation) = self.optimize_connection_pool(metrics) {
            recommendations.push(recommendation);
        }

        // Optimize cache size
        if let Some(recommendation) = self.optimize_cache_size(metrics) {
            recommendations.push(recommendation);
        }

        // Optimize request timeout
        if let Some(recommendation) = self.optimize_request_timeout(metrics) {
            recommendations.push(recommendation);
        }

        // Record optimization events
        for recommendation in &recommendations {
            let event = OptimizationEvent {
                timestamp: Instant::now(),
                service_id: service_id.to_string(),
                optimization_type: recommendation.optimization_type.clone(),
                old_value: recommendation.current_value,
                new_value: recommendation.recommended_value,
                improvement_percent: recommendation.expected_improvement,
            };
            self.optimization_history.push(event);
        }

        Ok(recommendations)
    }

    /// Optimize connection pool size
    fn optimize_connection_pool(&self, metrics: &PerformanceMetrics) -> Option<OptimizationRecommendation> {
        if metrics.connection_pool_utilization > 0.9 {  // 90% utilization
            let new_size = (self.performance_config.connection_pool_size as f64 * 1.5) as u32;
            Some(OptimizationRecommendation {
                optimization_type: OptimizationType::ConnectionPoolSize,
                current_value: self.performance_config.connection_pool_size as f64,
                recommended_value: new_size as f64,
                expected_improvement: 20.0,
                reason: "High connection pool utilization detected".to_string(),
            })
        } else if metrics.connection_pool_utilization < 0.3 {  // 30% utilization
            let new_size = (self.performance_config.connection_pool_size as f64 * 0.7) as u32;
            Some(OptimizationRecommendation {
                optimization_type: OptimizationType::ConnectionPoolSize,
                current_value: self.performance_config.connection_pool_size as f64,
                recommended_value: new_size as f64,
                expected_improvement: 10.0,
                reason: "Low connection pool utilization detected".to_string(),
            })
        } else {
            None
        }
    }

    /// Optimize cache size
    fn optimize_cache_size(&self, metrics: &PerformanceMetrics) -> Option<OptimizationRecommendation> {
        if metrics.cache_hit_rate < 0.7 {  // 70% cache hit rate
            let new_size = (self.performance_config.cache_size_mb as f64 * 1.3) as u32;
            Some(OptimizationRecommendation {
                optimization_type: OptimizationType::CacheSize,
                current_value: self.performance_config.cache_size_mb as f64,
                recommended_value: new_size as f64,
                expected_improvement: 15.0,
                reason: "Low cache hit rate detected".to_string(),
            })
        } else {
            None
        }
    }

    /// Optimize request timeout
    fn optimize_request_timeout(&self, metrics: &PerformanceMetrics) -> Option<OptimizationRecommendation> {
        if metrics.timeout_rate > 0.05 {  // 5% timeout rate
            let new_timeout = (self.performance_config.request_timeout_ms as f64 * 1.2) as u64;
            Some(OptimizationRecommendation {
                optimization_type: OptimizationType::RequestTimeout,
                current_value: self.performance_config.request_timeout_ms as f64,
                recommended_value: new_timeout as f64,
                expected_improvement: 8.0,
                reason: "High timeout rate detected".to_string(),
            })
        } else {
            None
        }
    }

    /// Get optimization history
    pub fn get_optimization_history(&self) -> &[OptimizationEvent] {
        &self.optimization_history
    }
}

/// Performance metrics for optimization
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub average_response_time: f64,
    pub request_rate: f64,
    pub error_rate: f64,
    pub timeout_rate: f64,
    pub connection_pool_utilization: f64,
    pub cache_hit_rate: f64,
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub optimization_type: OptimizationType,
    pub current_value: f64,
    pub recommended_value: f64,
    pub expected_improvement: f64,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_resource_pool() -> ResourcePool {
        ResourcePool {
            total_cpu_cores: 16,
            total_memory_mb: 32768,
            available_cpu_cores: 8,
            available_memory_mb: 16384,
        }
    }

    fn create_test_resource_usage(cpu: f64, memory: f64) -> ResourceUsage {
        ResourceUsage {
            cpu_percent: cpu,
            memory_percent: memory,
            network_io_mbps: 10.0,
            disk_io_mbps: 5.0,
        }
    }

    #[test]
    fn test_auto_scaler_creation() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let scaler = AutoScaler::new(config, resource_pool);

        assert_eq!(scaler.stats.total_scale_events, 0);
        assert!(scaler.scaling_history.is_empty());
    }

    #[test]
    fn test_scaling_decision_scale_up() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);

        let high_usage = create_test_resource_usage(85.0, 85.0);
        let decision = scaler.evaluate_scaling("test-service", 2, &high_usage, 100.0).expect("Failed to evaluate scaling decision");

        assert!(matches!(decision, ScalingDecision::ScaleUp(_)), 
            "Expected scale up decision, got: {:?}", decision);
    }

    #[test]
    fn test_scaling_decision_scale_down() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);

        let low_usage = create_test_resource_usage(20.0, 25.0);
        let decision = scaler.evaluate_scaling("test-service", 3, &low_usage, 10.0).expect("Failed to evaluate scaling decision");

        assert!(matches!(decision, ScalingDecision::ScaleDown(_)), 
            "Expected scale down decision, got: {:?}", decision);
    }

    #[test]
    fn test_scaling_decision_no_action() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);

        let normal_usage = create_test_resource_usage(50.0, 60.0);
        let decision = scaler.evaluate_scaling("test-service", 2, &normal_usage, 50.0).expect("Failed to evaluate scaling decision");

        assert!(matches!(decision, ScalingDecision::NoAction), 
            "Expected no action decision, got: {:?}", decision);
    }

    #[tokio::test]
    async fn test_scale_up_execution() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);

        let decision = ScalingDecision::ScaleUp(2);
        let result = scaler.execute_scaling("test-service", &decision).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scale_down_execution() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);

        let decision = ScalingDecision::ScaleDown(1);
        let result = scaler.execute_scaling("test-service", &decision).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_performance_optimizer() {
        let config = PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config);

        let metrics = PerformanceMetrics {
            average_response_time: 200.0,
            request_rate: 100.0,
            error_rate: 1.0,
            timeout_rate: 2.0,
            connection_pool_utilization: 95.0,
            cache_hit_rate: 60.0,
            memory_usage_percent: 70.0,
            cpu_usage_percent: 80.0,
        };

        let recommendations = optimizer.optimize_performance("test-service", &metrics).expect("Failed to optimize performance");
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_config_defaults() {
        let scaling_config = ServiceScalingConfig::default();
        assert_eq!(scaling_config.min_instances, 1);
        assert_eq!(scaling_config.max_instances, 10);
        assert_eq!(scaling_config.target_cpu_percent, 70.0);

        let resource_config = ResourceConfig::default();
        assert_eq!(resource_config.cpu_request, 0.5);
        assert_eq!(resource_config.memory_request_mb, 512);

        let performance_config = PerformanceConfig::default();
        assert_eq!(performance_config.max_concurrent_requests, 100);
        assert_eq!(performance_config.request_timeout_ms, 30000);
    }
} 