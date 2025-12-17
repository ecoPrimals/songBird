//! Tests for AI-First Metadata
//!
//! Comprehensive test coverage for AI response metadata and decision context.

use super::metadata::*;
use serde_json::json;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// AIResponseMetadata Tests
// ============================================================================

#[test]
fn test_ai_response_metadata_default() {
    let metadata = AIResponseMetadata::default();

    assert!(metadata.automation_capabilities.is_empty());
    assert!(metadata.custom_fields.is_empty());
    assert_eq!(metadata.decision_context.risk_level, RiskLevel::Low);
}

#[test]
fn test_ai_response_metadata_with_automation_capability() {
    let capability = AutomationCapability::new("auto-retry", "Automatic retry on failure", 0.8);
    let metadata = AIResponseMetadata::default().with_automation_capability(capability);

    assert_eq!(metadata.automation_capabilities.len(), 1);
    assert_eq!(metadata.automation_capabilities[0].capability, "auto-retry");
}

#[test]
fn test_ai_response_metadata_with_multiple_capabilities() {
    let cap1 = AutomationCapability::new("retry", "Auto retry", 0.8);
    let cap2 = AutomationCapability::new("fallback", "Auto fallback", 0.9);
    let cap3 = AutomationCapability::new("escalate", "Auto escalate", 0.7);

    let metadata = AIResponseMetadata::default()
        .with_automation_capability(cap1)
        .with_automation_capability(cap2)
        .with_automation_capability(cap3);

    assert_eq!(metadata.automation_capabilities.len(), 3);
}

#[test]
fn test_ai_response_metadata_with_custom_field() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default()
        .with_custom_field("user_id", json!("user-123"))
        .with_custom_field("priority", json!(5));

    assert_eq!(metadata.custom_fields.len(), 2);
    assert_eq!(metadata.custom_fields.get("user_id"), Some(&json!("user-123")));
    assert_eq!(metadata.custom_fields.get("priority"), Some(&json!(5)));
    Ok(())
}

#[test]
fn test_ai_response_metadata_serialization() -> SongbirdResult<()> {
    let metadata = AIResponseMetadata::default().with_custom_field("test", json!("value"));

    let json = serde_json::to_string(&metadata)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
    let deserialized: AIResponseMetadata = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;

    assert_eq!(metadata.custom_fields.len(), deserialized.custom_fields.len());
    Ok(())
}

#[test]
fn test_ai_response_metadata_clone() {
    let metadata = AIResponseMetadata::default().with_custom_field("key", json!("value"));
    let cloned = metadata.clone();

    assert_eq!(metadata.custom_fields.len(), cloned.custom_fields.len());
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
fn test_decision_context_with_factors() {
    let mut context = DecisionContext::default();
    context.influencing_factors.push("load-high".to_string());
    context.influencing_factors.push("latency-critical".to_string());

    assert_eq!(context.influencing_factors.len(), 2);
}

#[test]
fn test_decision_context_with_alternatives() -> SongbirdResult<()> {
    let mut context = DecisionContext::default();
    context.alternatives_considered.push("option-a".to_string());
    context.alternatives_considered.push("option-b".to_string());
    context.alternatives_considered.push("option-c".to_string());

    assert_eq!(context.alternatives_considered.len(), 3);
    Ok(())
}

#[test]
fn test_decision_context_with_reasoning() -> SongbirdResult<()> {
    let mut context = DecisionContext::default();
    context.reasoning = Some("Chose option A due to better performance".to_string());

    assert!(context.reasoning.is_some());
    assert!(context.reasoning.as_ref().unwrap().contains("performance"));
    Ok(())
}

#[test]
fn test_decision_context_risk_levels() -> SongbirdResult<()> {
    let mut context = DecisionContext::default();

    context.risk_level = RiskLevel::Low;
    assert_eq!(context.risk_level, RiskLevel::Low);

    context.risk_level = RiskLevel::Medium;
    assert_eq!(context.risk_level, RiskLevel::Medium);

    context.risk_level = RiskLevel::High;
    assert_eq!(context.risk_level, RiskLevel::High);

    context.risk_level = RiskLevel::Critical;
    assert_eq!(context.risk_level, RiskLevel::Critical);
    Ok(())
}

#[test]
fn test_decision_context_serialization() -> SongbirdResult<()> {
    let mut context = DecisionContext::default();
    context.influencing_factors.push("factor1".to_string());
    context.risk_level = RiskLevel::High;

    let json = serde_json::to_string(&context)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
    let deserialized: DecisionContext = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;

    assert_eq!(context.risk_level, deserialized.risk_level);
    assert_eq!(context.influencing_factors.len(), deserialized.influencing_factors.len());
    Ok(())
}

// ============================================================================
// RiskLevel Tests
// ============================================================================

#[test]
fn test_risk_level_variants() -> SongbirdResult<()> {
    let low = RiskLevel::Low;
    let medium = RiskLevel::Medium;
    let high = RiskLevel::High;
    let critical = RiskLevel::Critical;

    assert_eq!(low, RiskLevel::Low);
    assert_eq!(medium, RiskLevel::Medium);
    assert_eq!(high, RiskLevel::High);
    assert_eq!(critical, RiskLevel::Critical);
    Ok(())
}

#[test]
fn test_risk_level_equality() -> SongbirdResult<()> {
    assert_eq!(RiskLevel::Low, RiskLevel::Low);
    assert_ne!(RiskLevel::Low, RiskLevel::High);
    assert_ne!(RiskLevel::Medium, RiskLevel::Critical);
    Ok(())
}

#[test]
fn test_risk_level_serialization() -> SongbirdResult<()> {
    let levels = vec![RiskLevel::Low, RiskLevel::Medium, RiskLevel::High, RiskLevel::Critical];

    for level in levels {
        let json = serde_json::to_string(&level)
            .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
        let deserialized: RiskLevel = serde_json::from_str(&json)
            .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;
        assert_eq!(level, deserialized);
    }
    Ok(())
}

// ============================================================================
// AutomationCapability Tests
// ============================================================================

#[test]
fn test_automation_capability_new() {
    let capability = AutomationCapability::new("auto-scale", "Automatically scale resources", 0.85);

    assert_eq!(capability.capability, "auto-scale");
    assert_eq!(capability.description, "Automatically scale resources");
    assert_eq!(capability.confidence_threshold, 0.85);
    assert!(capability.prerequisites.is_empty());
}

#[test]
fn test_automation_capability_confidence_clamping() -> SongbirdResult<()> {
    let too_high = AutomationCapability::new("test", "Test", 1.5);
    let too_low = AutomationCapability::new("test", "Test", -0.5);
    let normal = AutomationCapability::new("test", "Test", 0.7);

    assert_eq!(too_high.confidence_threshold, 1.0);
    assert_eq!(too_low.confidence_threshold, 0.0);
    assert_eq!(normal.confidence_threshold, 0.7);
    Ok(())
}

#[test]
fn test_automation_capability_with_prerequisite() -> SongbirdResult<()> {
    let capability = AutomationCapability::new("deploy", "Auto deploy", 0.9)
        .with_prerequisite("tests-passed")
        .with_prerequisite("review-approved");

    assert_eq!(capability.prerequisites.len(), 2);
    assert!(capability.prerequisites.contains(&"tests-passed".to_string()));
    assert!(capability.prerequisites.contains(&"review-approved".to_string()));
    Ok(())
}

#[test]
fn test_automation_capability_serialization() -> SongbirdResult<()> {
    let capability = AutomationCapability::new("test", "Test capability", 0.8)
        .with_prerequisite("prerequisite-1");

    let json = serde_json::to_string(&capability)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
    let deserialized: AutomationCapability = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;

    assert_eq!(capability.capability, deserialized.capability);
    assert_eq!(capability.confidence_threshold, deserialized.confidence_threshold);
    Ok(())
}

#[test]
fn test_automation_capability_equality() {
    let cap1 = AutomationCapability::new("test", "Test", 0.8);
    let cap2 = AutomationCapability::new("test", "Test", 0.8);

    assert_eq!(cap1, cap2);
}

// ============================================================================
// QualityMetrics Tests
// ============================================================================

#[test]
fn test_quality_metrics_default() {
    let metrics = QualityMetrics::default();

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
    assert_eq!(metrics.overall_quality, Some(0.95));
}

#[test]
fn test_quality_metrics_with_completeness() {
    let metrics = QualityMetrics::default().with_completeness(0.88);

    assert_eq!(metrics.completeness, Some(0.88));
    assert_eq!(metrics.overall_quality, Some(0.88));
}

#[test]
fn test_quality_metrics_with_relevance() {
    let metrics = QualityMetrics::default().with_relevance(0.92);

    assert_eq!(metrics.relevance, Some(0.92));
    assert_eq!(metrics.overall_quality, Some(0.92));
}

#[test]
fn test_quality_metrics_with_timeliness() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_timeliness(0.85);

    assert_eq!(metrics.timeliness, Some(0.85));
    assert_eq!(metrics.overall_quality, Some(0.85));
    Ok(())
}

#[test]
fn test_quality_metrics_overall_calculation() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default()
        .with_accuracy(0.9)
        .with_completeness(0.8)
        .with_relevance(0.85)
        .with_timeliness(0.95);

    // Overall should be average of all metrics
    let expected = (0.9 + 0.8 + 0.85 + 0.95) / 4.0;
    assert!((metrics.overall_quality.unwrap() - expected).abs() < 0.001);
    Ok(())
}

#[test]
fn test_quality_metrics_partial_calculation() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(0.9).with_relevance(0.8);

    // Overall should be average of provided metrics only
    let expected = f64::midpoint(0.9, 0.8);
    assert!((metrics.overall_quality.unwrap() - expected).abs() < 0.001);
    Ok(())
}

#[test]
fn test_quality_metrics_clamping() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default()
        .with_accuracy(1.5) // Too high
        .with_completeness(-0.5); // Too low

    assert_eq!(metrics.accuracy, Some(1.0));
    assert_eq!(metrics.completeness, Some(0.0));
    Ok(())
}

#[test]
fn test_quality_metrics_calculate_overall_manual() -> SongbirdResult<()> {
    let mut metrics = QualityMetrics {
        accuracy: Some(0.9),
        completeness: Some(0.8),
        relevance: Some(0.85),
        timeliness: Some(0.75),
        overall_quality: None,
    };

    metrics.calculate_overall();

    assert!(metrics.overall_quality.is_some());
    let expected = (0.9 + 0.8 + 0.85 + 0.75) / 4.0;
    assert!((metrics.overall_quality.unwrap() - expected).abs() < 0.001);
    Ok(())
}

#[test]
fn test_quality_metrics_serialization() -> SongbirdResult<()> {
    let metrics = QualityMetrics::default().with_accuracy(0.9).with_completeness(0.8);

    let json = serde_json::to_string(&metrics)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
    let deserialized: QualityMetrics = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;

    assert_eq!(metrics.accuracy, deserialized.accuracy);
    assert_eq!(metrics.completeness, deserialized.completeness);
    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_ai_response_metadata_integration() {
    let capability1 =
        AutomationCapability::new("retry", "Auto retry", 0.9).with_prerequisite("idempotent");
    let capability2 = AutomationCapability::new("scale", "Auto scale", 0.8)
        .with_prerequisite("metrics-available");

    let metadata = AIResponseMetadata {
        decision_context: DecisionContext {
            influencing_factors: vec!["high-load".to_string(), "low-latency-required".to_string()],
            alternatives_considered: vec!["manual-scale".to_string(), "auto-scale".to_string()],
            reasoning: Some("Chose auto-scale due to predictable load patterns".to_string()),
            risk_level: RiskLevel::Medium,
        },
        automation_capabilities: vec![capability1, capability2],
        quality_metrics: QualityMetrics::default()
            .with_accuracy(0.95)
            .with_completeness(0.90)
            .with_relevance(0.92)
            .with_timeliness(0.88),
        custom_fields: {
            let mut fields = std::collections::HashMap::new();
            fields.insert("request_id".to_string(), json!("req-12345"));
            fields.insert("user_id".to_string(), json!("user-456"));
            fields
        },
    };

    assert_eq!(metadata.decision_context.risk_level, RiskLevel::Medium);
    assert_eq!(metadata.automation_capabilities.len(), 2);
    assert!(metadata.quality_metrics.overall_quality.is_some());
    assert_eq!(metadata.custom_fields.len(), 2);
}

#[test]
fn test_high_risk_decision_context() {
    let mut context = DecisionContext::default();
    context.risk_level = RiskLevel::Critical;
    context.reasoning = Some("Critical operation requires immediate human oversight".to_string());
    context.influencing_factors =
        vec!["financial-impact-high".to_string(), "data-sensitivity-critical".to_string()];

    assert_eq!(context.risk_level, RiskLevel::Critical);
    assert!(context.reasoning.is_some());
    assert_eq!(context.influencing_factors.len(), 2);
}

#[test]
fn test_low_confidence_automation_capability() {
    let capability = AutomationCapability::new(
        "experimental-feature",
        "New experimental automation",
        0.3, // Low confidence
    )
    .with_prerequisite("beta-opt-in")
    .with_prerequisite("monitoring-enabled")
    .with_prerequisite("rollback-plan-ready");

    assert!(capability.confidence_threshold < 0.5);
    assert_eq!(capability.prerequisites.len(), 3);
}
