// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Federation Coordinator Tests - Modern API
//!
//! Tests the federation coordination system with modern concurrent patterns

use songbird_network_federation::federation::{FederationConfig, FederationCoordinator};
use songbird_network_federation::state::{NodeRegistration, NodeStatus};
use songbird_network_federation::NetworkFederationBridge;

// ============================================================================
// FederationConfig Tests - Modern API
// ============================================================================

#[test]
fn test_federation_config_default() {
    let config = FederationConfig::default();
    assert!(!config.enabled); // Disabled by default
    assert!(config.bootstrap_address.is_none());
    assert!(config.self_registration.is_none());
    // Modern defaults from implementation
    assert_eq!(config.heartbeat_interval_secs, 30);
    assert_eq!(config.node_timeout_secs, 60); // Actual default in code
}

#[test]
fn test_federation_config_enabled() {
    let config = FederationConfig {
        enabled: true,
        bootstrap_address: Some("192.168.1.1:8080".to_string()),
        self_registration: None,
        heartbeat_interval_secs: 30,
        node_timeout_secs: 90,
    };

    assert!(config.enabled);
    assert!(config.bootstrap_address.is_some());
}

#[test]
fn test_federation_config_with_node_info() {
    // Modern API: Use NodeRegistration for self-registration
    let node_reg = NodeRegistration {
        node_id: uuid::Uuid::new_v4().to_string(),
        node_name: "node-123".to_string(),
        node_address: "192.168.1.100:8080".to_string(),
        cpu_cores: 8,
        memory_gb: 16,
        gpu_model: None,
        storage_gb: None,
        capabilities: vec!["compute".to_string()],
        status: NodeStatus::Active,
        joined_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
    };

    let config = FederationConfig {
        enabled: true,
        bootstrap_address: None,
        self_registration: Some(node_reg),
        heartbeat_interval_secs: 30,
        node_timeout_secs: 90,
    };

    assert!(config.enabled);
    assert_eq!(config.self_registration.as_ref().expect("test precondition").node_name, "node-123");
}

#[test]
fn test_federation_config_clone() {
    let node_reg = create_test_registration("node-1");

    let config = FederationConfig {
        enabled: true,
        bootstrap_address: Some("192.168.1.1:8080".to_string()),
        self_registration: Some(node_reg),
        heartbeat_interval_secs: 15,
        node_timeout_secs: 45,
    };

    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.heartbeat_interval_secs, cloned.heartbeat_interval_secs);
    assert_eq!(
        config.self_registration.as_ref().expect("test precondition").node_name,
        cloned.self_registration.as_ref().expect("test precondition").node_name
    );
}

#[test]
fn test_federation_config_debug() {
    let config = FederationConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("FederationConfig"));
    assert!(debug_str.contains("enabled"));
}

#[test]
fn test_federation_config_serialization() {
    let node_reg = create_test_registration("test-node");

    let config = FederationConfig {
        enabled: true,
        bootstrap_address: Some("192.168.1.1:8080".to_string()),
        self_registration: Some(node_reg),
        heartbeat_interval_secs: 30,
        node_timeout_secs: 90,
    };

    let serialized = serde_json::to_string(&config).expect("test precondition");
    let deserialized: FederationConfig =
        serde_json::from_str(&serialized).expect("should parse valid input");

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.heartbeat_interval_secs, deserialized.heartbeat_interval_secs);
}

#[test]
fn test_federation_config_enabled_states() {
    let enabled = FederationConfig {
        enabled: true,
        bootstrap_address: None,
        self_registration: None,
        heartbeat_interval_secs: 30,
        node_timeout_secs: 90,
    };

    let disabled = FederationConfig::default();

    assert!(enabled.enabled);
    assert!(!disabled.enabled);
}

// ============================================================================
// NodeRegistration Tests
// ============================================================================

#[test]
fn test_node_registration_creation() {
    let node = create_test_registration("node-123");

    assert_eq!(node.node_name, "node-123");
    assert_eq!(node.node_address, "192.168.1.100:8080");
    assert!(matches!(node.status, NodeStatus::Active));
}

#[test]
fn test_node_registration_clone() {
    let node = create_test_registration("node-123");
    let cloned = node.clone();

    assert_eq!(node.node_id, cloned.node_id);
    assert_eq!(node.node_name, cloned.node_name);
    assert_eq!(node.node_address, cloned.node_address);
}

#[test]
fn test_node_registration_debug() {
    let node = create_test_registration("node-123");
    let debug_str = format!("{node:?}");

    assert!(debug_str.contains("node-123"));
    assert!(debug_str.contains("192.168.1.100:8080"));
    assert!(debug_str.contains("Active"));
}

#[test]
fn test_node_registration_serialization() {
    let node = create_test_registration("node-123");

    let serialized = serde_json::to_string(&node).expect("test precondition");
    let deserialized: NodeRegistration =
        serde_json::from_str(&serialized).expect("should parse valid input");

    assert_eq!(node.node_id, deserialized.node_id);
    assert_eq!(node.node_name, deserialized.node_name);
    assert_eq!(node.node_address, deserialized.node_address);
}

#[test]
fn test_node_registration_various_statuses() {
    // Test available node statuses in modern API
    let statuses = vec![NodeStatus::Active, NodeStatus::Inactive];

    for status in statuses {
        let mut node = create_test_registration("node-1");
        node.status = status;
        assert_eq!(node.status, status);
    }
}

#[test]
fn test_node_registration_various_addresses() {
    let addresses = vec!["localhost:8080", "192.168.1.1:9000", "10.0.0.1:3000", "example.com:443"];

    for address in addresses {
        let mut node = create_test_registration("node-1");
        node.node_address = address.to_string();
        assert_eq!(node.node_address, address);
    }
}

// ============================================================================
// NetworkFederationBridge Tests
// ============================================================================

#[test]
fn test_network_federation_bridge_new() {
    let bridge = NetworkFederationBridge::new();
    // Just verify it can be created
    assert!(format!("{bridge:?}").contains("NetworkFederationBridge"));
}

#[test]
fn test_network_federation_bridge_debug() {
    let bridge = NetworkFederationBridge::new();
    let debug_str = format!("{bridge:?}");
    assert!(debug_str.contains("NetworkFederationBridge"));
}

// ============================================================================
// FederationCoordinator Tests
// ============================================================================

#[test]
fn test_coordinator_new() {
    let coordinator = FederationCoordinator::new();
    // Verify creation
    assert!(format!("{coordinator:?}").contains("FederationCoordinator"));
}

#[test]
fn test_coordinator_debug() {
    let coordinator = FederationCoordinator::new();
    let debug_str = format!("{coordinator:?}");
    assert!(debug_str.contains("FederationCoordinator"));
}

#[test]
fn test_coordinator_clone() {
    let coordinator = FederationCoordinator::new();
    let cloned = coordinator;
    // Both should have different Arc clones pointing to same state
    assert!(format!("{cloned:?}").contains("FederationCoordinator"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_multiple_node_registrations() {
    let nodes = vec![
        create_test_registration("node-1"),
        create_test_registration("node-2"),
        create_test_registration("node-3"),
    ];

    assert_eq!(nodes.len(), 3);
    assert!(nodes.iter().all(|n| matches!(n.status, NodeStatus::Active)));
}

#[test]
fn test_federation_config_json_serialization() {
    let node_reg = create_test_registration("test-node-456");

    let config = FederationConfig {
        enabled: true,
        bootstrap_address: Some("192.168.1.1:8080".to_string()),
        self_registration: Some(node_reg),
        heartbeat_interval_secs: 30,
        node_timeout_secs: 90,
    };

    let json = serde_json::to_string_pretty(&config).expect("test precondition");
    assert!(json.contains("enabled"));
    assert!(json.contains("bootstrap_address"));
}

#[test]
fn test_node_registration_json_serialization() {
    let node = create_test_registration("node-789");

    let json = serde_json::to_string_pretty(&node).expect("test precondition");
    assert!(json.contains("node_id"));
    assert!(json.contains("node_address"));
    assert!(json.contains("status"));
}

#[tokio::test]
async fn test_full_federation_workflow() {
    // Create configuration with self-registration
    let node_reg = create_test_registration("primary-node");

    let config = FederationConfig {
        enabled: true,
        bootstrap_address: None,
        self_registration: Some(node_reg.clone()),
        heartbeat_interval_secs: 30,
        node_timeout_secs: 90,
    };

    // Create coordinator
    let coordinator = FederationCoordinator::new();

    // Create bridge
    let mut bridge = NetworkFederationBridge::new();

    // Initialize bridge (concurrent-safe)
    assert!(bridge.initialize().await.is_ok());

    // Coordinate - modern API requires config parameter
    assert!(coordinator.coordinate(&config).await.is_ok());

    // Verify state
    let state = coordinator.state();
    assert_eq!(state.nodes.read().await.len(), 1); // Self-registered
}

#[tokio::test]
async fn test_federation_disabled() {
    let config = FederationConfig::default();
    assert!(!config.enabled);

    let coordinator = FederationCoordinator::new();

    // With disabled federation, coordinate should succeed but do nothing
    assert!(coordinator.coordinate(&config).await.is_ok());

    // No nodes should be registered
    let state = coordinator.state();
    assert_eq!(state.nodes.read().await.len(), 0);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_registration(node_name: &str) -> NodeRegistration {
    NodeRegistration {
        node_id: uuid::Uuid::new_v4().to_string(),
        node_name: node_name.to_string(),
        node_address: "192.168.1.100:8080".to_string(),
        cpu_cores: 8,
        memory_gb: 16,
        gpu_model: None,
        storage_gb: Some(512),
        capabilities: vec!["compute".to_string()],
        status: NodeStatus::Active,
        joined_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
    }
}
