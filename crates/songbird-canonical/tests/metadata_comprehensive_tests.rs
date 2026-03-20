// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
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
    clippy::must_use_candidate
)]

//! Comprehensive tests for AI metadata types
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde_json::json;
use songbird_canonical::metadata::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// AIResponseMetadata Tests
// ============================================================================

#[test]
fn test_ai_response_metadata_default() {
    let metadata = AIResponseMetadata::default();

    assert!(metadata.automation_capabilities.is_empty());
    assert!(metadata.custom_fields.is_empty());
}

#[test]
fn test_ai_response_metadata_with_automation_capability() {
    let capability = AutomationCapability::new("test-capability", "Test description", 0.8);
    let metadata = AIResponseMetadata::default().with_automation_capability(capability);

    assert_eq!(metadata.automation_capabilities.len(), 1);
    assert_eq!(metadata.automation_capabilities[0].capability, "test-capability");
}

#[test]
fn test_ai_response_metadata_with_multiple_automation_capabilities() {
    let cap1 = AutomationCapability::new("capability-1", "Description 1", 0.8);
    let cap2 = AutomationCapability::new("capability-2", "Description 2", 0.9);

    let metadata = AIResponseMetadata::default()
        .with_automation_capability(cap1)
        .with_automation_capability(cap2);

    assert_eq!(metadata.automation_capabilities.len(), 2);
}

#[test]
fn test_ai_response_metadata_with_custom_field() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default().with_custom_field("field1", json!("value1"));

    assert_eq!(metadata.custom_fields.len(), 1);
    assert!(metadata.custom_fields.contains_key("field1"));
    Ok(())
}

#[test]
fn test_ai_response_metadata_with_multiple_custom_fields() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default()
        .with_custom_field("field1", json!("value1"))
        .with_custom_field("field2", json!(42))
        .with_custom_field("field3", json!({"nested": "object"}));

    assert_eq!(metadata.custom_fields.len(), 3);
    Ok(())
}

#[test]
fn test_ai_response_metadata_custom_field_json_string() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default().with_custom_field("test", json!("string_value"));

    let value = metadata
        .custom_fields
        .get("test")
        .ok_or_else(|| SongbirdError::configuration("Should have field 'test'".to_string()))?;
    assert_eq!(value, &json!("string_value"));
    Ok(())
}

#[test]
fn test_ai_response_metadata_custom_field_json_number() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default().with_custom_field("count", json!(42));

    let value = metadata
        .custom_fields
        .get("count")
        .ok_or_else(|| SongbirdError::configuration("Should have field 'count'".to_string()))?;
    assert_eq!(value, &json!(42));
    Ok(())
}

#[test]
fn test_ai_response_metadata_custom_field_json_object() -> SongbirdResult<()> {
    let metadata =
        AIResponseMetadata::default().with_custom_field("config", json!({"key": "value"}));

    let value = metadata
        .custom_fields
        .get("config")
        .ok_or_else(|| SongbirdError::configuration("Should have field 'config'".to_string()))?;
    assert!(value.is_object());
    Ok(())
}

#[test]
fn test_ai_response_metadata_serialization() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default();
    let serialized = serde_json::to_string(&metadata);

    assert!(serialized.is_ok());
    Ok(())
}

#[test]
fn test_ai_response_metadata_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let metadata = AIResponseMetadata::default();
    let serialized =
        serde_json::to_string(&metadata).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {e}"),
            debug_info: None,
        })?;
    let deserialized: Result<AIResponseMetadata, _> = serde_json::from_str(&serialized);

    assert!(deserialized.is_ok());
    Ok(())
}

// ============================================================================
// DecisionContext Tests
// ============================================================================

#[test]
fn test_decision_context_default() {
    let context = DecisionContext::default();

    assert!(context.influencing_factors.is_empty());
    assert!(context.alternatives_considered.is_empty());
    assert!(context.reasoning.is_none());
    assert_eq!(context.risk_level, RiskLevel::Low);
}

#[test]
fn test_decision_context_with_influencing_factors() {
    let mut context = DecisionContext::default();
    context.influencing_factors.push("factor1".to_string());
    context.influencing_factors.push("factor2".to_string());

    assert_eq!(context.influencing_factors.len(), 2);
}

#[test]
fn test_decision_context_with_alternatives() -> SongbirdResult<()> {
    let mut context = DecisionContext::default();
    context.alternatives_considered.push("alternative1".to_string());
    context.alternatives_considered.push("alternative2".to_string());

    assert_eq!(context.alternatives_considered.len(), 2);
    Ok(())
}

#[test]
fn test_decision_context_with_reasoning() -> SongbirdResult<()> {
    let context = DecisionContext {
        reasoning: Some("This is the reasoning".to_string()),
        ..Default::default()
    };

    assert!(context.reasoning.is_some());
    assert_eq!(
        context
            .reasoning
            .as_ref()
            .ok_or_else(|| SongbirdError::configuration("Should have reasoning".to_string()))?,
        "This is the reasoning"
    );
    Ok(())
}

#[test]
fn test_decision_context_with_risk_level() -> SongbirdResult<()> {
    let context = DecisionContext {
        risk_level: RiskLevel::High,
        ..Default::default()
    };

    assert_eq!(context.risk_level, RiskLevel::High);
    Ok(())
}

#[test]
fn test_decision_context_serialization() -> SongbirdResult<()> {
    let context = DecisionContext::default();
    let serialized = serde_json::to_string(&context);

    assert!(serialized.is_ok());
    Ok(())
}

#[test]
fn test_decision_context_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let context = DecisionContext::default();
    let serialized = serde_json::to_string(&context).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {e}"),
        debug_info: None,
    })?;
    let deserialized: Result<DecisionContext, _> = serde_json::from_str(&serialized);

    assert!(deserialized.is_ok());
    Ok(())
}

// ============================================================================
// RiskLevel Tests
// ============================================================================

#[test]
fn test_risk_level_low() {
    let risk = RiskLevel::Low;
    assert_eq!(risk, RiskLevel::Low);
}

#[test]
fn test_risk_level_medium() {
    let risk = RiskLevel::Medium;
    assert_eq!(risk, RiskLevel::Medium);
}

#[test]
fn test_risk_level_high() -> SongbirdResult<()> {
    let risk = RiskLevel::High;
    assert_eq!(risk, RiskLevel::High);
    Ok(())
}

#[test]
fn test_risk_level_critical() -> SongbirdResult<()> {
    let risk = RiskLevel::Critical;
    assert_eq!(risk, RiskLevel::Critical);
    Ok(())
}

#[test]
fn test_risk_level_ordering() -> SongbirdResult<()> {
    let low = RiskLevel::Low;
    let medium = RiskLevel::Medium;

    assert_ne!(low, medium);
    Ok(())
}

#[test]
fn test_risk_level_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let risk = RiskLevel::High;
    let serialized = serde_json::to_string(&risk).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {e}"),
        debug_info: None,
    })?;

    assert!(serialized.contains("High"));
    Ok(())
}

#[test]
fn test_risk_level_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let risk = RiskLevel::Medium;
    let serialized = serde_json::to_string(&risk).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {e}"),
        debug_info: None,
    })?;
    let deserialized: RiskLevel = serde_json::from_str(&serialized)
        .map_err(|_e| SongbirdError::configuration("Error occurred".to_string()))?;

    assert_eq!(risk, deserialized);
    Ok(())
}

// ============================================================================
// AutomationCapability Tests
// ============================================================================

#[test]
fn test_automation_capability_new() {
    let capability = AutomationCapability::new("test-cap", "Test capability", 0.8);

    assert_eq!(capability.capability, "test-cap");
    assert_eq!(capability.description, "Test capability");
    assert!((capability.confidence_threshold - 0.8).abs() < f64::EPSILON);
}

#[test]
fn test_automation_capability_confidence_clamped_low() {
    let capability = AutomationCapability::new("test", "Test", -0.5);

    assert!((capability.confidence_threshold - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_automation_capability_confidence_clamped_high() {
    let capability = AutomationCapability::new("test", "Test", 1.5);

    assert!((capability.confidence_threshold - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_automation_capability_confidence_zero() {
    let capability = AutomationCapability::new("test", "Test", 0.0);

    assert!((capability.confidence_threshold - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_automation_capability_confidence_one() {
    let capability = AutomationCapability::new("test", "Test", 1.0);

    assert!((capability.confidence_threshold - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_automation_capability_with_prerequisite() {
    let capability =
        AutomationCapability::new("test", "Test", 0.8).with_prerequisite("prerequisite1");

    assert_eq!(capability.prerequisites.len(), 1);
    assert_eq!(capability.prerequisites[0], "prerequisite1");
}

#[test]
fn test_automation_capability_with_multiple_prerequisites() {
    let capability = AutomationCapability::new("test", "Test", 0.8)
        .with_prerequisite("prerequisite1")
        .with_prerequisite("prerequisite2")
        .with_prerequisite("prerequisite3");

    assert_eq!(capability.prerequisites.len(), 3);
}

#[test]
fn test_automation_capability_no_prerequisites_by_default() {
    let capability = AutomationCapability::new("test", "Test", 0.8);

    assert!(capability.prerequisites.is_empty());
}

#[test]
fn test_automation_capability_equality() {
    let cap1 = AutomationCapability::new("test", "Test", 0.8);
    let cap2 = AutomationCapability::new("test", "Test", 0.8);

    assert_eq!(cap1, cap2);
}

#[test]
fn test_automation_capability_inequality_different_name() {
    let cap1 = AutomationCapability::new("test1", "Test", 0.8);
    let cap2 = AutomationCapability::new("test2", "Test", 0.8);

    assert_ne!(cap1, cap2);
}

#[test]
fn test_automation_capability_inequality_different_confidence() -> SongbirdResult<()> {
    let cap1 = AutomationCapability::new("test", "Test", 0.7);
    let cap2 = AutomationCapability::new("test", "Test", 0.8);

    assert_ne!(cap1, cap2);
    Ok(())
}

#[test]
fn test_automation_capability_serialization() -> SongbirdResult<()> {
    let capability = AutomationCapability::new("test", "Test capability", 0.8);
    let serialized = serde_json::to_string(&capability);

    assert!(serialized.is_ok());
    Ok(())
}

#[test]
fn test_automation_capability_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let capability = AutomationCapability::new("test", "Test capability", 0.8);
    let serialized =
        serde_json::to_string(&capability).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {e}"),
            debug_info: None,
        })?;
    let deserialized: Result<AutomationCapability, _> = serde_json::from_str(&serialized);

    assert!(deserialized.is_ok());
    Ok(())
}

// ============================================================================
// QualityMetrics Tests
// ============================================================================

#[test]
fn test_quality_metrics_default() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default();

    assert!(metrics.accuracy.is_none());
    assert!(metrics.completeness.is_none());
    assert!(metrics.relevance.is_none());
    assert!(metrics.timeliness.is_none());
    assert!(metrics.overall_quality.is_none());
    Ok(())
}

#[test]
fn test_quality_metrics_with_accuracy() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(0.9);

    assert!(metrics.accuracy.is_some());
    let accuracy = metrics
        .accuracy
        .ok_or_else(|| SongbirdError::configuration("Should have accuracy".to_string()))?;
    assert!((accuracy - 0.9).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn test_quality_metrics_with_completeness() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_completeness(0.85);

    assert!(metrics.completeness.is_some());
    let completeness = metrics
        .completeness
        .ok_or_else(|| SongbirdError::configuration("Should have completeness".to_string()))?;
    assert!((completeness - 0.85).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn test_quality_metrics_with_relevance() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_relevance(0.92);

    assert!(metrics.relevance.is_some());
    let relevance = metrics
        .relevance
        .ok_or_else(|| SongbirdError::configuration("Should have relevance".to_string()))?;
    assert!((relevance - 0.92).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn test_quality_metrics_with_timeliness() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_timeliness(0.88);

    assert!(metrics.timeliness.is_some());
    let timeliness = metrics
        .timeliness
        .ok_or_else(|| SongbirdError::configuration("Should have timeliness".to_string()))?;
    assert!((timeliness - 0.88).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn test_quality_metrics_accuracy_clamped_low() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(-0.5);

    let accuracy = metrics
        .accuracy
        .ok_or_else(|| SongbirdError::configuration("Should have accuracy".to_string()))?;
    assert!((accuracy - 0.0).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn test_quality_metrics_accuracy_clamped_high() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(1.5);

    let accuracy = metrics
        .accuracy
        .ok_or_else(|| SongbirdError::configuration("Should have accuracy".to_string()))?;
    assert!((accuracy - 1.0).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn test_quality_metrics_calculate_overall_single_metric() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(0.8);

    assert!(metrics.overall_quality.is_some());
    let overall = metrics
        .overall_quality
        .ok_or_else(|| SongbirdError::configuration("Should have overall quality".to_string()))?;
    assert!((overall - 0.8).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn test_quality_metrics_calculate_overall_two_metrics() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(0.8).with_completeness(0.9);

    assert!(metrics.overall_quality.is_some());
    let overall = metrics
        .overall_quality
        .ok_or_else(|| SongbirdError::configuration("Should have overall quality".to_string()))?;
    assert!((overall - 0.85).abs() < 0.001);
    Ok(())
}

#[test]
fn test_quality_metrics_calculate_overall_all_metrics() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default()
        .with_accuracy(0.9)
        .with_completeness(0.85)
        .with_relevance(0.92)
        .with_timeliness(0.88);

    assert!(metrics.overall_quality.is_some());
    let expected = (0.9 + 0.85 + 0.92 + 0.88) / 4.0;
    let overall = metrics
        .overall_quality
        .ok_or_else(|| SongbirdError::configuration("Should have overall quality".to_string()))?;
    assert!((overall - expected).abs() < 0.001);
    Ok(())
}

#[test]
fn test_quality_metrics_calculate_overall_updates_on_change() -> SongbirdResult<()> {
    let mut metrics = QualityMetrics::default().with_accuracy(0.8);
    let first_overall = metrics.overall_quality.ok_or_else(|| {
        SongbirdError::configuration("Should have first overall quality".to_string())
    })?;

    metrics = metrics.with_completeness(0.9);
    let second_overall = metrics.overall_quality.ok_or_else(|| {
        SongbirdError::configuration("Should have second overall quality".to_string())
    })?;

    assert!((first_overall - second_overall).abs() > 0.001);
    Ok(())
}

#[test]
fn test_quality_metrics_calculate_overall_no_metrics() -> SongbirdResult<()> {
    let mut metrics = QualityMetrics::default();
    metrics.calculate_overall();

    assert!(metrics.overall_quality.is_none());
    Ok(())
}

#[test]
fn test_quality_metrics_serialization() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(0.9);
    let serialized = serde_json::to_string(&metrics);

    assert!(serialized.is_ok());
    Ok(())
}

#[test]
fn test_quality_metrics_deserialization() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(0.9);
    let serialized = serde_json::to_string(&metrics)
        .map_err(|_e| SongbirdError::configuration("Should serialize".to_string()))?;
    let deserialized: Result<QualityMetrics, _> = serde_json::from_str(&serialized);

    assert!(deserialized.is_ok());
    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_ai_response_metadata_workflow() {
    let capability1 = AutomationCapability::new("capability-1", "First capability", 0.8)
        .with_prerequisite("auth");
    let capability2 = AutomationCapability::new("capability-2", "Second capability", 0.9);

    let metadata = AIResponseMetadata::default()
        .with_automation_capability(capability1)
        .with_automation_capability(capability2)
        .with_custom_field("request_id", json!("req-12345"))
        .with_custom_field("priority", json!(5));

    assert_eq!(metadata.automation_capabilities.len(), 2);
    assert_eq!(metadata.custom_fields.len(), 2);
}

#[test]
fn test_decision_context_full_workflow() {
    let mut context = DecisionContext::default();
    context.influencing_factors.push("user_preference".to_string());
    context.influencing_factors.push("system_load".to_string());
    context.alternatives_considered.push("option_a".to_string());
    context.alternatives_considered.push("option_b".to_string());
    context.reasoning = Some("Option B provides better performance".to_string());
    context.risk_level = RiskLevel::Medium;

    assert_eq!(context.influencing_factors.len(), 2);
    assert_eq!(context.alternatives_considered.len(), 2);
    assert!(context.reasoning.is_some());
    assert_eq!(context.risk_level, RiskLevel::Medium);
}

#[test]
fn test_quality_metrics_comprehensive() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default()
        .with_accuracy(0.95)
        .with_completeness(0.90)
        .with_relevance(0.92)
        .with_timeliness(0.88);

    assert!(metrics.accuracy.is_some());
    assert!(metrics.completeness.is_some());
    assert!(metrics.relevance.is_some());
    assert!(metrics.timeliness.is_some());
    assert!(metrics.overall_quality.is_some());

    let expected_overall = (0.95 + 0.90 + 0.92 + 0.88) / 4.0;
    let overall = metrics
        .overall_quality
        .ok_or_else(|| SongbirdError::configuration("Should have overall quality".to_string()))?;
    assert!((overall - expected_overall).abs() < 0.001);
    Ok(())
}
