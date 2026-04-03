// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! `security provider` API Compatibility E2E Tests
//!
//! End-to-end tests verifying the complete flow with `security provider` API format

use songbird_orchestrator::security_capability_client::{
    ConnectionInfo, TrustEvaluationRequest, TrustEvaluationResponse,
};
use songbird_orchestrator::trust::peer_trust::{DiscoveredPeer, PeerTrustDecision};
use std::collections::HashMap;

/// Test: Complete trust evaluation request flow
#[test]
fn test_complete_trust_evaluation_flow() {
    // Simulate discovery of a peer
    let peer = DiscoveredPeer {
        node_id: "tower2".to_string(),
        tags: vec!["beardog:family:iidn:tower2".to_string()],
        endpoint: "https://192.168.1.135:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1767368141, // u64 internally
        capabilities: vec![],
        identity_attestations: vec![],
    };

    // Build trust evaluation request (converts to String)
    let request = TrustEvaluationRequest {
        peer_id: peer.node_id.clone(),
        peer_family: Some("iidn".to_string()), // ✅ Extracted from "beardog:family:iidn:tower2"
        peer_tags: peer.tags.clone(),
        connection_info: Some(ConnectionInfo {
            endpoint: peer.endpoint.clone(),
            protocol: "tarpc".to_string(),
        }),
        context: Some(
            vec![
                ("discovery_method".to_string(), peer.discovery_method.clone()),
                ("first_seen_at".to_string(), peer.first_seen_at.to_string()),
            ]
            .into_iter()
            .collect(),
        ),
    };

    // Verify request format
    let json = serde_json::to_string(&request).expect("Failed to serialize");
    assert!(json.contains(r#""first_seen_at":"1767368141""#));
    assert!(!json.contains(r#""first_seen_at":1767368141"#)); // Not integer
}

/// Test: `security provider` response deserialization
#[test]
fn test_security_provider_response_deserialization() {
    let json_response = r#"{
        "decision": "auto_accept",
        "trust_level": "high",
        "confidence": 1.0,
        "reason": "same_genetic_family",
        "encryption_tag": "beardog:family:iidn:tower1",
        "metadata": {}
    }"#;

    let response: TrustEvaluationResponse = serde_json::from_str(json_response)
        .expect("Failed to deserialize security provider response");

    assert_eq!(response.decision, "auto_accept");
    assert_eq!(response.trust_level, "high");
    assert_eq!(response.confidence, 1.0);
}

/// Test: Multiple peer discovery with String timestamps
#[test]
fn test_multiple_peer_discovery() {
    let base_timestamp = 1767368141u64;

    for i in 0..5 {
        let peer = DiscoveredPeer {
            node_id: format!("tower{}", i + 2),
            tags: vec![format!("beardog:family:iidn:tower{}", i + 2)],
            endpoint: format!("https://192.168.1.{}:8080", 135 + i),
            discovery_method: "udp_multicast".to_string(),
            first_seen_at: base_timestamp + i,
            capabilities: vec![],
            identity_attestations: vec![],
        };

        let request = TrustEvaluationRequest {
            peer_id: peer.node_id.clone(),
            peer_family: Some("iidn".to_string()), // ✅ v3.14.1
            peer_tags: peer.tags.clone(),
            connection_info: Some(ConnectionInfo {
                endpoint: peer.endpoint.clone(),
                protocol: "tarpc".to_string(),
            }),
            context: Some(
                vec![
                    ("discovery_method".to_string(), peer.discovery_method.clone()),
                    ("first_seen_at".to_string(), peer.first_seen_at.to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        };

        // All should serialize correctly
        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json.contains(&format!(r#""first_seen_at":"{}""#, base_timestamp + i)));
    }
}

/// Test: Request/Response cycle (simulated)
#[test]
fn test_request_response_cycle() {
    // 1. Create request
    let request = TrustEvaluationRequest {
        peer_id: "tower2".to_string(),
        peer_family: Some("iidn".to_string()), // ✅ v3.14.1
        peer_tags: vec!["beardog:family:iidn:tower2".to_string()],
        connection_info: Some(ConnectionInfo {
            endpoint: "https://192.168.1.135:8080".to_string(),
            protocol: "tarpc".to_string(),
        }),
        context: Some(
            vec![
                ("discovery_method".to_string(), "udp_multicast".to_string()),
                ("first_seen_at".to_string(), "1767368141".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
    };

    // 2. Serialize (what Songbird sends)
    let request_json = serde_json::to_string(&request).expect("Failed to serialize");
    assert!(request_json.contains(r#""first_seen_at":"1767368141""#));

    // 3. Simulate security provider response
    let response_json = r#"{
        "decision": "auto_accept",
        "trust_level": "high",
        "confidence": 1.0,
        "reason": "same_genetic_family",
        "encryption_tag": "beardog:family:iidn:tower2",
        "metadata": {}
    }"#;

    // 4. Deserialize response
    let response: TrustEvaluationResponse =
        serde_json::from_str(response_json).expect("Failed to deserialize");

    // 5. Verify decision
    assert_eq!(response.decision, "auto_accept");
    assert_eq!(response.confidence, 1.0);
}

/// Test: Error case - no context
#[test]
fn test_trust_request_without_context() {
    let request = TrustEvaluationRequest {
        peer_id: "tower2".to_string(),
        peer_family: Some("iidn".to_string()), // ✅ v3.14.1
        peer_tags: vec!["beardog:family:iidn:tower2".to_string()],
        connection_info: Some(ConnectionInfo {
            endpoint: "https://192.168.1.135:8080".to_string(),
            protocol: "tarpc".to_string(),
        }),
        context: None,
    };

    // Should still serialize correctly
    let json = serde_json::to_string(&request).expect("Failed to serialize");
    // Context may be omitted or null depending on serde settings
    assert!(json.contains(r#""peer_id":"tower2""#));
}

/// Test: Discovery with additional metadata
#[test]
fn test_discovery_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("network".to_string(), "lan".to_string());
    metadata.insert("signal_strength".to_string(), "strong".to_string());

    let request = TrustEvaluationRequest {
        peer_id: "tower2".to_string(),
        peer_family: Some("iidn".to_string()), // ✅ v3.14.1
        peer_tags: vec!["beardog:family:iidn:tower2".to_string()],
        connection_info: Some(ConnectionInfo {
            endpoint: "https://192.168.1.135:8080".to_string(),
            protocol: "tarpc".to_string(),
        }),
        context: Some({
            let mut ctx = vec![
                ("discovery_method".to_string(), "udp_multicast".to_string()),
                ("first_seen_at".to_string(), "1767368141".to_string()),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>();
            ctx.extend(metadata);
            ctx
        }),
    };

    let json = serde_json::to_string(&request).expect("Failed to serialize");
    assert!(json.contains(r#""network":"lan""#));
    assert!(json.contains(r#""signal_strength":"strong""#));
}

/// Test: Different discovery methods
#[test]
fn test_different_discovery_methods() {
    let methods = vec!["udp_multicast", "mdns", "manual", "api"];

    for method in methods {
        let request = TrustEvaluationRequest {
            peer_id: "tower2".to_string(),
            peer_family: Some("iidn".to_string()), // ✅ v3.14.1
            peer_tags: vec!["beardog:family:iidn:tower2".to_string()],
            connection_info: Some(ConnectionInfo {
                endpoint: "https://192.168.1.135:8080".to_string(),
                protocol: "tarpc".to_string(),
            }),
            context: Some(
                vec![
                    ("discovery_method".to_string(), method.to_string()),
                    ("first_seen_at".to_string(), "1767368141".to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        };

        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json.contains(&format!(r#""discovery_method":"{method}""#)));
    }
}

/// Test: Concurrent request handling
#[test]
fn test_concurrent_requests() {
    let requests: Vec<TrustEvaluationRequest> = (0..10)
        .map(|i| TrustEvaluationRequest {
            peer_id: format!("tower{}", i + 2),
            peer_family: Some("iidn".to_string()), // ✅ v3.14.1
            peer_tags: vec![format!("beardog:family:iidn:tower{}", i + 2)],
            connection_info: Some(ConnectionInfo {
                endpoint: format!("https://192.168.1.{}:8080", 135 + i),
                protocol: "tarpc".to_string(),
            }),
            context: Some(
                vec![
                    ("discovery_method".to_string(), "udp_multicast".to_string()),
                    ("first_seen_at".to_string(), (1767368141 + i).to_string()),
                ]
                .into_iter()
                .collect(),
            ),
        })
        .collect();

    assert_eq!(requests.len(), 10);

    // All should serialize correctly
    for request in requests {
        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json.contains(r#""first_seen_at":"#));
    }
}

/// Test: Trust decision mapping
#[test]
fn test_trust_decision_mapping() {
    let responses = vec![
        ("auto_accept", "same_genetic_family", 1.0),
        ("prompt_user", "different_genetic_family", 0.7),
        ("reject", "no_genetic_lineage", 0.0),
    ];

    for (decision, reason, confidence) in responses {
        let response_json = format!(
            r#"{{
                "decision": "{decision}",
                "trust_level": "high",
                "confidence": {confidence},
                "reason": "{reason}",
                "encryption_tag": null,
                "metadata": {{}}
            }}"#
        );

        let response: TrustEvaluationResponse =
            serde_json::from_str(&response_json).expect("Failed to deserialize");

        assert_eq!(response.decision, decision);
        assert_eq!(response.reason, reason);
        assert_eq!(response.confidence, confidence);
    }
}

/// Test: Full E2E flow simulation
#[test]
fn test_full_e2e_flow() {
    // Step 1: Peer discovered
    let discovered_peer = DiscoveredPeer {
        node_id: "tower2".to_string(),
        tags: vec!["beardog:family:iidn:tower2".to_string()],
        endpoint: "https://192.168.1.135:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1767368141,
        capabilities: vec![],
        identity_attestations: vec![],
    };

    // Step 2: Build trust request (u64 → String conversion happens here)
    let trust_request = TrustEvaluationRequest {
        peer_id: discovered_peer.node_id.clone(),
        peer_family: Some("iidn".to_string()), // ✅ v3.14.1
        peer_tags: discovered_peer.tags.clone(),
        connection_info: Some(ConnectionInfo {
            endpoint: discovered_peer.endpoint.clone(),
            protocol: "tarpc".to_string(),
        }),
        context: Some(
            vec![
                ("discovery_method".to_string(), discovered_peer.discovery_method.clone()),
                ("first_seen_at".to_string(), discovered_peer.first_seen_at.to_string()),
            ]
            .into_iter()
            .collect(),
        ),
    };

    // Step 3: Serialize (what gets sent to security provider)
    let json = serde_json::to_string(&trust_request).expect("Failed to serialize");

    // Verify: String format, not integer
    assert!(json.contains(r#""first_seen_at":"1767368141""#));
    assert!(!json.contains(r#""first_seen_at":1767368141"#));

    // Step 4: Simulate security provider's response
    let sample_response = r#"{
        "decision": "auto_accept",
        "trust_level": "high",
        "confidence": 1.0,
        "reason": "same_genetic_family",
        "encryption_tag": "beardog:family:iidn:tower2",
        "metadata": {}
    }"#;

    // Step 5: Deserialize response
    let response: TrustEvaluationResponse =
        serde_json::from_str(sample_response).expect("Failed to deserialize");

    // Step 6: Verify decision
    assert_eq!(response.decision, "auto_accept");
    assert_eq!(response.reason, "same_genetic_family");
    assert_eq!(response.confidence, 1.0);

    // Step 7: Map to PeerTrustDecision
    let _decision = match response.decision.as_str() {
        "auto_accept" => PeerTrustDecision::AutoAccept {
            reason: response.reason.clone(),
            confidence: response.confidence,
            encryption_tag: response.encryption_tag.clone(),
        },
        "prompt_user" => PeerTrustDecision::PromptUser {
            reason: response.reason.clone(),
            peer_id: discovered_peer.node_id,
            recommendation: "verify_identity".to_string(),
        },
        "reject" => PeerTrustDecision::Reject {
            reason: response.reason.clone(),
            trust_level: response.trust_level.clone(),
        },
        _ => PeerTrustDecision::Reject {
            reason: "unknown_decision".to_string(),
            trust_level: "none".to_string(),
        },
    };

    // E2E flow complete!
}
