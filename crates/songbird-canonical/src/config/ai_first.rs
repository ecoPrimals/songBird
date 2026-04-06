// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_json_roundtrip<T>(value: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let j = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&j).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), j);
    }

    #[test]
    fn ai_first_config_default_non_trivial() {
        let c = AIFirstConfig::default();
        assert!(c.enable_ai_responses);
        assert!(c.confidence_scoring.enabled);
        assert!((c.confidence_scoring.min_threshold - 0.7).abs() < f64::EPSILON);
        assert_eq!(c.workload_classification.model_update_interval, 24);
        assert_eq!(c.streaming_interface.buffer_size, 8192);
    }

    #[test]
    fn ai_first_config_roundtrip() {
        assert_json_roundtrip(&AIFirstConfig::default());
    }

    #[test]
    fn confidence_scoring_default_and_roundtrip() {
        assert_json_roundtrip(&ConfidenceScoringConfig::default());
    }

    #[test]
    fn human_collaboration_default_and_roundtrip() {
        assert_json_roundtrip(&HumanCollaborationConfig::default());
    }

    #[test]
    fn workload_classification_default_and_roundtrip() {
        assert_json_roundtrip(&WorkloadClassificationConfig::default());
    }

    #[test]
    fn streaming_interface_default_and_roundtrip() {
        assert_json_roundtrip(&StreamingInterfaceConfig::default());
    }

    #[test]
    fn classification_strategy_variants_roundtrip() {
        for s in [
            ClassificationStrategy::RuleBased,
            ClassificationStrategy::MachineLearning,
            ClassificationStrategy::Hybrid,
            ClassificationStrategy::Custom("ml-v2".to_string()),
        ] {
            assert_json_roundtrip(&s);
        }
    }
}
