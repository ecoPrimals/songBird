// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use chrono::Utc;

#[tokio::test]
async fn test_federation_state_creation() {
    let state = FederationState::new(String::from("test"));
    assert_eq!(state.nodes.read().await.len(), 0);
}

#[tokio::test]
async fn test_node_registration() {
    let state = FederationState::new(String::from("test"));

    let registration = NodeRegistration {
        node_id: String::from("test-node"),
        node_name: String::from("Test Node"),
        node_address: String::from("192.168.1.100:8080"),
        endpoints: None,
        cpu_cores: 8,
        memory_gb: 16,
        gpu_model: Some(String::from("RTX 3070")),
        storage_gb: Some(500),
        capabilities: vec![String::from("compute")],
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    };

    state.register_node(registration.clone()).await;

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 1);
    assert!(nodes.contains_key("test-node"));
}

#[tokio::test]
async fn test_heartbeat_update() {
    let state = FederationState::new(String::from("test"));

    let registration = NodeRegistration {
        node_id: String::from("test-node"),
        node_name: String::from("Test Node"),
        node_address: String::from("192.168.1.100:8080"),
        endpoints: None,
        cpu_cores: 8,
        memory_gb: 16,
        gpu_model: None,
        storage_gb: None,
        capabilities: vec![],
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now() - chrono::Duration::seconds(100),
    };

    state.register_node(registration).await;
    state.update_heartbeat("test-node").await;

    let nodes = state.nodes.read().await;
    let node = nodes.get("test-node").unwrap();

    let elapsed = (Utc::now() - node.last_heartbeat).num_seconds();
    assert!(elapsed < 5); // Should be very recent
}

#[tokio::test]
async fn register_node_merges_capabilities() {
    let state = FederationState::new(String::from("x"));
    let mut r1 = make_registration("n", "addr1");
    r1.capabilities = vec![String::from("a")];
    state.register_node(r1).await;
    let mut r2 = make_registration("n", "addr1");
    r2.capabilities = vec![String::from("b")];
    state.register_node(r2).await;
    let nodes = state.nodes.read().await;
    let n = nodes.get("n").unwrap();
    assert!(n.capabilities.contains(&String::from("a")));
    assert!(n.capabilities.contains(&String::from("b")));
}

fn make_registration(id: &str, addr: &str) -> NodeRegistration {
    NodeRegistration {
        node_id: id.into(),
        node_name: id.into(),
        node_address: addr.into(),
        endpoints: None,
        cpu_cores: 1,
        memory_gb: 1,
        gpu_model: None,
        storage_gb: None,
        capabilities: vec![],
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    }
}

#[test]
fn transport_endpoint_preference_sorts_in_add_endpoint() {
    let mut reg = make_registration_sync("n", "a");
    reg.add_endpoint(TransportEndpointInfo {
        interface_type: String::from("e"),
        address: String::from("192.168.1.1:1"),
        protocols: vec![],
        preference: 10,
        status: EndpointStatus::Active,
        last_check: Utc::now(),
    });
    reg.add_endpoint(TransportEndpointInfo {
        interface_type: String::from("e"),
        address: String::from("192.168.1.2:2"),
        protocols: vec![],
        preference: 200,
        status: EndpointStatus::Active,
        last_check: Utc::now(),
    });
    let pref = reg.preferred_endpoint().unwrap();
    assert_eq!(pref.address, "192.168.1.2:2");
}

fn make_registration_sync(id: &str, addr: &str) -> NodeRegistration {
    NodeRegistration {
        node_id: id.into(),
        node_name: id.into(),
        node_address: addr.into(),
        endpoints: None,
        cpu_cores: 0,
        memory_gb: 0,
        gpu_model: None,
        storage_gb: None,
        capabilities: vec![],
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    }
}

#[test]
fn federation_stats_serde_roundtrip() {
    let s = FederationStats {
        total_nodes: 3,
        active_nodes: 2,
        total_cpu_cores: 4,
        total_memory_gb: 8,
        total_storage_gb: 16,
        uptime_seconds: Some(42),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: FederationStats = serde_json::from_str(&json).unwrap();
    assert_eq!(back.active_nodes, 2);
}

#[tokio::test]
async fn cleanup_stale_nodes_removes_old() {
    let state = FederationState::new(String::from("x"));
    let mut r = make_registration("old", "a");
    r.last_heartbeat = Utc::now() - chrono::Duration::seconds(9999);
    state.register_node(r).await;
    let n = state.cleanup_stale_nodes(100).await;
    assert_eq!(n, 1);
    assert_eq!(state.nodes.read().await.len(), 0);
}

#[tokio::test]
async fn federation_state_default_creates_empty() {
    let state = FederationState::default();
    assert_eq!(state.nodes.read().await.len(), 0);
}

#[test]
fn node_status_display() {
    assert_eq!(NodeStatus::Active.to_string(), "active");
    assert_eq!(NodeStatus::Inactive.to_string(), "inactive");
}

#[tokio::test]
async fn get_stats_aggregates_resources() {
    let state = FederationState::new(String::from("fed"));
    let mut r = make_registration("n1", "https://a:1");
    r.cpu_cores = 4;
    r.memory_gb = 8;
    r.storage_gb = Some(100);
    state.register_node(r).await;
    let stats = state.get_stats().await;
    assert_eq!(stats.total_nodes, 1);
    assert_eq!(stats.active_nodes, 1);
    assert_eq!(stats.total_cpu_cores, 4);
    assert_eq!(stats.total_memory_gb, 8);
    assert_eq!(stats.total_storage_gb, 100);
}

#[tokio::test]
async fn get_best_endpoint_prefers_https_wrapped_preferred() {
    let state = FederationState::new(String::from("test"));
    let mut r = make_registration("node-x", "https://primary:443");
    r.endpoints = Some(vec![TransportEndpointInfo {
        interface_type: String::from("eth"),
        address: String::from("10.0.0.5:8443"),
        protocols: vec![String::from("https")],
        preference: 200,
        status: EndpointStatus::Active,
        last_check: Utc::now(),
    }]);
    state.register_node(r).await;
    let best = state.get_best_endpoint("node-x").await.unwrap();
    assert!(best.contains("10.0.0.5:8443"));
}

#[tokio::test]
async fn get_all_endpoints_includes_primary() {
    let state = FederationState::new(String::from("test"));
    let mut r = make_registration("n", "https://only:1");
    r.endpoints = Some(vec![TransportEndpointInfo {
        interface_type: String::from("eth"),
        address: String::from("192.168.1.1:1"),
        protocols: vec![],
        preference: 10,
        status: EndpointStatus::Active,
        last_check: Utc::now(),
    }]);
    state.register_node(r).await;
    let eps = state.get_all_endpoints("n").await;
    assert!(eps.iter().any(|e| e.contains("192.168.1.1")));
}

#[test]
fn federation_status_serde_roundtrip() {
    let s = FederationStatus {
        federation_id: String::from("fid"),
        active_nodes: 1,
        nodes: vec![],
        total_cpu_cores: 2,
        total_memory_gb: 4,
        total_storage_gb: 8,
        uptime_seconds: 60,
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: FederationStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back.active_nodes, 1);
}

#[test]
fn active_endpoints_returns_only_active() {
    let mut reg = make_registration_sync("n", "a");
    reg.endpoints = Some(vec![
        TransportEndpointInfo {
            interface_type: String::from("eth"),
            address: String::from("10.0.0.1:1"),
            protocols: vec![],
            preference: 100,
            status: EndpointStatus::Active,
            last_check: Utc::now(),
        },
        TransportEndpointInfo {
            interface_type: String::from("eth"),
            address: String::from("10.0.0.2:2"),
            protocols: vec![],
            preference: 50,
            status: EndpointStatus::Degraded,
            last_check: Utc::now(),
        },
        TransportEndpointInfo {
            interface_type: String::from("wifi"),
            address: String::from("10.0.0.3:3"),
            protocols: vec![],
            preference: 200,
            status: EndpointStatus::Failed,
            last_check: Utc::now(),
        },
        TransportEndpointInfo {
            interface_type: String::from("eth"),
            address: String::from("10.0.0.4:4"),
            protocols: vec![],
            preference: 80,
            status: EndpointStatus::Active,
            last_check: Utc::now(),
        },
    ]);
    let active = reg.active_endpoints();
    assert_eq!(active.len(), 2);
    assert!(active.iter().all(|e| e.status == EndpointStatus::Active));
}

#[test]
fn active_endpoints_empty_when_none() {
    let reg = make_registration_sync("n", "a");
    assert!(reg.active_endpoints().is_empty());
}

#[test]
fn active_endpoints_empty_when_all_failed() {
    let mut reg = make_registration_sync("n", "a");
    reg.endpoints = Some(vec![
        TransportEndpointInfo {
            interface_type: String::from("eth"),
            address: String::from("10.0.0.1:1"),
            protocols: vec![],
            preference: 100,
            status: EndpointStatus::Failed,
            last_check: Utc::now(),
        },
        TransportEndpointInfo {
            interface_type: String::from("eth"),
            address: String::from("10.0.0.2:2"),
            protocols: vec![],
            preference: 50,
            status: EndpointStatus::Standby,
            last_check: Utc::now(),
        },
    ]);
    assert!(reg.active_endpoints().is_empty());
}

#[test]
fn update_endpoint_status_changes_matching_address() {
    let mut reg = make_registration_sync("n", "a");
    reg.endpoints = Some(vec![
        TransportEndpointInfo {
            interface_type: String::from("eth"),
            address: String::from("10.0.0.1:1"),
            protocols: vec![],
            preference: 100,
            status: EndpointStatus::Active,
            last_check: Utc::now() - chrono::Duration::seconds(60),
        },
        TransportEndpointInfo {
            interface_type: String::from("eth"),
            address: String::from("10.0.0.2:2"),
            protocols: vec![],
            preference: 50,
            status: EndpointStatus::Active,
            last_check: Utc::now() - chrono::Duration::seconds(60),
        },
    ]);
    reg.update_endpoint_status("10.0.0.1:1", EndpointStatus::Failed);
    let eps = reg.endpoints.as_ref().unwrap();
    assert_eq!(eps[0].status, EndpointStatus::Failed);
    assert_eq!(eps[1].status, EndpointStatus::Active);
    let elapsed = (Utc::now() - eps[0].last_check).num_seconds();
    assert!(elapsed < 2, "last_check should be updated to now");
}

#[test]
fn update_endpoint_status_no_match_is_noop() {
    let mut reg = make_registration_sync("n", "a");
    reg.endpoints = Some(vec![TransportEndpointInfo {
        interface_type: String::from("eth"),
        address: String::from("10.0.0.1:1"),
        protocols: vec![],
        preference: 100,
        status: EndpointStatus::Active,
        last_check: Utc::now(),
    }]);
    reg.update_endpoint_status("nonexistent:999", EndpointStatus::Failed);
    assert_eq!(reg.endpoints.as_ref().unwrap()[0].status, EndpointStatus::Active);
}

#[test]
fn update_endpoint_status_with_no_endpoints_is_noop() {
    let mut reg = make_registration_sync("n", "a");
    reg.update_endpoint_status("10.0.0.1:1", EndpointStatus::Failed);
    assert!(reg.endpoints.is_none());
}

#[test]
fn preferred_endpoint_none_when_all_degraded() {
    let mut reg = make_registration_sync("n", "a");
    reg.endpoints = Some(vec![
        TransportEndpointInfo {
            interface_type: String::from("eth"),
            address: String::from("10.0.0.1:1"),
            protocols: vec![],
            preference: 200,
            status: EndpointStatus::Degraded,
            last_check: Utc::now(),
        },
        TransportEndpointInfo {
            interface_type: String::from("eth"),
            address: String::from("10.0.0.2:2"),
            protocols: vec![],
            preference: 100,
            status: EndpointStatus::Standby,
            last_check: Utc::now(),
        },
    ]);
    assert!(reg.preferred_endpoint().is_none());
}

#[test]
fn preferred_endpoint_none_when_no_endpoints() {
    let reg = make_registration_sync("n", "a");
    assert!(reg.preferred_endpoint().is_none());
}

#[test]
fn add_endpoint_replaces_existing_by_address() {
    let mut reg = make_registration_sync("n", "a");
    reg.add_endpoint(TransportEndpointInfo {
        interface_type: String::from("eth"),
        address: String::from("10.0.0.1:1"),
        protocols: vec![String::from("http")],
        preference: 50,
        status: EndpointStatus::Active,
        last_check: Utc::now(),
    });
    reg.add_endpoint(TransportEndpointInfo {
        interface_type: String::from("wifi"),
        address: String::from("10.0.0.1:1"),
        protocols: vec![String::from("https")],
        preference: 200,
        status: EndpointStatus::Active,
        last_check: Utc::now(),
    });
    let eps = reg.endpoints.as_ref().unwrap();
    assert_eq!(eps.len(), 1, "should deduplicate by address");
    assert_eq!(eps[0].interface_type, "wifi");
    assert_eq!(eps[0].preference, 200);
}
