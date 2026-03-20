// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Tests for AI-First Configuration
//!
//! Comprehensive tests for AI-First Citizen API configuration structures

use songbird_canonical::config::ai_first::{
    AIFirstConfig, ClassificationStrategy, ConfidenceScoringConfig, HumanCollaborationConfig,
    StreamingInterfaceConfig, WorkloadClassificationConfig,
};
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_ai_first_config_default() {
    let config = AIFirstConfig::default();

    assert!(config.enable_ai_responses);
    assert!(config.confidence_scoring.enabled);
    assert!(config.human_collaboration.enabled);
    assert!(config.workload_classification.enabled);
    assert!(config.streaming_interface.enabled);
}

#[test]
fn test_confidence_scoring_defaults() {
    let config = ConfidenceScoringConfig::default();

    assert!(config.enabled);
    assert!((config.min_threshold - 0.7).abs() < 0.001);
    assert!((config.high_threshold - 0.9).abs() < 0.001);
    assert!(config.use_ml_scoring);
    assert!(config.min_threshold < config.high_threshold);
}

#[test]
fn test_confidence_scoring_custom() {
    let config = ConfidenceScoringConfig {
        enabled: true,
        min_threshold: 0.5,
        high_threshold: 0.95,
        use_ml_scoring: false,
    };

    assert!(config.enabled);
    assert!((config.min_threshold - 0.5).abs() < 0.001);
    assert!((config.high_threshold - 0.95).abs() < 0.001);
    assert!(!config.use_ml_scoring);
}

#[test]
fn test_human_collaboration_defaults() {
    let config = HumanCollaborationConfig::default();

    assert!(config.enabled);
    assert!(!config.require_human_approval);
    assert!((config.escalation_threshold - 0.5).abs() < 0.001);
    assert_eq!(config.max_human_response_time, 300); // 5 minutes
}

#[test]
fn test_human_collaboration_strict_mode() {
    let config = HumanCollaborationConfig {
        enabled: true,
        require_human_approval: true,
        escalation_threshold: 0.3,
        max_human_response_time: 60,
    };

    assert!(config.enabled);
    assert!(config.require_human_approval);
    assert!((config.escalation_threshold - 0.3).abs() < 0.001);
    assert_eq!(config.max_human_response_time, 60);
}

#[test]
fn test_workload_classification_defaults() {
    let config = WorkloadClassificationConfig::default();

    assert!(config.enabled);
    assert_eq!(config.strategies.len(), 1);
    assert_eq!(config.model_update_interval, 24);
}

#[test]
fn test_workload_classification_custom() {
    let config = WorkloadClassificationConfig {
        enabled: true,
        strategies: vec![
            ClassificationStrategy::RuleBased,
            ClassificationStrategy::MachineLearning,
            ClassificationStrategy::Hybrid,
        ],
        model_update_interval: 12,
    };

    assert_eq!(config.strategies.len(), 3);
    assert_eq!(config.model_update_interval, 12);
}

#[test]
fn test_classification_strategy_variants() {
    let rule_based = ClassificationStrategy::RuleBased;
    let ml = ClassificationStrategy::MachineLearning;
    let hybrid = ClassificationStrategy::Hybrid;
    let custom = ClassificationStrategy::Custom("MyStrategy".to_string());

    // Verify all variants can be created
    assert!(matches!(rule_based, ClassificationStrategy::RuleBased));
    assert!(matches!(ml, ClassificationStrategy::MachineLearning));
    assert!(matches!(hybrid, ClassificationStrategy::Hybrid));
    assert!(matches!(custom, ClassificationStrategy::Custom(_)));
}

#[test]
fn test_streaming_interface_defaults() {
    let config = StreamingInterfaceConfig::default();

    assert!(config.enabled);
    assert_eq!(config.max_concurrent_streams, 100);
    assert_eq!(config.buffer_size, 8192);
    assert_eq!(config.heartbeat_interval, 30);
}

#[test]
fn test_streaming_interface_high_load() {
    let config = StreamingInterfaceConfig {
        enabled: true,
        max_concurrent_streams: 1000,
        buffer_size: 65536,
        heartbeat_interval: 10,
    };

    assert!(config.enabled);
    assert_eq!(config.max_concurrent_streams, 1000);
    assert_eq!(config.buffer_size, 65536);
    assert_eq!(config.heartbeat_interval, 10);
}

#[test]
fn test_ai_first_config_serialization() -> SongbirdResult<()> {
    let config = AIFirstConfig::default();

    // Test JSON serialization
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("enable_ai_responses"));
    assert!(json.contains("confidence_scoring"));

    // Test deserialization
    let deserialized: AIFirstConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Should deserialize: {}", e),
            debug_info: None,
        })?;
    assert_eq!(config.enable_ai_responses, deserialized.enable_ai_responses);
    Ok(())
}

#[test]
fn test_ai_first_config_clone() {
    let config = AIFirstConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enable_ai_responses, cloned.enable_ai_responses);
    assert!(
        (config.confidence_scoring.min_threshold - cloned.confidence_scoring.min_threshold).abs()
            < 0.001
    );
}

#[test]
fn test_confidence_thresholds_validation() {
    // Valid configuration
    let valid = ConfidenceScoringConfig {
        enabled: true,
        min_threshold: 0.6,
        high_threshold: 0.9,
        use_ml_scoring: true,
    };

    assert!(valid.min_threshold < valid.high_threshold);
    assert!(valid.min_threshold >= 0.0 && valid.min_threshold <= 1.0);
    assert!(valid.high_threshold >= 0.0 && valid.high_threshold <= 1.0);
}

#[test]
fn test_ai_first_config_debug() {
    let config = AIFirstConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("AIFirstConfig"));
    assert!(debug_str.contains("enable_ai_responses"));
}

#[test]
fn test_classification_strategy_custom() {
    let custom1 = ClassificationStrategy::Custom("Strategy1".to_string());
    let custom2 = ClassificationStrategy::Custom("Strategy2".to_string());

    if let ClassificationStrategy::Custom(name1) = custom1 {
        assert_eq!(name1, "Strategy1");
    }

    if let ClassificationStrategy::Custom(name2) = custom2 {
        assert_eq!(name2, "Strategy2");
    }
}

#[test]
fn test_streaming_interface_disabled() {
    let config = StreamingInterfaceConfig {
        enabled: false,
        max_concurrent_streams: 0,
        buffer_size: 1024,
        heartbeat_interval: 60,
    };

    assert!(!config.enabled);
    assert_eq!(config.max_concurrent_streams, 0);
}

#[test]
fn test_human_collaboration_timeout_variations() -> SongbirdResult<()> {
    let fast = HumanCollaborationConfig {
        enabled: true,
        require_human_approval: true,
        escalation_threshold: 0.8,
        max_human_response_time: 30,
    };

    let slow = HumanCollaborationConfig {
        enabled: true,
        require_human_approval: true,
        escalation_threshold: 0.2,
        max_human_response_time: 600,
    };

    assert!(fast.max_human_response_time < slow.max_human_response_time);
    assert!(fast.escalation_threshold > slow.escalation_threshold);
    Ok(())
}

#[test]
fn test_workload_classification_serialization() -> SongbirdResult<()> {
    let config = WorkloadClassificationConfig {
        enabled: true,
        strategies: vec![ClassificationStrategy::Hybrid],
        model_update_interval: 48,
    };

    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    let deserialized: WorkloadClassificationConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Should deserialize: {}", e),
            debug_info: None,
        })?;

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.model_update_interval, deserialized.model_update_interval);
    Ok(())
}
