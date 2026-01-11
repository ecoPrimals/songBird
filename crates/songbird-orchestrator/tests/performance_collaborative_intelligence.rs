//! Performance benchmarks for Collaborative Intelligence APIs
//!
//! Verifies that validation operations meet latency targets:
//! - graph.validate: < 50ms (small graphs)
//! - graph.check_availability: < 50ms
//! - graph.suggest_alternatives: < 30ms
//! - coordination.validate_pattern: < 100ms

use songbird_orchestrator::graph::{
    AvailabilityChecker, CoordinationValidator, Graph, GraphEdge, GraphMetadata, GraphNode,
    GraphValidator,
};
use songbird_orchestrator::ipc::registry::ServiceRegistry;
use std::sync::Arc;
use std::time::Instant;

fn create_small_graph(nodes: usize) -> Graph {
    let mut graph_nodes = Vec::new();
    let mut edges = Vec::new();

    for i in 0..nodes {
        graph_nodes.push(GraphNode {
            id: format!("node{}", i),
            primal_name: None,
            capability: "compute".to_string(),
            inputs: if i == 0 {
                vec![]
            } else {
                vec!["data".to_string()]
            },
            outputs: vec!["data".to_string()],
            config: serde_json::json!({}),
            preferred_protocol: None,
            timeout_secs: None,
        });

        if i > 0 {
            edges.push(GraphEdge {
                from: format!("node{}", i - 1),
                to: format!("node{}", i),
                data_mapping: None,
            });
        }
    }

    Graph::new(
        "benchmark-graph".to_string(),
        "Benchmark Graph".to_string(),
        graph_nodes,
        edges,
        GraphMetadata::default(),
    )
}

#[tokio::test]
async fn benchmark_graph_validation() {
    let validator = GraphValidator::new();
    let graph = create_small_graph(10);

    let start = Instant::now();
    let result = validator.validate(&graph);
    let duration = start.elapsed();

    assert!(result.valid, "Graph should be valid");
    println!("✅ graph.validate: {:?} (target: < 50ms)", duration);
    assert!(
        duration.as_millis() < 50,
        "Validation took {:?}, expected < 50ms",
        duration
    );
}

#[tokio::test]
async fn benchmark_availability_checking() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    // Register test primal
    registry
        .register_service(
            "TestPrimal".to_string(),
            vec!["compute".to_string()],
            "/tmp/test.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let graph = create_small_graph(5);

    let start = Instant::now();
    let _result = checker.check_availability(&graph).await.unwrap();
    let duration = start.elapsed();

    println!(
        "✅ graph.check_availability: {:?} (target: < 50ms)",
        duration
    );
    assert!(
        duration.as_millis() < 50,
        "Availability check took {:?}, expected < 50ms",
        duration
    );
}

#[tokio::test]
async fn benchmark_alternative_suggestions() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    // Register multiple primals
    for i in 0..3 {
        registry
            .register_service(
                format!("TestPrimal{}", i),
                vec!["compute".to_string()],
                format!("/tmp/test{}.sock", i),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();
    }

    let node = GraphNode {
        id: "test".to_string(),
        primal_name: None,
        capability: "compute".to_string(),
        inputs: vec![],
        outputs: vec![],
        config: serde_json::json!({}),
        preferred_protocol: Some("json-rpc".to_string()),
        timeout_secs: None,
    };

    let start = Instant::now();
    let _result = checker.suggest_alternatives(&node).await.unwrap();
    let duration = start.elapsed();

    println!(
        "✅ graph.suggest_alternatives: {:?} (target: < 30ms)",
        duration
    );
    assert!(
        duration.as_millis() < 30,
        "Alternative suggestion took {:?}, expected < 30ms",
        duration
    );
}

#[tokio::test]
async fn benchmark_coordination_validation() {
    let registry = Arc::new(ServiceRegistry::new());
    let validator = CoordinationValidator::new(registry.clone());

    // Register test primal
    registry
        .register_service(
            "TestPrimal".to_string(),
            vec!["compute".to_string()],
            "/tmp/test.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let graph = create_small_graph(10);

    let start = Instant::now();
    let result = validator.validate_pattern(&graph).await.unwrap();
    let duration = start.elapsed();

    assert!(result.valid, "Coordination should be valid");
    println!(
        "✅ coordination.validate_pattern: {:?} (target: < 100ms)",
        duration
    );
    assert!(
        duration.as_millis() < 100,
        "Coordination validation took {:?}, expected < 100ms",
        duration
    );
}

#[tokio::test]
async fn benchmark_full_validation_workflow() {
    let registry = Arc::new(ServiceRegistry::new());

    // Register test primals
    for capability in &["input", "compute", "storage"] {
        registry
            .register_service(
                format!("{}Primal", capability),
                vec![capability.to_string()],
                format!("/tmp/{}.sock", capability),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();
    }

    let graph = Graph::new(
        "full-workflow".to_string(),
        "Full Workflow".to_string(),
        vec![
            GraphNode {
                id: "input".to_string(),
                primal_name: None,
                capability: "input".to_string(),
                inputs: vec![],
                outputs: vec!["data".to_string()],
                config: serde_json::json!({}),
                preferred_protocol: None,
                timeout_secs: None,
            },
            GraphNode {
                id: "compute".to_string(),
                primal_name: None,
                capability: "compute".to_string(),
                inputs: vec!["data".to_string()],
                outputs: vec!["result".to_string()],
                config: serde_json::json!({}),
                preferred_protocol: None,
                timeout_secs: None,
            },
            GraphNode {
                id: "storage".to_string(),
                primal_name: None,
                capability: "storage".to_string(),
                inputs: vec!["result".to_string()],
                outputs: vec![],
                config: serde_json::json!({}),
                preferred_protocol: None,
                timeout_secs: None,
            },
        ],
        vec![
            GraphEdge {
                from: "input".to_string(),
                to: "compute".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "compute".to_string(),
                to: "storage".to_string(),
                data_mapping: None,
            },
        ],
        GraphMetadata::default(),
    );

    // Full workflow: validate → check availability → validate coordination
    let start = Instant::now();

    let validator = GraphValidator::new();
    let validation = validator.validate(&graph);
    assert!(validation.valid);

    let checker = AvailabilityChecker::new(registry.clone());
    let availability = checker.check_availability(&graph).await.unwrap();
    // Check that primals are available (should have some in 'available' list)
    assert!(!availability.available.is_empty(), "Should have available primals");

    let coord_validator = CoordinationValidator::new(registry.clone());
    let coordination = coord_validator.validate_pattern(&graph).await.unwrap();
    assert!(coordination.valid);

    let duration = start.elapsed();

    println!(
        "✅ Full validation workflow: {:?} (target: < 200ms)",
        duration
    );
    assert!(
        duration.as_millis() < 200,
        "Full workflow took {:?}, expected < 200ms",
        duration
    );
}

