// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for connection manager
//!
//! **v3.21.0**: Extracted from monolithic file

use super::*;
use crate::trust::peer_trust::PeerTrustDecision;

#[tokio::test]
async fn test_limited_connection_establishment() {
    let manager = ConnectionManager::new();

    let decision = PeerTrustDecision::AutoAccept {
        reason: "same_genetic_family".to_string(),
        confidence: 1.0,
        encryption_tag: Some("test_tag".to_string()),
    };

    manager
        .handle_trust_decision(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
            vec!["birdsong/*".to_string()],
            vec![], // v3.18.0: peer_tags (empty = no BTSP)
            &decision,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    let trust_level = manager.get_connection("test_peer").await;
    assert_eq!(trust_level, Some(TrustLevel::Limited));
}

#[tokio::test]
async fn test_reject_decision() {
    let manager = ConnectionManager::new();

    let decision = PeerTrustDecision::Reject {
        reason: "different_family".to_string(),
        trust_level: "none".to_string(),
    };

    manager
        .handle_trust_decision(
            "rejected_peer".to_string(),
            "http://localhost:8080".to_string(),
            vec![],
            vec![], // v3.18.0: peer_tags
            &decision,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    let trust_level = manager.get_connection("rejected_peer").await;
    assert_eq!(trust_level, None);

    let rejected = manager.get_rejected_peers().await;
    assert_eq!(rejected.get("rejected_peer"), Some(&"different_family".to_string()));
}

// ========================================================================
// Unit Tests for Peer Discovery API Methods (v3.8.0)
// ========================================================================

#[tokio::test]
async fn test_get_all_peers_empty() {
    let manager = ConnectionManager::new();

    let peers = manager.get_all_peers().await;
    assert_eq!(peers.len(), 0, "Should start with no peers");
}

#[tokio::test]
async fn test_get_all_peers_single() {
    let manager = ConnectionManager::new();

    // Establish a connection
    manager
        .establish_connection(
            "tower1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            vec![], // v3.18.0: peer_tags
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    let peers = manager.get_all_peers().await;
    assert_eq!(peers.len(), 1, "Should have 1 peer");
    assert_eq!(peers[0].peer_id, "tower1");
    assert_eq!(peers[0].endpoint, "https://192.168.1.100:8080");
    assert_eq!(peers[0].trust_level, TrustLevel::Limited);
    assert_eq!(peers[0].discovery_method, "udp_multicast");
}

#[tokio::test]
async fn test_get_all_peers_multiple() {
    let manager = ConnectionManager::new();

    // Establish multiple connections
    manager
        .establish_connection(
            "tower1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            vec![], // v3.18.0: peer_tags
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    manager
        .establish_connection(
            "tower2".to_string(),
            "https://192.168.1.101:8080".to_string(),
            vec!["data_store".to_string()],
            vec![], // v3.18.0: peer_tags
            TrustLevel::Elevated,
            "mdns".to_string(),
        )
        .await
        .unwrap();

    let peers = manager.get_all_peers().await;
    assert_eq!(peers.len(), 2, "Should have 2 peers");

    // Find each peer
    let tower1 = peers.iter().find(|p| p.peer_id == "tower1").unwrap();
    let tower2 = peers.iter().find(|p| p.peer_id == "tower2").unwrap();

    assert_eq!(tower1.trust_level, TrustLevel::Limited);
    assert_eq!(tower2.trust_level, TrustLevel::Elevated);
}

#[tokio::test]
async fn test_get_peer_count() {
    let manager = ConnectionManager::new();

    assert_eq!(manager.get_peer_count().await, 0, "Should start at 0");

    manager
        .establish_connection(
            "peer1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec![],
            vec![], // v3.18.0: peer_tags
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(manager.get_peer_count().await, 1, "Should have 1 peer");

    manager
        .establish_connection(
            "peer2".to_string(),
            "https://192.168.1.101:8080".to_string(),
            vec![],
            vec![], // v3.18.0: peer_tags
            TrustLevel::Elevated,
            "mdns".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(manager.get_peer_count().await, 2, "Should have 2 peers");
}

// ========================================================================
// v3.19.0: Test OnceCell lazy initialization
// ========================================================================

#[tokio::test]
async fn test_btsp_client_lazy_initialization() {
    let manager = ConnectionManager::new();

    // BTSP client should not be initialized yet
    // (Note: We can't directly test this without accessing internals,
    //  but we verify the behavior through connection establishment)

    // Try to establish BTSP connection (will initialize client on first use)
    let peer_tags = vec!["btsp_enabled".to_string()];

    // This will fail (no real security provider in tests)
    // But it should attempt initialization
    let _ = manager
        .establish_connection(
            "btsp_peer".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec![],
            peer_tags,
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await;

    // Even though BTSP failed, peer should be connected via HTTPS fallback
    let peers = manager.get_all_peers().await;
    assert_eq!(peers.len(), 1, "Should have 1 peer (HTTPS fallback)");
}

// ========================================================================
// v3.8.0: Test rejected peers tracking
// ========================================================================

#[tokio::test]
async fn test_rejected_peer_single() {
    let manager = ConnectionManager::new();

    let decision = PeerTrustDecision::Reject {
        reason: "different_family".to_string(),
        trust_level: "none".to_string(),
    };

    manager
        .handle_trust_decision(
            "rogue_device".to_string(),
            "https://192.168.1.200:8080".to_string(),
            vec![],
            vec![], // v3.18.0: peer_tags
            &decision,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    let rejected = manager.get_rejected_peers().await;
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected.get("rogue_device"), Some(&"different_family".to_string()));
}

#[tokio::test]
async fn test_rejected_peers_multiple() {
    let manager = ConnectionManager::new();

    let decision1 = PeerTrustDecision::Reject {
        reason: "different_family".to_string(),
        trust_level: "none".to_string(),
    };

    manager
        .handle_trust_decision(
            "rogue1".to_string(),
            "https://192.168.1.200:8080".to_string(),
            vec![],
            vec![], // v3.18.0: peer_tags
            &decision1,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    let decision2 = PeerTrustDecision::Reject {
        reason: "security_concern".to_string(),
        trust_level: "none".to_string(),
    };

    manager
        .handle_trust_decision(
            "rogue2".to_string(),
            "https://192.168.1.201:8080".to_string(),
            vec![],
            vec![], // v3.18.0: peer_tags
            &decision2,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    let decision3 = PeerTrustDecision::Reject {
        reason: "untrusted".to_string(),
        trust_level: "none".to_string(),
    };

    manager
        .handle_trust_decision(
            "rogue3".to_string(),
            "https://192.168.1.202:8080".to_string(),
            vec![],
            vec![], // v3.18.0: peer_tags
            &decision3,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    let rejected = manager.get_rejected_peers().await;
    assert_eq!(rejected.len(), 3);
}

// ========================================================================
// v3.18.0: Test BTSP connection creation at all trust levels
// ========================================================================

#[tokio::test]
async fn test_btsp_connection_all_trust_levels() {
    let manager = ConnectionManager::new();

    // Peer tags indicating BTSP support
    let peer_tags = vec!["btsp_enabled".to_string()];

    // Test Limited (Level 1)
    let result_limited = manager
        .establish_connection(
            "peer_limited_btsp".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            peer_tags.clone(),
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await;

    // Test Elevated (Level 2)
    let result_elevated = manager
        .establish_connection(
            "peer_elevated_btsp".to_string(),
            "https://192.168.1.101:8080".to_string(),
            vec!["orchestrator".to_string()],
            peer_tags.clone(),
            TrustLevel::Elevated,
            "udp_multicast".to_string(),
        )
        .await;

    // Test Highest (Level 3)
    let result_highest = manager
        .establish_connection(
            "peer_highest_btsp".to_string(),
            "https://192.168.1.102:8080".to_string(),
            vec!["orchestrator".to_string()],
            peer_tags,
            TrustLevel::Highest,
            "udp_multicast".to_string(),
        )
        .await;

    // All should succeed (will use HTTPS fallback in tests without real security provider)
    assert!(result_limited.is_ok());
    assert!(result_elevated.is_ok());
    assert!(result_highest.is_ok());

    // Verify all peers connected
    let peers = manager.get_all_peers().await;
    assert_eq!(peers.len(), 3);
}
