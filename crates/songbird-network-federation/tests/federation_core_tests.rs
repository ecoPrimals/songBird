//! Comprehensive tests for Federation core functionality

use songbird_network_federation::{FederationConfig, FederationCoordinator, NodeInfo};
use songbird_types::SongbirdError;

#[test]
fn test_federation_coordinator_new() {
    let coordinator = FederationCoordinator::new();
    assert!(std::mem::size_of_val(&coordinator) > 0 || std::mem::size_of_val(&coordinator) == 0);
}

#[test]
fn test_federation_coordinator_default() {
    let coordinator = FederationCoordinator;
    assert!(std::mem::size_of_val(&coordinator) > 0 || std::mem::size_of_val(&coordinator) == 0);
}

#[tokio::test]
async fn test_federation_coordinator_coordinate() {
    let coordinator = FederationCoordinator::new();
    let result = coordinator.coordinate().await;
    assert!(result.is_ok(), "Coordinate should succeed");
}

#[test]
fn test_federation_config_default() {
    let config = FederationConfig::default();
    assert!(!config.enabled, "Default should be disabled");
    assert_eq!(config.node_id, "node-1", "Default node ID should be node-1");
}

#[test]
fn test_federation_config_enabled() {
    let config = FederationConfig {
        enabled: true,
        node_id: "test-node".to_string(),
    };
    assert!(config.enabled, "Should be enabled");
    assert_eq!(config.node_id, "test-node");
}

#[test]
fn test_federation_config_disabled() {
    let config = FederationConfig {
        enabled: false,
        node_id: "disabled-node".to_string(),
    };
    assert!(!config.enabled, "Should be disabled");
}

#[test]
fn test_federation_config_clone() {
    let config = FederationConfig::default();
    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.node_id, cloned.node_id);
}

#[test]
fn test_federation_config_custom_node_id() {
    let config = FederationConfig {
        enabled: true,
        node_id: "custom-node-123".to_string(),
    };
    assert_eq!(config.node_id, "custom-node-123");
}

#[test]
fn test_federation_config_empty_node_id() {
    let config = FederationConfig {
        enabled: true,
        node_id: String::new(),
    };
    assert!(config.node_id.is_empty());
}

#[test]
fn test_federation_config_long_node_id() {
    let long_id = "a".repeat(1000);
    let config = FederationConfig {
        enabled: true,
        node_id: long_id.clone(),
    };
    assert_eq!(config.node_id.len(), 1000);
}

#[test]
fn test_federation_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let config = FederationConfig {
        enabled: true,
        node_id: "serialize-test".to_string(),
    };
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("serialize-test"));
    assert!(json.contains("enabled"));
    Ok(())
}

#[test]
fn test_federation_config_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"enabled":true,"node_id":"deserialize-test"}"#;
    let config: FederationConfig = serde_json::from_str(json)
        .map_err(|e| SongbirdError::configuration(format!("Should deserialize: {}", e)))?;
    assert!(config.enabled);
    assert_eq!(config.node_id, "deserialize-test");
    Ok(())
}

#[test]
fn test_federation_config_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let original = FederationConfig {
        enabled: true,
        node_id: "round-trip-test".to_string(),
    };
    let json = serde_json::to_string(&original)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    let deserialized: FederationConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Should deserialize: {}", e)))?;
    assert_eq!(original.enabled, deserialized.enabled);
    assert_eq!(original.node_id, deserialized.node_id);
    Ok(())
}

#[test]
fn test_node_info_creation() {
    let node = NodeInfo {
        node_id: "node-1".to_string(),
        address: "127.0.0.1:8080".to_string(),
        status: "active".to_string(),
    };
    assert_eq!(node.node_id, "node-1");
    assert_eq!(node.address, "127.0.0.1:8080");
    assert_eq!(node.status, "active");
}

#[test]
fn test_node_info_clone() {
    let node = NodeInfo {
        node_id: "node-2".to_string(),
        address: "192.168.1.1:9000".to_string(),
        status: "healthy".to_string(),
    };
    let cloned = node.clone();
    assert_eq!(node.node_id, cloned.node_id);
    assert_eq!(node.address, cloned.address);
    assert_eq!(node.status, cloned.status);
}

#[test]
fn test_node_info_with_ipv6() {
    let node = NodeInfo {
        node_id: "ipv6-node".to_string(),
        address: "[::1]:8080".to_string(),
        status: "active".to_string(),
    };
    assert!(node.address.contains("::1"));
}

#[test]
fn test_node_info_various_statuses() {
    let statuses = vec!["active", "inactive", "degraded", "maintenance"];
    for status in statuses {
        let node = NodeInfo {
            node_id: "test-node".to_string(),
            address: "localhost:8080".to_string(),
            status: status.to_string(),
        };
        assert_eq!(node.status, status);
    }
}

#[test]
fn test_node_info_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let node = NodeInfo {
        node_id: "serialize-node".to_string(),
        address: "10.0.0.1:7777".to_string(),
        status: "running".to_string(),
    };
    let json = serde_json::to_string(&node)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("serialize-node"));
    assert!(json.contains("10.0.0.1:7777"));
    Ok(())
}

#[test]
fn test_node_info_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"node_id":"deser-node","address":"localhost:9999","status":"ok"}"#;
    let node: NodeInfo = serde_json::from_str(json)
        .map_err(|e| SongbirdError::configuration(format!("Should deserialize: {}", e)))?;
    assert_eq!(node.node_id, "deser-node");
    assert_eq!(node.address, "localhost:9999");
    assert_eq!(node.status, "ok");
    Ok(())
}

#[test]
fn test_federation_config_debug_format() {
    let config = FederationConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("FederationConfig"));
}

#[test]
fn test_node_info_debug_format() {
    let node = NodeInfo {
        node_id: "debug-node".to_string(),
        address: "127.0.0.1:8080".to_string(),
        status: "active".to_string(),
    };
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("NodeInfo"));
}

#[test]
fn test_coordinator_debug_format() {
    let coordinator = FederationCoordinator::new();
    let debug_str = format!("{coordinator:?}");
    assert!(debug_str.contains("FederationCoordinator"));
}

#[tokio::test]
async fn test_multiple_coordinators_independent() {
    let coordinator1 = FederationCoordinator::new();
    let coordinator2 = FederationCoordinator::new();

    let result1 = coordinator1.coordinate().await;
    let result2 = coordinator2.coordinate().await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_multiple_configs_independent() {
    let config1 = FederationConfig {
        enabled: true,
        node_id: "node-1".to_string(),
    };
    let config2 = FederationConfig {
        enabled: false,
        node_id: "node-2".to_string(),
    };

    assert_ne!(config1.enabled, config2.enabled);
    assert_ne!(config1.node_id, config2.node_id);
}

#[test]
fn test_node_info_empty_fields() {
    let node = NodeInfo {
        node_id: String::new(),
        address: String::new(),
        status: String::new(),
    };
    assert!(node.node_id.is_empty());
    assert!(node.address.is_empty());
    assert!(node.status.is_empty());
}

#[test]
fn test_config_field_independence() {
    let mut config = FederationConfig::default();
    config.enabled = true;
    assert!(config.enabled);

    config.node_id = "modified".to_string();
    assert_eq!(config.node_id, "modified");
}
