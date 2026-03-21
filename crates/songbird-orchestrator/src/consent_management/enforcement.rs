// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Consent Enforcement
//!
//! Enforces consent requirements before task execution.
//! Integrates with task lifecycle to ensure operations
//! respect human consent and dignity principles.

use super::{ConsentManager, ConsentStatus};
use crate::task_lifecycle::TaskLifecycle;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Enforcement result
#[derive(Debug, Clone)]
pub enum EnforcementResult {
    /// Operation allowed (consent granted or not required)
    Allowed {
        reason: Arc<str>,
    },

    /// Operation blocked (consent denied or required but not granted)
    Blocked {
        reason: Arc<str>,
        consent_id: Option<Arc<str>>,
    },

    /// Waiting for consent decision
    Pending {
        consent_id: Arc<str>,
        timeout: Duration,
    },
}

/// Consent enforcement configuration
#[derive(Debug, Clone)]
pub struct EnforcementConfig {
    /// Cost threshold requiring consent (in dollars)
    pub consent_required_above_cost: f64,

    /// Operations that always require consent
    pub always_require_consent: Vec<Arc<str>>,

    /// Default timeout for consent decisions
    pub default_timeout: Duration,

    /// Fail-safe behavior: deny if no decision within timeout
    pub timeout_behavior: TimeoutBehavior,
}

impl Default for EnforcementConfig {
    fn default() -> Self {
        Self {
            consent_required_above_cost: 50.0, // Require consent above $50
            always_require_consent: vec![
                "delete_data".into(),
                "export_data".into(),
                "gpu_training".into(),
            ],
            default_timeout: Duration::from_secs(300), // 5 minutes
            timeout_behavior: TimeoutBehavior::Deny,   // Fail-safe: deny
        }
    }
}

/// Timeout behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutBehavior {
    /// Deny operation if no decision (fail-safe)
    Deny,

    /// Proceed anyway (not recommended for expensive operations)
    Proceed,
}

/// Consent enforcer
pub struct ConsentEnforcer {
    consent_manager: Arc<ConsentManager>,
    config: EnforcementConfig,
}

impl ConsentEnforcer {
    #[must_use]
    pub fn new(consent_manager: Arc<ConsentManager>) -> Self {
        Self {
            consent_manager,
            config: EnforcementConfig::default(),
        }
    }

    #[must_use]
    pub const fn with_config(
        consent_manager: Arc<ConsentManager>,
        config: EnforcementConfig,
    ) -> Self {
        Self {
            consent_manager,
            config,
        }
    }

    /// Check if consent is required for a task
    pub fn requires_consent(&self, task: &TaskLifecycle, estimated_cost: Option<f64>) -> bool {
        // Check if operation type always requires consent
        let operation_type = task.spec.task_type.as_ref();
        if self.config.always_require_consent.iter().any(|op| op.as_ref() == operation_type) {
            debug!("Operation {} always requires consent", operation_type);
            return true;
        }

        // Check if cost exceeds threshold
        if let Some(cost) = estimated_cost
            && cost > self.config.consent_required_above_cost
        {
            debug!(
                "Cost ${} exceeds threshold ${}, consent required",
                cost, self.config.consent_required_above_cost
            );
            return true;
        }

        false
    }

    /// Enforce consent before task execution
    pub async fn enforce(
        &self,
        task: &TaskLifecycle,
        estimated_cost: Option<f64>,
    ) -> Result<EnforcementResult> {
        // Check if consent required
        if !self.requires_consent(task, estimated_cost) {
            info!("Task {} does not require consent, allowing execution", task.id);
            return Ok(EnforcementResult::Allowed {
                reason: "Operation does not require consent".into(),
            });
        }

        info!("Task {} requires consent from user {}", task.id, task.owner);

        // Request consent
        let consent_id = self
            .consent_manager
            .request_consent(
                task.owner.clone(),
                task.id,
                task.spec.task_type.clone(),
                estimated_cost,
            )
            .await;

        // Check if auto-approved
        if let Some(status) = self.consent_manager.get_status(&consent_id).await
            && status == ConsentStatus::Approved
        {
            info!("Task {} auto-approved via user preferences", task.id);
            return Ok(EnforcementResult::Allowed {
                reason: "Auto-approved based on user preferences".into(),
            });
        }

        // Return pending status
        Ok(EnforcementResult::Pending {
            consent_id,
            timeout: self.config.default_timeout,
        })
    }

    /// Wait for consent decision with enforcement
    pub async fn wait_for_decision(&self, consent_id: &str) -> Result<EnforcementResult> {
        info!("Waiting for consent decision: {}", consent_id);

        let decision =
            self.consent_manager.wait_for_decision(consent_id, self.config.default_timeout).await;

        match decision {
            Some(ConsentStatus::Approved) => {
                info!("Consent {} approved, allowing operation", consent_id);
                Ok(EnforcementResult::Allowed {
                    reason: "User approved operation".into(),
                })
            }
            Some(ConsentStatus::Denied) => {
                warn!("Consent {} denied, blocking operation", consent_id);
                Ok(EnforcementResult::Blocked {
                    reason: "User denied operation".into(),
                    consent_id: Some(consent_id.into()),
                })
            }
            Some(ConsentStatus::Expired) | None => {
                warn!(
                    "Consent {} expired or timed out, applying timeout behavior: {:?}",
                    consent_id, self.config.timeout_behavior
                );

                match self.config.timeout_behavior {
                    TimeoutBehavior::Deny => Ok(EnforcementResult::Blocked {
                        reason: "Consent request expired or timed out (fail-safe)".into(),
                        consent_id: Some(consent_id.into()),
                    }),
                    TimeoutBehavior::Proceed => Ok(EnforcementResult::Allowed {
                        reason: "Proceeding despite timeout (configured behavior)".into(),
                    }),
                }
            }
            _ => {
                warn!("Unexpected consent status for {}", consent_id);
                Ok(EnforcementResult::Blocked {
                    reason: "Unexpected consent status".into(),
                    consent_id: Some(consent_id.into()),
                })
            }
        }
    }

    /// Check consent status (non-blocking)
    pub async fn check_consent(&self, consent_id: &str) -> Option<EnforcementResult> {
        let status = self.consent_manager.get_status(consent_id).await?;

        Some(match status {
            ConsentStatus::Approved => EnforcementResult::Allowed {
                reason: "Consent approved".into(),
            },
            ConsentStatus::Denied => EnforcementResult::Blocked {
                reason: "Consent denied".into(),
                consent_id: Some(consent_id.into()),
            },
            ConsentStatus::Pending => EnforcementResult::Pending {
                consent_id: consent_id.into(),
                timeout: self.config.default_timeout,
            },
            ConsentStatus::Expired => EnforcementResult::Blocked {
                reason: "Consent expired".into(),
                consent_id: Some(consent_id.into()),
            },
        })
    }
}

/// Human dignity principles checker
pub struct DignityChecker;

impl DignityChecker {
    /// Check if operation respects human dignity principles
    #[must_use]
    pub fn check_operation(operation: &str, cost: Option<f64>, transparent: bool) -> Vec<Arc<str>> {
        let mut violations = Vec::new();

        // Check for expensive operations without transparency
        if let Some(c) = cost
            && c > 100.0
            && !transparent
        {
            violations
                .push("Expensive operation ($100+) requires transparent cost disclosure".into());
        }

        // Check for operations that should always be explicit
        let sensitive_operations = ["delete", "export", "share", "gpu", "training", "large_scale"];

        for sensitive in &sensitive_operations {
            if operation.contains(sensitive) {
                violations.push(
                    format!(
                        "Sensitive operation containing '{sensitive}' requires explicit consent"
                    )
                    .into(),
                );
                break;
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::consent_management::UserPreferences;
    use crate::task_lifecycle::UserId;

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
}
