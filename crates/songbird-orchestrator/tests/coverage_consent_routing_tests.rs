// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for consent management, routing types, and core orchestrator
//!
//! Covers:
//! - `consent_management/preferences.rs`
//! - `consent_management/request.rs`
//! - `consent_management/rules.rs`
//! - core/orchestrator.rs
//! - core/routing/types.rs (`TaskBuilder`, `ResourceRequirements`, Task serde)

use songbird_orchestrator::consent_management::{
    AutoApprovalRule, ConsentRequestBuilder, UserPreferences,
};
use songbird_orchestrator::core::orchestrator::{CoreOrchestrator, OrchestratorConfig};
use songbird_orchestrator::core::routing::types::{ResourceRequirements, Task, TaskBuilder};

// ═══════════════════════════════════════════════════════════════════════════
// UserPreferences tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_user_preferences_default() {
    let prefs = UserPreferences::default();
    assert_eq!(prefs.auto_approve_under_cost, Some(10.0));
    assert!(prefs.always_require_consent.is_empty());
    assert!(prefs.blocked_operations.is_empty());
}

#[test]
fn test_user_preferences_custom() {
    let prefs = UserPreferences {
        auto_approve_under_cost: Some(50.0),
        always_require_consent: vec!["delete".to_string(), "deploy".to_string()],
        blocked_operations: vec!["format_disk".to_string()],
    };
    assert_eq!(prefs.auto_approve_under_cost, Some(50.0));
    assert_eq!(prefs.always_require_consent.len(), 2);
    assert_eq!(prefs.blocked_operations.len(), 1);
}

#[test]
fn test_user_preferences_no_auto_approve() {
    let prefs = UserPreferences {
        auto_approve_under_cost: None,
        always_require_consent: vec![],
        blocked_operations: vec![],
    };
    assert!(prefs.auto_approve_under_cost.is_none());
}

#[test]
fn test_user_preferences_clone() {
    let prefs = UserPreferences::default();
    let cloned = prefs.clone();
    assert_eq!(cloned.auto_approve_under_cost, prefs.auto_approve_under_cost);
}

#[test]
fn test_user_preferences_debug() {
    let prefs = UserPreferences::default();
    let debug_str = format!("{prefs:?}");
    assert!(debug_str.contains("UserPreferences"));
    assert!(debug_str.contains("10.0"));
}

#[test]
fn test_user_preferences_serde_roundtrip() {
    let prefs = UserPreferences {
        auto_approve_under_cost: Some(25.0),
        always_require_consent: vec!["admin_action".to_string()],
        blocked_operations: vec!["destroy".to_string()],
    };
    let json = serde_json::to_string(&prefs).expect("serialize");
    let deserialized: UserPreferences = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.auto_approve_under_cost, Some(25.0));
    assert_eq!(deserialized.always_require_consent, vec!["admin_action"]);
    assert_eq!(deserialized.blocked_operations, vec!["destroy"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// ConsentRequestBuilder tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_consent_request_builder_new() {
    let _builder = ConsentRequestBuilder::new("deploy");
    // Builder is created successfully (no public fields to assert on)
}

#[test]
fn test_consent_request_builder_with_cost() {
    let _builder = ConsentRequestBuilder::new("deploy").with_cost(99.99);
}

#[test]
fn test_consent_request_builder_with_justification() {
    let _builder =
        ConsentRequestBuilder::new("deploy").with_justification("Scheduled maintenance update");
}

#[test]
fn test_consent_request_builder_chained() {
    let _builder = ConsentRequestBuilder::new("scale_up")
        .with_cost(150.0)
        .with_justification("Traffic spike detected, need more capacity");
}

#[test]
fn test_consent_request_builder_string_into() {
    let _builder = ConsentRequestBuilder::new(String::from("operation"));
}

// ═══════════════════════════════════════════════════════════════════════════
// AutoApprovalRule tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_auto_approval_rule_matches_any_operation_any_cost() {
    let rule = AutoApprovalRule {
        name: "blanket".to_string(),
        max_cost: None,
        operations: vec![],
    };
    assert!(rule.matches("deploy", Some(1000.0)));
    assert!(rule.matches("delete", None));
    assert!(rule.matches("anything", Some(0.0)));
}

#[test]
fn test_auto_approval_rule_matches_specific_operations() {
    let rule = AutoApprovalRule {
        name: "deploy-only".to_string(),
        max_cost: None,
        operations: vec!["deploy".to_string(), "restart".to_string()],
    };
    assert!(rule.matches("deploy", None));
    assert!(rule.matches("restart", None));
    assert!(!rule.matches("delete", None));
    assert!(!rule.matches("scale", None));
}

#[test]
fn test_auto_approval_rule_matches_cost_limit() {
    let rule = AutoApprovalRule {
        name: "budget".to_string(),
        max_cost: Some(100.0),
        operations: vec![],
    };
    assert!(rule.matches("deploy", Some(50.0)));
    assert!(rule.matches("deploy", Some(100.0)));
    assert!(!rule.matches("deploy", Some(101.0)));
    // No cost provided = passes (no cost check needed)
    assert!(rule.matches("deploy", None));
}

#[test]
fn test_auto_approval_rule_matches_both_constraints() {
    let rule = AutoApprovalRule {
        name: "limited".to_string(),
        max_cost: Some(50.0),
        operations: vec!["deploy".to_string()],
    };
    // Correct operation, under cost
    assert!(rule.matches("deploy", Some(25.0)));
    // Correct operation, over cost
    assert!(!rule.matches("deploy", Some(75.0)));
    // Wrong operation
    assert!(!rule.matches("delete", Some(10.0)));
    // Correct operation, no cost
    assert!(rule.matches("deploy", None));
}

#[test]
fn test_auto_approval_rule_debug() {
    let rule = AutoApprovalRule {
        name: "test-rule".to_string(),
        max_cost: Some(42.0),
        operations: vec!["op1".to_string()],
    };
    let debug = format!("{rule:?}");
    assert!(debug.contains("test-rule"));
    assert!(debug.contains("42.0"));
}

#[test]
fn test_auto_approval_rule_clone() {
    let rule = AutoApprovalRule {
        name: "original".to_string(),
        max_cost: Some(100.0),
        operations: vec!["deploy".to_string()],
    };
    let cloned = rule;
    assert_eq!(cloned.name, "original");
    assert_eq!(cloned.max_cost, Some(100.0));
    assert_eq!(cloned.operations, vec!["deploy"]);
}

#[test]
fn test_auto_approval_rule_zero_cost() {
    let rule = AutoApprovalRule {
        name: "free-only".to_string(),
        max_cost: Some(0.0),
        operations: vec![],
    };
    assert!(rule.matches("deploy", Some(0.0)));
    assert!(!rule.matches("deploy", Some(0.01)));
}

// ═══════════════════════════════════════════════════════════════════════════
// OrchestratorConfig tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_orchestrator_config_default() {
    let config = OrchestratorConfig::default();
    assert_eq!(config.name, "Songbird Orchestrato");
    assert_eq!(config.max_services, 1000);
}

#[test]
fn test_orchestrator_config_custom() {
    let config = OrchestratorConfig {
        name: "Custom Orchestrator".to_string(),
        max_services: 500,
    };
    assert_eq!(config.name, "Custom Orchestrator");
    assert_eq!(config.max_services, 500);
}

#[test]
fn test_orchestrator_config_debug() {
    let config = OrchestratorConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("OrchestratorConfig"));
}

#[test]
fn test_orchestrator_config_clone() {
    let config = OrchestratorConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.name, config.name);
    assert_eq!(cloned.max_services, config.max_services);
}

#[test]
fn test_orchestrator_config_serde_roundtrip() {
    let config = OrchestratorConfig {
        name: "Test".to_string(),
        max_services: 42,
    };
    let json = serde_json::to_string(&config).expect("serialize");
    let deserialized: OrchestratorConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.name, "Test");
    assert_eq!(deserialized.max_services, 42);
}

// ═══════════════════════════════════════════════════════════════════════════
// CoreOrchestrator tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_core_orchestrator_new() {
    let orch = CoreOrchestrator::new();
    let debug = format!("{orch:?}");
    assert!(debug.contains("CoreOrchestrator"));
}

#[test]
fn test_core_orchestrator_default() {
    let orch = CoreOrchestrator;
    let debug = format!("{orch:?}");
    assert!(debug.contains("CoreOrchestrator"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Task & TaskBuilder tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_task_new() {
    let task = Task::new("health_check");
    assert_eq!(task.task_type.as_ref(), "health_check");
    assert!(task.resource_requirements.is_none());
    assert!(task.estimated_duration_secs.is_none());
    assert!(task.metadata.is_empty());
}

#[test]
fn test_task_builder_minimal() {
    let task = TaskBuilder::new("test").build();
    assert_eq!(task.task_type.as_ref(), "test");
}

#[test]
fn test_task_builder_with_payload() {
    let payload = serde_json::json!({"key": "value", "count": 42});
    let task = Task::builder("data_processing").with_payload(payload.clone()).build();
    assert_eq!(task.payload, payload);
}

#[test]
fn test_task_builder_with_resources() {
    let reqs = ResourceRequirements {
        cpu_cores: Some(8.0),
        memory_mb: Some(16384),
        gpu_required: true,
        storage_mb: Some(1000),
        network_mbps: Some(100.0),
    };
    let task = Task::builder("ml_training").with_resources(reqs).build();
    let r = task.resource_requirements.as_ref().expect("should have requirements");
    assert_eq!(r.cpu_cores, Some(8.0));
    assert!(r.gpu_required);
}

#[test]
fn test_task_builder_with_gpu() {
    let task = Task::builder("gpu_compute").with_gpu().build();
    let r = task.resource_requirements.as_ref().expect("should have requirements");
    assert!(r.gpu_required);
}

#[test]
fn test_task_builder_with_cpu() {
    let task = Task::builder("compute").with_cpu(16.0).build();
    let r = task.resource_requirements.as_ref().expect("should have requirements");
    assert_eq!(r.cpu_cores, Some(16.0));
}

#[test]
fn test_task_builder_with_memory() {
    let task = Task::builder("memory_intensive").with_memory(32768).build();
    let r = task.resource_requirements.as_ref().expect("should have requirements");
    assert_eq!(r.memory_mb, Some(32768));
}

#[test]
fn test_task_builder_with_duration() {
    let task = Task::builder("long_running").with_duration(3600).build();
    assert_eq!(task.estimated_duration_secs, Some(3600));
}

#[test]
fn test_task_builder_with_metadata() {
    let task = Task::builder("annotated")
        .with_metadata("owner", "team-alpha")
        .with_metadata("priority", "high")
        .build();
    assert_eq!(task.metadata.get("owner"), Some(&"team-alpha".to_string()));
    assert_eq!(task.metadata.get("priority"), Some(&"high".to_string()));
}

#[test]
fn test_task_builder_chained_full() {
    let task = Task::builder("ml_training")
        .with_gpu()
        .with_cpu(4.0)
        .with_memory(8192)
        .with_duration(600)
        .with_metadata("model", "resnet50")
        .with_metadata("dataset", "imagenet")
        .with_payload(serde_json::json!({"epochs": 10}))
        .build();

    assert_eq!(task.task_type.as_ref(), "ml_training");
    let r = task.resource_requirements.as_ref().expect("has requirements");
    assert!(r.gpu_required);
    assert_eq!(r.cpu_cores, Some(4.0));
    assert_eq!(r.memory_mb, Some(8192));
    assert_eq!(task.estimated_duration_secs, Some(600));
    assert_eq!(task.metadata.len(), 2);
}

#[test]
fn test_task_serde_roundtrip() {
    let task = Task::builder("data_processing")
        .with_cpu(2.0)
        .with_duration(120)
        .with_metadata("region", "us-east-1")
        .build();

    let json = serde_json::to_string(&task).expect("serialize");
    let deserialized: Task = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.task_type.as_ref(), "data_processing");
    assert_eq!(deserialized.estimated_duration_secs, Some(120));
    assert_eq!(deserialized.metadata.get("region"), Some(&"us-east-1".to_string()));
}

#[test]
fn test_task_serde_minimal() {
    let json = r#"{"task_type":"simple"}"#;
    let task: Task = serde_json::from_str(json).expect("deserialize minimal");
    assert_eq!(task.task_type.as_ref(), "simple");
    assert!(task.resource_requirements.is_none());
    assert!(task.metadata.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// ResourceRequirements tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_resource_requirements_default() {
    let reqs = ResourceRequirements::default();
    assert_eq!(reqs.cpu_cores, Some(1.0));
    assert_eq!(reqs.memory_mb, Some(512));
    assert!(!reqs.gpu_required);
    assert_eq!(reqs.storage_mb, Some(100));
    assert_eq!(reqs.network_mbps, Some(10.0));
}

#[test]
fn test_resource_requirements_clone() {
    let reqs = ResourceRequirements::default();
    let cloned = reqs.clone();
    assert_eq!(cloned.cpu_cores, reqs.cpu_cores);
    assert_eq!(cloned.gpu_required, reqs.gpu_required);
}

#[test]
fn test_resource_requirements_serde_roundtrip() {
    let reqs = ResourceRequirements {
        cpu_cores: Some(4.0),
        memory_mb: Some(2048),
        gpu_required: true,
        storage_mb: None,
        network_mbps: Some(1000.0),
    };
    let json = serde_json::to_string(&reqs).expect("serialize");
    let deserialized: ResourceRequirements = serde_json::from_str(&json).expect("deserialize");
    assert!(deserialized.gpu_required);
    assert_eq!(deserialized.cpu_cores, Some(4.0));
    assert!(deserialized.storage_mb.is_none());
}
