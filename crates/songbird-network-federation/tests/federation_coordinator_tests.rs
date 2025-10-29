//! Comprehensive tests for Federation Coordinator and Network-Federation Integration
//!
//! Tests federation coordination, node management, and network integration

use songbird_network_federation::{
    FederationConfig, FederationCoordinator, NetworkFederationBridge, NodeInfo,
};

// ============================================================================
// FederationCoordinator Tests
// ============================================================================

#[test]
fn test_federation_coordinator_new() {
    let coordinator = FederationCoordinator::new();
    assert!(format!("{coordinator:?}").contains("FederationCoordinator"));
}

#[test]
fn test_federation_coordinator_default() {
    let coordinator = FederationCoordinator;
    assert!(format!("{coordinator:?}").contains("FederationCoordinator"));
}

#[tokio::test]
async fn test_federation_coordinator_coordinate() {
    let coordinator = FederationCoordinator::new();
    let result = coordinator.coordinate().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_federation_coordinator_multiple_coordinat_calls() {
    let coordinator = FederationCoordinator::new();

    // Should be able to call coordinate multiple times
    for _ in 0..5 {
        let result = coordinator.coordinate().await;
        assert!(result.is_ok());
    }
}

#[test]
fn test_federation_coordinator_debug() {
    let coordinator = FederationCoordinator::new();
    let debug_str = format!("{coordinator:?}");
    assert!(!debug_str.is_empty());
}

// ============================================================================
// FederationConfig Tests
// ============================================================================

#[test]
fn test_federation_config_default() {
    let config = FederationConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.node_id, "node-1");
}

#[test]
fn test_federation_config_custom() {
    let config = FederationConfig {
        enabled: true,
        node_id: "custom-node-123".to_string(),
    };

    assert!(config.enabled);
    assert_eq!(config.node_id, "custom-node-123");
}

#[test]
fn test_federation_config_clone() {
    let config = FederationConfig {
        enabled: true,
        node_id: "node-1".to_string(),
    };

    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.node_id, cloned.node_id);
}

#[test]
fn test_federation_config_debug() {
    let config = FederationConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("FederationConfig"));
    assert!(debug_str.contains("node-1"));
}

#[test]
fn test_federation_config_serialization() {
    let config = FederationConfig {
        enabled: true,
        node_id: "test-node".to_string(),
    };

    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: FederationConfig = serde_json::from_str(&serialized).unwrap();

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.node_id, deserialized.node_id);
}

#[test]
fn test_federation_config_enabled_states() {
    let enabled = FederationConfig {
        enabled: true,
        node_id: "node-1".to_string(),
    };

    let disabled = FederationConfig {
        enabled: false,
        node_id: "node-1".to_string(),
    };

    assert!(enabled.enabled);
    assert!(!disabled.enabled);
}

// ============================================================================
// NodeInfo Tests
// ============================================================================

#[test]
fn test_node_info_creation() {
    let node = NodeInfo {
        node_id: "node-123".to_string(),
        address: "192.168.1.100:8080".to_string(),
        status: "active".to_string(),
    };

    assert_eq!(node.node_id, "node-123");
    assert_eq!(node.address, "192.168.1.100:8080");
    assert_eq!(node.status, "active");
}

#[test]
fn test_node_info_clone() {
    let node = NodeInfo {
        node_id: "node-123".to_string(),
        address: "192.168.1.100:8080".to_string(),
        status: "active".to_string(),
    };

    let cloned = node.clone();
    assert_eq!(node.node_id, cloned.node_id);
    assert_eq!(node.address, cloned.address);
    assert_eq!(node.status, cloned.status);
}

#[test]
fn test_node_info_debug() {
    let node = NodeInfo {
        node_id: "node-123".to_string(),
        address: "192.168.1.100:8080".to_string(),
        status: "active".to_string(),
    };

    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("node-123"));
    assert!(debug_str.contains("192.168.1.100:8080"));
    assert!(debug_str.contains("active"));
}

#[test]
fn test_node_info_serialization() {
    let node = NodeInfo {
        node_id: "node-123".to_string(),
        address: "192.168.1.100:8080".to_string(),
        status: "active".to_string(),
    };

    let serialized = serde_json::to_string(&node).unwrap();
    let deserialized: NodeInfo = serde_json::from_str(&serialized).unwrap();

    assert_eq!(node.node_id, deserialized.node_id);
    assert_eq!(node.address, deserialized.address);
    assert_eq!(node.status, deserialized.status);
}

#[test]
fn test_node_info_various_statuses() {
    let statuses = vec!["active", "inactive", "joining", "leaving", "error"];

    for status in statuses {
        let node = NodeInfo {
            node_id: "node-1".to_string(),
            address: "localhost:8080".to_string(),
            status: status.to_string(),
        };

        assert_eq!(node.status, status);
    }
}

#[test]
fn test_node_info_various_addresses() {
    let addresses = vec!["localhost:8080", "192.168.1.1:9000", "10.0.0.1:3000", "example.com:443"];

    for address in addresses {
        let node = NodeInfo {
            node_id: "node-1".to_string(),
            address: address.to_string(),
            status: "active".to_string(),
        };

        assert_eq!(node.address, address);
    }
}

// ============================================================================
// NetworkFederationBridge Tests
// ============================================================================

#[test]
fn test_network_federation_bridge_new() {
    let bridge = NetworkFederationBridge::new();
    assert!(format!("{bridge:?}").contains("NetworkFederationBridge"));
}

#[test]
fn test_network_federation_bridge_default() {
    let bridge = NetworkFederationBridge;
    assert!(format!("{bridge:?}").contains("NetworkFederationBridge"));
}

#[tokio::test]
async fn test_network_federation_bridge_initialize() {
    let mut bridge = NetworkFederationBridge::new();
    let result = bridge.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_network_federation_bridge_multiple_initialize() {
    let mut bridge = NetworkFederationBridge::new();

    // Should be able to initialize multiple times
    for _ in 0..3 {
        let result = bridge.initialize().await;
        assert!(result.is_ok());
    }
}

#[test]
fn test_network_federation_bridge_debug() {
    let bridge = NetworkFederationBridge::new();
    let debug_str = format!("{bridge:?}");
    assert!(!debug_str.is_empty());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_coordinator_and_bridge_integration() {
    let coordinator = FederationCoordinator::new();
    let mut bridge = NetworkFederationBridge::new();

    // Initialize bridge
    let bridge_result = bridge.initialize().await;
    assert!(bridge_result.is_ok());

    // Coordinate federation
    let coord_result = coordinator.coordinate().await;
    assert!(coord_result.is_ok());
}

#[test]
fn test_federation_config_with_node_info() {
    let config = FederationConfig {
        enabled: true,
        node_id: "node-123".to_string(),
    };

    let node = NodeInfo {
        node_id: config.node_id.clone(),
        address: "192.168.1.100:8080".to_string(),
        status: "active".to_string(),
    };

    assert_eq!(config.node_id, node.node_id);
}

#[test]
fn test_multiple_node_infos() {
    let nodes = vec![
        NodeInfo {
            node_id: "node-1".to_string(),
            address: "192.168.1.1:8080".to_string(),
            status: "active".to_string(),
        },
        NodeInfo {
            node_id: "node-2".to_string(),
            address: "192.168.1.2:8080".to_string(),
            status: "active".to_string(),
        },
        NodeInfo {
            node_id: "node-3".to_string(),
            address: "192.168.1.3:8080".to_string(),
            status: "joining".to_string(),
        },
    ];

    assert_eq!(nodes.len(), 3);
    assert!(nodes.iter().any(|n| n.status == "joining"));
    assert_eq!(nodes.iter().filter(|n| n.status == "active").count(), 2);
}

#[test]
fn test_federation_config_json_serialization() {
    let config = FederationConfig {
        enabled: true,
        node_id: "test-node-456".to_string(),
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    assert!(json.contains("enabled"));
    assert!(json.contains("node_id"));
    assert!(json.contains("test-node-456"));
}

#[test]
fn test_node_info_json_serialization() {
    let node = NodeInfo {
        node_id: "node-789".to_string(),
        address: "10.0.0.1:9000".to_string(),
        status: "inactive".to_string(),
    };

    let json = serde_json::to_string_pretty(&node).unwrap();
    assert!(json.contains("node_id"));
    assert!(json.contains("address"));
    assert!(json.contains("status"));
    assert!(json.contains("node-789"));
}

#[tokio::test]
async fn test_full_federation_workflow() {
    // Create configuration
    let config = FederationConfig {
        enabled: true,
        node_id: "primary-node".to_string(),
    };

    // Create node info
    let node = NodeInfo {
        node_id: config.node_id.clone(),
        address: "192.168.1.1:8080".to_string(),
        status: "active".to_string(),
    };

    // Create coordinator
    let coordinator = FederationCoordinator::new();

    // Create bridge
    let mut bridge = NetworkFederationBridge::new();

    // Initialize
    assert!(bridge.initialize().await.is_ok());

    // Coordinate
    assert!(coordinator.coordinate().await.is_ok());

    // Verify node info matches config
    assert_eq!(node.node_id, config.node_id);
    assert!(config.enabled);
}

#[test]
fn test_disabled_federation_config() {
    let config = FederationConfig::default();
    assert!(!config.enabled);

    // Even with disabled federation, we should be able to create nodes
    let node = NodeInfo {
        node_id: config.node_id.clone(),
        address: "localhost:8080".to_string(),
        status: "standby".to_string(),
    };

    assert_eq!(node.status, "standby");
}
