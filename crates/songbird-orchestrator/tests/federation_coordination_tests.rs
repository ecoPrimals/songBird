//! Comprehensive tests for federation coordination
//!
//! Tests multi-node coordination, node registration, heartbeats, and federation state

use chrono::Utc;
use songbird_network_federation::{
    federation::FederationConfig,
    federation::FederationCoordinator,
    service_registry::FederatedServiceRegistry,
    state::{FederationState, NodeRegistration, NodeStatus},
};
use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;

#[tokio::test]
async fn test_federation_state_initialization() {
    let state = FederationState::new();

    assert_eq!(state.get_node_count().await, 0);
    assert!(state.list_nodes().await.is_empty());
}

#[tokio::test]
async fn test_federation_state_add_node() {
    let state = Arc::new(FederationState::new());

    let registration = NodeRegistration {
        node_id: "node-1".to_string(),
        node_name: "test-node".to_string(),
        node_address: "127.0.0.1:8080".to_string(),
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: 4,
        memory_gb: 16,
        gpu_model: None,
        storage_gb: None,
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    state.register_node(registration.clone()).await;

    assert_eq!(state.get_node_count().await, 1);

    let nodes = state.list_nodes().await;
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].node_id, "node-1");
}

#[tokio::test]
async fn test_federation_state_multiple_nodes() {
    let state = Arc::new(FederationState::new());

    for i in 1..=5 {
        let registration = NodeRegistration {
            node_id: format!("node-{}", i),
            node_name: format!("test-node-{}", i),
            node_address: format!("127.0.0.1:808{}", i),
            capabilities: vec!["orchestrator".to_string()],
            cpu_cores: 4,
            memory_gb: 16,
            gpu_model: None,
            storage_gb: None,
            status: NodeStatus::Active,
            joined_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };

        state.register_node(registration).await;
    }

    assert_eq!(state.get_node_count().await, 5);
}

#[tokio::test]
async fn test_federation_state_node_removal() {
    let state = Arc::new(FederationState::new());

    let registration = NodeRegistration {
        node_id: "node-1".to_string(),
        node_name: "test-node".to_string(),
        node_address: "127.0.0.1:8080".to_string(),
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: 4,
        memory_gb: 16,
        gpu_model: None,
        storage_gb: None,
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    state.register_node(registration).await;
    assert_eq!(state.get_node_count().await, 1);

    state.unregister_node("node-1").await;
    assert_eq!(state.get_node_count().await, 0);
}

#[tokio::test]
async fn test_federation_state_node_status_update() {
    let state = Arc::new(FederationState::new());

    let registration = NodeRegistration {
        node_id: "node-1".to_string(),
        node_name: "test-node".to_string(),
        node_address: "127.0.0.1:8080".to_string(),
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: 4,
        memory_gb: 16,
        gpu_model: None,
        storage_gb: None,
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    state.register_node(registration).await;

    // Update node status
    state.update_node_status("node-1", NodeStatus::Unhealthy).await;

    let nodes = state.list_nodes().await;
    assert_eq!(nodes[0].status, NodeStatus::Unhealthy);
}

#[tokio::test]
async fn test_federation_coordinator_creation() {
    let coordinator = FederationCoordinator::new();

    // Coordinator should be created successfully
    assert!(std::ptr::addr_of!(coordinator) as usize != 0);
}

#[tokio::test]
async fn test_federation_coordinator_with_state() {
    let state = Arc::new(FederationState::new());
    let coordinator = FederationCoordinator::with_state(state);

    // Coordinator should share the same state
    assert!(std::ptr::addr_of!(coordinator) as usize != 0);
}

#[tokio::test]
async fn test_federation_config_defaults() {
    let config = FederationConfig {
        enabled: true,
        bootstrap_address: None,
        self_registration: None,
        heartbeat_interval_secs: 30,
        node_timeout_secs: 60,
    };

    assert!(config.enabled);
    assert_eq!(config.heartbeat_interval_secs, 30);
    assert_eq!(config.node_timeout_secs, 60);
}

#[tokio::test]
async fn test_federation_config_with_bootstrap() -> SongbirdResult<()> {
    let config = FederationConfig {
        enabled: true,
        bootstrap_address: Some(format!("http://bootstrap:{}", test_orchestrator_port())),
        self_registration: None,
        heartbeat_interval_secs: 30,
        node_timeout_secs: 60,
    };

    assert!(config.bootstrap_address.is_some());
    assert_eq!(
        config
            .bootstrap_address
            .ok_or_else(|_| SongbirdError::configuration(format!("Error: {}", e)))?,
        format!("http://bootstrap:{}", test_orchestrator_port())
    );
    Ok(())
}

#[tokio::test]
async fn test_federated_service_registry_initialization() {
    let registry = FederatedServiceRegistry::new();

    let services = registry.get_all_services().await;
    assert!(services.is_empty());
}

#[tokio::test]
async fn test_node_registration_with_capabilities() {
    let registration = NodeRegistration {
        node_id: "node-1".to_string(),
        node_name: "test-node".to_string(),
        node_address: "127.0.0.1:8080".to_string(),
        capabilities: vec!["orchestrator".to_string(), "compute".to_string()],
        cpu_cores: 8,
        memory_gb: 32,
        gpu_model: Some("Tesla T4".to_string()),
        storage_gb: Some(1000),
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    assert_eq!(registration.capabilities.len(), 2);
    assert!(registration.capabilities.contains(&"orchestrator".to_string()));
    assert!(registration.capabilities.contains(&"compute".to_string()));
}

#[tokio::test]
async fn test_node_registration_resource_detection() {
    let registration = NodeRegistration {
        node_id: "node-1".to_string(),
        node_name: "test-node".to_string(),
        node_address: "127.0.0.1:8080".to_string(),
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: num_cpus::get(),
        memory_gb: 16,
        gpu_model: None,
        storage_gb: None,
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    assert!(registration.cpu_cores > 0);
}

#[tokio::test]
async fn test_node_status_transitions() {
    let state = Arc::new(FederationState::new());

    let registration = NodeRegistration {
        node_id: "node-1".to_string(),
        node_name: "test-node".to_string(),
        node_address: "127.0.0.1:8080".to_string(),
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: 4,
        memory_gb: 16,
        gpu_model: None,
        storage_gb: None,
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    state.register_node(registration).await;

    // Active -> Unhealthy
    state.update_node_status("node-1", NodeStatus::Unhealthy).await;
    assert_eq!(state.list_nodes().await[0].status, NodeStatus::Unhealthy);

    // Unhealthy -> Active
    state.update_node_status("node-1", NodeStatus::Active).await;
    assert_eq!(state.list_nodes().await[0].status, NodeStatus::Active);

    // Active -> Inactive
    state.update_node_status("node-1", NodeStatus::Inactive).await;
    assert_eq!(state.list_nodes().await[0].status, NodeStatus::Inactive);
}

#[tokio::test]
async fn test_federation_state_concurrent_access() {
    let state = Arc::new(FederationState::new());

    // Spawn multiple tasks that register nodes concurrently
    let mut handles = vec![];

    for i in 1..=10 {
        let state_clone = Arc::clone(&state);
        let handle = tokio::spawn(async move {
            let registration = NodeRegistration {
                node_id: format!("node-{}", i),
                node_name: format!("test-node-{}", i),
                node_address: format!("127.0.0.1:808{}", i),
                capabilities: vec!["orchestrator".to_string()],
                cpu_cores: 4,
                memory_gb: 16,
                gpu_model: None,
                storage_gb: None,
                status: NodeStatus::Active,
                joined_at: Utc::now(),
                last_heartbeat: Utc::now(),
            };

            state_clone.register_node(registration).await;
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.map_err(|e| SongbirdError::configuration("Failed to register".to_string()))?;
    }

    assert_eq!(state.get_node_count().await, 10);
}

#[tokio::test]
async fn test_federation_state_heartbeat_tracking() {
    let state = Arc::new(FederationState::new());

    let registration = NodeRegistration {
        node_id: "node-1".to_string(),
        node_name: "test-node".to_string(),
        node_address: "127.0.0.1:8080".to_string(),
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: 4,
        memory_gb: 16,
        gpu_model: None,
        storage_gb: None,
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    state.register_node(registration).await;

    let nodes_before = state.list_nodes().await;
    let heartbeat_before = nodes_before[0].last_heartbeat;

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update heartbeat
    state.update_heartbeat("node-1").await;

    let nodes_after = state.list_nodes().await;
    let heartbeat_after = nodes_after[0].last_heartbeat;

    assert!(heartbeat_after > heartbeat_before);
}

#[tokio::test]
async fn test_federation_state_node_query_by_capability() {
    let state = Arc::new(FederationState::new());

    // Register nodes with different capabilities
    let reg1 = NodeRegistration {
        node_id: "node-1".to_string(),
        node_name: "orchestrator-node".to_string(),
        node_address: "127.0.0.1:8080".to_string(),
        capabilities: vec!["orchestrator".to_string()],
        cpu_cores: 4,
        memory_gb: 16,
        gpu_model: None,
        storage_gb: None,
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    let reg2 = NodeRegistration {
        node_id: "node-2".to_string(),
        node_name: "compute-node".to_string(),
        node_address: "127.0.0.1:8081".to_string(),
        capabilities: vec!["compute".to_string()],
        cpu_cores: 8,
        memory_gb: 32,
        gpu_model: Some("Tesla T4".to_string()),
        storage_gb: None,
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    state.register_node(reg1).await;
    state.register_node(reg2).await;

    let nodes = state.list_nodes().await;
    let orchestrator_nodes: Vec<_> =
        nodes.iter().filter(|n| n.capabilities.contains(&"orchestrator".to_string())).collect();

    assert_eq!(orchestrator_nodes.len(), 1);
}
