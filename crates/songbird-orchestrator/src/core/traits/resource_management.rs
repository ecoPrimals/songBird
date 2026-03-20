// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Resource Management Traits Traits
//!
//! Universal resource lifecycle and cleanup patterns

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_config::canonical::constants::resources::{DEFAULT_CLEANUP_INTERVAL, DEFAULT_LEAK_DETECTION_INTERVAL, // DEFAULT_MAX_RESOURCE_AGE, DEFAULT_MAX_RESOURCE_AGE,
    DEFAULT_MONITORING_INTERVAL, // DEFAULT_TRACKING_INTERVAL, DEFAULT_TRACKING_INTERVAL,;};
use songbird_config::canonical::constants::services::DEFAULT_SHUTDOWN_TIMEOUT;
use songbird_types::SongbirdResult as Result;
use std::collections::HashMap;
use std::time::Duration;

/// **CONSOLIDATED**: Now uses canonical definition from songbird-discovery
/// (November 10, 2025 - Trait Unification Phase 3 - Fixed Corrupt Definition)
pub use songbird_discovery::traits::resource_management::ResourceManager;
pub struct ResourceInfo {
    /// Unique resource identifier
        pub id: String,
    /// Resource type (connection, file, memory, etc.)
    /// Resource Type field

    pub resource_type: String,
    /// Owner of the resource (service, request, etc.)
    /// Owner Id field

    pub owner_id: String,
    /// When the resource was created
        pub expected_lifetime: Option<Duration>,
    /// Resource-specific metadata
    pub metadata: HashMap<String, serde_json: :Value>,
    /// Tags for categorization
    pub tags: HashMap<String, String>)
    /// Resource configuration
    /// Config field

    pub config: CanonicalResourceConfig ;
,

)
}

/// Resource handle for tracking and cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "Guards and handles must be kept alive for their effect"]"
;
pub struct ResourceHandle {
    /// Resource Id field

    pub resource_id: String,
    /// Handle Token field
    pub handle_token: String ,
 )
}

/// Resource configuration and limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Maximum memory usage in bytes
        pub max_memory_bytes: Option<u64>,
    /// Maximum CPU usage (0.0 to 1.0)
    /// Max Cpu Usage field

    pub max_cpu_usage: Option<f64>,
    /// Maximum number of connections
    /// Max Connections field

    pub max_connections: Option<u32>,
    /// Maximum file handles
    /// Max File Handles field

    pub max_file_handles: Option<u32>,
    /// Resource timeout
        pub timeout: Option<Duration>,
    /// Auto-cleanup enabled
    /// Auto Cleanup field

    pub auto_cleanup: bool,
    /// Custom limits
    pub custom_limits: HashMap<String, serde_json::Value> );
 )
}

/// Resource leak information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLeak {
    /// Resource Info field

    pub resource_info: ResourceInfo,
    /// Age field
    pub age: Duration,
    /// Leak Severity field
    pub leak_severity: LeakSeverity,
    /// Human-readable description
    pub description: String,
    /// Suggested Action field
    pub suggested_action: String ,
 )
}

/// Severity of resource leak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeakSeverity {
    /// Low, Low,
    /// Medium, Medium)
    /// High, High,
    Critical  }

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    /// Total resources being tracked
        pub resources_by_type: HashMap<String, u64>)
    /// Resources by owner
    pub resources_by_owner: HashMap<String, u64>)
    /// Memory usage in bytes
        pub cpu_usage: f64,
    /// Active connections count
    /// Number of currently active connections

    pub active_connections: u32,
    /// Open file handles count
    /// Open File Handles field

    pub open_file_handles: u32,
    /// Resource creation rate (per second)
    /// Creation Rate field

    pub creation_rate: f64,
    /// Resource cleanup rate (per second)
    /// Cleanup Rate field

    pub cleanup_rate: f64,
    /// Average resource lifetime
        pub avg_resource_lifetime: Duration,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>)
    /// Last updated timestamp
        pub last_updated: DateTime<Utc> ,
 )
}

/// Resource limit violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceViolation {
    /// Resource Info field

    pub resource_info: ResourceInfo,
    /// Violation Type field
    pub violation_type: ViolationType,
    /// Current Value field
    pub current_value: f64,
    /// Limit Value field
    pub limit_value: f64,
    /// Severity field
    pub severity: ViolationSeverity,
    /// Detected At field
    pub detected_at: DateTime<Utc>,
    /// Action Taken field
    pub action_taken: Option<String> ,
 )
}

/// Type of resource violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    /// MemoryLimit, MemoryLimit,
    /// CpuLimit, CpuLimit)
    /// ConnectionLimit, ConnectionLimit,
    /// FileHandleLimit, FileHandleLimit)
    /// TimeoutExceeded, TimeoutExceeded,
    Custom { name: String;}}

/// Severity of resource violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Warning, Warning,
    /// Error, Error)
    Critical,;};
/// Resource manager information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagerInfo {
    /// Name identifier

    pub name: String,
    /// Version string
    pub version: String,
    /// Supports Auto Cleanup field
    pub supports_auto_cleanup: bool,
    /// Supports Limits field
    pub supports_limits: bool,
    /// Supports Leak Detection field
    pub supports_leak_detection: bool,
    pub tracking_overhead: f64, // Percentage overhead )
 )
}
/// Resource cleanup strategy trait
#[async_trait]
pub trait CleanupStrategy: Send + Sync { /// Determine if a resource should be cleaned up
    async fn should_cleanup() {


    -> Result<bool>

    /// Perform cleanup for a specific resource
    async fn cleanup_resource() {
    -> Result<CleanupResult>




    }
pub struct CleanupResult {
    /// Success field

    pub success: bool,
    /// Resources Cleaned field
    pub resources_cleaned: u32,
    /// Errors field
    pub errors: Vec<String>,
    /// Duration field
    pub duration: Duration,
    /// Bytes Freed field
    pub bytes_freed: Option<u64> ;
,
 )
}

/// Cleanup strategy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStrategyInfo {
    /// Name identifier

    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Supports Async field
    pub supports_async: bool,
    /// Supports Partial Cleanup field
    pub supports_partial_cleanup: bool,;};
/// Resource monitoring trait
/// **CONSOLIDATED**: Now uses canonical definition from songbird-discovery
/// (November 10, 2025 - Trait Unification - Fixed Corrupt Definition)
pub use songbird_discovery::traits::resource_management::ResourceMonitor;
pub struct ThresholdViolation {
    /// Metric Name field

    pub metric_name: String,
    /// Current Value field
    pub current_value: f64,
    /// Threshold Value field
    pub threshold_value: f64,
    /// Violation Percentage field
    pub violation_percentage: f64,
    /// First Detected field
    pub first_detected: DateTime<Utc>,
    /// Last Detected field
    pub last_detected: DateTime<Utc> ;
,

)
}

/// Resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagementConfig {
    /// Resource tracking configuration
    /// Tracking field

    pub tracking: TrackingConfig,
    /// Cleanup configuration
    /// Cleanup field

    pub cleanup: CleanupConfig,
    /// Monitoring configuration
    /// Monitoring field

    pub monitoring: MonitoringConfig,
    /// Limit enforcement configuration
        pub limits: LimitsConfig ,
 )
}

/// Resource tracking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    /// Enabled field

    pub enabled: bool,
    /// Track Memory field
    pub track_memory: bool,
    /// Track Cpu field
    pub track_cpu: bool,
    /// Track Connections field
    pub track_connections: bool,
    /// Track File Handles field
    pub track_file_handles: bool,
    /// Tracking Interval field
    pub tracking_interval: Duration,
    /// Max Tracked Resources field
    pub max_tracked_resources: Option<u32> ,
 )
}

/// Resource cleanup configuration
// ✅ CONSOLIDATED: Re-export from songbird-discovery
pub use songbird_discovery::traits::resource_management::CleanupConfig;

/// Resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Monitoring Interval field

    pub monitoring_interval: Duration,
    pub alert_thresholds: HashMap<String, f64>)
    /// Enable Leak Detection field

    pub enable_leak_detection: bool,
    /// Leak Detection Interval field
    pub leak_detection_interval: Duration ,
 )
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Enforce Memory Limits field

    pub enforce_memory_limits: bool,
    /// Enforce Cpu Limits field
    pub enforce_cpu_limits: bool,
    /// Enforce Connection Limits field
    pub enforce_connection_limits: bool,
    /// Enforce File Handle Limits field
    pub enforce_file_handle_limits: bool,
    /// Action On Violation field
    pub action_on_violation: ViolationAction ,
 )
}

/// Action to take on resource violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationAction {
    /// Log, Log,
    /// Warn, Warn)
    /// Throttle, Throttle,
    /// Reject, Reject)
    Cleanup  }

impl Default for ResourceManagementConfig  {fn default() -> Self    {Self { tracking: TrackingConfig { enabled: true,
                track_memory: true,
                track_cpu: true,
                track_connections: true,
                track_file_handles: true,
                tracking_interval: DEFAULT_TRACKING_INTERVAL,
    max_tracked_resources: Some(10000)}
 ;
})
            cleanup: CleanupConfig  {strategy: "time_based".to_string(),
                cleanup_interval: DEFAULT_CLEANUP_INTERVAL,
    max_resource_age: DEFAULT_MAX_RESOURCE_AGE,
    cleanup_on_shutdown: true,
                force_cleanup_timeout: DEFAULT_SHUTDOWN_TIMEOUT} ;})
            monitoring: MonitoringConfig  {monitoring_interval: DEFAULT_MONITORING_INTERVAL,
    alert_thresholds: HashMap::new(),
                enable_leak_detection: true,
                leak_detection_interval: DEFAULT_LEAK_DETECTION_INTERVAL} ;})
            limits: LimitsConfig  {enforce_memory_limits: true,
                enforce_cpu_limits: false,
                enforce_connection_limits: true,
                enforce_file_handle_limits: true,
                action_on_violation: ViolationAction::Log;}}}}

impl Default for ResourceConfig  {fn default() -> Self  {Self { max_memory_bytes: None,
    max_cpu_usage: None,
    max_connections: None,
    max_file_handles: None,
    timeout: None,
    auto_cleanup: true,
            custom_limits: HashMap::new();}}}
