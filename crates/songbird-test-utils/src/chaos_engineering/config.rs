
// Chaos Engineering Configuration Types
//
// Canonical configuration types for chaos engineering experiments.
// Extracted from monolithic fault_injection.rs for maintainability.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Custom serialization for Option<SystemTime>
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

/// Experiment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    /// Experiment is being planned
    Planning,
    /// Experiment is running
    Running,
    /// Experiment completed successfully
    Completed,
    /// Experiment failed
    Failed,
    /// Experiment was stopped manually
    Stopped,
}

/// Experiment configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate ExperimentConfig to songbird_config::unified
pub struct ExperimentConfig {
    /// Network fault configuration
    pub network_fault: Option<NetworkFaultConfig>,
    /// Service failure configuration
    pub service_failure: Option<ServiceFailureConfig>,
    /// Resource constraint configuration
    pub resource_constraint: Option<ResourceConstraintConfig>,
    /// Byzantine failure configuration
    pub byzantine_failure: Option<ByzantineFailureConfig>,
    /// Performance degradation configuration
    pub performance_degradation: Option<PerformanceDegradationConfig>,
}

/// Network fault injection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate NetworkFaultConfig to songbird_config::unified
pub struct NetworkFaultConfig {
    /// Inject latency (milliseconds)
    pub latency_ms: Option<u64>,
    /// Packet loss percentage (0-100)
    pub packet_loss_percent: Option<f64>,
    /// Bandwidth limitation (bytes per second)
    pub bandwidth_limit_bps: Option<u64>,
    /// Connection drops
    pub drop_connections: bool,
    /// DNS resolution failures
    pub dns_failures: bool,
    /// SSL/TLS handshake failures
    pub ssl_failures: bool,
}

/// Service failure simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate ServiceFailureConfig to songbird_config::unified
pub struct ServiceFailureConfig {
    /// Crash the service
    pub crash_service: bool,
    /// Hang the service (stop responding)
    pub hang_service: bool,
    /// Return error responses
    pub error_responses: bool,
    /// Error response rate (0.0-1.0)
    pub error_rate: f64,
    /// Slow responses
    pub slow_responses: bool,
    /// Response delay (milliseconds)
    pub response_delay_ms: u64,
}

/// Resource constraint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate ResourceConstraintConfig to songbird_config::unified
pub struct ResourceConstraintConfig {
    /// CPU usage limit (percentage)
    pub cpu_limit_percent: Option<f64>,
    /// Memory usage limit (bytes)
    pub memory_limit_bytes: Option<u64>,
    /// Disk I/O limit (bytes per second)
    pub disk_io_limit_bps: Option<u64>,
    /// Network I/O limit (bytes per second)
    pub network_io_limit_bps: Option<u64>,
    /// File descriptor limit
    pub fd_limit: Option<u32>,
}

/// Byzantine failure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate ByzantineFailureConfig to songbird_config::unified
pub struct ByzantineFailureConfig {
    /// Corrupt data probability (0.0-1.0)
    pub corruption_probability: f64,
    /// Malicious response probability (0.0-1.0)
    pub malicious_response_probability: f64,
    /// Data tampering enabled
    pub data_tampering: bool,
    /// Timing attacks enabled
    pub timing_attacks: bool,
}

/// Performance degradation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate PerformanceDegradationConfig to songbird_config::unified
pub struct PerformanceDegradationConfig {
    /// CPU throttling percentage (0-100)
    pub cpu_throttle_percent: f64,
    /// Memory pressure level (0.0-1.0)
    pub memory_pressure: f64,
    /// Disk I/O slowdown factor
    pub disk_slowdown_factor: f64,
    /// Network bandwidth reduction factor
    pub network_slowdown_factor: f64,
}

/// Chaos experiment definition
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
    pub duration: std::time::Duration,
    /// Experiment status
    pub status: ExperimentStatus,
    /// Start time (using SystemTime for serialization compatibility)
    #[serde(with = "systemtime_option")]
    pub start_time: Option<SystemTime>,
    /// End time (using SystemTime for serialization compatibility)
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
