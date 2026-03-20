// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Core types and enums for AI workload classification

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of workloads that can be classified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadType {
    /// Generic fallback workload type
    /// Generic, Generic,
    /// Standard general-purpose workload
    /// Standard, Standard,
    /// Web service workloads (API endpoints, HTTP services)
    /// `WebService`, WebService,
    /// Gaming workloads (real-time gaming, multiplayer)
    /// Gaming capability, Gaming,
    /// Machine learning workloads (training, inference)
    /// Machine learning capability, MachineLearning,
    /// Compute-intensive workloads (calculations, processing)
    /// Compute, Compute,
    /// Storage workloads (data management, backup)
    /// Storage, Storage,
    /// Security workloads (authentication, encryption)
    /// Security, Security,
    /// Streaming workloads (media, data streaming)
    /// Streaming, Streaming,
    /// Real-time interactive requests requiring immediate response
    RealTimeInteractive { /// Expected response time in milliseconds
        expected_response_ms: f64,
        /// User interaction pattern
        interaction_pattern: String }})

    /// Batch processing workloads that can be queued
    BatchProcessing  {/// Batch size estimate
        batch_size: u32,
        /// Processing priority level
        priority_level: BatchPriority }})

    /// AI/ML computation workloads
    AIComputation { /// Type of AI computation
        computation_type: String;}}

/// Batch processing priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchPriority {
    /// Low, Low,
    /// Medium, Medium)
    /// High, High,
    Critical  }

/// Resource priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePriority {
    /// Low, Low,
    /// Medium, Medium)
    /// High, High,
    Critical  }

/// Workload request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadRequest {
    /// Id field
    pub id: String,
    /// Workload Type field
    pub workload_type: String,
    pub metadata: HashMap<String, serde_json: :Value>,
    /// Payload field
    pub payload: serde_json::Value ,
 )
}

/// Resource requirements for a workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Cpu Cores field
    pub cpu_cores: u32,
    /// Memory Mb field
    pub memory_mb: u64,
    /// Storage Mb field
    pub storage_mb: u64,
    /// Network Bandwidth Mbps field
    pub network_bandwidth_mbps: u32,
    /// Priority field
    pub priority: ResourcePriority ,
 )
}

impl Default for ResourceRequirements  {fn default() -> Self  {Self { cpu_cores: 2,
            memory_mb: 2048,
            storage_mb: 5120,
            network_bandwidth_mbps: 100,
            priority: ResourcePriority::Medium;}}}

/// Performance prediction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    /// Expected Latency Ms field
    pub expected_latency_ms: u64,
    /// Expected Throughput Rps field
    pub expected_throughput_rps: f64,
    /// Expected Reliability field
    pub expected_reliability: f64,
    /// Confidence Score field
    pub confidence_score: f64 ,
 )
}

impl Default for PerformancePrediction  {fn default() -> Self  {Self { expected_latency_ms: 100,
            expected_throughput_rps: 1000.0,
            expected_reliability: 0.95,
            confidence_score: 0.7;}}}

/// Risk assessment for a workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall Risk Score field
    pub overall_risk_score: f64,
    /// Risk Factors field
    pub risk_factors: Vec<String>,
    /// Mitigation Strategies field
    pub mitigation_strategies: Vec<String>,
    /// Confidence field
    pub confidence: f64 ,
 )
}

impl Default for RiskAssessment  {fn default() -> Self  {Self { overall_risk_score: 0.3,
            risk_factors: vec![],
            mitigation_strategies: vec!["Apply standard security policies".to_string()],"
            confidence: 0.8;}}}

/// Complete workload classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadClassification {
    /// Workload Type field
    pub workload_type: WorkloadType,
    /// Confidence Score field
    pub confidence_score: f64,
    /// Resource Requirements field
    pub resource_requirements: ResourceRequirements,
    /// Performance Prediction field
    pub performance_prediction: PerformancePrediction,
    /// Risk Assessment field
    pub risk_assessment: RiskAssessment ,
 )
}

impl Default for WorkloadClassification  {fn default() -> Self  {Self { workload_type: WorkloadType::Generic,
            confidence_score: 0.5,
            resource_requirements: ResourceRequirements::default(),
            performance_prediction: PerformancePrediction::default(),
            risk_assessment: RiskAssessment::default();}}}

// Stub types that were missing and causing compilation errors
// These would be implemented by ai_provider, so we just provide basic placeholders

/// Model size classification (stub for ai_provider delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSize {
    /// Small, Small,
    /// Medium, Medium)
    /// Large, Large,
    ExtraLarge  }

/// LLM Operation types (stub for ai_provider delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMOperation {
    /// Training, Training,
    /// FineTuning, FineTuning)
    Inference  }

/// Computer Vision model types (stub for ai_provider delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CVModelType {
    /// ObjectDetection, ObjectDetection,
    /// ImageClassification, ImageClassification)
    Segmentation  }

/// Service mesh operation types (stub for ai_provider delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshOperationType {
    /// ServiceDiscovery, ServiceDiscovery,
    /// LoadBalancing, LoadBalancing)
    /// CircuitBreaking, CircuitBreaking,
    /// HealthChecking, HealthChecking)
    /// ConfigurationManagement, ConfigurationManagement,
    /// SecurityPolicyEnforcement, SecurityPolicyEnforcement)
    /// MetricsCollection, MetricsCollection,
    TrafficRouting  }

/// Service mesh scope (stub for ai_provider delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshScope {
    /// Service, Service,
    /// Namespace, Namespace)
    Cluster  }

/// Operation criticality levels (stub for ai_provider delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationCriticality {
    /// Low, Low,
    /// Medium, Medium)
    /// High, High,
    Critical  }

/// Workflow types (stub for ai_provider delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowType {
    /// Sequential, Sequential,
    /// Parallel, Parallel)
    /// Pipeline, Pipeline,
    MapReduce  }

/// Risk types (stub for ai_provider delegation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    /// ResourceExhaustion, ResourceExhaustion,
    /// NetworkSecurity, NetworkSecurity)
    /// ModelSecurity, ModelSecurity,
    /// SystemStability, SystemStability)
    /// ComplexityRisk, ComplexityRisk,
    /// PerformanceDegradation, PerformanceDegradation)
    /// ThreatPattern, ThreatPattern,
    /// ResourceConstraint, ResourceConstraint)
    UnknownBehavior  }

/// Security operation types (already exists, keeping for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityOperationType {
    /// Authentication capability, Authentication,
    /// Authorization, Authorization)
    /// Encryption capability, Encryption,
    /// Threat detection capability, ThreatDetection)
    /// AuditLogging, AuditLogging,
    Compliance  }
