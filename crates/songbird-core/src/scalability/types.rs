//! Scalability Types and Configurations
//!
//! Type definitions and data structures for auto-scaling services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

/// Resource configuration for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu_request: f64,
    pub memory_request_mb: u32,
    pub cpu_limit: f64,
    pub memory_limit_mb: u32,
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

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_requests: u32,
    pub request_timeout_ms: u64,
    pub connection_pool_size: u32,
    pub cache_size_mb: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 1000,
            request_timeout_ms: 30000,
            connection_pool_size: 50,
            cache_size_mb: 128,
        }
    }
}

/// Scalability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityConfig {
    pub min_instances: u32,
    pub max_instances: u32,
    pub target_cpu_threshold: f64,
    pub target_memory_threshold: f64,
    pub scale_up_cooldown: Duration,
    pub scale_down_cooldown: Duration,
    pub metrics_interval: Duration,
    pub enable_predictive_scaling: bool,
}

impl Default for ScalabilityConfig {
    fn default() -> Self {
        Self {
            min_instances: 1,
            max_instances: 10,
            target_cpu_threshold: 70.0,
            target_memory_threshold: 80.0,
            scale_up_cooldown: Duration::from_secs(60),
            scale_down_cooldown: Duration::from_secs(120),
            metrics_interval: Duration::from_secs(30),
            enable_predictive_scaling: false,
        }
    }
}

/// Scaling action type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScalingActionType {
    ScaleUp,
    ScaleDown,
    NoAction,
}

/// Scaling action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingAction {
    pub action_type: ScalingActionType,
    pub target_instances: u32,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

/// Scaling decision
#[derive(Debug, Clone)]
pub enum ScalingDecision {
    ScaleUp(u32),
    ScaleDown(u32),
    NoAction,
}

/// Direction of scaling operation
#[derive(Debug, Clone, PartialEq)]
pub enum ScaleDirection {
    Up,
    Down,
    None,
}

/// Scaling event for history tracking
#[derive(Debug, Clone)]
pub struct ScalingEvent {
    pub timestamp: DateTime<Utc>,
    pub service_id: String,
    pub decision: ScalingDecision,
    pub reason: String,
    pub current_instances: u32,
    pub target_instances: u32,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Optimization event
#[derive(Debug, Clone)]
pub struct OptimizationEvent {
    pub timestamp: DateTime<Utc>,
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

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub optimization_type: OptimizationType,
    pub current_value: f64,
    pub recommended_value: f64,
    pub expected_improvement: f64,
    pub reason: String,
}
