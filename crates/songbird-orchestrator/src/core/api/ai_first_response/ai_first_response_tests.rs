// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

fn rid() -> Uuid {
    Uuid::nil()
}

#[test]
fn success_sets_flags_and_default_metadata() {
    let r = AIFirstResponse::success("payload", rid(), 12, 0.88);
    assert!(r.is_success());
    assert!(!r.is_error());
    assert_eq!(r.data, "payload");
    assert!(r.error.is_none());
    assert_eq!(r.processing_time_ms, 12);
    assert_eq!(r.confidence_score, 0.88);
    assert!(r.suggested_actions.is_empty());
}

#[test]
fn error_sets_failure_and_zero_confidence() {
    let err = AIFirstError {
        code: "E".to_string(),
        message: "m".to_string(),
        category: AIErrorCategory::SystemError,
        retry_strategy: RetryStrategy {
            should_retry: false,
            delay_ms: 0,
            max_attempts: 0,
            backoff_strategy: BackoffType::Linear,
            retry_conditions: vec![],
            success_probability: 0.0,
        },
        automation_hints: vec![],
        severity: ErrorSeverity::Low,
        requires_human_intervention: false,
        context: HashMap::new(),
    };
    let r = AIFirstResponse::error(42_i32, err, rid(), 5);
    assert!(r.is_error());
    assert!(!r.is_success());
    assert_eq!(r.confidence_score, 0.0);
    assert_eq!(r.error.as_ref().expect("err").code, "E");
}

#[test]
fn into_data_consumes_envelope() {
    let r = AIFirstResponse::success("x".to_string(), rid(), 1, 1.0);
    assert_eq!(r.into_data(), "x");
}

#[test]
fn with_human_context_round_trips() {
    let ctx = HumanInteractionContext {
        user_id: Some("u1".to_string()),
        approval_required: true,
        confidence_threshold: 0.7,
    };
    let r = AIFirstResponse::success((), rid(), 0, 1.0).with_human_context(ctx.clone());
    assert_eq!(r.human_context.as_ref().expect("ctx").user_id, ctx.user_id);
}

#[test]
fn with_ai_metadata_replaces_block() {
    let mut m = AIResponseMetadata::default();
    m.dependencies.push("dep".to_string());
    let r = AIFirstResponse::success(0_u8, rid(), 2, 0.5).with_ai_metadata(m.clone());
    assert_eq!(r.ai_metadata.dependencies, m.dependencies);
}

#[test]
fn with_suggested_actions_preserves_vec() {
    let actions = vec![SuggestedAction {
        action_type: "retry".to_string(),
        parameters: HashMap::from([("k".to_string(), json!("v"))]),
        priority: ActionPriority::High,
        expected_outcome: "ok".to_string(),
        confidence: 0.9,
        requires_human_approval: false,
        estimated_execution_time: Some(Duration::from_secs(1)),
    }];
    let r = AIFirstResponse::success((), rid(), 0, 1.0).with_suggested_actions(actions);
    assert_eq!(r.suggested_actions.len(), 1);
    assert_eq!(r.suggested_actions[0].action_type, "retry");
}

#[test]
fn builder_chain_order_independent_for_metadata_and_actions() {
    let mut meta = AIResponseMetadata::default();
    meta.performance.latency_ms = 3.0;
    let actions = vec![SuggestedAction {
        action_type: "a".to_string(),
        parameters: HashMap::new(),
        priority: ActionPriority::Low,
        expected_outcome: String::new(),
        confidence: 1.0,
        requires_human_approval: false,
        estimated_execution_time: None,
    }];
    let r = AIFirstResponse::success(1_i32, rid(), 9, 0.5)
        .with_ai_metadata(meta.clone())
        .with_suggested_actions(actions.clone());
    assert_eq!(r.ai_metadata.performance.latency_ms, 3.0);
    assert_eq!(r.suggested_actions.len(), 1);
    let r2 = AIFirstResponse::success(1_i32, rid(), 9, 0.5)
        .with_suggested_actions(actions)
        .with_ai_metadata(meta);
    assert_eq!(r2.suggested_actions.len(), 1);
    assert_eq!(r2.ai_metadata.performance.latency_ms, 3.0);
}

#[test]
fn service_mesh_failure_sets_category_and_context() {
    let e = AIFirstError::service_mesh_failure("payments", "upstream timeout");
    assert_eq!(e.category, AIErrorCategory::ServiceMeshFailure);
    assert_eq!(e.context.get("failed_service").and_then(|v| v.as_str()), Some("payments"));
    assert!(e.retry_strategy.should_retry);
    assert_eq!(
        e.retry_strategy.backoff_strategy,
        BackoffType::Exponential {
            base: 2.0
        }
    );
}

#[test]
fn human_intervention_required_sets_flags_and_linear_backoff() {
    let e = AIFirstError::human_intervention_required("quota");
    assert_eq!(e.category, AIErrorCategory::HumanInterventionRequired);
    assert!(e.requires_human_intervention);
    assert!(!e.retry_strategy.should_retry);
    assert_eq!(e.retry_strategy.backoff_strategy, BackoffType::Linear);
    assert!(e.message.contains("quota"));
}

#[test]
fn serde_roundtrip_ai_first_response() {
    let r = AIFirstResponse::success(json!({"a": 1}), rid(), 4, 0.33);
    let s = serde_json::to_string(&r).expect("serialize");
    let back: AIFirstResponse<serde_json::Value> = serde_json::from_str(&s).expect("deserialize");
    assert!(back.is_success());
    assert_eq!(back.data, json!({"a": 1}));
}

#[test]
fn error_serde_preserves_code_and_category() {
    let e = AIFirstError::human_intervention_required("x");
    let r = AIFirstResponse::error((), e, rid(), 1);
    let s = serde_json::to_string(&r).expect("serialize");
    let back: AIFirstResponse<()> = serde_json::from_str(&s).expect("deserialize");
    assert!(back.is_error());
    assert_eq!(back.error.expect("e").category, AIErrorCategory::HumanInterventionRequired);
}

#[test]
fn default_quality_metrics_are_sane() {
    let q = QualityMetrics::default();
    assert_eq!(q.accuracy, 1.0);
    assert_eq!(q.reliability, 1.0);
}

#[test]
fn default_routing_metadata_empty_maps() {
    let r = RoutingMetadata::default();
    assert!(r.service_health_scores.is_empty());
    assert!(r.decision_factors.is_empty());
}

#[test]
fn backoff_type_custom_roundtrip() {
    let b = BackoffType::Custom {
        formula: "x^2".to_string(),
    };
    let s = serde_json::to_string(&b).expect("serialize");
    let back: BackoffType = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(back, b);
}

#[test]
fn action_priority_all_variants_serde() {
    for p in
        [ActionPriority::Low, ActionPriority::Medium, ActionPriority::High, ActionPriority::Urgent]
    {
        let j = serde_json::to_string(&p).expect("serialize");
        let back: ActionPriority = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back, p);
    }
}

#[test]
fn rate_limit_status_default_skips_window_duration_on_wire() {
    let r = RateLimitStatus::default();
    let j = serde_json::to_string(&r).expect("serialize");
    assert!(!j.contains("window_duration"));
    let back: RateLimitStatus = serde_json::from_str(&j).expect("deserialize");
    assert!(!back.is_rate_limited);
}

#[test]
fn routing_decision_factor_roundtrip() {
    let f = RoutingDecisionFactor {
        name: "latency".to_string(),
        weight: 0.4,
        value: json!("12ms"),
        impact: "prefer_closer".to_string(),
    };
    let j = serde_json::to_string(&f).expect("serialize");
    let back: RoutingDecisionFactor = serde_json::from_str(&j).expect("deserialize");
    assert_eq!(back.name, f.name);
    assert!((back.weight - 0.4).abs() < f64::EPSILON);
}

#[test]
fn routing_metadata_with_scores_roundtrip() {
    let mut m = RoutingMetadata::default();
    m.selected_endpoint = Some("ep1".to_string());
    m.available_endpoints = 2;
    m.load_balancing_algorithm = Some("weighted_round_robin".to_string());
    m.service_health_scores.insert("a".to_string(), 0.9);
    m.decision_factors.push(RoutingDecisionFactor {
        name: "health".to_string(),
        weight: 1.0,
        value: json!(0.9),
        impact: "high".to_string(),
    });
    let j = serde_json::to_string(&m).expect("serialize");
    let back: RoutingMetadata = serde_json::from_str(&j).expect("deserialize");
    assert_eq!(back.available_endpoints, 2);
    assert_eq!(back.service_health_scores.get("a").copied(), Some(0.9));
}

#[test]
fn ai_error_category_exhaustive_sample_serde() {
    for cat in [
        AIErrorCategory::ServiceDiscoveryFailure,
        AIErrorCategory::LoadBalancingFailure,
        AIErrorCategory::CircuitBreakerOpen,
        AIErrorCategory::Unknown,
    ] {
        let j = serde_json::to_string(&cat).expect("serialize");
        let back: AIErrorCategory = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back, cat);
    }
}

#[test]
fn confidence_score_boundary_values_in_success_constructor() {
    let r = AIFirstResponse::success((), rid(), 0, 0.0);
    assert!((r.confidence_score - 0.0).abs() < f64::EPSILON);
    let r2 = AIFirstResponse::success((), rid(), 0, 1.0);
    assert!((r2.confidence_score - 1.0).abs() < f64::EPSILON);
}
