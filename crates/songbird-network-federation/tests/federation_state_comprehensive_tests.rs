//! Comprehensive Federation State Tests
#![allow(clippy::similar_names)]
//!
//! Tests for federation state management, node registration, health checking, and statistics

use chrono::Utc;
use songbird_network_federation::state::{FederationState, NodeRegistration, NodeStatus};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};

fn create_test_node(id: &str, name: &str) -> NodeRegistration {
    NodeRegistration {
        node_id: id.to_string(),
        node_name: name.to_string(),
        node_address: format!("{}:8080", id),
        cpu_cores: 4,
        memory_gb: 8,
        gpu_model: None,
        storage_gb: Some(100),
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    }
}

#[tokio::test]
async fn test_federation_state_creation() {
    let state = FederationState::new();
    assert!(state.federation_id.to_string().len() > 0);
}

#[tokio::test]
async fn test_federation_state_default() {
    let state = FederationState::default();
    assert!(state.federation_id.to_string().len() > 0);
}

#[tokio::test]
async fn test_register_single_node() {
    let state = FederationState::new();
    let node = create_test_node("node-1", "Test Node 1");

    state.register_node(node.clone()).await;

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 1);
    assert!(nodes.contains_key("node-1"));
}

#[tokio::test]
async fn test_register_multiple_nodes() {
    let state = FederationState::new();

    for i in 1..=5 {
        let node = create_test_node(&format!("node-{}", i), &format!("Test Node {}", i));
        state.register_node(node).await;
    }

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 5);
}

#[tokio::test]
async fn test_remove_node() {
    let state = FederationState::new();
    let node = create_test_node("node-1", "Test Node 1");

    state.register_node(node).await;
    state.remove_node("node-1").await;

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 0);
}

#[tokio::test]
async fn test_remove_nonexistent_node() {
    let state = FederationState::new();

    // Should not panic
    state.remove_node("nonexistent").await;

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 0);
}

#[tokio::test]
async fn test_update_heartbeat() {
    let state = FederationState::new();
    let mut node = create_test_node("node-1", "Test Node 1");
    node.status = NodeStatus::Inactive;

    state.register_node(node).await;

    // Small delay to ensure heartbeat time difference
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    state.update_heartbeat("node-1").await;

    let nodes = state.nodes.read().await;
    let updated_node = nodes.get("node-1").unwrap();
    assert_eq!(updated_node.status, NodeStatus::Active);
}

#[tokio::test]
async fn test_update_heartbeat_nonexistent_node() {
    let state = FederationState::new();

    // Should not panic
    state.update_heartbeat("nonexistent").await;
}

#[tokio::test]
async fn test_update_node_status() {
    let state = FederationState::new();
    let node = create_test_node("node-1", "Test Node 1");

    state.register_node(node).await;
    state.update_node_status("node-1", NodeStatus::Unhealthy).await;

    let nodes = state.nodes.read().await;
    let updated_node = nodes.get("node-1").unwrap();
    assert_eq!(updated_node.status, NodeStatus::Unhealthy);
}

#[tokio::test]
async fn test_node_status_display() {
    assert_eq!(NodeStatus::Active.to_string(), "active");
    assert_eq!(NodeStatus::Inactive.to_string(), "inactive");
    assert_eq!(NodeStatus::Unhealthy.to_string(), "unhealthy");
}

#[tokio::test]
async fn test_node_status_equality() {
    assert_eq!(NodeStatus::Active, NodeStatus::Active);
    assert_ne!(NodeStatus::Active, NodeStatus::Inactive);
    assert_ne!(NodeStatus::Active, NodeStatus::Unhealthy);
}

#[tokio::test]
async fn test_node_status_clone() {
    let status = NodeStatus::Active;
    let cloned = status;
    assert_eq!(status, cloned);
}

#[tokio::test]
async fn test_get_node_count() {
    let state = FederationState::new();

    for i in 1..=10 {
        let node = create_test_node(&format!("node-{}", i), &format!("Node {}", i));
        state.register_node(node).await;
    }

    let count = state.get_node_count().await;
    assert_eq!(count, 10);
}

#[tokio::test]
async fn test_get_active_node_count() {
    let state = FederationState::new();

    // Register 5 active nodes
    for i in 1..=5 {
        let node = create_test_node(&format!("node-{}", i), &format!("Node {}", i));
        state.register_node(node).await;
    }

    // Register 3 inactive nodes
    for i in 6..=8 {
        let mut node = create_test_node(&format!("node-{}", i), &format!("Node {}", i));
        node.status = NodeStatus::Inactive;
        state.register_node(node).await;
    }

    let count = state.active_nodes().await.len();
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_get_all_nodes() {
    let state = FederationState::new();

    for i in 1..=3 {
        let node = create_test_node(&format!("node-{}", i), &format!("Node {}", i));
        state.register_node(node).await;
    }

    let all_nodes = state.list_nodes().await;
    assert_eq!(all_nodes.len(), 3);
}

#[tokio::test]
async fn test_get_active_nodes() {
    let state = FederationState::new();

    // Active nodes
    for i in 1..=3 {
        let node = create_test_node(&format!("node-{}", i), &format!("Node {}", i));
        state.register_node(node).await;
    }

    // Inactive node
    let mut inactive_node = create_test_node("node-4", "Inactive Node");
    inactive_node.status = NodeStatus::Inactive;
    state.register_node(inactive_node).await;

    let active_nodes = state.active_nodes().await;
    assert_eq!(active_nodes.len(), 3);
}

#[tokio::test]
async fn test_get_stats() {
    let state = FederationState::new();

    // Register nodes with varying resources
    let mut node1 = create_test_node("node-1", "Node 1");
    node1.cpu_cores = 8;
    node1.memory_gb = 16;
    node1.storage_gb = Some(500);
    state.register_node(node1).await;

    let mut node2 = create_test_node("node-2", "Node 2");
    node2.cpu_cores = 4;
    node2.memory_gb = 8;
    node2.storage_gb = Some(250);
    state.register_node(node2).await;

    let stats = state.get_stats().await;
    assert_eq!(stats.total_nodes, 2);
    assert_eq!(stats.active_nodes, 2);
    assert_eq!(stats.total_cpu_cores, 12);
    assert_eq!(stats.total_memory_gb, 24);
    assert_eq!(stats.total_storage_gb, 750);
}

#[tokio::test]
async fn test_statistics_with_inactive_nodes() {
    let state = FederationState::new();

    // Active node
    let mut active = create_test_node("active", "Active Node");
    active.cpu_cores = 8;
    active.memory_gb = 16;
    state.register_node(active).await;

    // Inactive node
    let mut inactive = create_test_node("inactive", "Inactive Node");
    inactive.cpu_cores = 4;
    inactive.memory_gb = 8;
    inactive.status = NodeStatus::Inactive;
    state.register_node(inactive).await;

    let stats = state.get_stats().await;
    assert_eq!(stats.total_nodes, 2);
    assert_eq!(stats.active_nodes, 1);
    // Only active node resources counted
    assert_eq!(stats.total_cpu_cores, 8);
    assert_eq!(stats.total_memory_gb, 16);
}

#[tokio::test]
async fn test_node_registration_with_gpu() {
    let state = FederationState::new();
    let mut node = create_test_node("gpu-node", "GPU Node");
    node.gpu_model = Some("NVIDIA RTX 4090".to_string());

    state.register_node(node.clone()).await;

    let nodes = state.nodes.read().await;
    let registered = nodes.get("gpu-node").unwrap();
    assert_eq!(registered.gpu_model, Some("NVIDIA RTX 4090".to_string()));
}

#[tokio::test]
async fn test_node_registration_without_storage() {
    let state = FederationState::new();
    let mut node = create_test_node("no-storage", "No Storage Node");
    node.storage_gb = None;

    state.register_node(node).await;

    let nodes = state.nodes.read().await;
    let registered = nodes.get("no-storage").unwrap();
    assert_eq!(registered.storage_gb, None);
}

#[tokio::test]
async fn test_node_capabilities() {
    let state = FederationState::new();
    let mut node = create_test_node("cap-node", "Capabilities Node");
    node.capabilities = vec![
        "compute".to_string(),
        "storage".to_string(),
        "networking".to_string(),
        "ai-inference".to_string(),
    ];

    state.register_node(node).await;

    let nodes = state.nodes.read().await;
    let registered = nodes.get("cap-node").unwrap();
    assert_eq!(registered.capabilities.len(), 4);
    assert!(registered.capabilities.contains(&"ai-inference".to_string()));
}

#[tokio::test]
async fn test_node_address_formats() {
    let state = FederationState::new();

    let addresses =
        vec!["192.168.1.100:8080", "node.example.com:9000", "[::1]:8080", "localhost:3000"];

    for (i, addr) in addresses.iter().enumerate() {
        let mut node = create_test_node(&format!("node-{}", i), &format!("Node {}", i));
        node.node_address = addr.to_string();
        state.register_node(node).await;
    }

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 4);
}

#[tokio::test]
async fn test_concurrent_node_registration() {
    let state = FederationState::new();

    let handles: Vec<_> = (1..=20)
        .map(|i| {
            let state_clone = state.clone();
            tokio::spawn(async move {
                let node = create_test_node(&format!("node-{}", i), &format!("Node {}", i));
                state_clone.register_node(node).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    let count = state.get_node_count().await;
    assert_eq!(count, 20);
}

#[tokio::test]
async fn test_concurrent_heartbeat_updates() {
    let state = FederationState::new();

    // Register nodes first
    for i in 1..=10 {
        let node = create_test_node(&format!("node-{}", i), &format!("Node {}", i));
        state.register_node(node).await;
    }

    let handles: Vec<_> = (1..=10)
        .map(|i| {
            let state_clone = state.clone();
            tokio::spawn(async move {
                state_clone.update_heartbeat(&format!("node-{}", i)).await;
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    let active_count = state.active_nodes().await.len();
    assert_eq!(active_count, 10);
}

#[tokio::test]
async fn test_update_same_node_multiple_times() {
    let state = FederationState::new();
    let node = create_test_node("node-1", "Test Node");

    state.register_node(node.clone()).await;

    // Update the same node multiple times
    let mut current_cores = node.cpu_cores;
    for _ in 0..5 {
        current_cores += 2;
        let mut updated_node = node.clone();
        updated_node.cpu_cores = current_cores;
        state.register_node(updated_node).await;
    }

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 1); // Still just one node
    let final_node = nodes.get("node-1").unwrap();
    assert_eq!(final_node.cpu_cores, 14); // 4 + (2 * 5)
}

#[tokio::test]
async fn test_federation_state_with_empty_capabilities() {
    let state = FederationState::new();
    let mut node = create_test_node("minimal", "Minimal Node");
    node.capabilities = vec![];

    state.register_node(node).await;

    let nodes = state.nodes.read().await;
    let registered = nodes.get("minimal").unwrap();
    assert_eq!(registered.capabilities.len(), 0);
}

#[tokio::test]
async fn test_federation_state_with_large_resources() {
    let state = FederationState::new();
    let mut node = create_test_node("large", "Large Node");
    node.cpu_cores = 128;
    node.memory_gb = 1024;
    node.storage_gb = Some(10_000);

    state.register_node(node).await;

    let stats = state.get_stats().await;
    assert_eq!(stats.total_cpu_cores, 128);
    assert_eq!(stats.total_memory_gb, 1024);
    assert_eq!(stats.total_storage_gb, 10_000);
}

#[tokio::test]
async fn test_node_registration_preserves_timestamps() {
    let state = FederationState::new();
    let node = create_test_node("node-1", "Test Node");
    let original_joined_at = node.joined_at;

    state.register_node(node).await;

    let nodes = state.nodes.read().await;
    let registered = nodes.get("node-1").unwrap();
    assert_eq!(registered.joined_at, original_joined_at);
}

#[tokio::test]
async fn test_multiple_status_transitions() {
    let state = FederationState::new();
    let node = create_test_node("node-1", "Test Node");

    state.register_node(node).await;

    // Active -> Inactive
    state.update_node_status("node-1", NodeStatus::Inactive).await;
    let nodes = state.nodes.read().await;
    assert_eq!(nodes.get("node-1").unwrap().status, NodeStatus::Inactive);
    drop(nodes);

    // Inactive -> Unhealthy
    state.update_node_status("node-1", NodeStatus::Unhealthy).await;
    let nodes = state.nodes.read().await;
    assert_eq!(nodes.get("node-1").unwrap().status, NodeStatus::Unhealthy);
    drop(nodes);

    // Unhealthy -> Active
    state.update_node_status("node-1", NodeStatus::Active).await;
    let nodes = state.nodes.read().await;
    assert_eq!(nodes.get("node-1").unwrap().status, NodeStatus::Active);
}

#[tokio::test]
async fn test_federation_id_uniqueness() {
    let state1 = FederationState::new();
    let state2 = FederationState::new();

    assert_ne!(state1.federation_id, state2.federation_id);
}

#[tokio::test]
async fn test_federation_created_at_timestamp() {
    let before = Utc::now();
    let state = FederationState::new();
    let after = Utc::now();

    assert!(state.created_at >= before);
    assert!(state.created_at <= after);
}

#[tokio::test]
async fn test_node_with_special_characters_in_name() {
    let state = FederationState::new();
    let node = create_test_node("node-1", "Test Node™ (Prod) [EU]");

    state.register_node(node).await;

    let nodes = state.nodes.read().await;
    let registered = nodes.get("node-1").unwrap();
    assert_eq!(registered.node_name, "Test Node™ (Prod) [EU]");
}

#[tokio::test]
async fn test_remove_and_re_register_node() {
    let state = FederationState::new();
    let node1 = create_test_node("node-1", "First Registration");

    state.register_node(node1).await;
    state.remove_node("node-1").await;

    let node2 = create_test_node("node-1", "Second Registration");
    state.register_node(node2).await;

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes.get("node-1").unwrap().node_name, "Second Registration");
}

#[tokio::test]
async fn test_statistics_with_no_nodes() -> SongbirdResult<()> {
    let state = FederationState::new();
    let stats = state.get_stats().await;

    assert_eq!(stats.total_nodes, 0);
    assert_eq!(stats.active_nodes, 0);
    assert_eq!(stats.total_cpu_cores, 0);
    assert_eq!(stats.total_memory_gb, 0);
    assert_eq!(stats.total_storage_gb, 0);
    Ok(())
}

#[tokio::test]
async fn test_get_active_nodes_with_all_inactive() -> SongbirdResult<()> {
    let state = FederationState::new();

    for i in 1..=5 {
        let mut node = create_test_node(&format!("node-{}", i), &format!("Node {}", i));
        node.status = NodeStatus::Inactive;
        state.register_node(node).await;
    }

    let active_nodes = state.active_nodes().await;
    assert_eq!(active_nodes.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_node_status_debug_format() -> SongbirdResult<()> {
    let status = NodeStatus::Active;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Active"));
    Ok(())
}

#[tokio::test]
async fn test_node_registration_debug_format() -> SongbirdResult<()> {
    let node = create_test_node("node-1", "Test Node");
    let debug_str = format!("{:?}", node);
    assert!(debug_str.contains("node-1"));
    assert!(debug_str.contains("Test Node"));
    Ok(())
}

#[tokio::test]
async fn test_federation_state_debug_format() -> SongbirdResult<()> {
    let state = FederationState::new();
    let debug_str = format!("{:?}", state);
    assert!(!debug_str.is_empty());
    Ok(())
}
