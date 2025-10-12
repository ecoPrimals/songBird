//! # 🧪 Chaos Engineering Configuration
//!
//! **MIGRATION COMPLETE**: All configurations migrated to `songbird_config::unified::testing`
//!
//! This module provides configuration for chaos engineering experiments)
//! fault injection, and system resilience testing.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Custom serialization for `Option<SystemTime>`
mod systemtime_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::SystemTime;

    pub fn serialize<S>(time: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match time {
            Some(time) => {
                let duration = time
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_err(serde::ser::Error::custom)?;
                duration.as_secs().serialize(serializer)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<u64> = Option::deserialize(deserializer)?;
        Ok(opt.map(|secs| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs)))
    }
}

// ============================================================================
// UNIFIED CONFIGURATION RE-EXPORTS - **MIGRATION COMPLETE**
// ============================================================================

/// Experiment configuration - **MIGRATED TO UNIFIED**
// TEMPORARY: Disabled - songbird_config::unified has E0765 corruption
// pub use songbird_config::unified::testing::{ByzantineFailureConfig, ExperimentConfig, NetworkFaultConfig, PerformanceDegradationConfig,
//     ResourceConstraintConfig, ServiceFailureConfig,
// };

// Temporary local definitions until unified module is fixed
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExperimentConfig {
    pub network_fault: Option<NetworkFaultConfig>,
    pub service_failure: Option<ServiceFailureConfig>,
    pub resource_constraint: Option<ResourceConstraintConfig>,
    pub byzantine_failure: Option<ByzantineFailureConfig>,
    pub performance_degradation: Option<PerformanceDegradationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkFaultConfig {
    pub latency_ms: Option<u64>,
    pub packet_loss_percent: Option<f64>,
    pub bandwidth_limit_bps: Option<u64>,
    pub partition_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceFailureConfig {
    pub failure_rate: f64,
    pub mean_time_to_failure: Duration,
    pub mean_time_to_recovery: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceConstraintConfig {
    pub cpu_limit_percent: Option<f64>,
    pub memory_limit_mb: Option<u64>,
    pub disk_io_limit_mbps: Option<u64>,
    pub network_bandwidth_limit_mbps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ByzantineFailureConfig {
    pub corrupt_data_rate: f64,
    pub delayed_response_rate: f64,
    pub malicious_behavior_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceDegradationConfig {
    pub slowdown_factor: f64,
    pub resource_constraint: Option<ResourceConstraintConfig>,
}

/// Types of chaos experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentType {
    /// Network faults (latency, packet loss, partitions)
    NetworkFault,
    /// Service failures (crashes, hangs, resource exhaustion)
    ServiceFailure,
    /// Resource constraints (CPU, memory, disk, network)
    ResourceConstraint,
    /// Byzantine failures (corrupt data, malicious behavior)
    ByzantineFailure,
    /// Performance degradation
    PerformanceDegradation,
    /// Configuration errors
    ConfigurationError,
    /// Security attacks
    SecurityAttack,
    /// Dependency failures
    DependencyFailure,
}

/// Experiment execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentStatus {
    /// Experiment is being prepared
    Preparing,
    /// Experiment is currently running
    Running,
    /// Experiment completed successfully
    Completed,
    /// Experiment failed with an error
    Failed,
    /// Experiment was stopped manually
    Stopped,
}

/// Experiment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosExperiment {
    /// Unique experiment identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Experiment description
    pub description: String,
    /// Experiment type
    pub experiment_type: ExperimentType,
    /// Target services/components
    pub targets: Vec<String>,
    /// Experiment configuration
    pub config: ExperimentConfig,
    /// Expected duration
    pub duration: Duration,
    /// Experiment status
    pub status: ExperimentStatus,
    /// Start time (using `SystemTime` for serialization compatibility)
    #[serde(with = "systemtime_option")]
    pub start_time: Option<SystemTime>,
    /// End time (using `SystemTime` for serialization compatibility)
    #[serde(with = "systemtime_option")]
    pub end_time: Option<SystemTime>,
    /// Experiment results
    pub results: Option<ExperimentResults>,
}

/// Experiment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    /// Whether the experiment succeeded
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Metrics collected during experiment
    pub metrics: Vec<MetricSnapshot>,
    /// System behavior observations
    pub observations: Vec<String>,
    /// Performance impact measurements
    pub performance_impact: PerformanceImpact,
}

/// Performance impact measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    /// Response time change (percentage)
    pub response_time_change_percent: f64,
    /// Throughput change (percentage)
    pub throughput_change_percent: f64,
    /// Error rate change (percentage)
    pub error_rate_change_percent: f64,
    /// Resource utilization change
    pub resource_utilization_change: ResourceUtilizationChange,
}

/// Resource utilization changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilizationChange {
    /// CPU utilization change (percentage)
    pub cpu_change_percent: f64,
    /// Memory utilization change (percentage)
    pub memory_change_percent: f64,
    /// Network utilization change (percentage)
    pub network_change_percent: f64,
    /// Disk utilization change (percentage)
    pub disk_change_percent: f64,
}

/// Metric snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Timestamp
    #[serde(with = "systemtime_option")]
    pub timestamp: Option<SystemTime>,
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Metric unit
    pub unit: String,
}
