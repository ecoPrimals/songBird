//! Core types and enums for AI workload classification

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of workloads that can be classified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadType {
    /// Generic fallback workload type
    Generic,

    /// Standard general-purpose workload
    Standard,

    /// Web service workloads (API endpoints, HTTP services)
    WebService,

    /// Gaming workloads (real-time gaming, multiplayer)
    Gaming,

    /// Machine learning workloads (training, inference)
    MachineLearning,

    /// Compute-intensive workloads (calculations, processing)
    Compute,

    /// Storage workloads (data management, backup)
    Storage,

    /// Security workloads (authentication, encryption)
    Security,

    /// Streaming workloads (media, data streaming)
    Streaming,

    /// Real-time interactive requests requiring immediate response
    RealTimeInteractive {
        /// Expected response time in milliseconds
        expected_response_ms: f64,
        /// User interaction pattern
        interaction_pattern: String,
    },

    /// Batch processing workloads that can be queued
    BatchProcessing {
        /// Batch size estimate
        batch_size: u32,
        /// Processing priority level
        priority_level: BatchPriority,
    },

    /// AI/ML computation workloads
    AIComputation {
        /// Type of AI computation
        computation_type: String,
    },
}

/// Batch processing priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Resource priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Workload request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadRequest {
    pub id: String,
    pub workload_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub payload: serde_json::Value,
}

/// Resource requirements for a workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub storage_mb: u64,
    pub network_bandwidth_mbps: u32,
    pub priority: ResourcePriority,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: 2,
            memory_mb: 2048,
            storage_mb: 5120,
            network_bandwidth_mbps: 100,
            priority: ResourcePriority::Medium,
        }
    }
}

/// Performance prediction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    pub expected_latency_ms: u64,
    pub expected_throughput_rps: f64,
    pub expected_reliability: f64,
    pub confidence_score: f64,
}

impl Default for PerformancePrediction {
    fn default() -> Self {
        Self {
            expected_latency_ms: 100,
            expected_throughput_rps: 1000.0,
            expected_reliability: 0.95,
            confidence_score: 0.7,
        }
    }
}

/// Risk assessment for a workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_score: f64,
    pub risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub confidence: f64,
}

impl Default for RiskAssessment {
    fn default() -> Self {
        Self {
            overall_risk_score: 0.3,
            risk_factors: vec![],
            mitigation_strategies: vec!["Apply standard security policies".to_string()],
            confidence: 0.8,
        }
    }
}

/// Complete workload classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadClassification {
    pub workload_type: WorkloadType,
    pub confidence_score: f64,
    pub resource_requirements: ResourceRequirements,
    pub performance_prediction: PerformancePrediction,
    pub risk_assessment: RiskAssessment,
}

impl Default for WorkloadClassification {
    fn default() -> Self {
        Self {
            workload_type: WorkloadType::Generic,
            confidence_score: 0.5,
            resource_requirements: ResourceRequirements::default(),
            performance_prediction: PerformancePrediction::default(),
            risk_assessment: RiskAssessment::default(),
        }
    }
}

// Stub types that were missing and causing compilation errors
// These would be implemented by Squirrel, so we just provide basic placeholders

/// Model size classification (stub for Squirrel delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

/// LLM Operation types (stub for Squirrel delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMOperation {
    Training,
    FineTuning,
    Inference,
}

/// Computer Vision model types (stub for Squirrel delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CVModelType {
    ObjectDetection,
    ImageClassification,
    Segmentation,
}

/// Service mesh operation types (stub for Squirrel delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshOperationType {
    ServiceDiscovery,
    LoadBalancing,
    CircuitBreaking,
    HealthChecking,
    ConfigurationManagement,
    SecurityPolicyEnforcement,
    MetricsCollection,
    TrafficRouting,
}

/// Service mesh scope (stub for Squirrel delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshScope {
    Service,
    Namespace,
    Cluster,
}

/// Operation criticality levels (stub for Squirrel delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationCriticality {
    Low,
    Medium,
    High,
    Critical,
}

/// Workflow types (stub for Squirrel delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowType {
    Sequential,
    Parallel,
    Pipeline,
    MapReduce,
}

/// Risk types (stub for Squirrel delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    ResourceExhaustion,
    NetworkSecurity,
    ModelSecurity,
    SystemStability,
    ComplexityRisk,
    PerformanceDegradation,
    ThreatPattern,
    ResourceConstraint,
    UnknownBehavior,
}

/// Security operation types (already exists, keeping for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityOperationType {
    Authentication,
    Authorization,
    Encryption,
    ThreatDetection,
    AuditLogging,
    Compliance,
}
