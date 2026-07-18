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

//! Comprehensive tests for federation state management
//!
//! Covers edge cases and low-coverage paths in state.rs:
//! - `FederationState`: `register_node` coalescence, remove, cleanup, stats
//! - `NodeRegistration`: endpoints, `add_endpoint`, `preferred_endpoint`, `update_endpoint_status`
//! - `FederationStats`, `FederationStatus` serde
//! - `NodeStatus` Display, `EndpointStatus` variants

use chrono::{Duration, Utc};
use songbird_network_federation::state::{
    EndpointStatus, FederationState, FederationStats, FederationStatus, NodeRegistration,
    NodeStatus, TransportEndpointInfo,
};

fn make_node(id: &str, name: &str, addr: &str) -> NodeRegistration {
    NodeRegistration {
        node_id: id.to_string(),
        node_name: name.to_string(),
        node_address: addr.to_string(),
        endpoints: None,
        cpu_cores: 4,
        memory_gb: 8,
        gpu_model: None,
        storage_gb: Some(100),
        capabilities: vec!["compute".to_string()],
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    }
}

fn make_endpoint(iface: &str, addr: &str, pref: u8) -> TransportEndpointInfo {
    TransportEndpointInfo {
        interface_type: iface.to_string(),
        address: addr.to_string(),
        protocols: vec!["https".to_string()],
        preference: pref,
        status: EndpointStatus::Active,
        last_check: Utc::now(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FederationState tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_federation_state_default() {
    let state = FederationState::default();
    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 0);
}

#[tokio::test]
async fn test_federation_state_register_and_remove() {
    let state = FederationState::new("test-fed".to_string());
    let node = make_node("n1", "Node 1", "10.0.0.1:8080");
    state.register_node(node).await;

    assert_eq!(state.nodes.read().await.len(), 1);

    state.remove_node("n1").await;
    assert_eq!(state.nodes.read().await.len(), 0);
}

#[tokio::test]
async fn test_federation_state_remove_nonexistent() {
    let state = FederationState::default();
    state.remove_node("nonexistent").await; // No panic
    assert_eq!(state.nodes.read().await.len(), 0);
}

#[tokio::test]
async fn test_federation_state_register_node_coalescence() {
    let state = FederationState::new("test-fed".to_string());

    // Register initial node
    let node = make_node("n1", "Node 1", "10.0.0.1:8080");
    state.register_node(node).await;

    // Re-register same node_id with new endpoints and capabilities
    let mut updated = make_node("n1", "Node 1", "10.0.0.2:8080");
    updated.endpoints = Some(vec![make_endpoint("wifi", "10.0.0.3:8080", 50)]);
    updated.capabilities = vec!["storage".to_string()];
    state.register_node(updated).await;

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 1); // Coalesced, not duplicated

    let n = nodes.get("n1").unwrap();
    assert_eq!(n.node_address, "10.0.0.2:8080"); // Updated
    assert!(n.capabilities.contains(&"compute".to_string())); // Original preserved
    assert!(n.capabilities.contains(&"storage".to_string())); // New added
    assert!(n.endpoints.is_some());
    assert_eq!(n.endpoints.as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn test_federation_state_update_heartbeat_existing() {
    let state = FederationState::new("test-fed".to_string());

    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    node.last_heartbeat = Utc::now() - Duration::seconds(600);
    state.register_node(node).await;

    state.update_heartbeat("n1").await;

    let nodes = state.nodes.read().await;
    let n = nodes.get("n1").unwrap();
    let elapsed = (Utc::now() - n.last_heartbeat).num_seconds();
    assert!(elapsed < 2);
}

#[tokio::test]
async fn test_federation_state_update_heartbeat_nonexistent() {
    let state = FederationState::default();
    state.update_heartbeat("nonexistent").await; // No panic
}

#[tokio::test]
async fn test_federation_state_check_node_health() {
    let state = FederationState::new("test-fed".to_string());

    let mut stale_node = make_node("stale", "Stale", "10.0.0.1:8080");
    stale_node.last_heartbeat = Utc::now() - Duration::seconds(120);
    state.register_node(stale_node).await;

    let fresh_node = make_node("fresh", "Fresh", "10.0.0.2:8080");
    state.register_node(fresh_node).await;

    state.check_node_health(60).await;

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.get("stale").unwrap().status, NodeStatus::Inactive);
    assert_eq!(nodes.get("fresh").unwrap().status, NodeStatus::Active);
}

#[tokio::test]
async fn test_federation_state_cleanup_stale_nodes() {
    let state = FederationState::new("test-fed".to_string());

    let mut stale_node = make_node("stale", "Stale", "10.0.0.1:8080");
    stale_node.last_heartbeat = Utc::now() - Duration::seconds(1200);
    state.register_node(stale_node).await;

    let fresh_node = make_node("fresh", "Fresh", "10.0.0.2:8080");
    state.register_node(fresh_node).await;

    let removed = state.cleanup_stale_nodes(600).await;
    assert_eq!(removed, 1);

    let nodes = state.nodes.read().await;
    assert_eq!(nodes.len(), 1);
    assert!(nodes.contains_key("fresh"));
    assert!(!nodes.contains_key("stale"));
}

#[tokio::test]
async fn test_federation_state_cleanup_no_stale() {
    let state = FederationState::new("test-fed".to_string());
    let node = make_node("n1", "Node 1", "10.0.0.1:8080");
    state.register_node(node).await;

    let removed = state.cleanup_stale_nodes(600).await;
    assert_eq!(removed, 0);
    assert_eq!(state.nodes.read().await.len(), 1);
}

#[tokio::test]
async fn test_federation_state_active_nodes() {
    let state = FederationState::new("test-fed".to_string());

    let active_node = make_node("a1", "Active", "10.0.0.1:8080");
    state.register_node(active_node).await;

    let mut inactive_node = make_node("i1", "Inactive", "10.0.0.2:8080");
    inactive_node.status = NodeStatus::Inactive;
    state.register_node(inactive_node).await;

    let active = state.active_nodes().await;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].node_id, "a1");
}

#[tokio::test]
async fn test_federation_state_get_stats() {
    let state = FederationState::new("test-fed".to_string());

    let mut node1 = make_node("n1", "Node 1", "10.0.0.1:8080");
    node1.cpu_cores = 8;
    node1.memory_gb = 32;
    node1.storage_gb = Some(500);
    state.register_node(node1).await;

    let mut node2 = make_node("n2", "Node 2", "10.0.0.2:8080");
    node2.cpu_cores = 16;
    node2.memory_gb = 64;
    node2.storage_gb = Some(1000);
    state.register_node(node2).await;

    let mut inactive = make_node("n3", "Inactive", "10.0.0.3:8080");
    inactive.status = NodeStatus::Inactive;
    inactive.cpu_cores = 4;
    inactive.memory_gb = 8;
    state.register_node(inactive).await;

    let fed_stats = state.get_stats().await;
    assert_eq!(fed_stats.total_nodes, 3);
    assert_eq!(fed_stats.active_nodes, 2);
    assert_eq!(fed_stats.total_cpu_cores, 24); // 8 + 16 (inactive excluded)
    assert_eq!(fed_stats.total_memory_gb, 96); // 32 + 64
    assert_eq!(fed_stats.total_storage_gb, 1500); // 500 + 1000
}

#[tokio::test]
async fn test_federation_state_get_best_endpoint_with_endpoints() {
    let state = FederationState::new("test-fed".to_string());

    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    node.endpoints = Some(vec![
        make_endpoint("ethernet", "10.0.0.1:8443", 100),
        make_endpoint("wifi", "10.0.0.2:8443", 50),
    ]);
    state.register_node(node).await;

    let best = state.get_best_endpoint("n1").await;
    assert!(best.is_some());
    assert!(best.unwrap().contains("10.0.0.1:8443")); // Higher preference
}

#[tokio::test]
async fn test_federation_state_get_best_endpoint_fallback() {
    let state = FederationState::new("test-fed".to_string());

    let node = make_node("n1", "Node 1", "10.0.0.1:8080");
    state.register_node(node).await;

    let best = state.get_best_endpoint("n1").await;
    assert_eq!(best, Some("10.0.0.1:8080".to_string()));
}

#[tokio::test]
async fn test_federation_state_get_best_endpoint_nonexistent() {
    let state = FederationState::default();
    assert!(state.get_best_endpoint("nonexistent").await.is_none());
}

#[tokio::test]
async fn test_federation_state_get_all_endpoints() {
    let state = FederationState::new("test-fed".to_string());

    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    node.endpoints = Some(vec![
        make_endpoint("ethernet", "10.0.0.1:8443", 100),
        make_endpoint("wifi", "10.0.0.2:8443", 50),
    ]);
    state.register_node(node).await;

    let endpoints = state.get_all_endpoints("n1").await;
    assert!(endpoints.len() >= 2); // At least 2 endpoints + maybe primary
}

#[tokio::test]
async fn test_federation_state_get_all_endpoints_nonexistent() {
    let state = FederationState::default();
    let endpoints = state.get_all_endpoints("nonexistent").await;
    assert!(endpoints.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// NodeRegistration tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_node_registration_add_endpoint_new() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    assert!(node.endpoints.is_none());

    node.add_endpoint(make_endpoint("ethernet", "10.0.0.1:8443", 100));
    assert_eq!(node.endpoints.as_ref().unwrap().len(), 1);
}

#[test]
fn test_node_registration_add_endpoint_merge() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    node.endpoints = Some(vec![make_endpoint("ethernet", "10.0.0.1:8443", 100)]);

    node.add_endpoint(make_endpoint("wifi", "10.0.0.2:8443", 50));
    assert_eq!(node.endpoints.as_ref().unwrap().len(), 2);
}

#[test]
fn test_node_registration_add_endpoint_replace_same_address() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    node.endpoints = Some(vec![make_endpoint("ethernet", "10.0.0.1:8443", 50)]);

    // Add with same address but higher preference
    node.add_endpoint(make_endpoint("ethernet", "10.0.0.1:8443", 200));
    assert_eq!(node.endpoints.as_ref().unwrap().len(), 1);
    assert_eq!(node.endpoints.as_ref().unwrap()[0].preference, 200);
}

#[test]
fn test_node_registration_preferred_endpoint() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    node.endpoints = Some(vec![
        make_endpoint("wifi", "10.0.0.2:8443", 50),
        make_endpoint("ethernet", "10.0.0.1:8443", 100),
    ]);

    let pref = node.preferred_endpoint();
    assert!(pref.is_some());
    assert_eq!(pref.unwrap().address, "10.0.0.1:8443");
}

#[test]
fn test_node_registration_preferred_endpoint_none_when_no_endpoints() {
    let node = make_node("n1", "Node 1", "10.0.0.1:8080");
    assert!(node.preferred_endpoint().is_none());
}

#[test]
fn test_node_registration_preferred_endpoint_none_when_all_inactive() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    let mut ep = make_endpoint("ethernet", "10.0.0.1:8443", 100);
    ep.status = EndpointStatus::Failed;
    node.endpoints = Some(vec![ep]);

    assert!(node.preferred_endpoint().is_none());
}

#[test]
fn test_node_registration_active_endpoints() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    let mut failed_ep = make_endpoint("wifi", "10.0.0.2:8443", 50);
    failed_ep.status = EndpointStatus::Failed;

    node.endpoints = Some(vec![make_endpoint("ethernet", "10.0.0.1:8443", 100), failed_ep]);

    let active = node.active_endpoints();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].address, "10.0.0.1:8443");
}

#[test]
fn test_node_registration_active_endpoints_empty() {
    let node = make_node("n1", "Node 1", "10.0.0.1:8080");
    assert!(node.active_endpoints().is_empty());
}

#[test]
fn test_node_registration_update_endpoint_status() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    node.endpoints = Some(vec![make_endpoint("ethernet", "10.0.0.1:8443", 100)]);

    node.update_endpoint_status("10.0.0.1:8443", EndpointStatus::Degraded);

    let ep = &node.endpoints.as_ref().unwrap()[0];
    assert_eq!(ep.status, EndpointStatus::Degraded);
}

#[test]
fn test_node_registration_update_endpoint_status_nonexistent() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    node.endpoints = Some(vec![make_endpoint("ethernet", "10.0.0.1:8443", 100)]);

    // Updating non-existent address should be a no-op
    node.update_endpoint_status("10.99.99.99:8443", EndpointStatus::Failed);

    let ep = &node.endpoints.as_ref().unwrap()[0];
    assert_eq!(ep.status, EndpointStatus::Active); // Unchanged
}

#[test]
fn test_node_registration_update_endpoint_status_no_endpoints() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    // No endpoints — no panic
    node.update_endpoint_status("10.0.0.1:8443", EndpointStatus::Failed);
}

// ═══════════════════════════════════════════════════════════════════════════
// NodeStatus tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_node_status_display() {
    assert_eq!(format!("{}", NodeStatus::Active), "active");
    assert_eq!(format!("{}", NodeStatus::Inactive), "inactive");
    assert_eq!(format!("{}", NodeStatus::Unhealthy), "unhealthy");
}

#[test]
fn test_node_status_serde_roundtrip() {
    let statuses = [NodeStatus::Active, NodeStatus::Inactive, NodeStatus::Unhealthy];
    for status in statuses {
        let json = serde_json::to_string(&status).expect("serialize");
        let deserialized: NodeStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, status);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EndpointStatus tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_endpoint_status_serde_roundtrip() {
    let statuses = [
        EndpointStatus::Active,
        EndpointStatus::Standby,
        EndpointStatus::Degraded,
        EndpointStatus::Failed,
    ];
    for status in statuses {
        let json = serde_json::to_string(&status).expect("serialize");
        let deserialized: EndpointStatus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, status);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FederationStats & FederationStatus serde tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_federation_stats_serde_roundtrip() {
    let stats = FederationStats {
        total_nodes: 10,
        active_nodes: 8,
        total_cpu_cores: 64,
        total_memory_gb: 256,
        total_storage_gb: 4000,
        uptime_seconds: Some(120),
    };
    let json = serde_json::to_string(&stats).expect("serialize");
    let deserialized: FederationStats = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.total_nodes, 10);
    assert_eq!(deserialized.active_nodes, 8);
    assert_eq!(deserialized.total_cpu_cores, 64);
}

#[test]
fn test_federation_status_serde_roundtrip() {
    let status = FederationStatus {
        federation_id: "test-fed-123".to_string(),
        active_nodes: 3,
        nodes: vec![],
        total_cpu_cores: 24,
        total_memory_gb: 96,
        total_storage_gb: 1500,
        uptime_seconds: 86400,
    };
    let json = serde_json::to_string(&status).expect("serialize");
    let deserialized: FederationStatus = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.federation_id, "test-fed-123");
    assert_eq!(deserialized.uptime_seconds, 86400);
}

#[test]
fn test_node_registration_serde_roundtrip() {
    let node = make_node("n1", "Node 1", "10.0.0.1:8080");
    let json = serde_json::to_string(&node).expect("serialize");
    let deserialized: NodeRegistration = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.node_id, "n1");
    assert_eq!(deserialized.node_name, "Node 1");
    assert_eq!(deserialized.cpu_cores, 4);
}

#[test]
fn test_node_registration_with_endpoints_serde() {
    let mut node = make_node("n1", "Node 1", "10.0.0.1:8080");
    node.endpoints = Some(vec![
        make_endpoint("ethernet", "10.0.0.1:8443", 100),
        make_endpoint("wifi", "10.0.0.2:8443", 50),
    ]);
    let json = serde_json::to_string(&node).expect("serialize");
    let deserialized: NodeRegistration = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.endpoints.as_ref().unwrap().len(), 2);
}

#[test]
fn test_transport_endpoint_info_eq() {
    let now = Utc::now();
    let ep1 = TransportEndpointInfo {
        interface_type: "ethernet".to_string(),
        address: "10.0.0.1:8443".to_string(),
        protocols: vec!["https".to_string()],
        preference: 100,
        status: EndpointStatus::Active,
        last_check: now,
    };
    let ep2 = TransportEndpointInfo {
        interface_type: "ethernet".to_string(),
        address: "10.0.0.1:8443".to_string(),
        protocols: vec!["https".to_string()],
        preference: 100,
        status: EndpointStatus::Active,
        last_check: now,
    };
    let ep3 = make_endpoint("wifi", "10.0.0.2:8443", 50);
    assert_eq!(ep1, ep2);
    assert_ne!(ep1, ep3);
}
