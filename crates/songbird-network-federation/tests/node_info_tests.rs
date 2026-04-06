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
    reason = "test assertions and harness ergonomics"
)]

//! Node information tests

#![allow(clippy::unwrap_used, reason = "test assertions and harness ergonomics")]
use songbird_network_federation::NodeInfo;
#[test]
fn test_node_info_creation() {
    let node = NodeInfo {
        node_id: "test-node-1".to_string(),
        address: "http://localhost:8080".to_string(),
        status: "active".to_string(),
    };
    assert_eq!(node.node_id, "test-node-1");
    assert_eq!(node.address, "http://localhost:8080");
    assert_eq!(node.status, "active");
}
#[test]
fn test_node_info_clone() {
    let node1 = NodeInfo {
        node_id: "node-1".to_string(),
        address: "http://localhost:8080".to_string(),
        status: "active".to_string(),
    };
    let node2 = node1.clone();
    assert_eq!(node1.node_id, node2.node_id);
    assert_eq!(node1.address, node2.address);
    assert_eq!(node1.status, node2.status);
}

#[test]
fn test_node_info_debug() {
    let node = NodeInfo {
        node_id: "test".to_string(),
        address: "http://localhost:8080".to_string(),
        status: "active".to_string(),
    };
    let debug_output = format!("{:?}", node);
    assert!(debug_output.contains("NodeInfo") || debug_output.contains("test"));
}

#[test]
fn test_node_with_multiple_capabilities() {
    let node = NodeInfo {
        node_id: "multi-cap-node".to_string(),
        address: "http://localhost:9000".to_string(),
        status: "active".to_string(),
    };
    assert_eq!(node.node_id, "multi-cap-node");
    assert_eq!(node.address, "http://localhost:9000");
}

#[test]
fn test_node_with_metadata() {
    let node = NodeInfo {
        node_id: "meta-node".to_string(),
        address: "http://localhost:8080".to_string(),
        status: "production".to_string(),
    };
    assert_eq!(node.node_id, "meta-node");
    assert_eq!(node.status, "production");
}

#[test]
fn test_node_with_empty_capabilities() {
    let node = NodeInfo {
        node_id: "empty-cap-node".to_string(),
        address: "http://localhost:8080".to_string(),
        status: "inactive".to_string(),
    };
    assert_eq!(node.node_id, "empty-cap-node");
    assert_eq!(node.status, "inactive");
}

#[test]
fn test_multiple_nodes() {
    let nodes = [
        NodeInfo {
            node_id: "node-1".to_string(),
            address: "http://localhost:8080".to_string(),
            status: "active".to_string(),
        },
        NodeInfo {
            node_id: "node-2".to_string(),
            address: "http://localhost:8081".to_string(),
            status: "active".to_string(),
        },
    ];
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].node_id, "node-1");
    assert_eq!(nodes[1].node_id, "node-2");
}

#[test]
fn test_node_addresses() {
    let addresses =
        ["http://localhost:8080", "http://192.168.1.100:9000", "https://node.example.com:443"];
    for (i, addr) in addresses.iter().enumerate() {
        let node = NodeInfo {
            node_id: format!("node-{}", i),
            address: (*addr).to_string(),
            status: "active".to_string(),
        };
        assert_eq!(&node.address, addr);
    }
}

#[test]
fn test_node_serialization() {
    let node = NodeInfo {
        node_id: "test-node".to_string(),
        address: "http://localhost:8080".to_string(),
        status: "active".to_string(),
    };
    let json = serde_json::to_string(&node).expect("test precondition");
    assert!(json.contains("test-node"));
}

#[test]
fn test_node_deserialization() {
    let json = r#"{
        "node_id": "test-node",
        "address": "http://localhost:8080",
        "status": "active"
    }"#;
    let node: NodeInfo = serde_json::from_str(json).expect("should parse valid input");
    assert_eq!(node.node_id, "test-node");
}
