//! Resource Management Traits
//!
//! Universal resource lifecycle and cleanup patterns

use crate::errors::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Universal resource manager trait
#[async_trait]
pub trait ResourceManager: Send + Sync {
    /// Initialize resource tracking
    async fn initialize(&mut self) -> Result<()>;

    /// Register a resource for tracking
    async fn track_resource(&self, resource: ResourceInfo) -> Result<ResourceHandle>;

    /// Unregister and cleanup a resource
    async fn cleanup_resource(&self, handle: &ResourceHandle) -> Result<()>;

    /// Cleanup all resources for a specific owner
    async fn cleanup_resources_for_owner(&self, owner_id: &str) -> Result<Vec<ResourceInfo>>;

    /// Cleanup all resources
    async fn cleanup_all_resources(&self) -> Result<Vec<ResourceInfo>>;

    /// Check for resource leaks
    async fn check_resource_leaks(&self) -> Result<Vec<ResourceLeak>>;

    /// Get resource usage statistics
    async fn get_resource_stats(&self) -> Result<ResourceStats>;

    /// Enforce resource limits
    async fn enforce_resource_limits(&self) -> Result<Vec<ResourceViolation>>;

    /// Get manager information
    fn manager_info(&self) -> ResourceManagerInfo;
}

/// Resource information for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Unique resource identifier
    pub id: String,
    /// Resource type (connection, file, memory, etc.)
    pub resource_type: String,
    /// Owner of the resource (service, request, etc.)
    pub owner_id: String,
    /// When the resource was created
    pub created_at: DateTime<Utc>,
    /// Expected lifetime of the resource
    pub expected_lifetime: Option<Duration>,
    /// Resource-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Tags for categorization
    pub tags: HashMap<String, String>,
    /// Resource configuration
    pub config: ResourceConfig,
}

/// Resource handle for tracking and cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHandle {
    pub resource_id: String,
    pub handle_token: String,
}

/// Resource configuration and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU usage (0.0 to 1.0)
    pub max_cpu_usage: Option<f64>,
    /// Maximum number of connections
    pub max_connections: Option<u32>,
    /// Maximum file handles
    pub max_file_handles: Option<u32>,
    /// Resource timeout
    pub timeout: Option<Duration>,
    /// Auto-cleanup enabled
    pub auto_cleanup: bool,
    /// Custom limits
    pub custom_limits: HashMap<String, serde_json::Value>,
}

/// Resource leak information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLeak {
    pub resource_info: ResourceInfo,
    pub age: Duration,
    pub leak_severity: LeakSeverity,
    pub description: String,
    pub suggested_action: String,
}

/// Severity of resource leak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeakSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    /// Total resources being tracked
    pub total_resources: u64,
    /// Resources by type
    pub resources_by_type: HashMap<String, u64>,
    /// Resources by owner
    pub resources_by_owner: HashMap<String, u64>,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU usage (0.0 to 1.0)
    pub cpu_usage: f64,
    /// Active connections count
    pub active_connections: u32,
    /// Open file handles count
    pub open_file_handles: u32,
    /// Resource creation rate (per second)
    pub creation_rate: f64,
    /// Resource cleanup rate (per second)
    pub cleanup_rate: f64,
    /// Average resource lifetime
    pub avg_resource_lifetime: Duration,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Resource limit violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceViolation {
    pub resource_info: ResourceInfo,
    pub violation_type: ViolationType,
    pub current_value: f64,
    pub limit_value: f64,
    pub severity: ViolationSeverity,
    pub detected_at: DateTime<Utc>,
    pub action_taken: Option<String>,
}

/// Type of resource violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    MemoryLimit,
    CpuLimit,
    ConnectionLimit,
    FileHandleLimit,
    TimeoutExceeded,
    Custom { name: String },
}

/// Severity of resource violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Error,
    Critical,
}

/// Resource manager information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagerInfo {
    pub name: String,
    pub version: String,
    pub supports_auto_cleanup: bool,
    pub supports_limits: bool,
    pub supports_leak_detection: bool,
    pub tracking_overhead: f64, // Percentage overhead
}

/// Resource cleanup strategy trait
#[async_trait]
pub trait CleanupStrategy: Send + Sync {
    /// Determine if a resource should be cleaned up
    async fn should_cleanup(&self, resource: &ResourceInfo) -> Result<bool>;

    /// Perform cleanup for a specific resource
    async fn cleanup_resource(&self, resource: &ResourceInfo) -> Result<CleanupResult>;

    /// Get strategy information
    fn strategy_info(&self) -> CleanupStrategyInfo;
}

/// Result of resource cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub success: bool,
    pub resources_cleaned: u32,
    pub errors: Vec<String>,
    pub duration: Duration,
    pub bytes_freed: Option<u64>,
}

/// Cleanup strategy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStrategyInfo {
    pub name: String,
    pub description: String,
    pub supports_async: bool,
    pub supports_partial_cleanup: bool,
}

/// Resource monitoring trait
#[async_trait]
pub trait ResourceMonitor: Send + Sync {
    /// Start monitoring resources
    async fn start_monitoring(&mut self) -> Result<()>;

    /// Stop monitoring resources
    async fn stop_monitoring(&mut self) -> Result<()>;

    /// Get current resource metrics
    async fn get_metrics(&self) -> Result<HashMap<String, f64>>;

    /// Set resource threshold alerts
    async fn set_thresholds(&self, thresholds: HashMap<String, f64>) -> Result<()>;

    /// Check if thresholds are exceeded
    async fn check_thresholds(&self) -> Result<Vec<ThresholdViolation>>;
}

/// Threshold violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdViolation {
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub violation_percentage: f64,
    pub first_detected: DateTime<Utc>,
    pub last_detected: DateTime<Utc>,
}

/// Resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagementConfig {
    /// Resource tracking configuration
    pub tracking: TrackingConfig,
    /// Cleanup configuration
    pub cleanup: CleanupConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Limit enforcement configuration
    pub limits: LimitsConfig,
}

/// Resource tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    pub enabled: bool,
    pub track_memory: bool,
    pub track_cpu: bool,
    pub track_connections: bool,
    pub track_file_handles: bool,
    pub tracking_interval: Duration,
    pub max_tracked_resources: Option<u32>,
}

/// Resource cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    pub strategy: String,
    pub cleanup_interval: Duration,
    pub max_resource_age: Duration,
    pub cleanup_on_shutdown: bool,
    pub force_cleanup_timeout: Duration,
}

/// Resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub monitoring_interval: Duration,
    pub alert_thresholds: HashMap<String, f64>,
    pub enable_leak_detection: bool,
    pub leak_detection_interval: Duration,
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    pub enforce_memory_limits: bool,
    pub enforce_cpu_limits: bool,
    pub enforce_connection_limits: bool,
    pub enforce_file_handle_limits: bool,
    pub action_on_violation: ViolationAction,
}

/// Action to take on resource violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationAction {
    Log,
    Warn,
    Throttle,
    Reject,
    Cleanup,
}

impl Default for ResourceManagementConfig {
    fn default() -> Self {
        Self {
            tracking: TrackingConfig {
                enabled: true,
                track_memory: true,
                track_cpu: true,
                track_connections: true,
                track_file_handles: true,
                tracking_interval: Duration::from_secs(10),
                max_tracked_resources: Some(10000),
            },
            cleanup: CleanupConfig {
                strategy: "time_based".to_string(),
                cleanup_interval: Duration::from_secs(60),
                max_resource_age: Duration::from_secs(3600),
                cleanup_on_shutdown: true,
                force_cleanup_timeout: Duration::from_secs(30),
            },
            monitoring: MonitoringConfig {
                monitoring_interval: Duration::from_secs(30),
                alert_thresholds: HashMap::new(),
                enable_leak_detection: true,
                leak_detection_interval: Duration::from_secs(300),
            },
            limits: LimitsConfig {
                enforce_memory_limits: true,
                enforce_cpu_limits: false,
                enforce_connection_limits: true,
                enforce_file_handle_limits: true,
                action_on_violation: ViolationAction::Log,
            },
        }
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: None,
            max_cpu_usage: None,
            max_connections: None,
            max_file_handles: None,
            timeout: None,
            auto_cleanup: true,
            custom_limits: HashMap::new(),
        }
    }
}
