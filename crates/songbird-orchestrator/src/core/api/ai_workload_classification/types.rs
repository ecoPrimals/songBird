// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core types and enums for AI workload classification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of workloads that can be classified.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadType {
    /// Generic fallback when classification is unknown.
    Generic,
    /// Standard general-purpose workload.
    Standard,
    /// Web service workloads (API endpoints, HTTP services).
    WebService,
    /// Gaming workloads (real-time gaming, multiplayer).
    Gaming,
    /// Machine learning workloads (training, inference).
    MachineLearning,
    /// Compute-intensive workloads (calculations, processing).
    Compute,
    /// Storage workloads (data management, backup).
    Storage,
    /// Security workloads (authentication, encryption).
    Security,
    /// Streaming workloads (media, data streaming).
    Streaming,
    /// Real-time interactive requests requiring immediate response.
    RealTimeInteractive {
        /// Expected response time in milliseconds.
        expected_response_ms: f64,
        /// User interaction pattern label.
        interaction_pattern: String,
    },
    /// Batch processing workloads that can be queued.
    BatchProcessing {
        /// Batch size estimate.
        batch_size: u32,
        /// Processing priority level.
        priority_level: BatchPriority,
    },
    /// AI/ML computation workloads.
    AIComputation {
        /// Type of AI computation (e.g. inference, training).
        computation_type: String,
    },
}

/// Batch processing priority levels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BatchPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Resource scheduling priority (distinct from batch priority when both apply).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourcePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Workload request structure sent to an AI-capable primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadRequest {
    /// Stable identifier for correlation.
    pub id: String,
    /// Serialized or symbolic workload type (parsed via [`WorkloadRequest::parsed_workload_type`]).
    pub workload_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub payload: serde_json::Value,
}

impl WorkloadRequest {
    /// Best-effort parse of `workload_type` and metadata into a [`WorkloadType`].
    #[must_use]
    pub fn parsed_workload_type(&self) -> WorkloadType {
        let key = self.workload_type.to_ascii_lowercase();
        match key.as_str() {
            "standard" => WorkloadType::Standard,
            "web" | "webservice" | "web_service" => WorkloadType::WebService,
            "gaming" => WorkloadType::Gaming,
            "ml" | "machine_learning" | "machinelearning" => WorkloadType::MachineLearning,
            "compute" => WorkloadType::Compute,
            "storage" => WorkloadType::Storage,
            "security" => WorkloadType::Security,
            "streaming" => WorkloadType::Streaming,
            _ => WorkloadType::Generic,
        }
    }
}

/// Resource requirements for a workload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores requested.
    pub cpu_cores: u32,
    /// Memory in MiB.
    pub memory_mb: u64,
    /// Storage in MiB.
    pub storage_mb: u64,
    /// Network bandwidth in Mbps.
    pub network_bandwidth_mbps: u32,
    /// Scheduling priority.
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

impl ResourceRequirements {
    /// Heuristic pressure score in \[0.0, 1.0\] from requested resources.
    #[must_use]
    pub fn resource_pressure_score(&self) -> f64 {
        let cpu = (f64::from(self.cpu_cores) / 128.0).min(1.0);
        let mem = (self.memory_mb as f64 / (512.0 * 1024.0)).min(1.0); // vs 512 GiB
        let net = (f64::from(self.network_bandwidth_mbps) / 40_000.0).min(1.0);
        ((cpu + mem + net) / 3.0).clamp(0.0, 1.0)
    }

    /// Baseline estimate when no AI primal is available.
    #[must_use]
    pub fn basic_estimation(workload_type: &WorkloadType) -> Self {
        let mut r = Self::default();
        match workload_type {
            WorkloadType::MachineLearning
            | WorkloadType::AIComputation {
                ..
            } => {
                r.cpu_cores = 8;
                r.memory_mb = 16 * 1024;
                r.storage_mb = 50 * 1024;
                r.network_bandwidth_mbps = 1000;
                r.priority = ResourcePriority::High;
            }
            WorkloadType::Gaming
            | WorkloadType::RealTimeInteractive {
                ..
            } => {
                r.cpu_cores = 4;
                r.memory_mb = 8192;
                r.priority = ResourcePriority::High;
            }
            WorkloadType::BatchProcessing {
                batch_size,
                ..
            } => {
                r.cpu_cores = (2 + batch_size / 1000).min(32);
                r.memory_mb = 4096 + u64::from(*batch_size) * 2;
            }
            _ => {}
        }
        r
    }
}

/// Performance prediction data.
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

impl PerformancePrediction {
    /// Derive prediction from resource pressure (higher pressure → higher latency, lower confidence).
    #[must_use]
    pub fn from_resource_requirements(req: &ResourceRequirements) -> Self {
        let pressure = req.resource_pressure_score();
        let mut p = Self::default();
        p.expected_latency_ms = pressure.mul_add(900.0, 100.0) as u64;
        p.expected_throughput_rps = 1000.0 * pressure.mul_add(-0.5, 1.0);
        p.confidence_score = pressure.mul_add(-0.4, 0.9).clamp(0.2, 0.95);
        p
    }
}

/// Risk assessment for a workload.
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
            mitigation_strategies: vec![String::from("Apply standard security policies")],
            confidence: 0.8,
        }
    }
}

impl RiskAssessment {
    /// Baseline assessment when no AI primal is available.
    #[must_use]
    pub fn basic_assessment() -> Self {
        Self::default()
    }

    /// Compute risk from resource pressure and explicit factors.
    #[must_use]
    pub fn from_pressure(req: &ResourceRequirements, factors: &[String]) -> Self {
        let pressure = req.resource_pressure_score();
        Self {
            overall_risk_score: pressure.mul_add(0.55, 0.15).clamp(0.0, 1.0),
            risk_factors: factors.to_vec(),
            mitigation_strategies: vec![
                String::from("Apply standard security policies"),
                String::from("Enforce resource quotas"),
            ],
            confidence: pressure.mul_add(-0.25, 0.85).clamp(0.4, 0.95),
        }
    }
}

/// Complete workload classification result.
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

impl WorkloadClassification {
    /// Fallback classification when delegation to an AI primal is unavailable.
    #[must_use]
    pub fn basic_fallback(workload: &WorkloadRequest) -> Self {
        let wt = workload.parsed_workload_type();
        let resource_requirements = ResourceRequirements::basic_estimation(&wt);
        let performance_prediction =
            PerformancePrediction::from_resource_requirements(&resource_requirements);
        let risk_assessment = RiskAssessment::from_pressure(
            &resource_requirements,
            &[String::from("No AI provider; heuristic classification only")],
        );
        Self {
            workload_type: wt,
            confidence_score: 0.45,
            resource_requirements,
            performance_prediction,
            risk_assessment,
        }
    }
}

/// Model size classification for LLM-style workloads.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

/// LLM operation types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LLMOperation {
    Training,
    FineTuning,
    Inference,
}

/// Computer vision model categories.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CVModelType {
    ObjectDetection,
    ImageClassification,
    Segmentation,
}

/// Service mesh operation kinds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

/// Scope for mesh operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ServiceMeshScope {
    Service,
    Namespace,
    Cluster,
}

/// Operation criticality for scheduling and policy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationCriticality {
    Low,
    Medium,
    High,
    Critical,
}

/// Workflow orchestration patterns.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkflowType {
    Sequential,
    Parallel,
    Pipeline,
    MapReduce,
}

/// Risk categories for AI / mesh workloads.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

/// Security-related operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityOperationType {
    Authentication,
    Authorization,
    Encryption,
    ThreatDetection,
    AuditLogging,
    Compliance,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use std::collections::HashMap;

    use super::*;

    fn sample_request(workload_type: &str) -> WorkloadRequest {
        WorkloadRequest {
            id: String::from("t1"),
            workload_type: workload_type.to_string(),
            metadata: HashMap::new(),
            payload: serde_json::json!({}),
        }
    }

    fn sample_request_rich(workload_type: &str) -> WorkloadRequest {
        let mut meta = HashMap::new();
        meta.insert(String::from("k"), serde_json::json!({"nested": true}));
        WorkloadRequest {
            id: String::from("corr-42"),
            workload_type: workload_type.to_string(),
            metadata: meta,
            payload: serde_json::json!({"items": [1, 2, 3]}),
        }
    }

    #[test]
    fn parsed_workload_type_standard_and_aliases() {
        assert_eq!(sample_request("STANDARD").parsed_workload_type(), WorkloadType::Standard);
        assert_eq!(sample_request("web").parsed_workload_type(), WorkloadType::WebService);
        assert_eq!(sample_request("WEB_SERVICE").parsed_workload_type(), WorkloadType::WebService);
        assert_eq!(sample_request("webservice").parsed_workload_type(), WorkloadType::WebService);
        assert_eq!(sample_request("gaming").parsed_workload_type(), WorkloadType::Gaming);
        assert_eq!(sample_request("ML").parsed_workload_type(), WorkloadType::MachineLearning);
        assert_eq!(
            sample_request("machine_learning").parsed_workload_type(),
            WorkloadType::MachineLearning
        );
    }

    #[test]
    fn parsed_workload_type_compute_storage_security_streaming() {
        assert_eq!(sample_request("compute").parsed_workload_type(), WorkloadType::Compute);
        assert_eq!(sample_request("storage").parsed_workload_type(), WorkloadType::Storage);
        assert_eq!(sample_request("Security").parsed_workload_type(), WorkloadType::Security);
        assert_eq!(sample_request("streaming").parsed_workload_type(), WorkloadType::Streaming);
    }

    #[test]
    fn parsed_workload_type_unknown_is_generic() {
        assert_eq!(
            sample_request("totally-unknown-workload").parsed_workload_type(),
            WorkloadType::Generic
        );
    }

    #[test]
    fn resource_pressure_score_clamped_high_inputs() {
        let mut r = ResourceRequirements::default();
        r.cpu_cores = 256;
        r.memory_mb = 600 * 1024;
        r.network_bandwidth_mbps = 100_000;
        let p = r.resource_pressure_score();
        assert!(p <= 1.0);
        assert!(p >= 0.0);
    }

    #[test]
    fn resource_pressure_score_zero_is_zero() {
        let r = ResourceRequirements {
            cpu_cores: 0,
            memory_mb: 0,
            storage_mb: 0,
            network_bandwidth_mbps: 0,
            priority: ResourcePriority::Low,
        };
        assert_eq!(r.resource_pressure_score(), 0.0);
    }

    #[test]
    fn basic_estimation_machine_learning() {
        let r = ResourceRequirements::basic_estimation(&WorkloadType::MachineLearning);
        assert_eq!(r.cpu_cores, 8);
        assert_eq!(r.memory_mb, 16 * 1024);
        assert_eq!(r.priority, ResourcePriority::High);
    }

    #[test]
    fn basic_estimation_ai_computation_variant() {
        let wt = WorkloadType::AIComputation {
            computation_type: String::from("inference"),
        };
        let r = ResourceRequirements::basic_estimation(&wt);
        assert!(r.cpu_cores >= 8);
        assert_eq!(r.priority, ResourcePriority::High);
    }

    #[test]
    fn basic_estimation_gaming_and_realtime() {
        let r = ResourceRequirements::basic_estimation(&WorkloadType::Gaming);
        assert_eq!(r.cpu_cores, 4);
        assert_eq!(r.memory_mb, 8192);

        let wt = WorkloadType::RealTimeInteractive {
            expected_response_ms: 10.0,
            interaction_pattern: String::from("fps"),
        };
        let r2 = ResourceRequirements::basic_estimation(&wt);
        assert_eq!(r2.cpu_cores, 4);
    }

    #[test]
    fn basic_estimation_batch_processing_scales_with_batch_size() {
        let wt = WorkloadType::BatchProcessing {
            batch_size: 5000,
            priority_level: BatchPriority::Medium,
        };
        let r = ResourceRequirements::basic_estimation(&wt);
        assert_eq!(r.cpu_cores, 7);
        assert_eq!(r.memory_mb, 4096 + 5000 * 2);
    }

    #[test]
    fn basic_estimation_batch_processing_cpu_caps_at_32() {
        let wt = WorkloadType::BatchProcessing {
            batch_size: 100_000,
            priority_level: BatchPriority::Low,
        };
        let r = ResourceRequirements::basic_estimation(&wt);
        assert_eq!(r.cpu_cores, 32);
    }

    #[test]
    fn basic_estimation_default_branch_matches_default_for_plain_variants() {
        let d = ResourceRequirements::default();
        for wt in [
            WorkloadType::Generic,
            WorkloadType::Standard,
            WorkloadType::WebService,
            WorkloadType::Compute,
            WorkloadType::Storage,
            WorkloadType::Security,
            WorkloadType::Streaming,
        ] {
            let r = ResourceRequirements::basic_estimation(&wt);
            assert_eq!(r.cpu_cores, d.cpu_cores, "{wt:?}");
            assert_eq!(r.memory_mb, d.memory_mb, "{wt:?}");
            assert_eq!(r.storage_mb, d.storage_mb, "{wt:?}");
            assert_eq!(r.network_bandwidth_mbps, d.network_bandwidth_mbps, "{wt:?}");
            assert_eq!(r.priority, d.priority, "{wt:?}");
        }
    }

    #[test]
    fn performance_prediction_from_resources_monotonic_with_pressure() {
        let low = ResourceRequirements {
            cpu_cores: 1,
            memory_mb: 1024,
            storage_mb: 1024,
            network_bandwidth_mbps: 10,
            priority: ResourcePriority::Low,
        };
        let high = ResourceRequirements {
            cpu_cores: 64,
            memory_mb: 200 * 1024,
            storage_mb: 1024,
            network_bandwidth_mbps: 10_000,
            priority: ResourcePriority::Critical,
        };
        let p_low = PerformancePrediction::from_resource_requirements(&low);
        let p_high = PerformancePrediction::from_resource_requirements(&high);
        assert!(p_high.expected_latency_ms >= p_low.expected_latency_ms);
        assert!(p_high.confidence_score <= p_low.confidence_score);
    }

    #[test]
    fn risk_assessment_from_pressure_bounds() {
        let req = ResourceRequirements::default();
        let r = RiskAssessment::from_pressure(&req, &[String::from("factor a")]);
        assert!(r.overall_risk_score >= 0.0 && r.overall_risk_score <= 1.0);
        assert!(r.confidence >= 0.4 && r.confidence <= 0.95);
        assert_eq!(r.risk_factors.len(), 1);
        assert_eq!(r.mitigation_strategies.len(), 2);
    }

    #[test]
    fn risk_assessment_from_pressure_zero_and_high_resource_pressure() {
        let zero = ResourceRequirements {
            cpu_cores: 0,
            memory_mb: 0,
            storage_mb: 0,
            network_bandwidth_mbps: 0,
            priority: ResourcePriority::Low,
        };
        let z = RiskAssessment::from_pressure(&zero, &[]);
        assert!((z.overall_risk_score - 0.15).abs() < 1e-9);
        assert!(z.risk_factors.is_empty());

        let mut hi = ResourceRequirements::default();
        hi.cpu_cores = 128;
        hi.memory_mb = 512 * 1024;
        hi.network_bandwidth_mbps = 40_000;
        let h = RiskAssessment::from_pressure(&hi, &[String::from("overload")]);
        assert!(h.overall_risk_score > z.overall_risk_score);
        assert_eq!(h.risk_factors, vec![String::from("overload")]);
    }

    #[test]
    fn workload_classification_basic_fallback_pipeline() {
        let w = sample_request("ml");
        let c = WorkloadClassification::basic_fallback(&w);
        assert_eq!(c.workload_type, WorkloadType::MachineLearning);
        assert_eq!(c.confidence_score, 0.45);
        assert!(c.resource_requirements.cpu_cores >= 8);
        assert!(!c.risk_assessment.risk_factors.is_empty());
    }

    #[test]
    fn workload_classification_basic_fallback_generic() {
        let w = sample_request("unknown-label");
        let c = WorkloadClassification::basic_fallback(&w);
        assert_eq!(c.workload_type, WorkloadType::Generic);
        let d = ResourceRequirements::default();
        assert_eq!(c.resource_requirements.cpu_cores, d.cpu_cores);
        assert_eq!(c.resource_requirements.memory_mb, d.memory_mb);
        assert_eq!(c.resource_requirements.storage_mb, d.storage_mb);
        assert_eq!(c.resource_requirements.network_bandwidth_mbps, d.network_bandwidth_mbps);
        assert_eq!(c.resource_requirements.priority, d.priority);
    }

    #[test]
    fn workload_classification_basic_fallback_web_compute_gaming_standard() {
        for (label, expected) in [
            ("web", WorkloadType::WebService),
            ("compute", WorkloadType::Compute),
            ("gaming", WorkloadType::Gaming),
            ("standard", WorkloadType::Standard),
            ("storage", WorkloadType::Storage),
        ] {
            let c = WorkloadClassification::basic_fallback(&sample_request_rich(label));
            assert_eq!(c.workload_type, expected, "{label}");
            assert_eq!(c.confidence_score, 0.45);
        }
    }

    #[test]
    fn risk_assessment_basic_assessment_is_default_shape() {
        let r = RiskAssessment::basic_assessment();
        assert_eq!(r.overall_risk_score, 0.3);
        assert_eq!(r.confidence, 0.8);
    }

    #[test]
    fn defaults_match_documented_baselines() {
        let rr = ResourceRequirements::default();
        assert_eq!(rr.cpu_cores, 2);
        assert_eq!(rr.memory_mb, 2048);
        assert_eq!(rr.priority, ResourcePriority::Medium);

        let pp = PerformancePrediction::default();
        assert_eq!(pp.expected_latency_ms, 100);
        assert!((pp.expected_throughput_rps - 1000.0).abs() < f64::EPSILON);

        let ra = RiskAssessment::default();
        assert_eq!(ra.overall_risk_score, 0.3);
        assert_eq!(ra.mitigation_strategies.len(), 1);

        let wc = WorkloadClassification::default();
        assert_eq!(wc.workload_type, WorkloadType::Generic);
        assert!((wc.confidence_score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn serde_round_trip_workload_request() {
        let w = sample_request_rich("machine_learning");
        let s = serde_json::to_string(&w).unwrap();
        let w2: WorkloadRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(w.id, w2.id);
        assert_eq!(w.workload_type, w2.workload_type);
        assert_eq!(w.metadata, w2.metadata);
        assert_eq!(w.payload, w2.payload);
    }

    #[test]
    fn serde_round_trip_workload_type_variants() {
        let cases = vec![
            WorkloadType::Generic,
            WorkloadType::Standard,
            WorkloadType::RealTimeInteractive {
                expected_response_ms: 12.5,
                interaction_pattern: String::from("voice"),
            },
            WorkloadType::BatchProcessing {
                batch_size: 99,
                priority_level: BatchPriority::Critical,
            },
            WorkloadType::AIComputation {
                computation_type: String::from("train"),
            },
        ];
        for wt in cases {
            let s = serde_json::to_string(&wt).unwrap();
            let wt2: WorkloadType = serde_json::from_str(&s).unwrap();
            assert_eq!(wt, wt2);
        }
    }

    #[test]
    fn serde_round_trip_workload_classification() {
        let c = WorkloadClassification::basic_fallback(&sample_request("ml"));
        let s = serde_json::to_string(&c).unwrap();
        let c2: WorkloadClassification = serde_json::from_str(&s).unwrap();
        assert_eq!(c.workload_type, c2.workload_type);
        assert!((c.confidence_score - c2.confidence_score).abs() < f64::EPSILON);
        assert_eq!(c.resource_requirements.cpu_cores, c2.resource_requirements.cpu_cores);
        assert_eq!(c.risk_assessment.risk_factors, c2.risk_assessment.risk_factors);
    }

    #[test]
    fn serde_round_trip_resource_requirements_risk_and_prediction() {
        let req = ResourceRequirements::basic_estimation(&WorkloadType::MachineLearning);
        let s = serde_json::to_string(&req).unwrap();
        let req2: ResourceRequirements = serde_json::from_str(&s).unwrap();
        assert_eq!(req.cpu_cores, req2.cpu_cores);
        assert_eq!(req.memory_mb, req2.memory_mb);

        let pred = PerformancePrediction::from_resource_requirements(&req);
        let s = serde_json::to_string(&pred).unwrap();
        let pred2: PerformancePrediction = serde_json::from_str(&s).unwrap();
        assert_eq!(pred.expected_latency_ms, pred2.expected_latency_ms);
        assert!((pred.confidence_score - pred2.confidence_score).abs() < f64::EPSILON);

        let risk = RiskAssessment::from_pressure(&req, &[String::from("x")]);
        let s = serde_json::to_string(&risk).unwrap();
        let risk2: RiskAssessment = serde_json::from_str(&s).unwrap();
        assert!((risk.overall_risk_score - risk2.overall_risk_score).abs() < f64::EPSILON);
        assert_eq!(risk.risk_factors, risk2.risk_factors);
    }

    #[test]
    fn serde_round_trip_small_enums() {
        for v in [
            serde_json::to_string(&BatchPriority::Low).unwrap(),
            serde_json::to_string(&ResourcePriority::High).unwrap(),
            serde_json::to_string(&ModelSize::Large).unwrap(),
            serde_json::to_string(&LLMOperation::FineTuning).unwrap(),
            serde_json::to_string(&RiskType::ModelSecurity).unwrap(),
        ] {
            assert!(!v.is_empty());
        }
        assert_eq!(
            serde_json::from_str::<BatchPriority>(
                &serde_json::to_string(&BatchPriority::Medium).unwrap()
            )
            .unwrap(),
            BatchPriority::Medium
        );
        assert_eq!(
            serde_json::from_str::<ResourcePriority>(
                &serde_json::to_string(&ResourcePriority::Critical).unwrap()
            )
            .unwrap(),
            ResourcePriority::Critical
        );
    }
}
