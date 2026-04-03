// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use std::net::SocketAddr;

use crate::anonymous::broadcaster::AnonymousDiscoveryBroadcaster;
use crate::anonymous::messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};

#[test]
fn test_broadcaster_new_v2() {
    let broadcaster = AnonymousDiscoveryBroadcaster::new(
        vec!["orchestration".to_string()],
        vec!["https".to_string()],
        8080,
        vec!["224.0.0.251:2300".parse().unwrap()],
        30,
    );

    assert_eq!(broadcaster.version, "2.1");
    assert!(broadcaster.node_id.is_none());
    assert_eq!(broadcaster.capabilities.len(), 1);
    assert_eq!(broadcaster.interval_secs, 30);
}

#[test]
fn test_broadcaster_new_v3() {
    let endpoints = vec![TransportEndpointMessage {
        interface_type: "ethernet".to_string(),
        address: "192.168.1.100:8080".to_string(),
        protocols: vec!["https".to_string()],
        preference: 255,
    }];

    let broadcaster = AnonymousDiscoveryBroadcaster::new_v3(
        "node-123".to_string(),
        "testnode".to_string(),
        endpoints,
        vec!["orchestration".to_string()],
        vec!["224.0.0.251:2300".parse().unwrap()],
        30,
    );

    assert_eq!(broadcaster.version, "3.0");
    assert_eq!(broadcaster.node_id, Some("node-123".to_string()));
    assert_eq!(broadcaster.node_name, Some("testnode".to_string()));
    assert!(broadcaster.endpoints.is_some());
}

#[test]
fn test_broadcaster_with_known_peers() {
    let broadcaster = AnonymousDiscoveryBroadcaster::new(
        vec!["orchestration".to_string()],
        vec!["https".to_string()],
        8080,
        vec!["224.0.0.251:2300".parse().unwrap()],
        30,
    )
    .with_known_peers(vec!["192.168.1.10:2300".parse().unwrap()]);

    assert_eq!(broadcaster.known_peers.len(), 1);
}

#[test]
fn test_broadcaster_with_identity_attestations() {
    use crate::IdentityAttestation;

    let attestation = IdentityAttestation {
        provider_capability: "security/identity".to_string(),
        format: "tag_list".to_string(),
        data: serde_json::json!({"family_id": "test-family"}),
    };

    let broadcaster = AnonymousDiscoveryBroadcaster::new(
        vec!["orchestration".to_string()],
        vec!["https".to_string()],
        8080,
        vec!["224.0.0.251:2300".parse().unwrap()],
        30,
    )
    .with_identity_attestations(vec![attestation]);

    assert!(broadcaster.identity_attestations.is_some());
    assert_eq!(broadcaster.identity_attestations.unwrap().len(), 1);
}

#[test]
fn with_identity_tags_empty_clears() {
    let b = AnonymousDiscoveryBroadcaster::new(
        vec!["c".into()],
        vec!["https".into()],
        8080,
        vec!["224.0.0.251:2300".parse().expect("addr")],
        30,
    )
    .with_identity_tags(vec!["t".into()])
    .with_identity_tags(vec![]);
    assert!(b.tags.is_none());
}

#[test]
fn v3_message_to_bytes_roundtrip_preserves_version() {
    let endpoints = vec![TransportEndpointMessage {
        interface_type: "ethernet".into(),
        address: "192.168.1.2:8443".into(),
        protocols: vec!["https".into()],
        preference: 1,
    }];
    let msg = AnonymousDiscoveryMessage::new_v3("nid", "nn", endpoints, vec!["cap".into()]);
    let bytes = msg.to_bytes().expect("to_bytes");
    let back = AnonymousDiscoveryMessage::from_bytes(&bytes).expect("from_bytes");
    assert_eq!(back.version, "3.0");
    assert_eq!(back.node_id.as_deref(), Some("nid"));
    assert!(back.validate().is_ok());
}

#[test]
fn v3_defaults_port_when_address_has_no_port() {
    let endpoints = vec![TransportEndpointMessage {
        interface_type: "ethernet".into(),
        address: "192.168.1.1".into(),
        protocols: vec!["https".into()],
        preference: 1,
    }];
    let b = AnonymousDiscoveryBroadcaster::new_v3(
        "id".into(),
        "name".into(),
        endpoints,
        vec![],
        vec!["224.0.0.251:2300".parse().unwrap()],
        10,
    );
    assert_eq!(b.port, 8080);
}

#[test]
fn v3_uses_primary_protocols_for_fallback() {
    let endpoints = vec![TransportEndpointMessage {
        interface_type: "wifi".into(),
        address: "10.0.0.1:9443".into(),
        protocols: vec!["btsp".into(), "https".into()],
        preference: 2,
    }];
    let b = AnonymousDiscoveryBroadcaster::new_v3(
        "id".into(),
        "n".into(),
        endpoints,
        vec!["cap".into()],
        vec!["224.0.0.251:2300".parse().unwrap()],
        5,
    );
    assert!(b.protocols.contains(&"btsp".into()));
}

#[test]
fn with_known_peers_preserves_order() {
    let a: SocketAddr = "192.168.0.1:1".parse().unwrap();
    let b: SocketAddr = "192.168.0.2:2".parse().unwrap();
    let br = AnonymousDiscoveryBroadcaster::new(
        vec![],
        vec!["https".into()],
        443,
        vec!["224.0.0.251:2300".parse().unwrap()],
        1,
    )
    .with_known_peers(vec![a, b]);
    assert_eq!(br.known_peers, vec![a, b]);
}

#[test]
fn v2_constructor_sets_version_and_interval() {
    let br = AnonymousDiscoveryBroadcaster::new(
        vec!["a".into()],
        vec!["p".into()],
        9090,
        vec!["224.0.0.251:2300".parse().unwrap()],
        42,
    );
    assert_eq!(br.version, "2.1");
    assert_eq!(br.interval_secs, 42);
    assert_eq!(br.port, 9090);
}
