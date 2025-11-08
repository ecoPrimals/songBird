//! Tests for AI-First Configuration
//!
//! Comprehensive test coverage for AI-First Citizen API configuration.

use super::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// AIFirstConfig Tests
// ============================================================================

#[test]
fn test_ai_first_config_default() -> SongbirdResult<()> {
    let config = AIFirstConfig::default();

    assert!(config.enable_ai_responses);
    assert!(config.confidence_scoring.enabled);
    assert!(config.human_collaboration.enabled);
    assert!(config.workload_classification.enabled);
    assert!(config.streaming_interface.enabled);
    Ok(())
}

#[test]
fn test_ai_first_config_serialization() -> SongbirdResult<()> {
    let config = AIFirstConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
    let deserialized: AIFirstConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {}", e)))?;

    assert_eq!(config.enable_ai_responses, deserialized.enable_ai_responses);
    Ok(())
}

#[test]
fn test_ai_first_config_clone() {
    let config = AIFirstConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enable_ai_responses, cloned.enable_ai_responses);
}

// ============================================================================
// ConfidenceScoringConfig Tests
// ============================================================================

#[test]
fn test_confidence_scoring_default() {
    let config = ConfidenceScoringConfig::default();

    assert!(config.enabled);
    assert_eq!(config.min_threshold, 0.7);
    assert_eq!(config.high_threshold, 0.9);
    assert!(config.use_ml_scoring);
}

#[test]
fn test_confidence_scoring_custom_thresholds() {
    let mut config = ConfidenceScoringConfig::default();
    config.min_threshold = 0.5;
    config.high_threshold = 0.95;

    assert_eq!(config.min_threshold, 0.5);
    assert_eq!(config.high_threshold, 0.95);
}

#[test]
fn test_confidence_scoring_ml_disabled() {
    let mut config = ConfidenceScoringConfig::default();
    config.use_ml_scoring = false;

    assert!(!config.use_ml_scoring);
}

#[test]
fn test_confidence_scoring_disabled() {
    let mut config = ConfidenceScoringConfig::default();
    config.enabled = false;

    assert!(!config.enabled);
}

// ============================================================================
// HumanCollaborationConfig Tests
// ============================================================================

#[test]
fn test_human_collaboration_default() {
    let config = HumanCollaborationConfig::default();

    assert!(config.enabled);
    assert!(!config.require_human_approval);
    assert_eq!(config.escalation_threshold, 0.5);
    assert_eq!(config.max_human_response_time, 300);
}

#[test]
fn test_human_collaboration_with_approval() {
    let mut config = HumanCollaborationConfig::default();
    config.require_human_approval = true;

    assert!(config.require_human_approval);
}

#[test]
fn test_human_collaboration_custom_escalation() {
    let mut config = HumanCollaborationConfig::default();
    config.escalation_threshold = 0.3;

    assert_eq!(config.escalation_threshold, 0.3);
}

#[test]
fn test_human_collaboration_response_timeout() {
    let mut config = HumanCollaborationConfig::default();
    config.max_human_response_time = 600; // 10 minutes

    assert_eq!(config.max_human_response_time, 600);
}

// ============================================================================
// WorkloadClassificationConfig Tests
// ============================================================================

#[test]
fn test_workload_classification_default() {
    let config = WorkloadClassificationConfig::default();

    assert!(config.enabled);
    assert_eq!(config.strategies.len(), 1);
    assert_eq!(config.model_update_interval, 24);
}

#[test]
fn test_workload_classification_multiple_strategies() {
    let mut config = WorkloadClassificationConfig::default();
    config.strategies = vec![
        ClassificationStrategy::RuleBased,
        ClassificationStrategy::MachineLearning,
        ClassificationStrategy::Hybrid,
    ];

    assert_eq!(config.strategies.len(), 3);
}

#[test]
fn test_workload_classification_custom_interval() {
    let mut config = WorkloadClassificationConfig::default();
    config.model_update_interval = 12; // 12 hours

    assert_eq!(config.model_update_interval, 12);
}

// ============================================================================
// StreamingInterfaceConfig Tests
// ============================================================================

#[test]
fn test_streaming_interface_default() {
    let config = StreamingInterfaceConfig::default();

    assert!(config.enabled);
    assert_eq!(config.max_concurrent_streams, 100);
    assert_eq!(config.buffer_size, 8192);
    assert_eq!(config.heartbeat_interval, 30);
}

#[test]
fn test_streaming_interface_custom_streams() {
    let mut config = StreamingInterfaceConfig::default();
    config.max_concurrent_streams = 200;

    assert_eq!(config.max_concurrent_streams, 200);
}

#[test]
fn test_streaming_interface_buffer_size() {
    let mut config = StreamingInterfaceConfig::default();
    config.buffer_size = 16384;

    assert_eq!(config.buffer_size, 16384);
}

#[test]
fn test_streaming_interface_heartbeat() {
    let mut config = StreamingInterfaceConfig::default();
    config.heartbeat_interval = 60;

    assert_eq!(config.heartbeat_interval, 60);
}

// ============================================================================
// ClassificationStrategy Tests
// ============================================================================

#[test]
fn test_classification_strategy_variants() -> SongbirdResult<()> {
    let rule_based = ClassificationStrategy::RuleBased;
    let ml = ClassificationStrategy::MachineLearning;
    let hybrid = ClassificationStrategy::Hybrid;
    let custom = ClassificationStrategy::Custom("my-strategy".to_string());

    // Just verify they can be created
    let _ = rule_based;
    let _ = ml;
    let _ = hybrid;
    let _ = custom;
    Ok(())
}

#[test]
fn test_classification_strategy_serialization() -> SongbirdResult<()> {
    let strategy = ClassificationStrategy::Hybrid;
    let json = serde_json::to_string(&strategy)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
    let deserialized: ClassificationStrategy = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {}", e)))?;

    // Verify deserialization works
    let _ = deserialized;
    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_ai_first_config_full_custom() {
    let config = AIFirstConfig {
        enable_ai_responses: true,
        confidence_scoring: ConfidenceScoringConfig {
            enabled: true,
            min_threshold: 0.6,
            high_threshold: 0.95,
            use_ml_scoring: true,
        },
        human_collaboration: HumanCollaborationConfig {
            enabled: true,
            require_human_approval: true,
            escalation_threshold: 0.4,
            max_human_response_time: 600,
        },
        workload_classification: WorkloadClassificationConfig {
            enabled: true,
            strategies: vec![
                ClassificationStrategy::RuleBased,
                ClassificationStrategy::MachineLearning,
            ],
            model_update_interval: 12,
        },
        streaming_interface: StreamingInterfaceConfig {
            enabled: true,
            max_concurrent_streams: 150,
            buffer_size: 16384,
            heartbeat_interval: 45,
        },
    };

    assert_eq!(config.confidence_scoring.min_threshold, 0.6);
    assert!(config.human_collaboration.require_human_approval);
    assert_eq!(config.workload_classification.strategies.len(), 2);
    assert_eq!(config.streaming_interface.max_concurrent_streams, 150);
}

#[test]
fn test_ai_first_config_all_disabled() {
    let mut config = AIFirstConfig::default();
    config.enable_ai_responses = false;
    config.confidence_scoring.enabled = false;
    config.human_collaboration.enabled = false;
    config.workload_classification.enabled = false;
    config.streaming_interface.enabled = false;

    assert!(!config.enable_ai_responses);
    assert!(!config.confidence_scoring.enabled);
    assert!(!config.human_collaboration.enabled);
    assert!(!config.workload_classification.enabled);
    assert!(!config.streaming_interface.enabled);
}

#[test]
fn test_ai_first_config_conservative_settings() {
    let config = AIFirstConfig {
        enable_ai_responses: true,
        confidence_scoring: ConfidenceScoringConfig {
            enabled: true,
            min_threshold: 0.9, // Very high threshold
            high_threshold: 0.99,
            use_ml_scoring: false, // Rule-based only
        },
        human_collaboration: HumanCollaborationConfig {
            enabled: true,
            require_human_approval: true,  // Always require approval
            escalation_threshold: 0.1,     // Escalate frequently
            max_human_response_time: 1800, // 30 minutes
        },
        workload_classification: WorkloadClassificationConfig {
            enabled: true,
            strategies: vec![ClassificationStrategy::RuleBased], // Conservative approach
            model_update_interval: 168,                          // Weekly
        },
        streaming_interface: StreamingInterfaceConfig {
            enabled: false, // Disabled for safety
            max_concurrent_streams: 10,
            buffer_size: 4096,
            heartbeat_interval: 15,
        },
    };

    assert_eq!(config.confidence_scoring.min_threshold, 0.9);
    assert!(config.human_collaboration.require_human_approval);
    assert!(!config.streaming_interface.enabled);
}
