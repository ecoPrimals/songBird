// SPDX-License-Identifier: AGPL-3.0-or-later
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
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for AI-first metadata types
//!
//! This test suite provides thorough coverage of the metadata system
//! used for AI decision-making and automation in Songbird.

use serde_json::json;
use songbird_canonical::metadata::{
    AIResponseMetadata, AutomationCapability, DecisionContext, QualityMetrics, RiskLevel,
};
use songbird_types::{SongbirdError, SongbirdResult};

// ========== AIResponseMetadata Tests ==========

#[test]
fn test_ai_response_metadata_default() {
    let metadata = AIResponseMetadata::default();

    assert!(metadata.automation_capabilities.is_empty());
    assert!(metadata.custom_fields.is_empty());
    assert_eq!(metadata.decision_context.risk_level, RiskLevel::Low);
}

#[test]
fn test_ai_response_metadata_with_automation_capability() {
    let capability =
        AutomationCapability::new("data_processing", "Automated data transformation", 0.95);
    let metadata = AIResponseMetadata::default().with_automation_capability(capability);

    assert_eq!(metadata.automation_capabilities.len(), 1);
    assert_eq!(metadata.automation_capabilities[0].capability, "data_processing");
    assert!((metadata.automation_capabilities[0].confidence_threshold - 0.95).abs() < f64::EPSILON);
}

#[test]
fn test_ai_response_metadata_with_multiple_capabilities() {
    let cap1 = AutomationCapability::new("data_validation", "Automated data validation", 0.98);
    let cap2 = AutomationCapability::new("error_recovery", "Automated error recovery", 0.85);
    let cap3 =
        AutomationCapability::new("optimization", "Automated performance optimization", 0.90);

    let metadata = AIResponseMetadata::default()
        .with_automation_capability(cap1)
        .with_automation_capability(cap2)
        .with_automation_capability(cap3);

    assert_eq!(metadata.automation_capabilities.len(), 3);
    assert_eq!(metadata.automation_capabilities[0].capability, "data_validation");
    assert_eq!(metadata.automation_capabilities[1].capability, "error_recovery");
    assert_eq!(metadata.automation_capabilities[2].capability, "optimization");
}

#[test]
fn test_ai_response_metadata_with_custom_field() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default()
        .with_custom_field("session_id", json!("abc-123"))
        .with_custom_field("priority", json!(5));

    assert_eq!(metadata.custom_fields.len(), 2);
    assert_eq!(metadata.custom_fields.get("session_id"), Some(&json!("abc-123")));
    assert_eq!(metadata.custom_fields.get("priority"), Some(&json!(5)));
    Ok(())
}

#[test]
fn test_ai_response_metadata_with_complex_custom_fields() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default()
        .with_custom_field("user_context", json!({"user_id": 42, "role": "admin"}))
        .with_custom_field("tags", json!(["production", "critical", "monitored"]))
        .with_custom_field("threshold", json!(0.75));

    assert_eq!(metadata.custom_fields.len(), 3);

    let user_context = metadata
        .custom_fields
        .get("user_context")
        .ok_or_else(|| SongbirdError::configuration("user_context field not found".to_string()))?;
    assert!(user_context.is_object());

    let tags = metadata
        .custom_fields
        .get("tags")
        .ok_or_else(|| SongbirdError::configuration("tags field not found".to_string()))?;
    assert!(tags.is_array());
    Ok(())
}

#[test]
fn test_ai_response_metadata_serialization() -> SongbirdResult<()> {
    let capability = AutomationCapability::new("test_capability", "Test description", 0.90);
    let metadata = AIResponseMetadata::default()
        .with_automation_capability(capability)
        .with_custom_field("test_key", json!("test_value"));

    let json_str = serde_json::to_string(&metadata)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
    assert!(json_str.contains("automation_capabilities"));
    assert!(json_str.contains("decision_context"));
    assert!(json_str.contains("quality_metrics"));
    Ok(())
}

#[test]
fn test_ai_response_metadata_deserialization() -> SongbirdResult<()> {
    let json_data = r#"{
        "decision_context": {
            "influencing_factors": ["factor1"],
            "alternatives_considered": ["alt1"],
            "reasoning": "Test reasoning",
            "risk_level": "Medium"
        },
        "automation_capabilities": [],
        "quality_metrics": {
            "accuracy": 0.95,
            "confidence": 0.90,
            "completeness": 0.85,
            "reliability": 0.88
        },
        "custom_fields": {}
    }"#;

    let metadata: AIResponseMetadata = serde_json::from_str(json_data)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {}", e)))?;
    assert_eq!(metadata.decision_context.risk_level, RiskLevel::Medium);
    assert!(metadata.decision_context.reasoning.is_some());
    Ok(())
}

// ========== DecisionContext Tests ==========

#[test]
fn test_decision_context_default() {
    let context = DecisionContext::default();

    assert!(context.influencing_factors.is_empty());
    assert!(context.alternatives_considered.is_empty());
    assert!(context.reasoning.is_none());
    assert_eq!(context.risk_level, RiskLevel::Low);
}

#[test]
fn test_decision_context_with_factors() {
    let context = DecisionContext {
        influencing_factors: vec![
            "user_preference".to_string(),
            "system_load".to_string(),
            "security_policy".to_string(),
        ],
        alternatives_considered: vec!["option_a".to_string(), "option_b".to_string()],
        reasoning: Some("Chose based on security and performance balance".to_string()),
        risk_level: RiskLevel::Medium,
    };

    assert_eq!(context.influencing_factors.len(), 3);
    assert_eq!(context.alternatives_considered.len(), 2);
    assert!(context.reasoning.is_some());
    assert_eq!(context.risk_level, RiskLevel::Medium);
}

#[test]
fn test_decision_context_high_risk() -> SongbirdResult<()> {
    let context = DecisionContext {
        influencing_factors: vec!["data_sensitivity".to_string()],
        alternatives_considered: vec![],
        reasoning: Some("Critical operation requires human oversight".to_string()),
        risk_level: RiskLevel::High,
    };

    assert_eq!(context.risk_level, RiskLevel::High);
    assert!(context.reasoning.expect("test precondition").contains("human oversight"));
    Ok(())
}

#[test]
fn test_decision_context_critical_risk() {
    let context = DecisionContext {
        influencing_factors: vec!["system_security".to_string(), "data_integrity".to_string()],
        alternatives_considered: vec![],
        reasoning: Some("Requires immediate human intervention".to_string()),
        risk_level: RiskLevel::Critical,
    };

    assert_eq!(context.risk_level, RiskLevel::Critical);
}

#[test]
fn test_decision_context_serialization() {
    let context = DecisionContext {
        influencing_factors: vec!["factor1".to_string()],
        alternatives_considered: vec!["alt1".to_string()],
        reasoning: Some("test reasoning".to_string()),
        risk_level: RiskLevel::Low,
    };

    let serialized = serde_json::to_string(&context);
    assert!(serialized.is_ok());
}

// ========== RiskLevel Tests ==========

#[test]
fn test_risk_level_equality() -> SongbirdResult<()> {
    assert_eq!(RiskLevel::Low, RiskLevel::Low);
    assert_eq!(RiskLevel::Medium, RiskLevel::Medium);
    assert_eq!(RiskLevel::High, RiskLevel::High);
    assert_eq!(RiskLevel::Critical, RiskLevel::Critical);
    Ok(())
}

#[test]
fn test_risk_level_inequality() -> SongbirdResult<()> {
    assert_ne!(RiskLevel::Low, RiskLevel::Medium);
    assert_ne!(RiskLevel::Medium, RiskLevel::High);
    assert_ne!(RiskLevel::High, RiskLevel::Critical);
    Ok(())
}

#[test]
fn test_risk_level_debug() -> SongbirdResult<()> {
    let low = format!("{:?}", RiskLevel::Low);
    assert_eq!(low, "Low");

    let critical = format!("{:?}", RiskLevel::Critical);
    assert_eq!(critical, "Critical");
    Ok(())
}

#[test]
fn test_risk_level_clone() -> SongbirdResult<()> {
    let original = RiskLevel::High;
    let cloned = original.clone();
    assert_eq!(original, cloned);
    Ok(())
}

#[test]
fn test_risk_level_serialization() -> SongbirdResult<()> {
    let low = RiskLevel::Low;
    let serialized = serde_json::to_string(&low)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
    assert_eq!(serialized, "\"Low\"");

    let critical = RiskLevel::Critical;
    let serialized = serde_json::to_string(&critical)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
    assert_eq!(serialized, "\"Critical\"");
    Ok(())
}

#[test]
fn test_risk_level_deserialization() -> SongbirdResult<()> {
    let json_low = "\"Low\"";
    let low: RiskLevel = serde_json::from_str(json_low)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {}", e)))?;
    assert_eq!(low, RiskLevel::Low);

    let json_critical = "\"Critical\"";
    let critical: RiskLevel = serde_json::from_str(json_critical)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {}", e)))?;
    assert_eq!(critical, RiskLevel::Critical);
    Ok(())
}

// ========== AutomationCapability Tests ==========

#[test]
fn test_automation_capability_creation() {
    let capability = AutomationCapability::new(
        "data_transformation",
        "Automated data transformation between formats",
        0.92,
    );

    assert_eq!(capability.capability, "data_transformation");
    assert_eq!(capability.description, "Automated data transformation between formats");
    assert!((capability.confidence_threshold - 0.92).abs() < f64::EPSILON);
    assert!(capability.prerequisites.is_empty());
}

#[test]
fn test_automation_capability_with_prerequisites() {
    let mut capability =
        AutomationCapability::new("complex_analysis", "Automated complex data analysis", 0.85);
    capability.prerequisites.push("data_validation".to_string());
    capability.prerequisites.push("schema_verification".to_string());

    assert_eq!(capability.prerequisites.len(), 2);
    assert_eq!(capability.prerequisites[0], "data_validation");
    assert_eq!(capability.prerequisites[1], "schema_verification");
}

#[test]
fn test_automation_capability_high_confidence() {
    let capability = AutomationCapability::new("simple_validation", "Simple data validation", 0.99);

    assert!(capability.confidence_threshold > 0.95);
    assert!((capability.confidence_threshold - 0.99).abs() < f64::EPSILON);
}

#[test]
fn test_automation_capability_low_confidence() {
    let capability =
        AutomationCapability::new("experimental_feature", "Experimental AI feature", 0.60);

    assert!(capability.confidence_threshold < 0.70);
    assert!((capability.confidence_threshold - 0.60).abs() < f64::EPSILON);
}

#[test]
fn test_automation_capability_equality() {
    let cap1 = AutomationCapability::new("test", "Test capability", 0.90);
    let cap2 = AutomationCapability::new("test", "Test capability", 0.90);

    assert_eq!(cap1, cap2);
}

#[test]
fn test_automation_capability_inequality() -> SongbirdResult<()> {
    let cap1 = AutomationCapability::new("test1", "Test capability 1", 0.90);
    let cap2 = AutomationCapability::new("test2", "Test capability 2", 0.90);

    assert_ne!(cap1, cap2);
    Ok(())
}

#[test]
fn test_automation_capability_clone() -> SongbirdResult<()> {
    let original = AutomationCapability::new("test", "Test description", 0.85);
    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.capability, cloned.capability);
    assert!((original.confidence_threshold - cloned.confidence_threshold).abs() < 1e-10);
    Ok(())
}

#[test]
fn test_automation_capability_serialization() -> SongbirdResult<()> {
    let capability = AutomationCapability::new("test_capability", "Test description", 0.90);

    let json_str = serde_json::to_string(&capability)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
    assert!(json_str.contains("test_capability"));
    assert!(json_str.contains("0.9"));
    Ok(())
}

#[test]
fn test_automation_capability_deserialization() -> SongbirdResult<()> {
    let json_data = r#"{
        "capability": "test_cap",
        "description": "Test description",
        "prerequisites": ["prereq1", "prereq2"],
        "confidence_threshold": 0.85
    }"#;

    let result: Result<AutomationCapability, _> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let capability = result
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {}", e)))?;
    assert_eq!(capability.capability, "test_cap");
    assert_eq!(capability.prerequisites.len(), 2);
    assert!((capability.confidence_threshold - 0.85).abs() < f64::EPSILON);
    Ok(())
}

// ========== QualityMetrics Tests ==========

#[test]
fn test_quality_metrics_default() {
    let metrics = QualityMetrics::default();

    // All metrics should default to None
    assert!(metrics.accuracy.is_none());
    assert!(metrics.completeness.is_none());
    assert!(metrics.relevance.is_none());
    assert!(metrics.timeliness.is_none());
    assert!(metrics.overall_quality.is_none());
}

#[test]
fn test_quality_metrics_with_accuracy() {
    let metrics = QualityMetrics::default().with_accuracy(0.95);

    assert_eq!(metrics.accuracy, Some(0.95));
    assert!(metrics.overall_quality.is_some());
}

#[test]
fn test_quality_metrics_with_completeness() {
    let metrics = QualityMetrics::default().with_completeness(0.88);

    assert_eq!(metrics.completeness, Some(0.88));
    assert!(metrics.overall_quality.is_some());
}

#[test]
fn test_quality_metrics_with_relevance() {
    let metrics = QualityMetrics::default().with_relevance(0.92);

    assert_eq!(metrics.relevance, Some(0.92));
    assert!(metrics.overall_quality.is_some());
}

#[test]
fn test_quality_metrics_with_timeliness() {
    let metrics = QualityMetrics::default().with_timeliness(0.90);

    assert_eq!(metrics.timeliness, Some(0.90));
    assert!(metrics.overall_quality.is_some());
}

#[test]
fn test_quality_metrics_perfect_scores() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default()
        .with_accuracy(1.0)
        .with_completeness(1.0)
        .with_relevance(1.0)
        .with_timeliness(1.0);

    assert_eq!(metrics.accuracy, Some(1.0));
    assert_eq!(metrics.completeness, Some(1.0));
    assert_eq!(metrics.relevance, Some(1.0));
    assert_eq!(metrics.timeliness, Some(1.0));
    assert_eq!(metrics.overall_quality, Some(1.0));
    Ok(())
}

#[test]
fn test_quality_metrics_realistic_scores() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default()
        .with_accuracy(0.92)
        .with_completeness(0.95)
        .with_relevance(0.88)
        .with_timeliness(0.90);

    assert!(metrics.accuracy.expect("test precondition") > 0.90);
    assert!(metrics.completeness.expect("test precondition") > 0.90);
    assert!(metrics.relevance.expect("test precondition") > 0.85);
    assert!(metrics.timeliness.expect("test precondition") > 0.85);
    Ok(())
}

#[test]
fn test_quality_metrics_clamping_high() {
    let metrics = QualityMetrics::default().with_accuracy(1.5);

    // Should clamp to 1.0
    assert_eq!(metrics.accuracy, Some(1.0));
}

#[test]
fn test_quality_metrics_clamping_low() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(-0.5);

    // Should clamp to 0.0
    assert_eq!(metrics.accuracy, Some(0.0));
    Ok(())
}

#[test]
fn test_quality_metrics_calculate_overall() -> SongbirdResult<()> {
    let mut metrics = QualityMetrics {
        accuracy: Some(0.90),
        completeness: Some(0.80),
        relevance: Some(0.85),
        timeliness: Some(0.75),
        ..Default::default()
    };

    metrics.calculate_overall();

    // Average should be (0.90 + 0.80 + 0.85 + 0.75) / 4 = 0.825
    let overall = metrics.overall_quality.expect("test precondition");
    assert!((overall - 0.825).abs() < 0.001);
    Ok(())
}

#[test]
fn test_quality_metrics_overall_with_partial() -> SongbirdResult<()> {
    let mut metrics = QualityMetrics {
        accuracy: Some(0.90),
        completeness: Some(0.80),
        ..Default::default()
    };
    // relevance and timeliness are None

    metrics.calculate_overall();

    // Average should be (0.90 + 0.80) / 2 = 0.85
    let overall = metrics.overall_quality.expect("test precondition");
    assert!((overall - 0.85).abs() < 0.001);
    Ok(())
}

#[test]
fn test_quality_metrics_clone() -> SongbirdResult<()> {
    let original = QualityMetrics::default().with_accuracy(0.85).with_completeness(0.90);

    let cloned = original.clone();
    assert_eq!(original.accuracy, cloned.accuracy);
    assert_eq!(original.completeness, cloned.completeness);
    Ok(())
}

#[test]
fn test_quality_metrics_serialization() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(0.95).with_completeness(0.85);

    let json_str = serde_json::to_string(&metrics)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
    assert!(json_str.contains("accuracy"));
    assert!(json_str.contains("0.95"));
    Ok(())
}

#[test]
fn test_quality_metrics_deserialization() -> SongbirdResult<()> {
    let json_data = r#"{
        "accuracy": 0.92,
        "completeness": 0.95,
        "relevance": 0.88,
        "timeliness": 0.90,
        "overall_quality": 0.9125
    }"#;

    let metrics: QualityMetrics = serde_json::from_str(json_data)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {}", e)))?;
    assert_eq!(metrics.accuracy, Some(0.92));
    assert_eq!(metrics.completeness, Some(0.95));
    assert_eq!(metrics.relevance, Some(0.88));
    assert_eq!(metrics.timeliness, Some(0.90));
    Ok(())
}

// ========== Integration Tests ==========

#[test]
fn test_complete_ai_metadata_workflow() -> SongbirdResult<()> {
    // Build a complete AI metadata structure
    let cap1 = AutomationCapability::new("data_validation", "Automated validation", 0.95);
    let cap2 = AutomationCapability::new("error_recovery", "Automated recovery", 0.88);

    let metadata = AIResponseMetadata::default()
        .with_automation_capability(cap1)
        .with_automation_capability(cap2)
        .with_custom_field("session_id", json!("session-123"))
        .with_custom_field("user_id", json!(42));

    // Verify structure
    assert_eq!(metadata.automation_capabilities.len(), 2);
    assert_eq!(metadata.custom_fields.len(), 2);

    // Verify serialization round-trip
    let serialized = serde_json::to_string(&metadata).map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let deserialized: AIResponseMetadata =
        serde_json::from_str(&serialized).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {}", e),
            debug_info: None,
        })?;

    assert_eq!(deserialized.automation_capabilities.len(), 2);
    assert_eq!(deserialized.custom_fields.len(), 2);
    Ok(())
}

#[test]
fn test_risk_escalation_workflow() {
    // Low risk operation
    let low_risk_context = DecisionContext {
        influencing_factors: vec!["routine_operation".to_string()],
        alternatives_considered: vec![],
        reasoning: None,
        risk_level: RiskLevel::Low,
    };
    assert_eq!(low_risk_context.risk_level, RiskLevel::Low);

    // Escalate to medium
    let medium_risk_context = DecisionContext {
        risk_level: RiskLevel::Medium,
        ..low_risk_context
    };
    assert_eq!(medium_risk_context.risk_level, RiskLevel::Medium);

    // Escalate to high
    let high_risk_context = DecisionContext {
        risk_level: RiskLevel::High,
        reasoning: Some("Requires human oversight".to_string()),
        ..medium_risk_context
    };
    assert_eq!(high_risk_context.risk_level, RiskLevel::High);
    assert!(high_risk_context.reasoning.is_some());
}

#[test]
fn test_automation_decision_based_on_confidence() -> SongbirdResult<()> {
    let high_confidence =
        AutomationCapability::new("safe_operation", "Safe automated operation", 0.98);
    let low_confidence = AutomationCapability::new("risky_operation", "Risky operation", 0.60);

    // High confidence should allow automation
    assert!(high_confidence.confidence_threshold > 0.95);

    // Low confidence should require human review
    assert!(low_confidence.confidence_threshold < 0.70);
    Ok(())
}

#[test]
fn test_quality_metrics_builder_pattern() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default()
        .with_accuracy(0.90)
        .with_completeness(0.95)
        .with_relevance(0.85)
        .with_timeliness(0.88);

    assert_eq!(metrics.accuracy, Some(0.90));
    assert_eq!(metrics.completeness, Some(0.95));
    assert_eq!(metrics.relevance, Some(0.85));
    assert_eq!(metrics.timeliness, Some(0.88));

    // Overall should be calculated automatically
    let overall = metrics.overall_quality.expect("test precondition");
    assert!((overall - 0.895).abs() < 0.001);
    Ok(())
}

#[test]
fn test_complex_metadata_with_multiple_custom_fields() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default()
        .with_custom_field("request_id", json!("req-12345"))
        .with_custom_field(
            "user_context",
            json!({
                "user_id": 42,
                "role": "admin",
                "permissions": ["read", "write", "execute"]
            }),
        )
        .with_custom_field(
            "performance_metrics",
            json!({
                "latency_ms": 150,
                "memory_mb": 256
            }),
        )
        .with_custom_field("tags", json!(["production", "monitored", "critical"]));

    assert_eq!(metadata.custom_fields.len(), 4);

    // Verify complex field structure
    let user_context = metadata
        .custom_fields
        .get("user_context")
        .ok_or_else(|| SongbirdError::configuration("user_context field not found".to_string()))?;
    assert!(user_context.is_object());

    let tags = metadata
        .custom_fields
        .get("tags")
        .ok_or_else(|| SongbirdError::configuration("tags field not found".to_string()))?;
    assert!(tags.is_array());
    Ok(())
}

#[test]
fn test_decision_context_with_no_alternatives() -> SongbirdResult<()> {
    let context = DecisionContext {
        influencing_factors: vec!["only_option".to_string()],
        alternatives_considered: vec![],
        reasoning: Some("No alternatives available".to_string()),
        risk_level: RiskLevel::Medium,
    };

    assert!(context.alternatives_considered.is_empty());
    assert!(context.reasoning.expect("test precondition").contains("No alternatives"));
    Ok(())
}

#[test]
fn test_decision_context_with_many_alternatives() {
    let alternatives: Vec<String> = (1..=10).map(|i| format!("alternative_{}", i)).collect();

    let context = DecisionContext {
        influencing_factors: vec!["multiple_options".to_string()],
        alternatives_considered: alternatives,
        reasoning: Some("Evaluated all 10 alternatives".to_string()),
        risk_level: RiskLevel::Low,
    };

    assert_eq!(context.alternatives_considered.len(), 10);
}
