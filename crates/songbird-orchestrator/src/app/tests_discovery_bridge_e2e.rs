// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
#![allow(dead_code, reason = "enum variants model production trust outcomes for test coverage")]

//! Simulated end-to-end tests for the discovery → federation bridge (`discovery_bridge.rs`).
//! They mirror the production control flow (identity extraction, same-family skip, `/health`
//! gate, trust branches) without spawning orchestrators or using the network.
//!
//! Live multi-process E2E remains in `tests/discovery_e2e_test.rs` (often `#[ignore = "..."]`).

/// Minimal stand-in for a UDP-discovered peer (`songbird_discovery` shape).
struct MockDiscoveredPeer {
    version: &'static str,
    session_id: String,
    node_id: Option<String>,
    node_name: Option<String>,
    tags: Option<Vec<String>>,
    https_base: String,
}

impl MockDiscoveredPeer {
    fn https_endpoint(&self) -> String {
        self.https_base.clone()
    }
}

/// Outcome of the security / dev trust path (maps to `PeerTrustDecision` branches).
#[derive(Debug, Clone)]
enum MockSecurityTrustOutcome {
    /// `SONGBIRD_SECURITY_PROVIDER` not set — anonymous dev accept in production bridge.
    NotConfigured,
    AutoAccept {
        reason: String,
        confidence: f64,
    },
    Reject {
        reason: String,
    },
    PromptUser {
        reason: String,
        recommendation: String,
    },
    /// Trust client or `evaluate_peer_trust` failed — safe reject (`None` branch).
    EvaluationFailed,
}

/// Final per-peer result comparable to bridge + federation registration.
#[derive(Debug, PartialEq, Eq)]
enum MockBridgeOutcome {
    /// Would call `connection_manager.handle_trust_decision` + `federation_state.register_node`.
    FederationRegistered {
        node_id: String,
        connectivity_check_ran: bool,
    },
    /// Trust rejected or audit-only path; not registered as active member.
    TrustRejectedOrHeld {
        connectivity_check_ran: bool,
        would_audit_reject: bool,
    },
    /// Failed `/health` (non-2xx, error, or timeout) before trust.
    BlockedByConnectivity,
}

/// Executes one peer through the same decision tree as `start_discovery_federation_bridge`.
struct MockDiscoveryBridge;

impl MockDiscoveryBridge {
    fn extract_identity(peer: &MockDiscoveredPeer) -> (String, String) {
        if peer.version == "3.0" {
            match (&peer.node_id, &peer.node_name) {
                (Some(id), Some(name)) => (id.clone(), name.clone()),
                _ => (
                    peer.session_id.clone(),
                    format!("peer-{}", &peer.session_id[..8.min(peer.session_id.len())]),
                ),
            }
        } else {
            (
                peer.session_id.clone(),
                format!("peer-{}", &peer.session_id[..8.min(peer.session_id.len())]),
            )
        }
    }

    fn same_family(my_family: Option<&str>, peer: &MockDiscoveredPeer) -> bool {
        my_family.is_some_and(|family| {
            peer.tags.as_ref().is_some_and(|tags| {
                tags.iter().any(|tag| {
                    tag.contains(&format!(":family:{family}:"))
                        || tag.contains(&format!("family_{family}"))
                })
            })
        })
    }

    /// `health_check` is only invoked when connectivity check is required (not same-family).
    fn process_peer(
        my_family: Option<&str>,
        security: MockSecurityTrustOutcome,
        peer: &MockDiscoveredPeer,
        health_check: &mut impl FnMut() -> Result<u16, ()>,
    ) -> MockBridgeOutcome {
        let (node_id, _node_name) = Self::extract_identity(peer);
        let same_family = Self::same_family(my_family, peer);
        let skip_connectivity = same_family;

        let connectivity_ok = if skip_connectivity {
            true
        } else {
            // Production: GET `{https_endpoint()}/health` (see `discovery_bridge.rs`).
            let _health_url = format!("{}/health", peer.https_endpoint());
            match health_check() {
                Ok(status) => (200..300).contains(&status),
                Err(()) => false,
            }
        };

        if !connectivity_ok {
            return MockBridgeOutcome::BlockedByConnectivity;
        }

        let trust_result: Option<MockSecurityTrustOutcome> = match &security {
            MockSecurityTrustOutcome::NotConfigured => {
                Some(MockSecurityTrustOutcome::NotConfigured)
            }
            MockSecurityTrustOutcome::EvaluationFailed => None,
            other => Some(other.clone()),
        };

        match trust_result {
            None => MockBridgeOutcome::TrustRejectedOrHeld {
                connectivity_check_ran: !skip_connectivity,
                would_audit_reject: false,
            },
            Some(MockSecurityTrustOutcome::NotConfigured) => {
                MockBridgeOutcome::FederationRegistered {
                    node_id,
                    connectivity_check_ran: !skip_connectivity,
                }
            }
            Some(MockSecurityTrustOutcome::AutoAccept {
                ..
            }) => MockBridgeOutcome::FederationRegistered {
                node_id,
                connectivity_check_ran: !skip_connectivity,
            },
            Some(MockSecurityTrustOutcome::Reject {
                ..
            }) => MockBridgeOutcome::TrustRejectedOrHeld {
                connectivity_check_ran: !skip_connectivity,
                would_audit_reject: true,
            },
            Some(MockSecurityTrustOutcome::PromptUser {
                ..
            }) => MockBridgeOutcome::TrustRejectedOrHeld {
                connectivity_check_ran: !skip_connectivity,
                would_audit_reject: false,
            },
            Some(MockSecurityTrustOutcome::EvaluationFailed) => {
                MockBridgeOutcome::TrustRejectedOrHeld {
                    connectivity_check_ran: !skip_connectivity,
                    would_audit_reject: false,
                }
            }
        }
    }
}

/// In-memory “API” layer: peers exposed after a successful bridge registration.
#[derive(Default)]
struct MockDiscoveryApiState {
    registered_peers: Vec<String>,
}

impl MockDiscoveryApiState {
    fn list_peers(&self) -> &[String] {
        &self.registered_peers
    }

    fn register_from_bridge(&mut self, node_id: String) {
        self.registered_peers.push(node_id);
    }

    fn ping_peer(&self, node_id: &str) -> Result<(), &'static str> {
        if self.registered_peers.iter().any(|id| id == node_id) {
            Ok(())
        } else {
            Err("unknown peer")
        }
    }
}

#[tokio::test]
async fn test_e2e_same_family_peer_discovery() {
    let peer = MockDiscoveredPeer {
        version: "3.0",
        session_id: "sess-ignored".into(),
        node_id: Some("tower-a".into()),
        node_name: Some("Tower A".into()),
        tags: Some(vec!["crypto:family:nat0:tower-a".into()]),
        https_base: "https://10.0.0.5:8443".into(),
    };

    let mut calls = 0u8;
    let mut check = || {
        calls += 1;
        Ok(503)
    };

    let out = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::NotConfigured,
        &peer,
        &mut check,
    );

    assert_eq!(calls, 0, "same-family peer must skip HTTPS /health");
    assert_eq!(
        out,
        MockBridgeOutcome::FederationRegistered {
            node_id: "tower-a".into(),
            connectivity_check_ran: false,
        }
    );
}

#[tokio::test]
async fn test_e2e_different_family_peer_discovery() {
    let peer = MockDiscoveredPeer {
        version: "3.0",
        session_id: "sess1".into(),
        node_id: Some("foreign".into()),
        node_name: Some("Foreign".into()),
        tags: Some(vec!["crypto:family:other:node".into()]),
        https_base: "https://192.168.1.10:8443".into(),
    };

    let mut calls = 0u8;
    let mut check = || {
        calls += 1;
        Ok(200)
    };

    let out = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::Reject {
            reason: "no_genetic_lineage".into(),
        },
        &peer,
        &mut check,
    );

    assert_eq!(calls, 1, "non-family peer must run connectivity check");
    assert_eq!(
        out,
        MockBridgeOutcome::TrustRejectedOrHeld {
            connectivity_check_ran: true,
            would_audit_reject: true,
        }
    );
}

#[tokio::test]
async fn test_e2e_trust_evaluation_with_security_provider() {
    let peer = MockDiscoveredPeer {
        version: "3.0",
        session_id: "s2".into(),
        node_id: Some("trusted-node".into()),
        node_name: Some("Trusted".into()),
        tags: Some(vec!["crypto:family:nat0:x".into()]),
        https_base: "https://127.0.0.1:9".into(),
    };

    let mut check = || Ok(500);

    let out_skip = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::AutoAccept {
            reason: "lineage_ok".into(),
            confidence: 1.0,
        },
        &peer,
        &mut check,
    );

    assert_eq!(
        out_skip,
        MockBridgeOutcome::FederationRegistered {
            node_id: "trusted-node".into(),
            connectivity_check_ran: false,
        },
        "same-family skips /health even if mock would return 500"
    );

    let peer2 = MockDiscoveredPeer {
        version: "3.0",
        session_id: "s3".into(),
        node_id: Some("lineage-fail".into()),
        node_name: Some("Bad".into()),
        tags: Some(vec!["crypto:family:other:y".into()]),
        https_base: "https://10.0.0.2:8443".into(),
    };

    let mut check_ok = || Ok(200);
    let out_reject = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::Reject {
            reason: "invalid_lineage".into(),
        },
        &peer2,
        &mut check_ok,
    );

    assert_eq!(
        out_reject,
        MockBridgeOutcome::TrustRejectedOrHeld {
            connectivity_check_ran: true,
            would_audit_reject: true,
        }
    );
}

#[tokio::test]
async fn test_e2e_discovery_to_api_flow() {
    let peer = MockDiscoveredPeer {
        version: "3.0",
        session_id: "s4".into(),
        node_id: Some("api-peer".into()),
        node_name: Some("API Peer".into()),
        tags: Some(vec!["crypto:family:nat0:z".into()]),
        https_base: "https://10.0.0.20:8443".into(),
    };

    let mut noop_health = || Ok(599);

    let outcome = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::NotConfigured,
        &peer,
        &mut noop_health,
    );

    let mut api = MockDiscoveryApiState::default();
    if let MockBridgeOutcome::FederationRegistered {
        node_id,
        ..
    } = outcome
    {
        api.register_from_bridge(node_id);
    } else {
        panic!("expected registration path, got {outcome:?}");
    }

    assert_eq!(api.list_peers(), &["api-peer"]);
    assert!(api.ping_peer("api-peer").is_ok());
    assert!(api.ping_peer("missing").is_err());
}

#[tokio::test]
async fn test_e2e_connectivity_check_failure_handling() {
    let peer = MockDiscoveredPeer {
        version: "3.0",
        session_id: "s5".into(),
        node_id: Some("unreachable".into()),
        node_name: Some("Unreachable".into()),
        tags: Some(vec!["crypto:family:wan:peer".into()]),
        https_base: "https://192.0.2.1:8443".into(),
    };

    let mut timeout_or_err = || Err(());

    let out = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::AutoAccept {
            reason: "would_accept".into(),
            confidence: 1.0,
        },
        &peer,
        &mut timeout_or_err,
    );

    assert_eq!(out, MockBridgeOutcome::BlockedByConnectivity);

    let mut bad_status = || Ok(503);
    let out2 = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::AutoAccept {
            reason: "would_accept".into(),
            confidence: 1.0,
        },
        &peer,
        &mut bad_status,
    );
    assert_eq!(out2, MockBridgeOutcome::BlockedByConnectivity);
}

#[tokio::test]
async fn test_e2e_legacy_v2_identity_uses_session_not_node_fields() {
    let peer = MockDiscoveredPeer {
        version: "2.1",
        session_id: "legacy-sess-abcdef".into(),
        node_id: Some("ignored-id".into()),
        node_name: Some("Ignored Name".into()),
        tags: Some(vec!["crypto:family:nat0:legacy".into()]),
        https_base: "https://10.0.0.1:8443".into(),
    };

    let (id, name) = MockDiscoveryBridge::extract_identity(&peer);
    assert_eq!(id, "legacy-sess-abcdef");
    assert_eq!(name, "peer-legacy-s");

    let mut calls = 0u8;
    let mut health = || {
        calls += 1;
        Ok(200)
    };
    let out = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::NotConfigured,
        &peer,
        &mut health,
    );
    assert_eq!(calls, 0, "same-family skips connectivity");
    assert_eq!(
        out,
        MockBridgeOutcome::FederationRegistered {
            node_id: "legacy-sess-abcdef".into(),
            connectivity_check_ran: false,
        }
    );
}

#[tokio::test]
async fn test_e2e_v3_partial_node_metadata_falls_back_to_session() {
    let peer = MockDiscoveredPeer {
        version: "3.0",
        session_id: "partial-sess-xyz".into(),
        node_id: Some("only-id".into()),
        node_name: None,
        tags: Some(vec!["crypto:family:nat0:partial".into()]),
        https_base: "https://10.0.0.2:8443".into(),
    };

    let (id, name) = MockDiscoveryBridge::extract_identity(&peer);
    assert_eq!(id, "partial-sess-xyz");
    assert_eq!(name, "peer-partial-");

    let mut calls = 0u8;
    let mut health = || {
        calls += 1;
        Ok(200)
    };
    let out = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::AutoAccept {
            reason: "ok".into(),
            confidence: 1.0,
        },
        &peer,
        &mut health,
    );
    assert_eq!(calls, 0);
    assert_eq!(
        out,
        MockBridgeOutcome::FederationRegistered {
            node_id: "partial-sess-xyz".into(),
            connectivity_check_ran: false,
        }
    );
}

#[tokio::test]
async fn test_e2e_prompt_user_trust_held_without_audit_reject() {
    let peer = MockDiscoveredPeer {
        version: "3.0",
        session_id: "prompt-s".into(),
        node_id: Some("node-p".into()),
        node_name: Some("Prompt".into()),
        tags: Some(vec!["crypto:family:nat0:p".into()]),
        https_base: "https://10.0.0.3:8443".into(),
    };

    let mut health = || Ok(200);
    let out = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::PromptUser {
            reason: "unknown_peer".into(),
            recommendation: "Review".into(),
        },
        &peer,
        &mut health,
    );

    assert_eq!(
        out,
        MockBridgeOutcome::TrustRejectedOrHeld {
            connectivity_check_ran: false,
            would_audit_reject: false,
        }
    );
}

#[tokio::test]
async fn test_e2e_trust_evaluation_failed_blocks_federation() {
    let peer = MockDiscoveredPeer {
        version: "3.0",
        session_id: "fail-s".into(),
        node_id: Some("node-f".into()),
        node_name: Some("Fail".into()),
        tags: Some(vec!["crypto:family:other:f".into()]),
        https_base: "https://10.0.0.4:8443".into(),
    };

    let mut health = || Ok(200);
    let out = MockDiscoveryBridge::process_peer(
        Some("nat0"),
        MockSecurityTrustOutcome::EvaluationFailed,
        &peer,
        &mut health,
    );

    assert_eq!(
        out,
        MockBridgeOutcome::TrustRejectedOrHeld {
            connectivity_check_ran: true,
            would_audit_reject: false,
        }
    );
}

#[tokio::test]
async fn test_e2e_no_family_env_forces_connectivity_check() {
    let peer = MockDiscoveredPeer {
        version: "3.0",
        session_id: "nofam-s".into(),
        node_id: Some("n1".into()),
        node_name: Some("NoFam".into()),
        tags: Some(vec!["crypto:family:nat0:still-tagged".into()]),
        https_base: "https://10.0.0.6:8443".into(),
    };

    let mut calls = 0u8;
    let mut health = || {
        calls += 1;
        Ok(200)
    };

    let out = MockDiscoveryBridge::process_peer(
        None,
        MockSecurityTrustOutcome::NotConfigured,
        &peer,
        &mut health,
    );

    assert_eq!(calls, 1, "without MY_FAMILY, same-family is false → /health path runs");
    assert_eq!(
        out,
        MockBridgeOutcome::FederationRegistered {
            node_id: "n1".into(),
            connectivity_check_ran: true,
        }
    );
}
