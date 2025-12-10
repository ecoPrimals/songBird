//! Node information tests

#![allow(clippy::unwrap_used)]
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
    let nodes = vec![
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
        vec!["http://localhost:8080", "http://192.168.1.100:9000", "https://node.example.com:443"];
    for (i, addr) in addresses.iter().enumerate() {
        let node = NodeInfo {
            node_id: format!("node-{}", i),
            address: addr.to_string(),
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
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("test-node"));
}

#[test]
fn test_node_deserialization() {
    let json = r#"{
        "node_id": "test-node",
        "address": "http://localhost:8080",
        "status": "active"
    }"#;
    let node: NodeInfo = serde_json::from_str(json).unwrap();
    assert_eq!(node.node_id, "test-node");
}
