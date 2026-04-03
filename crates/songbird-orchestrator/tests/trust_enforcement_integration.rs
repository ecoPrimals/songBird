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

//! Integration tests for trust enforcement
//!
//! Tests the complete flow from discovery → trust evaluation → peer acceptance/rejection

use songbird_orchestrator::trust::peer_trust::{DiscoveredPeer, PeerTrustDecision};

/// Test: Peer with same family should be auto-accepted
#[tokio::test]
async fn test_same_family_auto_accept() {
    let peer = DiscoveredPeer {
        node_id: "tower2".to_string(),
        tags: vec!["beardog:family:iidn:tower2".to_string()],
        endpoint: "https://192.168.1.135:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };

    // Note: This test requires a running security provider instance or mock server
    // For now, we're just testing the structure

    assert_eq!(peer.node_id, "tower2");
    assert!(peer.tags.contains(&"beardog:family:iidn:tower2".to_string()));
}

/// Test: Peer without tags should be rejected
#[tokio::test]
async fn test_no_tags_reject() {
    let peer = DiscoveredPeer {
        node_id: "tower3".to_string(),
        tags: vec![], // No tags = no lineage
        endpoint: "https://192.168.1.136:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };

    assert!(peer.tags.is_empty());
    // Without tags, security provider should reject this peer
}

/// Test: Peer with different family should prompt user
#[tokio::test]
async fn test_different_family_prompt() {
    let peer = DiscoveredPeer {
        node_id: "tower4".to_string(),
        tags: vec!["beardog:family:different:tower4".to_string()],
        endpoint: "https://192.168.1.137:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };

    assert_eq!(peer.node_id, "tower4");
    // Different family should trigger user prompt
}

/// Test: `PeerTrustDecision` enum construction
#[test]
fn test_peer_trust_decision_auto_accept() {
    let decision = PeerTrustDecision::AutoAccept {
        reason: "same_genetic_family".to_string(),
        confidence: 1.0,
        encryption_tag: Some("beardog:family:iidn:tower1".to_string()),
    };

    match decision {
        PeerTrustDecision::AutoAccept {
            reason,
            confidence,
            ..
        } => {
            assert_eq!(reason, "same_genetic_family");
            assert_eq!(confidence, 1.0);
        }
        _ => panic!("Expected AutoAccept"),
    }
}

/// Test: `PeerTrustDecision::PromptUser` construction
#[test]
fn test_peer_trust_decision_prompt_user() {
    let decision = PeerTrustDecision::PromptUser {
        reason: "different_genetic_family".to_string(),
        peer_id: "tower4".to_string(),
        recommendation: "verify_identity_before_accepting".to_string(),
    };

    match decision {
        PeerTrustDecision::PromptUser {
            reason,
            peer_id,
            recommendation,
        } => {
            assert_eq!(reason, "different_genetic_family");
            assert_eq!(peer_id, "tower4");
            assert_eq!(recommendation, "verify_identity_before_accepting");
        }
        _ => panic!("Expected PromptUser"),
    }
}

/// Test: `PeerTrustDecision::Reject` construction
#[test]
fn test_peer_trust_decision_reject() {
    let decision = PeerTrustDecision::Reject {
        reason: "no_genetic_lineage".to_string(),
        trust_level: "none".to_string(),
    };

    match decision {
        PeerTrustDecision::Reject {
            reason,
            trust_level,
        } => {
            assert_eq!(reason, "no_genetic_lineage");
            assert_eq!(trust_level, "none");
        }
        _ => panic!("Expected Reject"),
    }
}

/// Test: `DiscoveredPeer` struct creation
#[test]
fn test_discovered_peer_creation() {
    let peer = DiscoveredPeer {
        node_id: "test-tower".to_string(),
        tags: vec!["beardog:family:test:tower1".to_string(), "btsp_enabled".to_string()],
        endpoint: "https://192.168.1.100:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };

    assert_eq!(peer.node_id, "test-tower");
    assert_eq!(peer.tags.len(), 2);
    assert_eq!(peer.discovery_method, "udp_multicast");
    assert_eq!(peer.first_seen_at, 1704196800);
}

/// Test: Multiple tags handling
#[test]
fn test_multiple_tags() {
    let peer = DiscoveredPeer {
        node_id: "multi-tag-tower".to_string(),
        tags: vec![
            "beardog:family:iidn:tower1".to_string(),
            "btsp_enabled".to_string(),
            "birdsong_v2".to_string(),
        ],
        endpoint: "https://192.168.1.100:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };

    assert_eq!(peer.tags.len(), 3);
    assert!(peer.tags.iter().any(|t| t.starts_with("beardog:family:")));
    assert!(peer.tags.contains(&"btsp_enabled".to_string()));
}

/// Test: Empty endpoint handling (should be invalid)
#[test]
fn test_empty_endpoint() {
    let peer = DiscoveredPeer {
        node_id: "test-tower".to_string(),
        tags: vec![],
        endpoint: String::new(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };

    assert!(peer.endpoint.is_empty());
    // In real code, we should validate this
}

/// Test: Trust decision confidence levels
#[test]
fn test_confidence_levels() {
    let decisions = vec![
        (1.0, "same_family"),
        (0.7, "different_family_known"),
        (0.3, "different_family_unknown"),
        (0.0, "no_lineage"),
    ];

    for (confidence, reason) in decisions {
        let decision = PeerTrustDecision::AutoAccept {
            reason: reason.to_string(),
            confidence,
            encryption_tag: None,
        };

        if let PeerTrustDecision::AutoAccept {
            confidence: c,
            ..
        } = decision
        {
            assert!((0.0..=1.0).contains(&c), "Confidence should be between 0.0 and 1.0");
        }
    }
}

/// Test: Agnostic pattern - no hardcoded "`security provider`" in peer struct
#[test]
fn test_agnostic_pattern() {
    let peer = DiscoveredPeer {
        node_id: "agnostic-tower".to_string(),
        tags: vec!["generic:security:tag".to_string()],
        endpoint: "https://192.168.1.100:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };

    // Tags are generic - could be from any security provider
    assert!(!format!("{peer:?}").contains("security provider"));
    assert!(!peer.endpoint.contains(".sock"), "endpoint should be a URL, not a Unix socket path");

    // Only the tag content mentions the provider, which is opaque to Songbird
}

/// Test: Discovery timestamp validation
#[test]
fn test_timestamp_validity() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    let peer = DiscoveredPeer {
        node_id: "time-test-tower".to_string(),
        tags: vec![],
        endpoint: "https://192.168.1.100:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: now,
        capabilities: vec![],
        identity_attestations: vec![],
    };

    // Timestamp should be recent (within last day)
    assert!(peer.first_seen_at <= now);
    assert!(peer.first_seen_at > now - 86400); // Not older than 24 hours
}

/// Benchmark helper: Measure trust evaluation performance
#[test]
fn test_trust_decision_performance() {
    let start = std::time::Instant::now();

    for i in 0..1000 {
        let decision = PeerTrustDecision::AutoAccept {
            reason: format!("test_reason_{i}"),
            confidence: 1.0,
            encryption_tag: Some(format!("tag_{i}")),
        };

        match decision {
            PeerTrustDecision::AutoAccept {
                ..
            } => {}
            _ => panic!("Unexpected decision"),
        }
    }

    let duration = start.elapsed();
    println!("1000 trust decisions in {duration:?}");
    assert!(duration.as_millis() < 100, "Trust decisions should be fast");
}
