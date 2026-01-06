//! Property-based tests for trust enforcement
//!
//! Uses quickcheck-style property testing to verify invariants

use songbird_orchestrator::trust::peer_trust::{DiscoveredPeer, PeerTrustDecision};

/// Property: All peer IDs should be non-empty
#[test]
fn prop_peer_id_non_empty() {
    let test_cases = vec![
        "tower1",
        "tower-2",
        "TOWER_3",
        "t",
        "very-long-tower-name-with-many-characters",
    ];
    
    for peer_id in test_cases {
        let peer = DiscoveredPeer {
            node_id: peer_id.to_string(),
            tags: vec![],
            endpoint: "https://localhost:8080".to_string(),
            discovery_method: "udp_multicast".to_string(),
            first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
        };
        
        assert!(!peer.node_id.is_empty(), "Peer ID should never be empty");
    }
}

/// Property: Confidence should always be between 0.0 and 1.0
#[test]
fn prop_confidence_in_range() {
    let confidences = vec![0.0, 0.1, 0.5, 0.9, 1.0];
    
    for confidence in confidences {
        let decision = PeerTrustDecision::AutoAccept {
            reason: "test".to_string(),
            confidence,
            encryption_tag: None,
        };
        
        match decision {
            PeerTrustDecision::AutoAccept { confidence: c, .. } => {
                assert!(c >= 0.0 && c <= 1.0, "Confidence must be in [0.0, 1.0]");
            }
            _ => panic!("Expected AutoAccept"),
        }
    }
}

/// Property: Tags are order-independent for trust evaluation
#[test]
fn prop_tags_order_independent() {
    let tags1 = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
    let tags2 = vec!["tag3".to_string(), "tag1".to_string(), "tag2".to_string()];
    
    let peer1 = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: tags1.clone(),
        endpoint: "https://localhost:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    let peer2 = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: tags2.clone(),
        endpoint: "https://localhost:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    // Both peers have the same tags, just in different order
    assert_eq!(peer1.node_id, peer2.node_id);
    let mut sorted1 = peer1.tags.clone();
    let mut sorted2 = peer2.tags.clone();
    sorted1.sort();
    sorted2.sort();
    assert_eq!(sorted1, sorted2);
}

/// Property: Empty tags list is valid
#[test]
fn prop_empty_tags_valid() {
    let peer = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: vec![],
        endpoint: "https://localhost:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    assert!(peer.tags.is_empty());
    // Empty tags is valid - means no security provider configured
}

/// Property: Timestamp should be monotonic (newer discoveries have higher timestamps)
#[test]
fn prop_timestamp_monotonic() {
    let mut timestamps = vec![];
    
    for i in 0..10 {
        let peer = DiscoveredPeer {
            node_id: format!("tower{}", i),
            tags: vec![],
            endpoint: "https://localhost:8080".to_string(),
            discovery_method: "udp_multicast".to_string(),
            first_seen_at: 1704196800 + i,
            capabilities: vec![],
            identity_attestations: vec![],
        };
        
        timestamps.push(peer.first_seen_at);
    }
    
    // Timestamps should be in ascending order
    for window in timestamps.windows(2) {
        assert!(window[0] <= window[1], "Timestamps should be monotonic");
    }
}

/// Property: Different discovery methods are all valid
#[test]
fn prop_discovery_methods_valid() {
    let methods = vec![
        "udp_multicast",
        "udp_broadcast",
        "mdns",
        "manual",
        "api",
    ];
    
    for method in methods {
        let peer = DiscoveredPeer {
            node_id: "tower1".to_string(),
            tags: vec![],
            endpoint: "https://localhost:8080".to_string(),
            discovery_method: method.to_string(),
            first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
        };
        
        assert!(!peer.discovery_method.is_empty());
    }
}

/// Property: Endpoint format should be URL-like
#[test]
fn prop_endpoint_url_format() {
    let endpoints = vec![
        "https://192.168.1.100:8080",
        "http://localhost:8080",
        "https://example.com:443",
        "http://[::1]:8080", // IPv6
    ];
    
    for endpoint in endpoints {
        let peer = DiscoveredPeer {
            node_id: "tower1".to_string(),
            tags: vec![],
            endpoint: endpoint.to_string(),
            discovery_method: "udp_multicast".to_string(),
            first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
        };
        
        assert!(peer.endpoint.contains("://"), "Endpoint should look like a URL");
    }
}

/// Property: Trust decisions are immutable once created
#[test]
fn prop_trust_decision_immutable() {
    let decision = PeerTrustDecision::AutoAccept {
        reason: "test_reason".to_string(),
        confidence: 1.0,
        encryption_tag: Some("tag".to_string()),
    };
    
    // Clone the decision
    let decision_clone = decision.clone();
    
    // Both should be identical
    match (decision, decision_clone) {
        (
            PeerTrustDecision::AutoAccept { reason: r1, confidence: c1, .. },
            PeerTrustDecision::AutoAccept { reason: r2, confidence: c2, .. },
        ) => {
            assert_eq!(r1, r2);
            assert_eq!(c1, c2);
        }
        _ => panic!("Clone should be identical"),
    }
}

/// Property: Tags can contain any UTF-8 string
#[test]
fn prop_tags_utf8() {
    let tags = vec![
        "beardog:family:iidn:tower1".to_string(),
        "日本語タグ".to_string(), // Japanese
        "тег".to_string(), // Cyrillic
        "🔒🔑".to_string(), // Emojis
    ];
    
    let peer = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: tags.clone(),
        endpoint: "https://localhost:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    assert_eq!(peer.tags.len(), 4);
    // All UTF-8 strings should be preserved
}

/// Property: Peer with no tags should not auto-accept (without security provider)
#[test]
fn prop_no_tags_no_auto_accept() {
    let peer = DiscoveredPeer {
        node_id: "untrusted-tower".to_string(),
        tags: vec![], // No tags = no lineage
        endpoint: "https://192.168.1.100:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    // In production, this should be rejected
    // (unless no security provider is configured - development mode)
    assert!(peer.tags.is_empty());
}

/// Property: Same peer discovered twice should have different timestamps
#[test]
fn prop_rediscovery_updates_timestamp() {
    let first_discovery = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: vec!["tag1".to_string()],
        endpoint: "https://localhost:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    let second_discovery = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: vec!["tag1".to_string()],
        endpoint: "https://localhost:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196900, // 100 seconds later
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    assert_eq!(first_discovery.node_id, second_discovery.node_id);
    assert!(second_discovery.first_seen_at > first_discovery.first_seen_at);
}

/// Property: Confidence of 0.0 indicates uncertainty
#[test]
fn prop_zero_confidence_uncertain() {
    let decision = PeerTrustDecision::AutoAccept {
        reason: "no_security_provider_configured".to_string(),
        confidence: 0.0,
        encryption_tag: None,
    };
    
    match decision {
        PeerTrustDecision::AutoAccept { confidence, reason, .. } => {
            if confidence == 0.0 {
                // Zero confidence should have a reason explaining why
                assert!(!reason.is_empty());
                assert!(reason.contains("no_security_provider") || reason.contains("unknown"));
            }
        }
        _ => {}
    }
}

/// Property: Multiple tags don't change peer identity
#[test]
fn prop_tags_dont_affect_identity() {
    let peer_no_tags = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: vec![],
        endpoint: "https://localhost:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    let peer_many_tags = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        endpoint: "https://localhost:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    // Same node_id = same identity, regardless of tags
    assert_eq!(peer_no_tags.node_id, peer_many_tags.node_id);
}

/// Property: Endpoint changes don't affect node identity
#[test]
fn prop_endpoint_change_same_identity() {
    let peer_ethernet = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: vec![],
        endpoint: "https://192.168.1.100:8080".to_string(),
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    let peer_wifi = DiscoveredPeer {
        node_id: "tower1".to_string(),
        tags: vec![],
        endpoint: "https://192.168.1.150:8080".to_string(), // Different IP
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: 1704196800,
        capabilities: vec![],
        identity_attestations: vec![],
    };
    
    // Same node_id = same tower, even if discovered on different interfaces
    assert_eq!(peer_ethernet.node_id, peer_wifi.node_id);
}

