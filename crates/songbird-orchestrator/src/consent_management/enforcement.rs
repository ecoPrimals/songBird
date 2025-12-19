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
    pub fn new(consent_manager: Arc<ConsentManager>) -> Self {
        Self {
            consent_manager,
            config: EnforcementConfig::default(),
        }
    }

    pub fn with_config(consent_manager: Arc<ConsentManager>, config: EnforcementConfig) -> Self {
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
        if let Some(cost) = estimated_cost {
            if cost > self.config.consent_required_above_cost {
                debug!(
                    "Cost ${} exceeds threshold ${}, consent required",
                    cost, self.config.consent_required_above_cost
                );
                return true;
            }
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
        if let Some(status) = self.consent_manager.get_status(&consent_id).await {
            if status == ConsentStatus::Approved {
                info!("Task {} auto-approved via user preferences", task.id);
                return Ok(EnforcementResult::Allowed {
                    reason: "Auto-approved based on user preferences".into(),
                });
            }
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
    pub fn check_operation(operation: &str, cost: Option<f64>, transparent: bool) -> Vec<Arc<str>> {
        let mut violations = Vec::new();

        // Check for expensive operations without transparency
        if let Some(c) = cost {
            if c > 100.0 && !transparent {
                violations.push(
                    "Expensive operation ($100+) requires transparent cost disclosure".into(),
                );
            }
        }

        // Check for operations that should always be explicit
        let sensitive_operations = ["delete", "export", "share", "gpu", "training", "large_scale"];

        for sensitive in &sensitive_operations {
            if operation.contains(sensitive) {
                violations.push(
                    format!(
                        "Sensitive operation containing '{}' requires explicit consent",
                        sensitive
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
    use super::*;
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
}
