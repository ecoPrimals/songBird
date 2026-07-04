// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::needless_pass_by_value,
    clippy::similar_names,
    reason = "test assertions and harness ergonomics"
)]

//! Network Federation integration tests for `FederationState`.

use chrono::Utc;
use songbird_network_federation::state::{
    EndpointStatus, FederationState, FederationStats, NodeRegistration, NodeStatus,
    TransportEndpointInfo,
};
use std::sync::Arc;

fn make_registration(id: &str, addr: &str) -> NodeRegistration {
    NodeRegistration {
        node_id: String::from(id),
        node_name: String::from(id),
        node_address: String::from(addr),
        endpoints: None,
        cpu_cores: 2,
        memory_gb: 4,
        gpu_model: None,
        storage_gb: Some(50),
        capabilities: vec![],
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    }
}

fn endpoint(
    interface_type: &str,
    address: &str,
    preference: u8,
    status: EndpointStatus,
) -> TransportEndpointInfo {
    TransportEndpointInfo {
        interface_type: String::from(interface_type),
        address: String::from(address),
        protocols: vec![],
        preference,
        status,
        last_check: Utc::now(),
    }
}

#[tokio::test]
async fn test_peer_discovery() {
    let state = FederationState::new(String::from("peer-discovery"));

    for (id, addr) in [
        ("node-a", "192.168.1.10:8080"),
        ("node-b", "192.168.1.11:8080"),
        ("node-c", "192.168.1.12:8080"),
    ] {
        state.register_node(make_registration(id, addr)).await;
    }

    let active = state.active_nodes().await;
    assert_eq!(active.len(), 3);

    let stats = state.get_stats().await;
    assert_eq!(stats.total_nodes, 3);
    assert_eq!(stats.active_nodes, 3);
    assert_eq!(stats.total_cpu_cores, 6);
    assert_eq!(stats.total_memory_gb, 12);
    assert_eq!(stats.total_storage_gb, 150);
}

#[tokio::test]
async fn test_network_partition_handling() {
    let state = FederationState::new(String::from("partition"));

    let mut alive = make_registration("alive-node", "10.0.0.1:9000");
    alive.last_heartbeat = Utc::now() - chrono::Duration::seconds(500);
    state.register_node(alive).await;

    let mut partitioned = make_registration("partitioned-node", "10.0.0.2:9000");
    partitioned.last_heartbeat = Utc::now() - chrono::Duration::seconds(500);
    state.register_node(partitioned).await;

    state.update_heartbeat("alive-node").await;

    let removed = state.cleanup_stale_nodes(100).await;
    assert_eq!(removed, 1);

    let nodes = state.nodes.read().await;
    assert!(nodes.contains_key("alive-node"));
    assert!(!nodes.contains_key("partitioned-node"));
}

#[tokio::test]
async fn test_federation_handshake() {
    let state = FederationState::new(String::from("handshake"));

    let mut registration = make_registration("mesh-relay", "10.0.0.5:443");
    registration.capabilities = vec![String::from("mesh"), String::from("relay")];
    registration.endpoints = Some(vec![
        endpoint("eth", "10.0.0.5:8443", 50, EndpointStatus::Active),
        endpoint("wifi", "10.0.0.6:8443", 200, EndpointStatus::Active),
    ]);
    state.register_node(registration).await;

    let best = state.get_best_endpoint("mesh-relay").await.unwrap();
    assert!(best.contains("10.0.0.6:8443"));

    let nodes = state.nodes.read().await;
    let node = nodes.get("mesh-relay").unwrap();
    assert!(node.capabilities.contains(&String::from("mesh")));
    assert!(node.capabilities.contains(&String::from("relay")));
}

#[tokio::test]
async fn test_cross_network_communication() {
    let state = FederationState::new(String::from("cross-network"));

    let mut registration = make_registration("multi-transport", "127.0.0.1:8080");
    registration.endpoints = Some(vec![
        endpoint("tcp", "127.0.0.1:8080", 100, EndpointStatus::Active),
        endpoint("uds", "/tmp/songbird.sock", 80, EndpointStatus::Active),
        endpoint("relay", "relay.example.com:9000", 60, EndpointStatus::Active),
    ]);
    state.register_node(registration).await;

    let endpoints = state.get_all_endpoints("multi-transport").await;
    assert!(endpoints.iter().any(|e| e.contains("127.0.0.1:8080")));
    assert!(endpoints.iter().any(|e| e.contains("/tmp/songbird.sock")));
    assert!(endpoints.iter().any(|e| e.contains("relay.example.com:9000")));

    let nodes = state.nodes.read().await;
    let node = nodes.get("multi-transport").unwrap();
    let types: Vec<_> =
        node.endpoints.as_ref().unwrap().iter().map(|e| e.interface_type.as_str()).collect();
    assert!(types.contains(&"tcp"));
    assert!(types.contains(&"uds"));
    assert!(types.contains(&"relay"));
}

#[tokio::test]
async fn test_mesh_network_formation() {
    let state = FederationState::new(String::from("mesh"));

    for i in 0..5 {
        let id = format!("mesh-node-{i}");
        let addr = format!("10.1.0.{i}:7000");
        state.register_node(make_registration(&id, &addr)).await;
    }

    assert_eq!(state.active_nodes().await.len(), 5);

    {
        let mut nodes = state.nodes.write().await;
        for id in ["mesh-node-0", "mesh-node-1"] {
            if let Some(node) = nodes.get_mut(id) {
                node.last_heartbeat = Utc::now() - chrono::Duration::seconds(9999);
            }
        }
    }

    let removed = state.cleanup_stale_nodes(100).await;
    assert_eq!(removed, 2);
    assert_eq!(state.active_nodes().await.len(), 3);
    assert_eq!(state.nodes.read().await.len(), 3);
}

#[tokio::test]
async fn test_endpoint_failover() {
    let state = FederationState::new(String::from("failover"));

    let mut registration = make_registration("failover-node", "10.0.0.1:8080");
    registration.endpoints = Some(vec![
        endpoint("eth", "10.0.0.1:8080", 200, EndpointStatus::Active),
        endpoint("wifi", "10.0.0.2:8080", 100, EndpointStatus::Active),
    ]);
    state.register_node(registration).await;

    {
        let mut nodes = state.nodes.write().await;
        nodes
            .get_mut("failover-node")
            .unwrap()
            .update_endpoint_status("10.0.0.1:8080", EndpointStatus::Degraded);
    }

    let best = state.get_best_endpoint("failover-node").await.unwrap();
    assert!(best.contains("10.0.0.2:8080"));
    assert!(!best.contains("10.0.0.1:8080"));
}

#[tokio::test]
async fn test_concurrent_node_registration() {
    let state = Arc::new(FederationState::new(String::from("concurrent")));

    let mut handles = Vec::new();
    for i in 0..10 {
        let state = Arc::clone(&state);
        handles.push(tokio::spawn(async move {
            let id = format!("concurrent-node-{i}");
            let addr = format!("10.2.0.{i}:8000");
            state.register_node(make_registration(&id, &addr)).await;
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(state.nodes.read().await.len(), 10);
    assert_eq!(state.active_nodes().await.len(), 10);
}

#[tokio::test]
async fn test_service_capability_routing() {
    let state = FederationState::new(String::from("capability-routing"));

    let mut compute = make_registration("compute-node", "10.3.0.1:9000");
    compute.capabilities = vec![String::from("compute"), String::from("gpu")];
    state.register_node(compute).await;

    let mut relay = make_registration("relay-node", "10.3.0.2:9000");
    relay.capabilities = vec![String::from("relay"), String::from("mesh")];
    state.register_node(relay).await;

    let mut storage = make_registration("storage-node", "10.3.0.3:9000");
    storage.capabilities = vec![String::from("storage")];
    state.register_node(storage).await;

    let active = state.active_nodes().await;

    let relay_nodes: Vec<_> =
        active.iter().filter(|node| node.capabilities.iter().any(|c| c == "relay")).collect();
    assert_eq!(relay_nodes.len(), 1);
    assert_eq!(relay_nodes[0].node_id, "relay-node");

    let gpu_nodes: Vec<_> =
        active.iter().filter(|node| node.capabilities.iter().any(|c| c == "gpu")).collect();
    assert_eq!(gpu_nodes.len(), 1);
    assert_eq!(gpu_nodes[0].node_id, "compute-node");

    let mesh_nodes: Vec<_> =
        active.iter().filter(|node| node.capabilities.iter().any(|c| c == "mesh")).collect();
    assert_eq!(mesh_nodes.len(), 1);
    assert_eq!(mesh_nodes[0].node_id, "relay-node");
}

#[test]
fn test_federation_state_persistence() {
    let stats = FederationStats {
        total_nodes: 5,
        active_nodes: 4,
        total_cpu_cores: 16,
        total_memory_gb: 32,
        total_storage_gb: 500,
        uptime_seconds: Some(3600),
    };

    let json = serde_json::to_string(&stats).unwrap();
    let restored: FederationStats = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.total_nodes, stats.total_nodes);
    assert_eq!(restored.active_nodes, stats.active_nodes);
    assert_eq!(restored.total_cpu_cores, stats.total_cpu_cores);
    assert_eq!(restored.total_memory_gb, stats.total_memory_gb);
    assert_eq!(restored.total_storage_gb, stats.total_storage_gb);
    assert_eq!(restored.uptime_seconds, stats.uptime_seconds);
}
