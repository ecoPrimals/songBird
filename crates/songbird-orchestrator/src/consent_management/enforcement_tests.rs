// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::enforcement::*;
use super::{ConsentManager, ConsentStatus, UserPreferences};
use crate::task_lifecycle::{TaskLifecycle, UserId};
use std::sync::Arc;
use std::time::Duration;

fn test_task(user_id: UserId, task_type: &str) -> TaskLifecycle {
    use crate::task_lifecycle::types::{Priority, ResourceRequirements, TaskSpec};

    TaskLifecycle::new(
        user_id,
        TaskSpec {
            task_type: task_type.into(),
            config: serde_json::json!({}),
            required_capabilities: vec![task_type.into()],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        },
    )
}

#[tokio::test]
async fn test_consent_not_required_for_cheap_operations() {
    let consent_manager = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(consent_manager);

    let task = test_task(UserId::from("alice"), "simple_task");

    assert!(!enforcer.requires_consent(&task, Some(10.0)));
}

#[tokio::test]
async fn test_consent_required_for_expensive_operations() {
    let consent_manager = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(consent_manager);

    let task = test_task(UserId::from("alice"), "expensive_task");

    assert!(enforcer.requires_consent(&task, Some(100.0)));
}

#[tokio::test]
async fn test_consent_required_for_sensitive_operations() {
    let consent_manager = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(consent_manager);

    let task = test_task(UserId::from("alice"), "gpu_training");

    // Should require consent even with low cost
    assert!(enforcer.requires_consent(&task, Some(5.0)));
}

#[tokio::test]
async fn test_enforcement_allows_cheap_operations() {
    let consent_manager = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(consent_manager);

    let task = test_task(UserId::from("alice"), "simple_task");

    let result = enforcer.enforce(&task, Some(10.0)).await.unwrap();

    assert!(matches!(result, EnforcementResult::Allowed { .. }));
}

#[tokio::test]
async fn test_enforcement_requests_consent_for_expensive() {
    let consent_manager = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(consent_manager);

    let task = test_task(UserId::from("alice"), "expensive_task");

    let result = enforcer.enforce(&task, Some(100.0)).await.unwrap();

    assert!(matches!(result, EnforcementResult::Pending { .. }));
}

#[tokio::test]
async fn test_dignity_checker_expensive_operations() {
    let violations = DignityChecker::check_operation("expensive_compute", Some(150.0), false);

    assert!(!violations.is_empty());
    assert!(violations[0].contains("transparent"));
}

#[tokio::test]
async fn test_dignity_checker_sensitive_operations() {
    let violations = DignityChecker::check_operation("delete_user_data", None, true);

    assert!(!violations.is_empty());
    assert!(violations[0].contains("delete"));
}

#[tokio::test]
async fn test_dignity_checker_safe_operations() {
    let violations = DignityChecker::check_operation("simple_task", Some(5.0), true);

    assert!(violations.is_empty());
}

#[test]
fn requires_consent_cost_at_threshold_not_required() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("alice"), "other");
    assert!(!enforcer.requires_consent(&task, Some(50.0)));
}

#[test]
fn requires_consent_cost_just_above_threshold() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("alice"), "other");
    assert!(enforcer.requires_consent(&task, Some(50.01)));
}

#[test]
fn dignity_expensive_but_transparent_no_cost_violation() {
    let v = DignityChecker::check_operation("compute", Some(150.0), true);
    assert!(!v.iter().any(|m| m.contains("transparent")), "{v:?}");
}

#[test]
fn dignity_sensitive_keyword_share() {
    let v = DignityChecker::check_operation("share_data", None, true);
    assert!(v.iter().any(|m| m.contains("share")));
}

#[tokio::test]
async fn wait_for_decision_approved() {
    let cm = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(cm.clone());
    let task = test_task(UserId::from("alice"), "op");
    let consent_id = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    assert!(cm.approve(consent_id.as_ref(), None).await);
    let r = enforcer.wait_for_decision(consent_id.as_ref()).await.unwrap();
    assert!(matches!(r, EnforcementResult::Allowed { .. }));
}

#[tokio::test]
async fn wait_for_decision_denied() {
    let cm = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(cm.clone());
    let task = test_task(UserId::from("alice"), "op");
    let consent_id = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    assert!(cm.deny(consent_id.as_ref(), None).await);
    let r = enforcer.wait_for_decision(consent_id.as_ref()).await.unwrap();
    assert!(matches!(r, EnforcementResult::Blocked { .. }));
}

#[tokio::test]
async fn wait_for_decision_times_out_fail_safe_deny() {
    let cm = Arc::new(ConsentManager::new());
    let config = EnforcementConfig {
        default_timeout: std::time::Duration::from_millis(30),
        ..Default::default()
    };
    let enforcer = ConsentEnforcer::with_config(cm.clone(), config);
    let task = test_task(UserId::from("alice"), "op");
    let _consent_id = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    // Still pending — wait_for_decision should time out
    let r = enforcer.wait_for_decision(_consent_id.as_ref()).await.unwrap();
    assert!(matches!(
        r,
        EnforcementResult::Blocked {
            reason,
            ..
        } if reason.as_ref().contains("expired") || reason.as_ref().contains("timed out")
    ));
}

#[tokio::test]
async fn wait_for_decision_times_out_proceed() {
    let cm = Arc::new(ConsentManager::new());
    let config = EnforcementConfig {
        default_timeout: std::time::Duration::from_millis(30),
        timeout_behavior: TimeoutBehavior::Proceed,
        ..Default::default()
    };
    let enforcer = ConsentEnforcer::with_config(cm.clone(), config);
    let task = test_task(UserId::from("alice"), "op");
    let cid = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    let r = enforcer.wait_for_decision(cid.as_ref()).await.unwrap();
    assert!(matches!(r, EnforcementResult::Allowed { .. }));
}

#[tokio::test]
async fn check_consent_all_statuses() {
    let cm = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(cm.clone());
    let task = test_task(UserId::from("alice"), "op");

    let pending_id = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    assert!(matches!(
        enforcer.check_consent(pending_id.as_ref()).await,
        Some(EnforcementResult::Pending { .. })
    ));

    let approve_id = cm.request_consent(task.owner.clone(), task.id, "op2", Some(100.0)).await;
    cm.approve(approve_id.as_ref(), None).await;
    assert!(matches!(
        enforcer.check_consent(approve_id.as_ref()).await,
        Some(EnforcementResult::Allowed { .. })
    ));

    let deny_id = cm.request_consent(task.owner.clone(), task.id, "op3", Some(100.0)).await;
    cm.deny(deny_id.as_ref(), None).await;
    assert!(matches!(
        enforcer.check_consent(deny_id.as_ref()).await,
        Some(EnforcementResult::Blocked { .. })
    ));
}

#[tokio::test]
async fn with_config_exposes_custom_threshold() {
    let enforcer = ConsentEnforcer::with_config(
        Arc::new(ConsentManager::new()),
        EnforcementConfig {
            consent_required_above_cost: 500.0,
            ..Default::default()
        },
    );
    let task = test_task(UserId::from("alice"), "x");
    assert!(!enforcer.requires_consent(&task, Some(100.0)));
    assert!(enforcer.requires_consent(&task, Some(600.0)));
}

#[test]
fn requires_consent_none_cost_below_threshold_not_always_list() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("u"), "misc");
    assert!(!enforcer.requires_consent(&task, None));
}

#[test]
fn always_require_includes_delete_data() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("u"), "delete_data");
    assert!(enforcer.requires_consent(&task, None));
}

#[test]
fn always_require_includes_export_data() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("u"), "export_data");
    assert!(enforcer.requires_consent(&task, Some(0.0)));
}

#[test]
fn always_require_includes_gpu_training() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("u"), "gpu_training");
    assert!(enforcer.requires_consent(&task, Some(0.0)));
}

#[test]
fn enforcement_config_default_timeout_is_positive() {
    let c = EnforcementConfig::default();
    assert!(c.default_timeout.as_secs() > 0);
    assert_eq!(c.timeout_behavior, TimeoutBehavior::Deny);
}

#[test]
fn dignity_cost_exactly_100_non_transparent_no_violation() {
    let v = DignityChecker::check_operation("job", Some(100.0), false);
    assert!(v.is_empty());
}

#[test]
fn dignity_cost_100_01_non_transparent_flags() {
    let v = DignityChecker::check_operation("job", Some(100.01), false);
    assert!(v.iter().any(|m| m.contains("transparent")));
}

#[test]
fn dignity_keywords_gpu_and_training() {
    let v1 = DignityChecker::check_operation("run_gpu_step", None, true);
    assert!(v1.iter().any(|m| m.contains("gpu")));
    let v2 = DignityChecker::check_operation("model_training", None, true);
    assert!(v2.iter().any(|m| m.contains("training")));
}

#[test]
fn dignity_large_scale_keyword() {
    let v = DignityChecker::check_operation("large_scale_import", None, true);
    assert!(v.iter().any(|m| m.contains("large_scale")));
}

#[test]
fn dignity_sensitive_stops_after_first_match() {
    let v = DignityChecker::check_operation("delete_and_export", None, true);
    assert_eq!(v.len(), 1);
}

#[tokio::test]
async fn enforce_auto_approved_when_under_user_threshold() {
    let cm = Arc::new(ConsentManager::new());
    cm.set_user_preferences(
        UserId::from("bob"),
        UserPreferences {
            auto_approve_under_cost: Some(200.0),
            ..UserPreferences::default()
        },
    )
    .await;
    let enforcer = ConsentEnforcer::new(cm);
    let task = test_task(UserId::from("bob"), "pay");
    let r = enforcer.enforce(&task, Some(50.0)).await.unwrap();
    assert!(matches!(r, EnforcementResult::Allowed { .. }));
}

#[tokio::test]
async fn check_consent_unknown_id_returns_none() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    assert!(enforcer.check_consent("00000000-0000-0000-0000-000000000000").await.is_none());
}

#[tokio::test]
async fn wait_for_decision_blocked_preserves_consent_id() {
    let cm = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(cm.clone());
    let task = test_task(UserId::from("u"), "op");
    let cid = cm.request_consent(task.owner.clone(), task.id, "op", Some(99.0)).await;
    assert!(cm.deny(cid.as_ref(), None).await);
    let r = enforcer.wait_for_decision(cid.as_ref()).await.unwrap();
    assert!(matches!(
        r,
        EnforcementResult::Blocked {
            consent_id: Some(id),
            ..
        } if id.as_ref() == cid.as_ref()
    ));
}

#[test]
fn cost_nan_does_not_trigger_threshold_compare() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("u"), "misc");
    assert!(!enforcer.requires_consent(&task, Some(f64::NAN)));
}

#[test]
fn custom_always_require_list_respected() {
    let enforcer = ConsentEnforcer::with_config(
        Arc::new(ConsentManager::new()),
        EnforcementConfig {
            always_require_consent: vec!["custom_op".into()],
            ..Default::default()
        },
    );
    let task = test_task(UserId::from("u"), "custom_op");
    assert!(enforcer.requires_consent(&task, None));
}

#[test]
fn cost_infinity_exceeds_threshold() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("u"), "misc");
    assert!(enforcer.requires_consent(&task, Some(f64::INFINITY)));
}

#[test]
fn dignity_export_keyword() {
    let v = DignityChecker::check_operation("export", None, true);
    assert!(v.iter().any(|m| m.contains("export")));
}

#[tokio::test]
async fn enforce_pending_carries_custom_default_timeout() {
    let cm = Arc::new(ConsentManager::new());
    let custom = Duration::from_secs(777);
    let enforcer = ConsentEnforcer::with_config(
        cm,
        EnforcementConfig {
            default_timeout: custom,
            ..Default::default()
        },
    );
    let task = test_task(UserId::from("alice"), "expensive_task");
    let result = enforcer.enforce(&task, Some(100.0)).await.expect("enforce");
    match result {
        EnforcementResult::Pending {
            timeout,
            ..
        } => assert_eq!(timeout, custom),
        other => panic!("expected Pending, got {other:?}"),
    }
}

#[test]
fn dignity_expensive_non_transparent_and_sensitive_yields_multiple_violations() {
    let v = DignityChecker::check_operation("delete_all", Some(200.0), false);
    assert!(v.len() >= 2);
    assert!(v.iter().any(|m| m.contains("transparent")));
    assert!(v.iter().any(|m| m.contains("delete")));
}

#[test]
fn requires_consent_zero_cost_does_not_exceed_threshold() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("u"), "misc");
    assert!(!enforcer.requires_consent(&task, Some(0.0)));
}

#[test]
fn dignity_operation_without_lowercase_sensitive_keyword_is_clean() {
    let v = DignityChecker::check_operation("DeletionTask", Some(5.0), true);
    assert!(v.is_empty());
}

#[tokio::test]
async fn wait_for_decision_blocked_includes_consent_id_in_reason_path() {
    let cm = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(cm.clone());
    let task = test_task(UserId::from("u"), "op");
    let cid = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    assert!(cm.deny(cid.as_ref(), None).await);
    let r = enforcer.wait_for_decision(cid.as_ref()).await.expect("wait");
    assert!(matches!(
        r,
        EnforcementResult::Blocked {
            reason,
            consent_id: Some(id),
            ..
        } if reason.as_ref() == "User denied operation" && id.as_ref() == cid.as_ref()
    ));
}

// --- Additional coverage (reason strings, Expired paths, types) ---

#[test]
fn timeout_behavior_variants_are_distinct() {
    assert_ne!(TimeoutBehavior::Deny, TimeoutBehavior::Proceed);
}

#[test]
fn enforcement_result_allowed_clones_with_same_reason() {
    let a = EnforcementResult::Allowed {
        reason: "x".into(),
    };
    let b = a.clone();
    assert!(matches!(
        (a, b),
        (
            EnforcementResult::Allowed { reason: r1 },
            EnforcementResult::Allowed { reason: r2 }
        ) if r1.as_ref() == "x" && r2.as_ref() == "x"
    ));
}

#[tokio::test]
async fn enforce_allowed_includes_standard_reason_when_not_required() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("a"), "simple");
    let r = enforcer.enforce(&task, Some(10.0)).await.unwrap();
    match r {
        EnforcementResult::Allowed {
            reason,
        } => assert_eq!(reason.as_ref(), "Operation does not require consent"),
        other => panic!("expected Allowed, got {other:?}"),
    }
}

#[tokio::test]
async fn enforce_allowed_includes_auto_approve_reason_after_request() {
    let cm = Arc::new(ConsentManager::new());
    cm.set_user_preferences(
        UserId::from("u1"),
        UserPreferences {
            auto_approve_under_cost: Some(500.0),
            ..UserPreferences::default()
        },
    )
    .await;
    let enforcer = ConsentEnforcer::new(cm);
    let task = test_task(UserId::from("u1"), "costly");
    let r = enforcer.enforce(&task, Some(100.0)).await.unwrap();
    match r {
        EnforcementResult::Allowed {
            reason,
        } => assert_eq!(reason.as_ref(), "Auto-approved based on user preferences"),
        other => panic!("expected Allowed, got {other:?}"),
    }
}

#[tokio::test]
async fn wait_for_decision_approved_includes_user_approved_reason() {
    let cm = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(cm.clone());
    let task = test_task(UserId::from("alice"), "op");
    let consent_id = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    assert!(cm.approve(consent_id.as_ref(), None).await);
    let r = enforcer.wait_for_decision(consent_id.as_ref()).await.unwrap();
    match r {
        EnforcementResult::Allowed {
            reason,
        } => assert_eq!(reason.as_ref(), "User approved operation"),
        other => panic!("expected Allowed, got {other:?}"),
    }
}

#[tokio::test]
async fn check_consent_expired_maps_to_blocked_with_consent_id() {
    let cm = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(cm.clone());
    let task = test_task(UserId::from("e"), "op");
    let cid = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    assert!(cm.test_set_consent_status(cid.as_ref(), ConsentStatus::Expired).await);
    let r = enforcer.check_consent(cid.as_ref()).await.expect("status");
    assert!(matches!(
        r,
        EnforcementResult::Blocked {
            reason,
            consent_id: Some(id),
        } if reason.as_ref() == "Consent expired" && id.as_ref() == cid.as_ref()
    ));
}

#[tokio::test]
async fn wait_for_decision_expired_applies_fail_safe_deny() {
    let cm = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::new(cm.clone());
    let task = test_task(UserId::from("ex"), "op");
    let cid = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    assert!(cm.test_set_consent_status(cid.as_ref(), ConsentStatus::Expired).await);
    let r = enforcer.wait_for_decision(cid.as_ref()).await.unwrap();
    assert!(matches!(
        r,
        EnforcementResult::Blocked {
            reason,
            consent_id: Some(id),
        } if reason.as_ref().contains("expired") && id.as_ref() == cid.as_ref()
    ));
}

#[tokio::test]
async fn wait_for_decision_expired_applies_proceed_when_configured() {
    let cm = Arc::new(ConsentManager::new());
    let config = EnforcementConfig {
        timeout_behavior: TimeoutBehavior::Proceed,
        ..Default::default()
    };
    let enforcer = ConsentEnforcer::with_config(cm.clone(), config);
    let task = test_task(UserId::from("ex2"), "op");
    let cid = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    assert!(cm.test_set_consent_status(cid.as_ref(), ConsentStatus::Expired).await);
    let r = enforcer.wait_for_decision(cid.as_ref()).await.unwrap();
    match r {
        EnforcementResult::Allowed {
            reason,
        } => assert_eq!(reason.as_ref(), "Proceeding despite timeout (configured behavior)"),
        other => panic!("expected Allowed with proceed-on-timeout, got {other:?}"),
    }
}

#[tokio::test]
async fn check_consent_pending_includes_configured_timeout() {
    let custom = Duration::from_secs(42);
    let cm = Arc::new(ConsentManager::new());
    let enforcer = ConsentEnforcer::with_config(
        cm.clone(),
        EnforcementConfig {
            default_timeout: custom,
            ..Default::default()
        },
    );
    let task = test_task(UserId::from("p"), "op");
    let cid = cm.request_consent(task.owner.clone(), task.id, "op", Some(100.0)).await;
    match enforcer.check_consent(cid.as_ref()).await {
        Some(EnforcementResult::Pending {
            timeout,
            consent_id,
        }) => {
            assert_eq!(timeout, custom);
            assert_eq!(consent_id.as_ref(), cid.as_ref());
        }
        other => panic!("expected Pending, got {other:?}"),
    }
}

#[test]
fn requires_consent_negative_estimated_cost_does_not_exceed_threshold() {
    let enforcer = ConsentEnforcer::new(Arc::new(ConsentManager::new()));
    let task = test_task(UserId::from("u"), "misc");
    assert!(!enforcer.requires_consent(&task, Some(-10.0)));
}

#[test]
fn dignity_high_cost_with_none_omits_expensive_rule() {
    let v = DignityChecker::check_operation("harmless", None, false);
    assert!(v.is_empty());
}

#[test]
fn enforcement_config_default_lists_always_require_ops() {
    let c = EnforcementConfig::default();
    assert_eq!(c.always_require_consent.len(), 3);
    assert!(c.always_require_consent.iter().any(|s| s.as_ref() == "delete_data"));
    assert!(c.always_require_consent.iter().any(|s| s.as_ref() == "export_data"));
    assert!(c.always_require_consent.iter().any(|s| s.as_ref() == "gpu_training"));
}
