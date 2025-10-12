//! AI-First /// Configuration capability Configuration
//!
//! Configuration structures for AI-First Citizen API compliance, including
//! confidence scoring, human-AI collaboration, and workload classification.

use serde::{Deserialize, Serialize};

/// AI-First Citizen API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFirstConfig {
    /// Enable AI-First response formatting
    /// Enable Ai Responses field
    pub enable_ai_responses: bool,

    /// Confidence scoring settings
    /// Confidence Scoring field
    pub confidence_scoring: ConfidenceScoringConfig,
    /// Human-AI collaboration settings
    /// Human Collaboration field
    pub human_collaboration: HumanCollaborationConfig,
    /// AI workload classification settings
    /// Workload Classification field
    pub workload_classification: WorkloadClassificationConfig,
    /// Real-time streaming interface settings
    /// Streaming Interface field
    pub streaming_interface: StreamingInterfaceConfig,
}

/// Confidence scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScoringConfig {
    /// Enable confidence scoring
    /// Enabled field
    pub enabled: bool,
    /// Minimum confidence threshold (0.0-1.0)
    /// Min Threshold field
    pub min_threshold: f64,
    /// High confidence threshold (0.0-1.0)
    /// High Threshold field
    pub high_threshold: f64,
    /// Use machine learning for scoring
    /// Use Ml Scoring field
    pub use_ml_scoring: bool,
}

/// Human-AI collaboration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanCollaborationConfig {
    /// Enable human-AI collaboration features
    /// Enabled field
    pub enabled: bool,
    /// Require human approval for high-impact decisions
    /// Require Human Approval field
    pub require_human_approval: bool,
    /// Escalation threshold for human intervention
    /// Escalation Threshold field
    pub escalation_threshold: f64,
    /// Maximum wait time for human response (seconds)
    /// Max Human Response Time field
    pub max_human_response_time: u64,
}

/// Workload classification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadClassificationConfig {
    /// Enable automatic workload classification
    /// Enabled field
    pub enabled: bool,
    /// Classification strategies
    pub strategies: Vec<ClassificationStrategy>,
    /// Update classification model interval (hours)
    /// Model Update Interval field
    pub model_update_interval: u64,
}

/// Streaming interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingInterfaceConfig {
    /// Enable real-time streaming
    /// Enabled field
    pub enabled: bool,
    /// Maximum concurrent streams
    pub max_concurrent_streams: usize,
    /// Stream buffer size
    pub buffer_size: usize,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval: u64,
}

/// Classification strategies for workload analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClassificationStrategy {
    /// Rule-based classification
    RuleBased,
    /// Machine learning classification
    MachineLearning,
    /// Hybrid approach
    Hybrid,
    /// Custom strategy
    Custom(String),
}

impl Default for AIFirstConfig {
    fn default() -> Self {
        Self {
            enable_ai_responses: true,
            confidence_scoring: ConfidenceScoringConfig::default(),
            human_collaboration: HumanCollaborationConfig::default(),
            workload_classification: WorkloadClassificationConfig::default(),
            streaming_interface: StreamingInterfaceConfig::default(),
        }
    }
}

impl Default for ConfidenceScoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_threshold: 0.7,
            high_threshold: 0.9,
            use_ml_scoring: true,
        }
    }
}

impl Default for HumanCollaborationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_human_approval: false,
            escalation_threshold: 0.5,
            max_human_response_time: 300, // 5 minutes
        }
    }
}

impl Default for WorkloadClassificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategies: vec![ClassificationStrategy::Hybrid],
            model_update_interval: 24, // 24 hours
        }
    }
}

impl Default for StreamingInterfaceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_streams: 100,
            buffer_size: 8192,
            heartbeat_interval: 30,
        }
    }
}
