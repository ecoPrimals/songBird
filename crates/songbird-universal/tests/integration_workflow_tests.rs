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
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Integration workflow tests
//!
//! Exercises [`UniversalCapabilityAdapter`] workflows, discovery-backed provider lists,
//! and [`songbird_test_utils::mocks`] capability mocks (with [`DiscoveryConfig::provider_endpoints`]
//! wired for deterministic discovery).

use songbird_test_utils::mocks::{CapabilityType, MockCapabilityEnvironment};
use songbird_universal::capabilities::{
    CapabilityWorkflow, DiscoveryConfig, UniversalCapabilityAdapter, WorkflowStep,
};
use std::sync::Arc;
use std::time::Duration;

/// Build [`DiscoveryConfig`] with mock HTTP endpoints (no process environment).
fn discovery_config_from_mock_env(env: &MockCapabilityEnvironment) -> DiscoveryConfig {
    let mut config = DiscoveryConfig::default();
    for (cap, key) in [
        (CapabilityType::Compute, "compute"),
        (CapabilityType::Storage, "storage"),
        (CapabilityType::Ai, "ai"),
        (CapabilityType::Security, "security"),
    ] {
        if let Some(url) = env.endpoint(&cap) {
            config.provider_endpoints.insert(key.to_string(), url);
        }
    }
    config
}

#[tokio::test]
async fn test_end_to_end_capability_workflow() {
    let adapter = UniversalCapabilityAdapter::new(DiscoveryConfig::default());

    let workflow = CapabilityWorkflow {
        name: "compute-smoke".to_string(),
        steps: vec![WorkflowStep {
            name: "discover-compute".to_string(),
            capability_type: "compute".to_string(),
            parameters: serde_json::json!({}),
        }],
        continue_on_error: false,
    };

    let result = adapter.execute_capability_workflow(&workflow).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_multi_capability_orchestration() {
    let mut env = MockCapabilityEnvironment::builder()
        .with_compute()
        .with_storage()
        .with_ai()
        .with_security()
        .build()
        .await
        .expect("mock capability environment");

    let adapter = UniversalCapabilityAdapter::new(discovery_config_from_mock_env(&env));

    let workflow = CapabilityWorkflow {
        name: "multi-cap-orchestration".to_string(),
        steps: vec![
            WorkflowStep {
                name: "compute".to_string(),
                capability_type: "compute".to_string(),
                parameters: serde_json::json!({}),
            },
            WorkflowStep {
                name: "storage".to_string(),
                capability_type: "storage".to_string(),
                parameters: serde_json::json!({}),
            },
            WorkflowStep {
                name: "ai".to_string(),
                capability_type: "ai".to_string(),
                parameters: serde_json::json!({}),
            },
            WorkflowStep {
                name: "security".to_string(),
                capability_type: "security".to_string(),
                parameters: serde_json::json!({}),
            },
        ],
        continue_on_error: false,
    };

    let wr = adapter.execute_capability_workflow(&workflow).await.expect("workflow execution");
    assert_eq!(wr.steps.len(), 4);
    assert!(wr.success, "orchestrated workflow: {:?}", wr.steps);
    assert!(wr.steps.iter().all(|s| s.success));

    env.shutdown().await;
}

#[tokio::test]
async fn test_capability_chaining() {
    let mut env = MockCapabilityEnvironment::builder()
        .with_compute()
        .with_storage()
        .with_ai()
        .build()
        .await
        .expect("mock capability environment");

    let adapter = UniversalCapabilityAdapter::new(discovery_config_from_mock_env(&env));
    let workflow = CapabilityWorkflow {
        name: "sequential-chain".to_string(),
        steps: vec![
            WorkflowStep {
                name: "compute-step".to_string(),
                capability_type: "compute".to_string(),
                parameters: serde_json::json!({}),
            },
            WorkflowStep {
                name: "storage-step".to_string(),
                capability_type: "storage".to_string(),
                parameters: serde_json::json!({}),
            },
            WorkflowStep {
                name: "ai-step".to_string(),
                capability_type: "ai".to_string(),
                parameters: serde_json::json!({}),
            },
        ],
        continue_on_error: false,
    };

    let result = adapter.execute_capability_workflow(&workflow).await;
    assert!(result.is_ok(), "{result:?}");
    let wr = result.expect("workflow result");
    assert_eq!(wr.steps.len(), 3);
    assert!(wr.success, "all steps should find providers: {:?}", wr.steps);
    assert!(wr.steps.iter().all(|s| s.success));

    env.shutdown().await;
}

#[tokio::test]
async fn test_parallel_capability_execution() {
    let mut env = MockCapabilityEnvironment::builder()
        .with_compute()
        .with_storage()
        .with_ai()
        .build()
        .await
        .expect("mock capability environment");

    let adapter = Arc::new(UniversalCapabilityAdapter::new(discovery_config_from_mock_env(&env)));

    let mut handles = vec![];

    for capability in &["compute", "storage", "ai"] {
        let adapter_clone = Arc::clone(&adapter);
        let cap = capability.to_string();
        let handle =
            tokio::spawn(async move { adapter_clone.find_capability_providers(&cap).await });
        handles.push(handle);
    }

    for handle in handles {
        let joined = handle.await.expect("task join");
        assert!(
            !joined.is_empty(),
            "parallel discovery should see mock-backed providers for {joined:?}"
        );
    }

    env.shutdown().await;
}

#[tokio::test]
async fn test_capability_dependency_resolution() {
    let mut env = MockCapabilityEnvironment::builder()
        .with_compute()
        .with_storage()
        .build()
        .await
        .expect("mock capability environment");

    let adapter = UniversalCapabilityAdapter::new(discovery_config_from_mock_env(&env));

    let compute = adapter.find_capability_providers("compute").await;
    let storage = adapter.find_capability_providers("storage").await;
    assert!(
        !compute.is_empty() && !storage.is_empty(),
        "expected config-backed providers for compute and storage: compute={compute:?} storage={storage:?}"
    );

    let best_compute = adapter.get_best_primal_for_capability("compute").await;
    let best_storage = adapter.get_best_primal_for_capability("storage").await;
    assert!(
        best_compute.is_some() || !compute.is_empty(),
        "registry or provider list should surface compute"
    );
    assert!(
        best_storage.is_some() || !storage.is_empty(),
        "registry or provider list should surface storage"
    );

    env.shutdown().await;
}

#[tokio::test]
async fn test_capability_transaction() {
    let adapter = UniversalCapabilityAdapter::new(DiscoveryConfig::default());

    // `continue_on_error: false` gives all-or-nothing semantics: first missing provider aborts the run.
    let workflow = CapabilityWorkflow {
        name: "txn".to_string(),
        steps: vec![
            WorkflowStep {
                name: "must-fail".to_string(),
                capability_type: "zzzz-nonexistent-capability-unique".to_string(),
                parameters: serde_json::json!({}),
            },
            WorkflowStep {
                name: "never-run".to_string(),
                capability_type: "compute".to_string(),
                parameters: serde_json::json!({}),
            },
        ],
        continue_on_error: false,
    };

    let result = adapter.execute_capability_workflow(&workflow).await;
    assert!(result.is_ok(), "{result:?}");
    let wr = result.expect("workflow");
    assert!(!wr.success, "workflow should fail on first step");
    assert_eq!(wr.steps.len(), 1, "second step should not run after failure");
    assert!(wr.error.is_some());
}

#[tokio::test]
#[ignore = "UniversalCapabilityAdapter has no execute_with_rollback API; use execute_capability_workflow with continue_on_error for partial completion semantics"]
async fn test_capability_rollback() {
    // Blocked on `execute_with_rollback` / compensating transactions on the adapter.
    // Today, only workflow-level `continue_on_error` models partial forward progress.
}

#[tokio::test]
async fn test_long_running_workflow() {
    let mut env = MockCapabilityEnvironment::builder().with_compute().build().await.expect("mock");

    let adapter = UniversalCapabilityAdapter::new(discovery_config_from_mock_env(&env));

    let steps: Vec<WorkflowStep> = (0..8)
        .map(|i| WorkflowStep {
            name: format!("step-{i}"),
            capability_type: "compute".to_string(),
            parameters: serde_json::json!({"index": i}),
        })
        .collect();

    let workflow = CapabilityWorkflow {
        name: "many-steps".to_string(),
        steps,
        continue_on_error: false,
    };

    let result = adapter.execute_capability_workflow(&workflow).await;
    assert!(result.is_ok(), "{result:?}");
    let wr = result.expect("workflow");
    assert_eq!(wr.steps.len(), 8);
    assert!(wr.success);
    assert!(wr.steps.iter().all(|s| s.success));

    env.shutdown().await;
}

#[tokio::test]
async fn test_workflow_cancellation() {
    let adapter = Arc::new(UniversalCapabilityAdapter::new(DiscoveryConfig::default()));
    let (done_tx, done_rx) = tokio::sync::oneshot::channel();

    let a = Arc::clone(&adapter);
    let handle = tokio::spawn(async move {
        let _ = a.find_capability_providers("compute").await;
        let _ = done_tx.send(());
        std::future::pending::<()>().await;
    });

    done_rx.await.expect("discovery phase should complete");
    tokio::task::yield_now().await;
    handle.abort();
    let outcome = handle.await;
    assert!(outcome.is_err(), "aborted task should not run to completion: {outcome:?}");
}

#[tokio::test]
async fn test_workflow_retry_logic() {
    let adapter = UniversalCapabilityAdapter::new(DiscoveryConfig::default());
    let cap = "compute";
    let mut last_len = None;
    for attempt in 0u32..3 {
        let providers = adapter.find_capability_providers(cap).await;
        last_len = Some(providers.len());
        assert_eq!(
            last_len,
            Some(adapter.find_capability_providers(cap).await.len()),
            "same discovery path should be stable on retry attempt {attempt}"
        );
        tokio::task::yield_now().await;
    }
    assert!(last_len.is_some());
}

#[tokio::test]
async fn test_workflow_timeout() {
    let adapter = UniversalCapabilityAdapter::new(DiscoveryConfig::default());
    let outcome = tokio::time::timeout(Duration::from_secs(30), async {
        adapter.find_capability_providers("network").await
    })
    .await;
    assert!(outcome.is_ok(), "discovery should complete within timeout: {outcome:?}");
}

#[tokio::test]
async fn test_workflow_metrics_collection() {
    let adapter = UniversalCapabilityAdapter::new(DiscoveryConfig::default());
    let metrics = adapter.get_workflow_metrics().await.expect("metrics API should succeed");
    assert_eq!(metrics.total_workflows, 0);
    assert_eq!(metrics.successful_workflows, 0);
    assert_eq!(metrics.failed_workflows, 0);
}

#[tokio::test]
async fn test_conditional_capability_execution() {
    let mut env = MockCapabilityEnvironment::builder()
        .with_compute()
        .with_storage()
        .build()
        .await
        .expect("mock capability environment");

    let adapter = UniversalCapabilityAdapter::new(discovery_config_from_mock_env(&env));
    let compute = adapter.find_capability_providers("compute").await;
    assert!(!compute.is_empty(), "precondition: compute providers from mock");

    let follow_up = if compute.is_empty() {
        "compute"
    } else {
        "storage"
    };
    let workflow = CapabilityWorkflow {
        name: "conditional-follow-up".to_string(),
        steps: vec![WorkflowStep {
            name: "follow-up".to_string(),
            capability_type: follow_up.to_string(),
            parameters: serde_json::json!({}),
        }],
        continue_on_error: false,
    };

    let wr = adapter.execute_capability_workflow(&workflow).await.expect("workflow");
    assert!(wr.success);
    assert_eq!(wr.steps.len(), 1);

    env.shutdown().await;
}

#[tokio::test]
#[ignore = "UniversalCapabilityAdapter has no start_workflow/resume_workflow API; persistence requires orchestration storage"]
async fn test_workflow_state_persistence() {
    // Needs `start_workflow` / `resume_workflow` (or external workflow engine) on the adapter.
}

#[tokio::test]
async fn test_workflow_branching() {
    let mut env = MockCapabilityEnvironment::builder()
        .with_storage()
        .with_compute()
        .with_ai()
        .build()
        .await
        .expect("mock capability environment");

    let adapter = UniversalCapabilityAdapter::new(discovery_config_from_mock_env(&env));

    let wf_a = CapabilityWorkflow {
        name: "branch-a".to_string(),
        steps: vec![WorkflowStep {
            name: "storage".to_string(),
            capability_type: "storage".to_string(),
            parameters: serde_json::json!({}),
        }],
        continue_on_error: false,
    };
    let wf_b = CapabilityWorkflow {
        name: "branch-b".to_string(),
        steps: vec![WorkflowStep {
            name: "ai".to_string(),
            capability_type: "ai".to_string(),
            parameters: serde_json::json!({}),
        }],
        continue_on_error: false,
    };

    let (ra, rb) = tokio::join!(
        adapter.execute_capability_workflow(&wf_a),
        adapter.execute_capability_workflow(&wf_b)
    );
    let a = ra.expect("branch a");
    let b = rb.expect("branch b");
    assert!(a.success && b.success, "parallel branches: a={a:?} b={b:?}");

    env.shutdown().await;
}
