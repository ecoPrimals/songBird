//! AI Workload Classification System
//!
//! Intelligently classifies incoming requests and workloads to optimize
//! routing, resource allocation, and human-AI collaboration patterns.

use crate::api::ai_first_response::{
    AIErrorCategory, AIFirstError, AIFirstResponse, AIResponseMetadata, ActionPriority,
    BackoffType, HumanInteractionContext, InteractionMode, RetryStrategy, SuggestedAction,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

// Re-export all module types for convenience
pub mod allocation;
pub mod characteristics;
pub mod collaboration;
pub mod engine;
pub mod predictions;
pub mod routing;
pub mod types;

pub use allocation::*;
pub use characteristics::*;
pub use collaboration::*;
pub use engine::*;
pub use predictions::*;
pub use routing::*;
pub use types::*;

/// AI Workload Classification Response Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadClassificationData {
    /// Unique workload identifier
    pub workload_id: String,

    /// Classified workload type
    pub workload_type: WorkloadType,

    /// AI confidence in classification (0.0 - 1.0)
    pub classification_confidence: f64,

    /// Workload characteristics analysis
    pub characteristics: WorkloadCharacteristics,

    /// Optimal routing strategy
    pub routing_strategy: RoutingStrategy,

    /// Resource allocation recommendations
    pub resource_allocation: ResourceAllocation,

    /// Human-AI collaboration requirements
    pub collaboration_requirements: CollaborationRequirements,

    /// Performance predictions
    pub performance_predictions: WorkloadPerformancePredictions,

    /// Risk assessment
    pub risk_assessment: WorkloadRiskAssessment,

    /// Processing timeline
    pub timeline: ProcessingTimeline,
}

/// Request structure for workload classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadClassificationRequest {
    /// Unique workload identifier
    pub workload_id: String,

    /// Request metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Request source information
    pub source: RequestSource,

    /// Performance requirements
    pub performance_requirements: PerformanceRequirements,

    /// Security requirements
    pub security_requirements: SecurityRequirements,

    /// Timestamp of the request
    pub timestamp: DateTime<Utc>,
}

/// Request source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSource {
    /// Source IP address
    pub ip_address: Option<String>,

    /// Source service or application
    pub source_service: Option<String>,

    /// User agent information
    pub user_agent: Option<String>,

    /// User context
    pub user_context: Option<String>,

    /// Request origin type
    pub origin_type: String,
}

/// Performance requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: Option<f64>,

    /// Required throughput (requests per second)
    pub required_throughput_rps: Option<f64>,

    /// Availability requirements (0.0 - 1.0)
    pub availability: Option<f64>,
}

/// Security requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Encryption required
    pub encryption_required: bool,

    /// Authentication level required
    pub auth_level: String,

    /// Data sensitivity level
    pub data_sensitivity: String,

    /// Compliance requirements
    pub compliance_requirements: Vec<String>,
}
