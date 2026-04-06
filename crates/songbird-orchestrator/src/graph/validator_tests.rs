// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::graph::types::{GraphEdge, GraphMetadata, GraphNode};

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

#[test]
fn test_valid_simple_graph() {
    let validator = GraphValidator::new();
    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![create_test_node("node-1", "encryption")],
        vec![],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(result.valid);
    assert!(result.issues.is_empty());
}

#[test]
fn test_detect_cycle() {
    let validator = GraphValidator::new();

    let mut node1 = create_test_node("node-1", "encryption");
    node1.outputs = vec!["data".to_string()];

    let mut node2 = create_test_node("node-2", "storage");
    node2.inputs = vec!["data".to_string()];
    node2.outputs = vec!["result".to_string()];

    let mut node3 = create_test_node("node-3", "compute");
    node3.inputs = vec!["result".to_string()];
    node3.outputs = vec!["data".to_string()];

    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![node1, node2, node3],
        vec![
            GraphEdge {
                from: "node-1".to_string(),
                to: "node-2".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "node-2".to_string(),
                to: "node-3".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "node-3".to_string(),
                to: "node-1".to_string(),
                data_mapping: None,
            },
        ],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(!result.valid);
    assert!(result.issues.iter().any(|i| i.code == "CYCLE_DETECTED"));
}

#[test]
fn test_duplicate_node_ids() {
    let validator = GraphValidator::new();
    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![create_test_node("node-1", "encryption"), create_test_node("node-1", "storage")],
        vec![],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(!result.valid);
    assert!(result.issues.iter().any(|i| i.code == "DUPLICATE_NODE_ID"));
}

#[test]
fn test_invalid_edge_reference() {
    let validator = GraphValidator::new();
    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![create_test_node("node-1", "encryption")],
        vec![GraphEdge {
            from: "node-1".to_string(),
            to: "node-2".to_string(),
            data_mapping: None,
        }],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(!result.valid);
    assert!(result.issues.iter().any(|i| i.code == "INVALID_EDGE_TARGET"));
}

#[test]
fn test_missing_capability() {
    let validator = GraphValidator::new();
    let mut node = create_test_node("node-1", "");
    node.capability = String::new();

    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![node],
        vec![],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(!result.valid);
    assert!(result.issues.iter().any(|i| i.code == "MISSING_CAPABILITY"));
}

#[test]
fn test_orphan_node() {
    let validator = GraphValidator::new();

    let mut node1 = create_test_node("node-1", "encryption");
    node1.outputs = vec!["data".to_string()];

    let mut node2 = create_test_node("node-2", "storage");
    node2.inputs = vec!["data".to_string()];

    let mut node3 = create_test_node("node-3", "compute");
    node3.inputs = vec!["input".to_string()];
    node3.outputs = vec!["output".to_string()];

    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![node1, node2, node3],
        vec![GraphEdge {
            from: "node-1".to_string(),
            to: "node-2".to_string(),
            data_mapping: None,
        }],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(!result.valid);
    assert!(result.issues.iter().any(|i| i.code == "ORPHAN_NODE"));
    assert!(result.issues.iter().any(|i| i.nodes.contains(&"node-3".to_string())));
}

#[test]
fn test_unsatisfied_input() {
    let validator = GraphValidator::new();

    let mut node1 = create_test_node("node-1", "encryption");
    node1.outputs = vec!["encrypted_data".to_string()];

    let mut node2 = create_test_node("node-2", "storage");
    node2.inputs = vec!["decrypted_data".to_string()];

    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![node1, node2],
        vec![GraphEdge {
            from: "node-1".to_string(),
            to: "node-2".to_string(),
            data_mapping: None,
        }],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(!result.valid);
    assert!(result.issues.iter().any(|i| i.code == "UNSATISFIED_INPUT"));
}

#[test]
fn test_complex_graph_validation() {
    let validator = GraphValidator::new();

    let mut nodes = Vec::new();
    for i in 1..=10 {
        let mut node = create_test_node(&format!("node-{i}"), "compute");
        if i > 1 {
            node.inputs = vec![format!("data-{}", i - 1)];
        }
        if i < 10 {
            node.outputs = vec![format!("data-{i}")];
        }
        nodes.push(node);
    }

    let mut edges = Vec::new();
    for i in 1..10 {
        edges.push(GraphEdge {
            from: format!("node-{i}"),
            to: format!("node-{}", i + 1),
            data_mapping: None,
        });
    }

    let graph = Graph::new(
        "complex".to_string(),
        "Complex Graph".to_string(),
        nodes,
        edges,
        GraphMetadata::default(),
    );
    let result = validator.validate(&graph);
    assert!(result.valid, "Complex graph should be valid");
    assert!(result.issues.is_empty());
    assert_eq!(result.info.as_ref().unwrap().node_count, 10);
    assert_eq!(result.info.as_ref().unwrap().edge_count, 9);
    assert!(!result.info.as_ref().unwrap().has_cycles);
}

#[test]
fn test_multiple_entry_points() {
    let validator = GraphValidator::new();
    let mut node1 = create_test_node("entry-1", "source");
    node1.outputs = vec!["data-1".to_string()];
    let mut node2 = create_test_node("entry-2", "source");
    node2.outputs = vec!["data-2".to_string()];
    let mut node3 = create_test_node("merge", "compute");
    node3.inputs = vec!["data-1".to_string(), "data-2".to_string()];
    node3.outputs = vec!["merged".to_string()];

    let graph = Graph::new(
        "multi-entry".to_string(),
        "Multiple Entry Points".to_string(),
        vec![node1, node2, node3],
        vec![
            GraphEdge {
                from: "entry-1".to_string(),
                to: "merge".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "entry-2".to_string(),
                to: "merge".to_string(),
                data_mapping: None,
            },
        ],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(result.valid);
    let info = result.info.unwrap();
    assert_eq!(info.entry_points.len(), 2);
}

#[test]
fn test_multiple_exit_points() {
    let validator = GraphValidator::new();
    let mut node1 = create_test_node("source", "source");
    node1.outputs = vec!["data".to_string()];
    let mut node2 = create_test_node("exit-1", "sink");
    node2.inputs = vec!["data".to_string()];
    let mut node3 = create_test_node("exit-2", "sink");
    node3.inputs = vec!["data".to_string()];

    let graph = Graph::new(
        "multi-exit".to_string(),
        "Multiple Exit Points".to_string(),
        vec![node1, node2, node3],
        vec![
            GraphEdge {
                from: "source".to_string(),
                to: "exit-1".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "source".to_string(),
                to: "exit-2".to_string(),
                data_mapping: None,
            },
        ],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(result.valid);
    let info = result.info.unwrap();
    assert_eq!(info.exit_points.len(), 2);
}

#[test]
fn test_data_mapping() {
    let validator = GraphValidator::new();
    let mut node1 = create_test_node("node-1", "encryption");
    node1.outputs = vec!["encrypted_data".to_string()];
    let mut node2 = create_test_node("node-2", "storage");
    node2.inputs = vec!["data_to_store".to_string()];
    let mut data_mapping = HashMap::new();
    data_mapping.insert("encrypted_data".to_string(), "data_to_store".to_string());

    let graph = Graph::new(
        "mapping".to_string(),
        "Data Mapping".to_string(),
        vec![node1, node2],
        vec![GraphEdge {
            from: "node-1".to_string(),
            to: "node-2".to_string(),
            data_mapping: Some(data_mapping),
        }],
        GraphMetadata::default(),
    );

    let result = validator.validate(&graph);
    assert!(result.valid, "Graph with proper data mapping should be valid");
}

#[test]
fn invalid_edge_source_node() {
    let validator = GraphValidator::new();
    let graph = Graph::new(
        "t".to_string(),
        "T".to_string(),
        vec![create_test_node("only", "cap")],
        vec![GraphEdge {
            from: "missing".to_string(),
            to: "only".to_string(),
            data_mapping: None,
        }],
        GraphMetadata::default(),
    );
    let result = validator.validate(&graph);
    assert!(!result.valid);
    assert!(result.issues.iter().any(|i| i.code == "INVALID_EDGE_SOURCE"));
}

#[test]
fn entry_point_with_inputs_emits_warning() {
    let validator = GraphValidator::new();
    let mut node = create_test_node("entry", "cap");
    node.outputs = vec!["o".to_string()];
    node.inputs = vec!["external".to_string()];
    let graph =
        Graph::new("w".to_string(), "W".to_string(), vec![node], vec![], GraphMetadata::default());
    let result = validator.validate(&graph);
    assert!(result.warnings.iter().any(|w| w.contains("entry point")));
}

#[test]
fn graph_validator_default_same_as_new() {
    let new_v = GraphValidator::new();
    let default_v = GraphValidator::default();
    let graph = Graph::new(
        "id".to_string(),
        "n".to_string(),
        vec![create_test_node("n1", "c")],
        vec![],
        GraphMetadata::default(),
    );
    let ra = new_v.validate(&graph);
    let rb = default_v.validate(&graph);
    assert_eq!(ra.valid, rb.valid);
    assert_eq!(ra.issues.len(), rb.issues.len());
}

#[test]
fn empty_graph_no_nodes_valid() {
    let validator = GraphValidator::new();
    let graph =
        Graph::new("e".to_string(), "empty".to_string(), vec![], vec![], GraphMetadata::default());
    let result = validator.validate(&graph);
    assert!(result.valid);
}

#[test]
fn self_loop_cycle_detected() {
    let validator = GraphValidator::new();
    let mut node = create_test_node("a", "c");
    node.outputs = vec!["x".to_string()];
    node.inputs = vec!["x".to_string()];
    let graph = Graph::new(
        "s".to_string(),
        "S".to_string(),
        vec![node],
        vec![GraphEdge {
            from: "a".to_string(),
            to: "a".to_string(),
            data_mapping: None,
        }],
        GraphMetadata::default(),
    );
    let result = validator.validate(&graph);
    assert!(!result.valid);
    assert!(result.issues.iter().any(|i| i.code == "CYCLE_DETECTED"));
}

#[test]
fn two_node_cycle() {
    let validator = GraphValidator::new();
    let mut n1 = create_test_node("x", "c");
    n1.outputs = vec!["d".to_string()];
    let mut n2 = create_test_node("y", "c");
    n2.inputs = vec!["d".to_string()];
    n2.outputs = vec!["d".to_string()];
    let graph = Graph::new(
        "c2".to_string(),
        "C".to_string(),
        vec![n1, n2],
        vec![
            GraphEdge {
                from: "x".to_string(),
                to: "y".to_string(),
                data_mapping: None,
            },
            GraphEdge {
                from: "y".to_string(),
                to: "x".to_string(),
                data_mapping: None,
            },
        ],
        GraphMetadata::default(),
    );
    let result = validator.validate(&graph);
    assert!(result.issues.iter().any(|i| i.code == "CYCLE_DETECTED"));
}

#[test]
fn entry_point_with_external_inputs_valid_when_wired() {
    let validator = GraphValidator::new();
    let mut entry = create_test_node("entry", "c");
    entry.inputs = vec!["ext".to_string()];
    entry.outputs = vec!["o".to_string()];
    let mut exit = create_test_node("exit", "c");
    exit.inputs = vec!["o".to_string()];
    let graph = Graph::new(
        "ent".to_string(),
        "E".to_string(),
        vec![entry, exit],
        vec![GraphEdge {
            from: "entry".to_string(),
            to: "exit".to_string(),
            data_mapping: None,
        }],
        GraphMetadata::default(),
    );
    let result = validator.validate(&graph);
    assert!(result.valid);
}

#[test]
fn orphan_not_reported_when_is_entry_and_exit() {
    let validator = GraphValidator::new();
    let node = create_test_node("solo", "c");
    let graph = Graph::new(
        "solo".to_string(),
        "S".to_string(),
        vec![node],
        vec![],
        GraphMetadata::default(),
    );
    let result = validator.validate(&graph);
    assert!(!result.issues.iter().any(|i| i.code == "ORPHAN_NODE"));
}

#[test]
fn edge_both_endpoints_invalid() {
    let validator = GraphValidator::new();
    let graph = Graph::new(
        "be".to_string(),
        "B".to_string(),
        vec![create_test_node("only", "c")],
        vec![GraphEdge {
            from: "a".to_string(),
            to: "b".to_string(),
            data_mapping: None,
        }],
        GraphMetadata::default(),
    );
    let result = validator.validate(&graph);
    assert!(result.issues.iter().any(|i| i.code == "INVALID_EDGE_SOURCE"));
    assert!(result.issues.iter().any(|i| i.code == "INVALID_EDGE_TARGET"));
}

#[test]
fn validate_result_combines_multiple_issue_types() {
    let validator = GraphValidator::new();
    let mut bad = create_test_node("n1", "");
    bad.outputs = vec!["o".to_string()];
    let mut n2 = create_test_node("n2", "c");
    n2.inputs = vec!["missing".to_string()];
    let graph = Graph::new(
        "combo".to_string(),
        "C".to_string(),
        vec![bad, n2],
        vec![GraphEdge {
            from: "n1".to_string(),
            to: "n2".to_string(),
            data_mapping: None,
        }],
        GraphMetadata::default(),
    );
    let result = validator.validate(&graph);
    assert!(result.issues.iter().any(|i| i.code == "MISSING_CAPABILITY"));
    assert!(result.issues.iter().any(|i| i.code == "UNSATISFIED_INPUT"));
}
