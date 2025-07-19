//! Workload characteristics analysis

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workload characteristics analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCharacteristics {
    /// CPU intensity (0.0 - 1.0)
    pub cpu_intensity: f64,

    /// Memory intensity (0.0 - 1.0)
    pub memory_intensity: f64,

    /// I/O intensity (0.0 - 1.0)
    pub io_intensity: f64,

    /// Network intensity (0.0 - 1.0)
    pub network_intensity: f64,

    /// Latency sensitivity (0.0 - 1.0)
    pub latency_sensitivity: f64,

    /// Throughput requirements
    pub throughput_requirements: ThroughputRequirements,

    /// Scalability characteristics
    pub scalability: ScalabilityCharacteristics,

    /// Service dependencies
    pub dependencies: Vec<ServiceDependency>,

    /// Failure tolerance requirements
    pub failure_tolerance: FailureTolerance,

    /// Processing pattern analysis
    pub processing_pattern: ProcessingPattern,

    /// Resource impact assessment
    pub resource_impact: ResourceImpact,

    /// Quality of Service requirements
    pub qos_requirements: QoSRequirements,
}

/// Throughput requirements analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputRequirements {
    /// Minimum acceptable throughput
    pub min_throughput: f64,

    /// Optimal throughput target
    pub optimal_throughput: f64,

    /// Maximum sustainable throughput
    pub max_throughput: f64,

    /// Throughput measurement unit
    pub unit: String,
}

/// Scalability characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityCharacteristics {
    /// Horizontal scaling potential (0.0 - 1.0)
    pub horizontal_scaling_potential: f64,

    /// Vertical scaling potential (0.0 - 1.0)
    pub vertical_scaling_potential: f64,

    /// Auto-scaling suitability (0.0 - 1.0)
    pub auto_scaling_suitability: f64,

    /// Scaling responsiveness (seconds to scale)
    pub scaling_responsiveness_seconds: f64,

    /// Resource elasticity requirements
    pub elasticity_requirements: Vec<String>,
}

/// Service dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// Dependent service name
    pub service_name: String,

    /// Dependency type
    pub dependency_type: DependencyType,

    /// Criticality level
    pub criticality: DependencyCriticality,

    /// Expected response time from dependency
    pub expected_response_ms: f64,

    /// Dependency health requirements
    pub health_requirements: Vec<String>,
}

/// Types of service dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Database,
    Cache,
    ExternalAPI,
    MessageQueue,
    FileSystem,
    ConfigService,
    AuthService,
    LoggingService,
    MonitoringService,
}

/// Dependency criticality levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyCriticality {
    Low,
    Medium,
    High,
    Critical,
}

/// Failure tolerance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureTolerance {
    /// Can tolerate partial failures
    pub partial_failure_tolerance: bool,

    /// Maximum acceptable failure rate (0.0 - 1.0)
    pub max_failure_rate: f64,

    /// Recovery time requirements (seconds)
    pub recovery_time_seconds: f64,

    /// Requires graceful degradation
    pub graceful_degradation_required: bool,
}

/// Processing pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPattern {
    /// Pattern type description
    pub pattern_type: String,

    /// Burstiness factor (0.0 - 1.0)
    pub burstiness: f64,

    /// Predictability score (0.0 - 1.0)
    pub predictability: f64,

    /// Seasonal patterns
    pub seasonal_patterns: Vec<String>,

    /// Peak load timing patterns
    pub peak_patterns: Vec<String>,
}

/// Resource impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceImpact {
    /// CPU usage patterns
    pub cpu_patterns: Vec<String>,

    /// Memory usage patterns
    pub memory_patterns: Vec<String>,

    /// I/O usage patterns
    pub io_patterns: Vec<String>,

    /// Network usage patterns
    pub network_patterns: Vec<String>,

    /// Resource contention risks
    pub contention_risks: Vec<String>,
}

/// Quality of Service requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoSRequirements {
    /// Availability requirements (0.0 - 1.0)
    pub availability: f64,

    /// Consistency requirements
    pub consistency: ConsistencyRequirement,

    /// Durability requirements (0.0 - 1.0)
    pub durability: f64,

    /// Performance guarantees
    pub performance_guarantees: Vec<String>,

    /// Error tolerance
    pub error_tolerance: ErrorTolerance,
}

/// Consistency requirement levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyRequirement {
    Eventual,
    Strong,
    Sequential,
    Linearizable,
}

/// Error tolerance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTolerance {
    /// Maximum acceptable error rate (0.0 - 1.0)
    pub max_error_rate: f64,

    /// Error types that can be tolerated
    pub tolerable_error_types: Vec<String>,

    /// Recovery strategies
    pub recovery_strategies: Vec<String>,
}
