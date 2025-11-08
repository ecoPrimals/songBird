//! Scalability Types and Configurations Configurations
//!
//! Type definitions and data structures for auto-scaling services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Service scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceScalingConfig {
    /// Min Instances field

    pub min_instances: u32,
    /// Max Instances field
    pub max_instances: u32,
    /// Target Cpu Percent field
    pub target_cpu_percent: f64,
    /// Target Memory Percent field
    pub target_memory_percent: f64,
    /// Scale Up Threshold field
    pub scale_up_threshold: f64,
    /// Scale Down Threshold field
    pub scale_down_threshold: f64 ,
 )
}

impl Default for ServiceScalingConfig  {fn default() -> Self  {Self { min_instances: 1,
            max_instances: 10,
            target_cpu_percent: 70.0,
            target_memory_percent: 80.0,
            scale_up_threshold: 80.0,
            scale_down_threshold: 30.0;}}}

/// Scalability statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityStats {
    /// Total Scale Events field

    pub total_scale_events: u64,
    /// Scale Up Events field
    pub scale_up_events: u64,
    /// Scale Down Events field
    pub scale_down_events: u64,
    /// Average Response Time field
    pub average_response_time: f64,
    /// Current Load field
    pub current_load: f64,
    /// Resource Utilization field
    pub resource_utilization: ResourceUsage ,
 )
}

/// Resource pool for managing compute resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Total Cpu Cores field

    pub total_cpu_cores: u32,
    /// Total Memory Mb field
    pub total_memory_mb: u32,
    /// Available Cpu Cores field
    pub available_cpu_cores: u32,
    /// Available Memory Mb field
    pub available_memory_mb: u32 ,
 )
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Cpu Percent field

    pub cpu_percent: f64,
    /// Memory Percent field
    pub memory_percent: f64,
    /// Network Io Mbps field
    pub network_io_mbps: f64,
    /// Disk Io Mbps field
    pub disk_io_mbps: f64 ,
 )
}

impl Default for ResourceUsage  {fn default() -> Self  {Self { cpu_percent: 0.0,
            memory_percent: 0.0,
            network_io_mbps: 0.0,
            disk_io_mbps: 0.0;}}}

/// Resource configuration for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Cpu Request field

    pub cpu_request: f64,
    /// Memory Request Mb field
    pub memory_request_mb: u32,
    /// Cpu Limit field
    pub cpu_limit: f64,
    /// Memory Limit Mb field
    pub memory_limit_mb: u32 ,
 )
}

impl Default for ResourceConfig  {fn default() -> Self  {Self { cpu_request: 0.5,
            memory_request_mb: 512,
            cpu_limit: 1.0,
            memory_limit_mb: 1024;}}}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPerformanceConfig {
    /// Max Concurrent Requests field

    pub max_concurrent_requests: u32,
    /// Request Timeout Ms field
    pub request_timeout_ms: u64,
    /// Connection Pool Size field
    pub connection_pool_size: u32,
    /// Cache Size Mb field
    pub cache_size_mb: u32 ,
 )
}

impl Default for CanonicalPerformanceConfig  {fn default() -> Self  {Self { max_concurrent_requests: 1000,
            request_timeout_ms: 30000,
            connection_pool_size: 50,
            cache_size_mb: 128;}}}

/// Scalability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityConfig {
    /// Min Instances field

    pub min_instances: u32,
    /// Max Instances field
    pub max_instances: u32,
    /// Target Cpu Threshold field
    pub target_cpu_threshold: f64,
    /// Target Memory Threshold field
    pub target_memory_threshold: f64,
    /// Scale Up Cooldown field
    pub scale_up_cooldown: Duration,
    /// Scale Down Cooldown field
    pub scale_down_cooldown: Duration,
    /// Metrics Interval field
    pub metrics_interval: Duration,
    /// Enable Predictive Scaling field
    pub enable_predictive_scaling: bool ,
 )
}

impl Default for ScalabilityConfig  {fn default() -> Self  {Self { min_instances: 1,
            max_instances: 10,
            target_cpu_threshold: 70.0,
            target_memory_threshold: 80.0,
            scale_up_cooldown: Duration::from_secs(60)
            scale_down_cooldown: Duration::from_secs(120,
            metrics_interval: Duration::from_secs(30)
            enable_predictive_scaling: false;}}}

/// Scaling action type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScalingActionType {
    /// ScaleUp, ScaleUp,
    /// ScaleDown, ScaleDown)
    NoAction  }

/// Scaling action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingAction {
    /// Action Type field

    pub action_type: ScalingActionType,
    /// Target Instances field
    pub target_instances: u32,
    /// Reason field
    pub reason: String,
    /// Timestamp when this was created or last updated
    pub timestamp: DateTime<Utc> ,
 )
}

/// Scaling decision
#[derive(Debug, Clone)]
pub enum ScalingDecision {
    /// ScaleUp
        ScaleUp(u32)
    /// ScaleDown
        ScaleDown(u32)
    NoAction);  }

/// Direction of scaling operation
#[derive(Debug, Clone, PartialEq)]
pub enum ScaleDirection {
    /// Up, Up,
    /// Down, Down)
    None  }

/// Scaling event for history tracking
#[derive(Debug, Clone)]
pub struct ScalingEvent {
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc>,
    /// Service Id field
    pub service_id: String,
    /// Decision field
    pub decision: ScalingDecision,
    /// Reason field
    pub reason: String,
    /// Current Instances field
    pub current_instances: u32,
    /// Target Instances field
    pub target_instances: u32 ,
 )
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average Response Time field

    pub average_response_time: f64,
    /// Request Rate field
    pub request_rate: f64,
    /// Error Rate field
    pub error_rate: f64,
    /// Timeout Rate field
    pub timeout_rate: f64,
    /// Connection Pool Utilization field
    pub connection_pool_utilization: f64,
    /// Cache Hit Rate field
    pub cache_hit_rate: f64,
    /// Memory Usage Percent field
    pub memory_usage_percent: f64,
    /// Cpu Usage Percent field
    pub cpu_usage_percent: f64 ,
 )
}

/// Optimization event
#[derive(Debug, Clone)]
pub struct OptimizationEvent {
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc>,
    /// Service Id field
    pub service_id: String,
    /// Optimization Type field
    pub optimization_type: OptimizationType,
    /// Old Value field
    pub old_value: f64,
    /// New Value field
    pub new_value: f64,
    /// Improvement Percent field
    pub improvement_percent: f64 ,
 )
}

/// Types of optimizations
#[derive(Debug, Clone)]
pub enum OptimizationType {
    /// ConnectionPoolSize, ConnectionPoolSize,
    /// CacheSize, CacheSize)
    /// RequestTimeout, RequestTimeout,
    ConcurrentRequests  }

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Optimization Type field

    pub optimization_type: OptimizationType,
    /// Current Value field
    pub current_value: f64,
    /// Recommended Value field
    pub recommended_value: f64,
    /// Expected Improvement field
    pub expected_improvement: f64,
    /// Reason field
    pub reason: String ,
 )
}
