//! Canonical Federation Tests - Modern Implementation
//!
//! This test suite demonstrates the canonical federation patterns using
//! modern async/await, proper error handling, and unified APIs.

use crate::canonical::{
    CanonicalDiscovery, CanonicalFederationConfig, CanonicalFederationManager,
    CanonicalHealthMonitor, DiscoveryInfo, FederationMessage, FederationMessageType,
    FederationNode, HealthStatus, MessageType, NodeStatus,
};
use songbird_errors::SongbirdResult;
use std::collections::HashMap;
use tokio::time::Duration;
use uuid;

/// Test the canonical federation manager creation and basic operations
#[tokio::test]
async fn test_canonical_federation_manager() -> SongbirdResult<()> {
    let config = CanonicalFederationConfig::default();
    let _manager = CanonicalFederationManager::new(config).await?;

    // Test basic manager functionality - manager should be created successfully
    // Note: local_node is private, so we test creation success instead

    Ok(())
}

/// Test canonical discovery service creation and node discovery
#[tokio::test]
async fn test_canonical_discovery_service() -> SongbirdResult<()> {
    let config = CanonicalFederationConfig::default();
    let discovery = CanonicalDiscovery::new(config).await?;

    // Test discovery info creation
    let discovery_info = DiscoveryInfo::new("test-node".to_string(), "127.0.0.1:8080".to_string());

    // Test node validation
    let is_valid = discovery.validate_node(&discovery_info).await?;
    assert!(is_valid, "Test node should be valid");

    Ok(())
}

/// Test canonical health monitoring functionality
#[tokio::test]
async fn test_canonical_health_monitoring() -> SongbirdResult<()> {
    let config = CanonicalFederationConfig::default();
    let _monitor = CanonicalHealthMonitor::new(config).await?;

    // Create a test node for health monitoring
    let test_node = FederationNode {
        id: "health-test-node".to_string(),
        address: "127.0.0.1:8081".to_string(),
        status: NodeStatus::Starting,
        capabilities: vec!["test".to_string()],
        last_seen: std::time::SystemTime::now(),
        metadata: HashMap::new(),
    };

    // Test health status creation
    let health_status = HealthStatus::new(test_node.id.clone(), NodeStatus::Healthy);

    // Verify health status properties
    assert_eq!(health_status.node_id, "health-test-node");
    assert_eq!(health_status.status, NodeStatus::Healthy);

    Ok(())
}

/// Test federation message creation and serialization
#[tokio::test]
async fn test_federation_messaging() -> SongbirdResult<()> {
    // Test targeted message creation (broadcast messages created via None target)
    let targeted_msg = FederationMessage::new_targeted(
        "sender-node".to_string(),
        "target-node".to_string(),
        MessageType::ServiceDiscovery,
        serde_json::json!({"request": "info"}),
    );

    assert!(
        !targeted_msg.is_broadcast(),
        "Message should be targeted type"
    );
    assert_eq!(targeted_msg.target, Some("target-node".to_string()));
    assert_eq!(targeted_msg.sender_id, "sender-node");

    // Test broadcast-style message (created with None target)
    let broadcast_msg = FederationMessage {
        message_id: uuid::Uuid::new_v4().to_string(),
        message_type: FederationMessageType::Heartbeat,
        sender_id: "sender-node".to_string(),
        target: None, // This makes it a broadcast
        timestamp: std::time::SystemTime::now(),
        payload: serde_json::json!({"test": "data"}),
    };

    assert!(
        broadcast_msg.is_broadcast(),
        "Message should be broadcast type"
    );

    Ok(())
}

/// Test node status transitions and display
#[tokio::test]
async fn test_node_status_management() -> SongbirdResult<()> {
    // Test default node status
    let default_status = NodeStatus::default();
    assert_eq!(default_status, NodeStatus::Starting);

    // Test status display
    assert_eq!(NodeStatus::Healthy.to_string(), "Healthy");
    assert_eq!(NodeStatus::Unhealthy.to_string(), "Unhealthy");
    assert_eq!(NodeStatus::Starting.to_string(), "Starting");
    assert_eq!(NodeStatus::Stopping.to_string(), "Stopping");
    assert_eq!(NodeStatus::Stopped.to_string(), "Stopped");

    Ok(())
}

/// Test federation configuration and environment integration
#[tokio::test]
async fn test_federation_configuration() -> SongbirdResult<()> {
    // Test default configuration
    let config = CanonicalFederationConfig::default();
    assert!(!config.node_id.is_empty(), "Config should have a node ID");
    assert!(config.max_nodes > 0, "Config should have a valid max_nodes");

    // Test configuration with environment variables
    std::env::set_var("SONGBIRD_FEDERATION_NODE_ID", "env-test-node");
    std::env::set_var("SONGBIRD_FEDERATION_PORT", "9999");

    // Note: Current implementation doesn't read from env vars yet
    // This test documents the expected behavior for future implementation

    // Clean up
    std::env::remove_var("SONGBIRD_FEDERATION_NODE_ID");
    std::env::remove_var("SONGBIRD_FEDERATION_PORT");

    Ok(())
}

/// Test federation service integration patterns
#[tokio::test]
async fn test_federation_service_integration() -> SongbirdResult<()> {
    let config = CanonicalFederationConfig::default();

    // Test that all core services can be created together
    let _manager = CanonicalFederationManager::new(config.clone()).await?;
    let discovery = CanonicalDiscovery::new(config.clone()).await?;
    let _health_monitor = CanonicalHealthMonitor::new(config.clone()).await?;

    // Verify services were created successfully
    // Note: Internal node details are private, so we test creation success

    // Test service coordination (simplified for canonical patterns)
    let test_info = DiscoveryInfo::new(
        "integration-test-node".to_string(),
        "127.0.0.1:8082".to_string(),
    );

    let is_valid = discovery.validate_node(&test_info).await?;
    assert!(is_valid, "Integration test node should be valid");

    Ok(())
}

/// Test error handling patterns in federation operations
#[tokio::test]
async fn test_federation_error_handling() -> SongbirdResult<()> {
    // Test invalid discovery info handling
    let config = CanonicalFederationConfig::default();
    let discovery = CanonicalDiscovery::new(config).await?;

    let invalid_info = DiscoveryInfo::new(
        "".to_string(), // Invalid empty node ID
        "127.0.0.1:8083".to_string(),
    );

    // This should return false for invalid node rather than error
    let is_valid = discovery.validate_node(&invalid_info).await?;
    assert!(!is_valid, "Invalid node should not be valid");

    Ok(())
}

/// Test performance characteristics of federation operations
#[tokio::test]
async fn test_federation_performance() -> SongbirdResult<()> {
    let start_time = std::time::Instant::now();

    // Test rapid service creation
    let config = CanonicalFederationConfig::default();
    let _manager = CanonicalFederationManager::new(config.clone()).await?;
    let _discovery = CanonicalDiscovery::new(config.clone()).await?;
    let _health_monitor = CanonicalHealthMonitor::new(config).await?;

    let creation_time = start_time.elapsed();

    // Federation services should initialize quickly (< 100ms)
    assert!(
        creation_time < Duration::from_millis(100),
        "Federation services should initialize quickly, took: {creation_time:?}"
    );

    Ok(())
}

/// Test concurrent federation operations
#[tokio::test]
async fn test_concurrent_federation_operations() -> SongbirdResult<()> {
    let config = CanonicalFederationConfig::default();

    // Test concurrent service creation
    let (manager_result, discovery_result, health_result) = tokio::join!(
        CanonicalFederationManager::new(config.clone()),
        CanonicalDiscovery::new(config.clone()),
        CanonicalHealthMonitor::new(config)
    );

    // All services should create successfully in parallel
    let _manager = manager_result?;
    let _discovery = discovery_result?;
    let _health_monitor = health_result?;

    Ok(())
}
