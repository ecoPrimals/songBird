//! Simple federation tests
//!
//! Modern, idiomatic tests for federation configuration and node management

#![allow(clippy::unwrap_used)]
use songbird_network_federation::{FederationConfig, NodeInfo};

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
fn test_node_info_statuses() {
    let statuses = vec!["active", "inactive", "starting", "stopping"];
    for status in statuses {
        let node = NodeInfo {
            node_id: "node".to_string(),
            address: "http://localhost:8080".to_string(),
            status: status.to_string(),
        };
        assert_eq!(node.status, status);
    }
}

#[test]
fn test_multiple_nodes() {
    let nodes = [
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
}

#[test]
fn test_federation_config_enabled() {
    let config = FederationConfig {
        discovery_mode: None,
        rendezvous_url: None,
        enabled: true,
        bootstrap_address: Some("http://bootstrap:8080".to_string()),
        self_registration: None,
        heartbeat_interval_secs: 30,
        node_timeout_secs: 60,
        ..Default::default()
    };
    assert!(config.enabled);
}

#[test]
fn test_federation_config_disabled() {
    let config = FederationConfig {
        discovery_mode: None,
        rendezvous_url: None,
        enabled: false,
        bootstrap_address: None,
        self_registration: None,
        heartbeat_interval_secs: 30,
        node_timeout_secs: 60,
    };
    assert!(!config.enabled);
}

#[test]
fn test_bootstrap_address() {
    let config = FederationConfig {
        discovery_mode: None,
        rendezvous_url: None,
        enabled: true,
        bootstrap_address: Some("http://main-node:9000".to_string()),
        self_registration: None,
        heartbeat_interval_secs: 30,
        node_timeout_secs: 60,
    };
    assert!(config.bootstrap_address.is_some());
    assert_eq!(
        config.bootstrap_address.expect("Expected bootstrap address"),
        "http://main-node:9000"
    );
}

#[test]
fn test_heartbeat_interval() {
    for interval in [10, 20, 30, 60, 120] {
        let config = FederationConfig {
            discovery_mode: None,
            rendezvous_url: None,
            enabled: true,
            bootstrap_address: None,
            self_registration: None,
            heartbeat_interval_secs: interval,
            node_timeout_secs: 120,
            ..Default::default()
        };
        assert_eq!(config.heartbeat_interval_secs, interval);
    }
}

#[test]
fn test_node_timeout() {
    for timeout in [30, 60, 120, 300] {
        let config = FederationConfig {
            discovery_mode: None,
            rendezvous_url: None,
            enabled: true,
            bootstrap_address: None,
            self_registration: None,
            heartbeat_interval_secs: 30,
            node_timeout_secs: timeout,
            ..Default::default()
        };
        assert_eq!(config.node_timeout_secs, timeout);
    }
}

#[test]
fn test_minimal_federation_config() {
    let config = FederationConfig {
        discovery_mode: None,
        rendezvous_url: None,
        enabled: true,
        bootstrap_address: None,
        self_registration: None,
        heartbeat_interval_secs: 30,
        node_timeout_secs: 60,
    };
    assert!(config.bootstrap_address.is_none());
}

#[test]
fn test_node_addresses() {
    let addresses =
        vec!["http://localhost:8080", "http://192.168.1.100:9000", "https://node.example.com:443"];
    for addr in addresses {
        let node = NodeInfo {
            node_id: "test".to_string(),
            address: addr.to_string(),
            status: "active".to_string(),
        };
        assert_eq!(node.address, addr);
    }
}
