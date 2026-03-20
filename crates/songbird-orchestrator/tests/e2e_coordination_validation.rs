// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! End-to-end tests for coordination pattern validation
//!
//! These tests verify the full workflow from graph creation to coordination
//! pattern validation, using real service registry integration.

use songbird_orchestrator::graph::{
    CoordinationPattern, CoordinationValidator, Graph, GraphEdge, GraphMetadata, GraphNode,
};
use songbird_orchestrator::ipc::registry::ServiceRegistry;
use std::sync::Arc;

fn create_test_node(id: &str, capability: &str) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        primal_name: None,
        capability: capability.to_string(),
        inputs: vec![],
        outputs: vec![],
        config: serde_json::json!({}),
        preferred_protocol: None,
        timeout_secs: None,
    }
}

#[tokio::test]
async fn test_e2e_sequential_pattern_validation() {
    // Step 1: Set up service registry with primals for sequential execution
    let registry = Arc::new(ServiceRegistry::new());
    let validator = CoordinationValidator::new(registry.clone());

    // Register primals for each stage of the sequential workflow
    for (i, capability) in ["data-ingestion", "data-processing", "data-storage"].iter().enumerate()
    {
        let primal_name = format!("Worker{}", i + 1);
        let service_id = registry
            .register_service(
                primal_name.clone(),
                vec![capability.to_string()],
                format!("/run/user/1000/{}.sock", primal_name.to_lowercase()),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Mark as healthy
        registry.update_health(&service_id, "healthy".to_string()).await.unwrap();
    }

    // Step 2: Create a sequential data processing graph
    let graph = Graph::new(
        "sequential-pipeline".to_string(),
        "Sequential Data Pipeline".to_string(),
        vec![
            create_test_node("ingest", "data-ingestion"),
            create_test_node("process", "data-processing"),
            create_test_node("store", "data-storage"),
        ],
        vec![
            GraphEdge {
                from: "ingest".to_string(),
                to: "process".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "process".to_string(),
                to: "store".to_string(),
                data_mapping: None,
            },
        ],
        GraphMetadata::default(),
    );

    // Step 3: Validate coordination pattern
    let result = validator.validate_pattern(&graph).await.unwrap();

    // Step 4: Verify results
    assert_eq!(result.pattern, CoordinationPattern::Sequential);
    assert!(result.valid, "Sequential pattern should be valid with all primals available");
    assert!(result.issues.is_empty(), "Should have no issues");
}

#[tokio::test]
async fn test_e2e_parallel_pattern_validation() {
    // Step 1: Set up service registry with multiple compute workers
    let registry = Arc::new(ServiceRegistry::new());
    let validator = CoordinationValidator::new(registry.clone());

    // Register input primal
    let input_id = registry
        .register_service(
            "InputProcessor".to_string(),
            vec!["input".to_string()],
            "/run/user/1000/input.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry.update_health(&input_id, "healthy".to_string()).await.unwrap();

    // Register 3 parallel compute workers
    for i in 1..=3 {
        let service_id = registry
            .register_service(
                format!("ComputeWorker{}", i),
                vec!["compute".to_string()],
                format!("/run/user/1000/compute{}.sock", i),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();
        registry.update_health(&service_id, "healthy".to_string()).await.unwrap();
    }

    // Register output aggregator
    let output_id = registry
        .register_service(
            "OutputAggregator".to_string(),
            vec!["aggregation".to_string()],
            "/run/user/1000/output.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry.update_health(&output_id, "healthy".to_string()).await.unwrap();

    // Step 2: Create a parallel computation graph (MapReduce pattern)
    let graph = Graph::new(
        "parallel-compute".to_string(),
        "Parallel Computation".to_string(),
        vec![
            create_test_node("input", "input"),
            create_test_node("compute1", "compute"),
            create_test_node("compute2", "compute"),
            create_test_node("compute3", "compute"),
            create_test_node("aggregate", "aggregation"),
        ],
        vec![
            GraphEdge {
                from: "input".to_string(),
                to: "compute1".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "input".to_string(),
                to: "compute2".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "input".to_string(),
                to: "compute3".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "compute1".to_string(),
                to: "aggregate".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "compute2".to_string(),
                to: "aggregate".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "compute3".to_string(),
                to: "aggregate".to_string(),
                data_mapping: None,
            },
        ],
        GraphMetadata::default(),
    );

    // Step 3: Validate coordination pattern
    let result = validator.validate_pattern(&graph).await.unwrap();

    // Step 4: Verify results
    assert_eq!(result.pattern, CoordinationPattern::MapReduce);
    assert!(result.valid, "MapReduce pattern should be valid with sufficient primals");
}

#[tokio::test]
async fn test_e2e_insufficient_resources_for_parallel() {
    // Step 1: Set up service registry with insufficient parallel workers
    let registry = Arc::new(ServiceRegistry::new());
    let validator = CoordinationValidator::new(registry.clone());

    // Register input primal
    let input_id = registry
        .register_service(
            "InputProcessor".to_string(),
            vec!["input".to_string()],
            "/run/user/1000/input.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry.update_health(&input_id, "healthy".to_string()).await.unwrap();

    // Register only 1 compute worker (insufficient for 3 parallel tasks)
    let compute_id = registry
        .register_service(
            "ComputeWorker1".to_string(),
            vec!["compute".to_string()],
            "/run/user/1000/compute1.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry.update_health(&compute_id, "healthy".to_string()).await.unwrap();

    // Register output aggregator
    let output_id = registry
        .register_service(
            "OutputAggregator".to_string(),
            vec!["aggregation".to_string()],
            "/run/user/1000/output.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry.update_health(&output_id, "healthy".to_string()).await.unwrap();

    // Step 2: Create a parallel graph requiring 3 workers
    let graph = Graph::new(
        "parallel-compute".to_string(),
        "Parallel Computation".to_string(),
        vec![
            create_test_node("input", "input"),
            create_test_node("compute1", "compute"),
            create_test_node("compute2", "compute"),
            create_test_node("compute3", "compute"),
            create_test_node("aggregate", "aggregation"),
        ],
        vec![
            GraphEdge {
                from: "input".to_string(),
                to: "compute1".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "input".to_string(),
                to: "compute2".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "input".to_string(),
                to: "compute3".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "compute1".to_string(),
                to: "aggregate".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "compute2".to_string(),
                to: "aggregate".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "compute3".to_string(),
                to: "aggregate".to_string(),
                data_mapping: None,
            },
        ],
        GraphMetadata::default(),
    );

    // Step 3: Validate coordination pattern
    let result = validator.validate_pattern(&graph).await.unwrap();

    // Step 4: Verify results
    assert_eq!(result.pattern, CoordinationPattern::MapReduce);
    // Should still be valid (can execute sequentially on one worker)
    // but will have warnings about limited parallelism
    assert!(result.valid, "Should be valid (can execute, just not optimally parallel)");
}

#[tokio::test]
async fn test_e2e_missing_capability_for_coordination() {
    // Step 1: Set up service registry missing a critical capability
    let registry = Arc::new(ServiceRegistry::new());
    let validator = CoordinationValidator::new(registry.clone());

    // Register only input primal (missing compute and aggregation)
    let input_id = registry
        .register_service(
            "InputProcessor".to_string(),
            vec!["input".to_string()],
            "/run/user/1000/input.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry.update_health(&input_id, "healthy".to_string()).await.unwrap();

    // Step 2: Create a sequential graph requiring missing capabilities
    let graph = Graph::new(
        "incomplete-pipeline".to_string(),
        "Incomplete Pipeline".to_string(),
        vec![
            create_test_node("input", "input"),
            create_test_node("process", "compute"), // Missing!
            create_test_node("store", "storage"),   // Missing!
        ],
        vec![
            GraphEdge {
                from: "input".to_string(),
                to: "process".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "process".to_string(),
                to: "store".to_string(),
                data_mapping: None,
            },
        ],
        GraphMetadata::default(),
    );

    // Step 3: Validate coordination pattern
    let result = validator.validate_pattern(&graph).await.unwrap();

    // Step 4: Verify results
    assert_eq!(result.pattern, CoordinationPattern::Sequential);
    assert!(!result.valid, "Should be invalid due to missing capabilities");
    assert!(!result.issues.is_empty(), "Should have issues about missing primals");

    // Check that issues mention the missing capabilities
    let issue_messages: Vec<_> = result.issues.iter().map(|i| &i.message).collect();
    let has_compute_issue =
        issue_messages.iter().any(|msg| msg.contains("compute") || msg.contains("capability"));
    assert!(has_compute_issue, "Should have issue about missing compute capability");
}
