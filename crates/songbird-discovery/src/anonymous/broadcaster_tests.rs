// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use std::net::SocketAddr;

use crate::anonymous::broadcaster::AnonymousDiscoveryBroadcaster;
use crate::anonymous::messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};

use super::protocol::build_discovery_plaintext;
use super::scheduling::{broadcast_interval, rotating_session_id};

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

#[test]
fn rotating_session_id_format_and_hour_bucket() {
    let id = rotating_session_id();
    assert!(id.starts_with("session-"), "expected hour bucket prefix, got {id:?}");
    let slot: u64 = id.strip_prefix("session-").expect("prefix").parse().expect("numeric slot");
    let expected_slot =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
            / 3600;
    assert_eq!(slot, expected_slot);
}

#[tokio::test]
async fn broadcast_interval_period_matches_secs() {
    let iv = broadcast_interval(47);
    assert_eq!(iv.period(), std::time::Duration::from_secs(47));
}

#[test]
fn build_discovery_plaintext_v21_with_attestations_validates() {
    use crate::IdentityAttestation;

    let att = IdentityAttestation {
        provider_capability: "security/identity".into(),
        format: "jwt".into(),
        data: serde_json::json!({"sub": "node"}),
    };
    let prep = build_discovery_plaintext(
        "2.1",
        None,
        None,
        None,
        vec!["compute".into()],
        vec!["https".into()],
        8443,
        Some(vec![att]),
        Some(vec!["crypto:family:f1".into(), "crypto:role:r".into()]),
    )
    .expect("plaintext");
    assert!(!prep.session_id.is_empty());
    let msg = AnonymousDiscoveryMessage::from_bytes(&prep.bytes).expect("parse");
    assert!(msg.validate().is_ok());
    assert_eq!(msg.tags, Some(vec!["crypto:family:f1".into(), "crypto:role:r".into()]));
    assert!(msg.identity_attestations.is_some());
}

#[test]
fn build_discovery_plaintext_v30_preserves_capabilities_order() {
    let eps = vec![TransportEndpointMessage {
        interface_type: "lo".into(),
        address: "127.0.0.1:1".into(),
        protocols: vec!["https".into()],
        preference: 0,
    }];
    let prep = build_discovery_plaintext(
        "3.0",
        Some("nid".into()),
        Some("nn".into()),
        Some(eps),
        vec!["z".into(), "a".into(), "m".into()],
        vec![],
        0,
        None,
        None,
    )
    .expect("v3 plaintext");
    let msg = AnonymousDiscoveryMessage::from_bytes(&prep.bytes).expect("parse");
    assert_eq!(msg.capabilities, vec!["z".to_string(), "a".to_string(), "m".to_string()]);
}

#[test]
fn with_identity_tags_preserves_order_when_non_empty() {
    let tags = vec!["p:a:1".into(), "p:b:2".into(), "p:c:3".into()];
    let b = AnonymousDiscoveryBroadcaster::new(
        vec!["c".into()],
        vec!["https".into()],
        443,
        vec!["224.0.0.251:2300".parse().unwrap()],
        15,
    )
    .with_identity_tags(tags.clone());
    assert_eq!(b.tags.as_ref().expect("tags"), &tags);
    assert_eq!(b.interval_secs, 15);
}
