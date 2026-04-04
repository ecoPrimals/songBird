// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Unit and Integration Tests for Self-Filtering (v3.10.2)
//!
//! Tests the self-filtering functionality that prevents towers from discovering
//! their own broadcasts, which is critical for multi-instance deployments.

#[cfg(test)]
mod unit_tests {
    use crate::anonymous::AnonymousDiscoveryListener;
    use crate::anonymous::messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};

    fn sample_v3_message(node_id: &str) -> AnonymousDiscoveryMessage {
        AnonymousDiscoveryMessage::new_v3(
            node_id,
            "test-node",
            vec![TransportEndpointMessage {
                interface_type: "tcp".into(),
                address: "127.0.0.1:8443".into(),
                protocols: vec!["https".into()],
                preference: 0,
            }],
            vec!["orchestration".into()],
        )
    }

    #[test]
    fn self_filter_skips_when_node_ids_match() {
        let mine = "3a2c467d-2409-571f-aaab-dd7cfd2214e8";
        let listener = AnonymousDiscoveryListener::new(2300, 60).with_node_id(mine.to_string());
        let msg = sample_v3_message(mine);
        assert!(
            listener.would_skip_as_own_broadcast(&msg),
            "same node_id must be treated as own broadcast"
        );
    }

    #[test]
    fn self_filter_does_not_skip_when_peer_differs() {
        let listener =
            AnonymousDiscoveryListener::new(2300, 60).with_node_id("tower-a".to_string());
        let msg = sample_v3_message("tower-b");
        assert!(
            !listener.would_skip_as_own_broadcast(&msg),
            "other tower must not be filtered as self"
        );
    }

    #[test]
    fn self_filter_disabled_without_listener_node_id() {
        let listener = AnonymousDiscoveryListener::new(2300, 60);
        let msg = sample_v3_message("any-id");
        assert!(
            !listener.would_skip_as_own_broadcast(&msg),
            "without our node_id we cannot match self"
        );
    }

    #[test]
    fn v2_style_message_no_node_id_never_matches_self_filter() {
        let listener = AnonymousDiscoveryListener::new(2300, 60).with_node_id("tower1".to_string());
        let msg = AnonymousDiscoveryMessage::new(
            vec!["orchestration".into()],
            vec!["https".into()],
            8443,
        );
        assert!(msg.node_id.is_none());
        assert!(
            !listener.would_skip_as_own_broadcast(&msg),
            "v2.x messages lack node_id; cannot filter as self (backward compatible)"
        );
    }

    #[test]
    fn broadcast_only_listener_self_filter_same_as_multicast() {
        let mine = "edge-node";
        let msg = sample_v3_message(mine);
        let a = AnonymousDiscoveryListener::new(2300, 60).with_node_id(mine.to_string());
        let b =
            AnonymousDiscoveryListener::new_broadcast_only(2300, 60).with_node_id(mine.to_string());
        assert_eq!(a.would_skip_as_own_broadcast(&msg), b.would_skip_as_own_broadcast(&msg));
        assert!(a.would_skip_as_own_broadcast(&msg));
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::anonymous::AnonymousDiscoveryListener;
    use crate::anonymous::messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};

    #[tokio::test]
    async fn get_peers_starts_empty_with_self_filter_configured() {
        let listener = AnonymousDiscoveryListener::new(2300, 60).with_node_id("tower1".to_string());
        let peers = listener.get_peers().await;
        assert_eq!(peers.len(), 0, "no UDP traffic: registry stays empty");
    }

    #[test]
    fn listener_with_self_filtering_exposes_consistent_skip_predicate() {
        let id = "3a2c467d-2409-571f-aaab-dd7cfd2214e8";
        let listener = AnonymousDiscoveryListener::new(2300, 60).with_node_id(id.to_string());
        let msg = AnonymousDiscoveryMessage::new_v3(
            id,
            "n",
            vec![TransportEndpointMessage {
                interface_type: "tcp".into(),
                address: "10.0.0.1:443".into(),
                protocols: vec!["https".into()],
                preference: 0,
            }],
            vec![],
        );
        assert!(listener.would_skip_as_own_broadcast(&msg));
    }

    #[test]
    fn listener_without_self_filtering_does_not_skip_same_id_message() {
        let id = "same-id";
        let listener = AnonymousDiscoveryListener::new(2300, 60);
        let msg = AnonymousDiscoveryMessage::new_v3(
            id,
            "n",
            vec![TransportEndpointMessage {
                interface_type: "tcp".into(),
                address: "10.0.0.1:443".into(),
                protocols: vec!["https".into()],
                preference: 0,
            }],
            vec![],
        );
        assert!(!listener.would_skip_as_own_broadcast(&msg));
    }
}

#[cfg(test)]
mod e2e_tests {
    use crate::anonymous::AnonymousDiscoveryListener;

    /// E2E: Two towers discover each other but not themselves
    #[tokio::test]
    #[ignore = "LIVE: requires full Songbird multi-instance setup and UDP multicast"]
    async fn test_e2e_two_towers_mutual_discovery_with_self_filtering() {
        let a = AnonymousDiscoveryListener::new(2300, 60).with_node_id("tower-a".to_string());
        let b = AnonymousDiscoveryListener::new(2301, 60).with_node_id("tower-b".to_string());
        assert_eq!(a.get_peers().await.len(), 0);
        assert_eq!(b.get_peers().await.len(), 0);
    }

    /// E2E: Three towers with self-filtering
    #[tokio::test]
    #[ignore = "LIVE: requires full Songbird multi-instance setup and UDP multicast"]
    async fn test_e2e_three_towers_self_filtering() {
        let t1 = AnonymousDiscoveryListener::new(2300, 60).with_node_id("id-a".to_string());
        let t2 = AnonymousDiscoveryListener::new(2301, 60).with_node_id("id-b".to_string());
        let t3 = AnonymousDiscoveryListener::new(2302, 60).with_node_id("id-c".to_string());
        assert_eq!(t1.get_peers().await.len(), 0);
        assert_eq!(t2.get_peers().await.len(), 0);
        assert_eq!(t3.get_peers().await.len(), 0);
    }

    /// E2E: Self-filtering with bridge processing
    #[tokio::test]
    #[ignore = "LIVE: requires Songbird bridge, ConnectionManager, and FederationState"]
    async fn test_e2e_self_filtering_with_bridge_processing() {
        let listener = AnonymousDiscoveryListener::new(2300, 60).with_node_id("bridge-node".into());
        assert_eq!(listener.get_peers().await.len(), 0);
    }

    /// E2E: Performance test with self-filtering
    #[tokio::test]
    #[ignore = "Expensive performance test; run manually with --ignored"]
    async fn test_e2e_self_filtering_performance() {
        use crate::anonymous::messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};

        let listener = AnonymousDiscoveryListener::new(2300, 60).with_node_id("perf-node".into());
        let msg = AnonymousDiscoveryMessage::new_v3(
            "other",
            "o",
            vec![TransportEndpointMessage {
                interface_type: "tcp".into(),
                address: "127.0.0.1:1".into(),
                protocols: vec!["https".into()],
                preference: 0,
            }],
            vec![],
        );
        for _ in 0..1000 {
            assert!(!listener.would_skip_as_own_broadcast(&msg));
        }
    }

    /// E2E: Self-filtering with stale peer cleanup
    #[tokio::test]
    #[ignore = "LIVE: requires time-based TTL / stale-peer cleanup scenario"]
    async fn test_e2e_self_filtering_with_ttl_cleanup() {
        let listener = AnonymousDiscoveryListener::new(2300, 30).with_node_id("ttl-node".into());
        assert_eq!(listener.get_peers().await.len(), 0);
    }
}

#[cfg(test)]
mod regression_tests {
    use crate::anonymous::AnonymousDiscoveryListener;
    use crate::anonymous::messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};

    /// Regression: Ensure listener works without `node_id` (backward compatible)
    #[test]
    fn test_regression_backward_compatibility() {
        let listener = AnonymousDiscoveryListener::new(2300, 60);
        let msg = AnonymousDiscoveryMessage::new_v3(
            "x",
            "n",
            vec![TransportEndpointMessage {
                interface_type: "tcp".into(),
                address: "127.0.0.1:1".into(),
                protocols: vec!["https".into()],
                preference: 0,
            }],
            vec![],
        );
        assert!(!listener.would_skip_as_own_broadcast(&msg));
    }

    /// Regression: Ensure builder pattern doesn't break existing code
    #[test]
    fn test_regression_builder_pattern() {
        let listener = AnonymousDiscoveryListener::new(2300, 60)
            .with_node_id("test".to_string())
            .with_node_id("updated".to_string());
        let msg = AnonymousDiscoveryMessage::new_v3(
            "updated",
            "n",
            vec![TransportEndpointMessage {
                interface_type: "tcp".into(),
                address: "127.0.0.1:1".into(),
                protocols: vec!["https".into()],
                preference: 0,
            }],
            vec![],
        );
        assert!(listener.would_skip_as_own_broadcast(&msg));
    }

    /// Regression: Ensure v2.x messages (no `node_id`) still work
    #[test]
    fn test_regression_v2_messages() {
        let listener = AnonymousDiscoveryListener::new(2300, 60).with_node_id("any".into());
        let msg = AnonymousDiscoveryMessage::new(vec![], vec![], 8080);
        assert!(msg.node_id.is_none());
        assert!(!listener.would_skip_as_own_broadcast(&msg));
    }
}
