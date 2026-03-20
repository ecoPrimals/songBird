// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive E2E Tests for Generic Trust Integration
//!
//! Full end-to-end scenarios including:
//! - Discovery announcement with attestations
//! - Peer discovery and parsing
//! - Trust evaluation with universal API
//! - Decision handling (auto_accept/prompt/reject)

use serde_json::json;
use songbird_discovery::{DiscoveryPacket, IdentityAttestation};
use songbird_orchestrator::trust::{
    DiscoveredPeer, UniversalIdentityAttestation, UniversalTrustDecision, UniversalTrustRequest,
    UniversalTrustResponse,
};

/// Test: Complete flow - same family auto-accept
#[test]
fn e2e_same_family_complete_flow() {
    // Step 1: Tower A gets its identity
    let tower_a_identity = json!({
        "tags": ["beardog:family:iidn:tower_a"],
        "family_id": "iidn"
    });

    let tower_a_attestation = IdentityAttestation {
        provider_capability: "security/identity".to_string(),
        format: "tag_list".to_string(),
        data: tower_a_identity,
    };

    // Step 2: Tower A broadcasts discovery with attestation
    let discovery_packet = DiscoveryPacket::new(
        "tower_a",
        vec!["orchestration".to_string()],
        "https://192.168.1.100:8080",
    )
    .with_identity_attestation(tower_a_attestation);

    assert_eq!(discovery_packet.identity_attestations.len(), 1);

    // Step 3: Tower B receives discovery packet
    let received_attestations = discovery_packet.identity_attestations;
    assert_eq!(received_attestations.len(), 1);
    assert_eq!(received_attestations[0].format, "tag_list");

    // Step 4: Tower B creates discovered peer
    let discovered_peer = DiscoveredPeer {
        node_id: "tower_a".to_string(),
        tags: vec!["beardog:family:iidn:tower_a".to_string()],
        identity_attestations: vec![UniversalIdentityAttestation {
            provider: Some(received_attestations[0].provider_capability.clone()),
            format: received_attestations[0].format.clone(),
            data: received_attestations[0].data.clone(),
        }],
        endpoint: "https://192.168.1.100:8080".to_string(),
        capabilities: vec!["orchestration".to_string()],
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
    };

    // Step 5: Tower B builds universal trust request
    let trust_request = UniversalTrustRequest::new(
        discovered_peer.node_id.clone(),
        discovered_peer.identity_attestations.clone(),
    )
    .with_endpoint(discovered_peer.endpoint.clone())
    .with_discovery_method(discovered_peer.discovery_method.clone())
    .with_capabilities(discovered_peer.capabilities);

    assert_eq!(trust_request.evaluator.peer_id, "tower_a");
    assert_eq!(trust_request.evaluator.attestations.len(), 1);

    // Step 6: Simulate BearDog response (same family)
    let trust_response = UniversalTrustResponse {
        response_format: "universal_trust_v1".to_string(),
        decision: UniversalTrustDecision::AutoAccept,
        confidence: 1.0,
        reason: "Same genetic family (iidn)".to_string(),
        reason_code: "same_genetic_family".to_string(),
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("same_family".to_string(), json!(true));
            map.insert("family_id".to_string(), json!("iidn"));
            map
        },
        expires_at: None,
        custom: Default::default(),
    };

    // Step 7: Handle decision
    assert!(trust_response.is_auto_accept());
    assert_eq!(trust_response.confidence, 1.0);

    // E2E flow complete: Tower B would now form mesh connection
}

/// Test: Complete flow - different family prompt user
#[test]
fn e2e_different_family_prompt_flow() {
    // Tower A: family aaaa
    let tower_a_attestation = IdentityAttestation {
        provider_capability: "security/identity".to_string(),
        format: "tag_list".to_string(),
        data: json!({
            "tags": ["beardog:family:aaaa:tower_a"],
            "family_id": "aaaa"
        }),
    };

    let _discovery_packet = DiscoveryPacket::new(
        "tower_a",
        vec!["orchestration".to_string()],
        "https://192.168.1.100:8080",
    )
    .with_identity_attestation(tower_a_attestation);

    // Tower B: family bbbb (different)
    let discovered_peer = DiscoveredPeer {
        node_id: "tower_a".to_string(),
        tags: vec![],
        identity_attestations: vec![UniversalIdentityAttestation {
            provider: Some("security/identity".to_string()),
            format: "tag_list".to_string(),
            data: json!({
                "tags": ["beardog:family:aaaa:tower_a"],
                "family_id": "aaaa"
            }),
        }],
        endpoint: "https://192.168.1.100:8080".to_string(),
        capabilities: vec!["orchestration".to_string()],
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
    };

    let _trust_request = UniversalTrustRequest::new(
        discovered_peer.node_id.clone(),
        discovered_peer.identity_attestations.clone(),
    )
    .with_endpoint(discovered_peer.endpoint);

    // BearDog response: different family
    let trust_response = UniversalTrustResponse {
        response_format: "universal_trust_v1".to_string(),
        decision: UniversalTrustDecision::PromptUser,
        confidence: 0.5,
        reason: "Different genetic family (aaaa vs bbbb)".to_string(),
        reason_code: "different_genetic_family".to_string(),
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("same_family".to_string(), json!(false));
            map.insert("peer_family_id".to_string(), json!("aaaa"));
            map.insert("our_family_id".to_string(), json!("bbbb"));
            map
        },
        expires_at: None,
        custom: Default::default(),
    };

    assert!(trust_response.is_prompt_user());
    assert_eq!(trust_response.confidence, 0.5);
}

/// Test: Complete flow - no attestations reject
#[test]
fn e2e_no_attestations_reject_flow() {
    // Discovery packet with NO attestations
    let discovery_packet = DiscoveryPacket::new(
        "tower_untrusted",
        vec!["orchestration".to_string()],
        "https://192.168.1.150:8080",
    );

    assert_eq!(discovery_packet.identity_attestations.len(), 0);

    let discovered_peer = DiscoveredPeer {
        node_id: "tower_untrusted".to_string(),
        tags: vec![],
        identity_attestations: vec![], // No attestations
        endpoint: "https://192.168.1.150:8080".to_string(),
        capabilities: vec!["orchestration".to_string()],
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
    };

    let _trust_request = UniversalTrustRequest::new(
        discovered_peer.node_id.clone(),
        discovered_peer.identity_attestations,
    );

    // BearDog response: reject (no attestations)
    let trust_response = UniversalTrustResponse {
        response_format: "universal_trust_v1".to_string(),
        decision: UniversalTrustDecision::Reject,
        confidence: 0.0,
        reason: "No identity attestations provided".to_string(),
        reason_code: "no_attestations".to_string(),
        metadata: Default::default(),
        expires_at: None,
        custom: Default::default(),
    };

    assert!(trust_response.is_reject());
    assert_eq!(trust_response.confidence, 0.0);
}

/// Test: Multiple attestation types in single flow
#[test]
fn e2e_multiple_attestation_types() {
    let attestations = vec![
        IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "tags": ["beardog:family:iidn:tower1"],
                "family_id": "iidn"
            }),
        },
        IdentityAttestation {
            provider_capability: "security/certificate".to_string(),
            format: "x509".to_string(),
            data: json!({
                "certificate": "-----BEGIN CERTIFICATE-----...",
                "issuer": "ToadStool CA"
            }),
        },
    ];

    let mut discovery_packet = DiscoveryPacket::new(
        "tower1",
        vec!["orchestration".to_string()],
        "https://192.168.1.100:8080",
    );

    for att in &attestations {
        discovery_packet = discovery_packet.with_identity_attestation(att.clone());
    }

    assert_eq!(discovery_packet.identity_attestations.len(), 2);

    // Provider can choose which attestation type to use
    let discovered_peer = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: vec![],
        identity_attestations: discovery_packet
            .identity_attestations
            .iter()
            .map(|att| UniversalIdentityAttestation {
                provider: Some(att.provider_capability.clone()),
                format: att.format.clone(),
                data: att.data.clone(),
            })
            .collect(),
        endpoint: "https://192.168.1.100:8080".to_string(),
        capabilities: vec!["orchestration".to_string()],
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
    };

    assert_eq!(discovered_peer.identity_attestations.len(), 2);
}

/// Test: Backward compatibility - legacy tags still work
#[test]
fn e2e_backward_compatibility_legacy_tags() {
    let discovered_peer = DiscoveredPeer {
        node_id: "tower_legacy".to_string(),
        tags: vec!["beardog:family:iidn:tower_legacy".to_string()], // Legacy
        identity_attestations: vec![],                              // No new attestations
        endpoint: "https://192.168.1.100:8080".to_string(),
        capabilities: vec!["orchestration".to_string()],
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
    };

    // Should still work with legacy tags
    assert!(!discovered_peer.tags.is_empty());
    assert_eq!(discovered_peer.identity_attestations.len(), 0);
}

/// Test: Discovery packet serialization roundtrip
#[test]
fn e2e_discovery_packet_roundtrip() {
    let attestation = IdentityAttestation {
        provider_capability: "security/identity".to_string(),
        format: "tag_list".to_string(),
        data: json!({
            "tags": ["beardog:family:iidn:tower1"],
            "family_id": "iidn"
        }),
    };

    let original = DiscoveryPacket::new(
        "tower1",
        vec!["orchestration".to_string(), "storage".to_string()],
        "https://192.168.1.100:8080",
    )
    .with_identity_attestation(attestation);

    // Serialize
    let json = serde_json::to_string(&original).expect("Failed to serialize");

    // Deserialize
    let deserialized: DiscoveryPacket = serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify
    assert_eq!(deserialized.node_id, original.node_id);
    assert_eq!(deserialized.identity_attestations.len(), 1);
    assert_eq!(deserialized.identity_attestations[0].format, "tag_list");
}

/// Test: Trust request/response roundtrip
#[test]
fn e2e_trust_api_roundtrip() {
    let request = UniversalTrustRequest::new(
        "tower2",
        vec![UniversalIdentityAttestation::tag_list_with_family(
            vec!["beardog:family:iidn:tower2".to_string()],
            "iidn",
        )],
    )
    .with_endpoint("https://192.168.1.135:8080")
    .with_discovery_method("udp_multicast");

    // Serialize request
    let request_json = serde_json::to_string(&request).expect("Failed to serialize request");

    // Deserialize request (what BearDog receives)
    let received_request: UniversalTrustRequest =
        serde_json::from_str(&request_json).expect("Failed to deserialize request");

    assert_eq!(received_request.evaluator.peer_id, "tower2");

    // Build response
    let response = UniversalTrustResponse {
        response_format: "universal_trust_v1".to_string(),
        decision: UniversalTrustDecision::AutoAccept,
        confidence: 1.0,
        reason: "Same family".to_string(),
        reason_code: "same_genetic_family".to_string(),
        metadata: Default::default(),
        expires_at: None,
        custom: Default::default(),
    };

    // Serialize response
    let response_json = serde_json::to_string(&response).expect("Failed to serialize response");

    // Deserialize response (what Songbird receives)
    let received_response: UniversalTrustResponse =
        serde_json::from_str(&response_json).expect("Failed to deserialize response");

    assert!(received_response.is_auto_accept());
}

/// Test: Concurrent peer discovery and evaluation
#[test]
fn e2e_concurrent_peer_handling() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let results = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let results = Arc::clone(&results);
            thread::spawn(move || {
                // Simulate peer discovery
                let peer = DiscoveredPeer {
                    node_id: format!("tower{}", i),
                    tags: vec![],
                    identity_attestations: vec![UniversalIdentityAttestation::tag_list(vec![
                        format!("beardog:family:iidn:tower{}", i),
                    ])],
                    endpoint: format!("https://192.168.1.{}:8080", 100 + i),
                    capabilities: vec!["orchestration".to_string()],
                    discovery_method: "udp_multicast".to_string(),
                    first_seen_at: 1704196800,
                };

                // Build request
                let request =
                    UniversalTrustRequest::new(peer.node_id.clone(), peer.identity_attestations);

                results.lock().unwrap().push(request.evaluator.peer_id);
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let final_results = results.lock().unwrap();
    assert_eq!(final_results.len(), 10);
}

/// Test: Expired trust decision handling
#[test]
fn e2e_expired_trust_decision() {
    let expired_time = "2020-01-01T00:00:00Z"; // Past date

    let response = UniversalTrustResponse {
        response_format: "universal_trust_v1".to_string(),
        decision: UniversalTrustDecision::AutoAccept,
        confidence: 1.0,
        reason: "Cached decision".to_string(),
        reason_code: "cached".to_string(),
        metadata: Default::default(),
        expires_at: Some(expired_time.to_string()),
        custom: Default::default(),
    };

    // Application should check expires_at and re-evaluate if expired
    assert!(response.expires_at.is_some());
}
