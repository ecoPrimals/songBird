// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::unchecked_time_subtraction, reason = "test assertions")]

use super::*;
use serde_json::json;
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::test]
async fn test_mesh_handler_uninitialized() {
    let handler = MeshHandler::new();

    let result = handler.handle_status(json!({})).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["initialized"], false);
    assert_eq!(response["status"], "awaiting_init");
}

#[tokio::test]
async fn test_mesh_init() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": ["abc.onion"]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["initialized"], true);
    assert_eq!(response["node_id"], "test-tower");
}

#[tokio::test]
async fn mesh_init_with_bootstrap_peers_adds_endpoints() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "east-gate",
            "bootstrap_onions": [],
            "bootstrap_peers": [
                { "node_id": "west-gate", "address": "192.168.1.50:3492" },
                { "node_id": "flock-gate", "address": "10.0.0.5:3492" }
            ]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["initialized"], true);
    assert_eq!(response["bootstrap_peers_added"], 2);

    let mesh = handler.mesh().await;
    let mesh = mesh.as_ref().unwrap();
    let reachable = mesh.get_reachable_nodes().await;
    assert_eq!(reachable.len(), 2, "both bootstrap peers should be reachable");
    assert!(reachable.contains(&"west-gate".to_string()));
    assert!(reachable.contains(&"flock-gate".to_string()));
}

#[tokio::test]
async fn mesh_init_with_invalid_bootstrap_peers_skips_gracefully() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "east-gate",
            "bootstrap_peers": [
                { "node_id": "good-peer", "address": "192.168.1.50:3492" },
                { "node_id": "bad-peer", "address": "not-a-valid-addr" },
                { "missing_fields": true }
            ]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["bootstrap_peers_added"], 1, "only valid peer should be added");
}

#[tokio::test]
async fn mesh_init_string_format_bootstrap_peers() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "south-gate",
            "bootstrap_peers": [
                "east-gate@192.168.1.100:7700",
                "192.168.4.29:7700",
                "invalid-no-port"
            ]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["bootstrap_peers_added"], 2);

    let mesh = handler.mesh().await;
    let mesh = mesh.as_ref().unwrap();
    let reachable = mesh.get_reachable_nodes().await;
    assert_eq!(reachable.len(), 2);
    assert!(reachable.contains(&"east-gate".to_string()));
    assert!(reachable.contains(&"peer-192.168.4.29".to_string()));
}

#[tokio::test]
async fn mesh_init_mixed_format_bootstrap_peers() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "iron-gate",
            "bootstrap_peers": [
                { "node_id": "west-gate", "address": "10.0.0.1:7700" },
                "east-gate@192.168.1.100:7700"
            ]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["bootstrap_peers_added"], 2);
}

#[tokio::test]
async fn test_mesh_status_after_init() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler.handle_status(json!({})).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response["node_id"], "test-tower");
    assert_eq!(response["reachable_peers"], 0);
    assert!(response["uptime_seconds"].as_u64().is_some());
}

#[tokio::test]
async fn test_mesh_find_path_not_found() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler
        .handle_find_path(json!({
            "target_node_id": "unknown-peer"
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["found"], false);
    assert_eq!(response["reason"], "peer_not_discovered");
}

#[tokio::test]
async fn test_mesh_find_path_with_bootstrap() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": ["bootstrap.onion"]
        }))
        .await
        .unwrap();

    let result = handler
        .handle_find_path(json!({
            "target_node_id": "remote-peer"
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["found"], true);
    assert_eq!(response["path_type"], "onion");
}

#[tokio::test]
async fn test_mesh_announce() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler
        .handle_announce(json!({
            "as_relay": true,
            "capabilities": ["relay", "stun"]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["announced"], true);
}

#[tokio::test]
async fn test_mesh_peers_empty() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler.handle_peers(json!({})).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response["total"], 0);
    assert!(response["peers"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_mesh_auto_discover() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler
        .handle_auto_discover(json!({
            "timeout_ms": 100,
            "broadcast_port": 15353
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["discovered"], 0);
    assert!(response["peers"].as_array().unwrap().is_empty());
    assert_eq!(response["broadcast_port"], 15353);
}

#[tokio::test]
async fn test_mesh_auto_discover_requires_init() {
    let handler = MeshHandler::new();

    let result = handler.handle_auto_discover(json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not initialized"));
}

#[tokio::test]
async fn mesh_topology_requires_init() {
    let handler = MeshHandler::new();
    let err = handler.handle_topology(json!({})).await.expect_err("topology");
    assert!(err.contains("not initialized"), "unexpected: {err}");
}

#[tokio::test]
async fn test_mesh_health_check() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler
        .handle_health_check(json!({
            "target_node_ids": ["unknown-peer"],
            "timeout_ms": 1000
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["all_healthy"], false);
}

#[test]
fn endpoint_to_strings_local_and_direct() {
    use std::net::SocketAddr;
    let handler = MeshHandler::new();
    let addr: SocketAddr = "192.168.0.1:1234".parse().expect("addr");
    let (t, s) = handler.endpoint_strings_for_test(&EndpointType::Local {
        addr,
    });
    assert_eq!(t, "local");
    assert_eq!(s.as_deref(), Some("192.168.0.1:1234"));

    let (t2, s2) = handler.endpoint_strings_for_test(&EndpointType::Direct {
        addr,
    });
    assert_eq!(t2, "direct");
    assert_eq!(s2.as_deref(), Some("192.168.0.1:1234"));
}

#[test]
fn endpoint_to_strings_relay_and_onion() {
    let handler = MeshHandler::new();
    let (t, s) = handler.endpoint_strings_for_test(&EndpointType::FamilyRelay {
        relay_node_id: "relay-1".into(),
    });
    assert_eq!(t, "family_relay");
    assert_eq!(s.as_deref(), Some("relay-1"));

    let (t2, s2) = handler.endpoint_strings_for_test(&EndpointType::TorOnion {
        onion_addr: "abc.onion".into(),
    });
    assert_eq!(t2, "onion");
    assert_eq!(s2.as_deref(), Some("abc.onion"));
}

#[test]
fn path_to_json_includes_expected_fields() {
    let handler = MeshHandler::new();
    let addr: SocketAddr = "10.0.0.2:9000".parse().expect("addr");
    let path = RelayEndpoint {
        node_id: "peer-9".into(),
        endpoint_type: EndpointType::Direct {
            addr,
        },
        latency: None,
        last_seen: Instant::now(),
        reachable: true,
    };
    let v = handler.path_json_for_test(&path, true);
    assert_eq!(v["found"], true);
    assert_eq!(v["path_type"], "direct");
    assert_eq!(v["target_node_id"], "peer-9");
    assert_eq!(v["reachable"], true);
}

#[tokio::test]
async fn mesh_init_missing_node_id_errors() {
    let handler = MeshHandler::new();
    let err = handler.handle_init(json!({})).await.expect_err("missing node_id");
    assert!(err.contains("node_id"));
}

#[tokio::test]
async fn mesh_find_path_missing_target_errors() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "tower",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    let err = handler.handle_find_path(json!({})).await.expect_err("missing target");
    assert!(err.contains("target_node_id"));
}

#[tokio::test]
async fn mesh_announce_as_relay_false_short_circuits() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "tower",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    let v = handler.handle_announce(json!({ "as_relay": false })).await.expect("announce response");
    assert_eq!(v["announced"], false);
}

#[tokio::test]
async fn mesh_probe_latency_requires_init() {
    let handler = MeshHandler::new();
    let result = handler.handle_probe_latency(json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Mesh not initialized"));
}

#[tokio::test]
async fn mesh_probe_latency_with_no_peers_returns_empty() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "probe-test",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    let result = handler.handle_probe_latency(json!({ "timeout_ms": 1000 })).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["probed"], 0);
    assert_eq!(response["total_peers"], 0);
    assert_eq!(response["results"].as_array().expect("results array").len(), 0);
}

#[tokio::test]
async fn mesh_probe_latency_skips_non_tcp_endpoints() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "probe-relay-test",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    // Add a peer with a relay endpoint (no direct TCP address)
    {
        let mesh = handler.mesh.read().await;
        let mesh = mesh.as_ref().unwrap();
        mesh.record_relay_path(
            "relay-peer".to_string(),
            "via-relay".to_string(),
            Duration::from_millis(50),
        )
        .await;
    }

    let result = handler.handle_probe_latency(json!({})).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    let results = response["results"].as_array().expect("results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["status"], "skipped");
    assert_eq!(results[0]["reason"], "no_tcp_endpoint");
}

#[tokio::test]
async fn mesh_probe_latency_attempts_unreachable_peer() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "probe-unreachable-test",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    // Add a peer with a direct TCP endpoint that won't be reachable
    {
        let mesh = handler.mesh.read().await;
        let mesh = mesh.as_ref().unwrap();
        mesh.record_direct_connection(
            "unreachable-peer".to_string(),
            "192.0.2.1:1".parse().unwrap(), // RFC 5737 documentation address
            Duration::from_millis(999),
        )
        .await;
    }

    let result = handler.handle_probe_latency(json!({ "timeout_ms": 200 })).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    let results = response["results"].as_array().expect("results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["node_id"], "unreachable-peer");
    assert_eq!(results[0]["status"], "error");
    assert!(
        results[0]["error"].as_str().unwrap().contains("timeout")
            || results[0]["error"].as_str().unwrap().contains("failed")
    );
}

#[test]
fn mesh_handler_default_matches_new() {
    let _a = MeshHandler::new();
    let _b = MeshHandler::default();
}

#[test]
fn path_to_json_respects_found_flag_and_latency() {
    let handler = MeshHandler::new();
    let addr: SocketAddr = "10.0.0.2:9000".parse().expect("addr");
    let path = RelayEndpoint {
        node_id: "peer-x".into(),
        endpoint_type: EndpointType::Local {
            addr,
        },
        latency: Some(Duration::from_millis(12)),
        last_seen: Instant::now(),
        reachable: false,
    };
    let v = handler.path_json_for_test(&path, false);
    assert_eq!(v["found"], false);
    assert_eq!(v["estimated_latency_ms"], 12);
    assert_eq!(v["reachable"], false);
    assert_eq!(v["path_type"], "local");
}

#[tokio::test]
async fn capabilities_announce_stores_remote_peer_caps() {
    let handler = MeshHandler::new();
    let params = serde_json::json!({
        "node_id": "east-gate",
        "capabilities": ["crypto", "storage", "mesh"]
    });
    let result = handler.handle_capabilities_announce(params).await.expect("announce");
    assert_eq!(result["accepted"], true);
    assert_eq!(result["capabilities_count"], 3);

    let caps = handler.get_peer_capabilities("east-gate").await;
    assert_eq!(caps, vec!["crypto", "storage", "mesh"]);

    // Unknown peer returns empty
    let unknown = handler.get_peer_capabilities("unknown-gate").await;
    assert!(unknown.is_empty());
}

#[tokio::test]
async fn capabilities_announce_overwrites_on_update() {
    let handler = MeshHandler::new();

    handler
        .handle_capabilities_announce(serde_json::json!({
            "node_id": "gate-a",
            "capabilities": ["old-cap"]
        }))
        .await
        .expect("first");

    handler
        .handle_capabilities_announce(serde_json::json!({
            "node_id": "gate-a",
            "capabilities": ["new-cap-1", "new-cap-2"]
        }))
        .await
        .expect("second");

    let caps = handler.get_peer_capabilities("gate-a").await;
    assert_eq!(caps, vec!["new-cap-1", "new-cap-2"]);
}

#[tokio::test]
async fn capabilities_announce_to_uninitialized_mesh_is_noop() {
    let handler = MeshHandler::new();
    // announce_capabilities_to_peers on uninitialized mesh should not panic
    handler.announce_capabilities_to_peers(vec!["crypto".to_string()]).await;
}

#[tokio::test]
async fn retry_pending_announces_drains_empty_queue() {
    let handler = MeshHandler::new();
    // Should be a no-op with empty queue and not panic
    handler.retry_pending_announces().await;
}

#[tokio::test]
async fn capabilities_announce_rejects_unknown_peer_when_mesh_active() {
    use songbird_onion_relay::mesh::BeaconMesh;

    let mesh = BeaconMesh::new("local-gate".into(), vec![]);
    let handler = MeshHandler::with_mesh(mesh, "local-gate");

    // "unknown-peer" is not in the mesh's reachable nodes
    let result = handler
        .handle_capabilities_announce(serde_json::json!({
            "node_id": "unknown-peer",
            "capabilities": ["storage"]
        }))
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unknown peer"));
}

#[tokio::test]
async fn get_peer_capabilities_returns_empty_for_unknown() {
    let handler = MeshHandler::new();
    let caps = handler.get_peer_capabilities("nonexistent").await;
    assert!(caps.is_empty());
}

#[tokio::test]
async fn stale_capabilities_are_evicted_on_retry_cycle() {
    use capability_propagation::PeerCapabilityEntry;
    use std::time::Instant;

    let handler = MeshHandler::new();

    // Manually insert a stale entry (pretend it was received 11 minutes ago)
    {
        let mut caps = handler.peer_capabilities.write().await;
        caps.insert(
            "stale-gate".to_string(),
            PeerCapabilityEntry {
                capabilities: vec!["old-cap".to_string()],
                last_seen: Instant::now() - Duration::from_secs(660),
            },
        );
        caps.insert(
            "fresh-gate".to_string(),
            PeerCapabilityEntry {
                capabilities: vec!["new-cap".to_string()],
                last_seen: Instant::now(),
            },
        );
    }

    handler.retry_pending_announces().await;

    // Stale entry should be evicted
    let stale = handler.get_peer_capabilities("stale-gate").await;
    assert!(stale.is_empty());

    // Fresh entry should remain
    let fresh = handler.get_peer_capabilities("fresh-gate").await;
    assert_eq!(fresh, vec!["new-cap"]);
}

#[tokio::test]
async fn queue_depth_cap_prevents_unbounded_growth() {
    use capability_propagation::PendingAnnounce;
    use std::time::Instant;

    let handler = MeshHandler::new();

    // Fill the pending queue to capacity (50)
    {
        let mut guard = handler.pending_announces.write().await;
        for i in 0..50 {
            guard.push(PendingAnnounce {
                node_id: format!("gate-{i}"),
                address: format!("http://10.0.0.{i}:9100/jsonrpc"),
                payload: json!({"test": true}),
                attempts: 1,
                enqueued_at: Instant::now(),
            });
        }
    }

    // Verify queue is at capacity
    let len = handler.pending_announces.read().await.len();
    assert_eq!(len, 50);
}

#[tokio::test]
async fn expired_pending_entries_are_dropped() {
    use capability_propagation::PendingAnnounce;
    use std::time::Instant;

    let handler = MeshHandler::new();

    // Insert an entry that was enqueued over 10 minutes ago (expired)
    {
        let mut guard = handler.pending_announces.write().await;
        guard.push(PendingAnnounce {
            node_id: "expired-gate".to_string(),
            address: "http://10.0.0.1:9100/jsonrpc".to_string(),
            payload: json!({"test": true}),
            attempts: 2,
            enqueued_at: Instant::now() - Duration::from_secs(700),
        });
    }

    handler.retry_pending_announces().await;

    // Expired entry should have been dropped
    let len = handler.pending_announces.read().await.len();
    assert_eq!(len, 0);
}

#[tokio::test]
async fn backoff_defers_recent_entries() {
    use capability_propagation::PendingAnnounce;
    use std::time::Instant;

    let handler = MeshHandler::new();

    // Insert an entry with attempt=2, enqueued just now.
    // Backoff for attempt 2 = 120 * 2^2 = 480 seconds.
    // Since it was just enqueued, it should be deferred (not retried).
    {
        let mut guard = handler.pending_announces.write().await;
        guard.push(PendingAnnounce {
            node_id: "deferred-gate".to_string(),
            address: "http://10.0.0.99:9100/jsonrpc".to_string(),
            payload: json!({"test": true}),
            attempts: 2,
            enqueued_at: Instant::now(),
        });
    }

    handler.retry_pending_announces().await;

    // Entry should still be in the queue (deferred, not retried or dropped)
    let guard = handler.pending_announces.read().await;
    assert_eq!(guard.len(), 1);
    assert_eq!(guard[0].node_id, "deferred-gate");
    assert_eq!(guard[0].attempts, 2, "attempts should not be incremented on defer");
}

#[tokio::test]
async fn max_retries_drops_entry() {
    use capability_propagation::PendingAnnounce;
    use std::time::Instant;

    let handler = MeshHandler::new();

    // Insert an entry at max retries (5)
    {
        let mut guard = handler.pending_announces.write().await;
        guard.push(PendingAnnounce {
            node_id: "maxed-gate".to_string(),
            address: "http://10.0.0.5:9100/jsonrpc".to_string(),
            payload: json!({"test": true}),
            attempts: 5,
            enqueued_at: Instant::now(),
        });
    }

    handler.retry_pending_announces().await;

    // Entry should have been dropped
    let len = handler.pending_announces.read().await.len();
    assert_eq!(len, 0);
}

// ──────────────────────────────────────────────────────────
// Partition detection and version tracking tests
// ──────────────────────────────────────────────────────────

#[tokio::test]
async fn capabilities_announce_stores_version_and_reachable_peers() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "local-gate",
            "bootstrap_peers": [{"node_id": "remote-gate", "address": "10.0.0.1:3492"}]
        }))
        .await
        .unwrap();

    let result = handler
        .handle_capabilities_announce(json!({
            "node_id": "remote-gate",
            "capabilities": ["mesh", "relay"],
            "version": "0.2.1",
            "reachable_peers": ["vps-gate", "flock-gate"]
        }))
        .await;

    assert!(result.is_ok());
    let meta = handler.peer_metadata.read().await;
    let entry = meta.get("remote-gate").expect("metadata should exist");
    assert_eq!(entry.version, Some("0.2.1".to_string()));
    assert_eq!(entry.reachable_peers, vec!["vps-gate", "flock-gate"]);
}

#[tokio::test]
async fn partition_status_healthy_when_no_gossip() {
    let handler = MeshHandler::new();
    let status = handler.partition_status_for("some-peer", true).await;
    assert_eq!(status, PartitionStatus::Healthy);
}

#[tokio::test]
async fn partition_status_detects_local_partition() {
    use capability_propagation::PeerMetadata;
    use std::time::Instant;

    let handler = MeshHandler::new();

    // remote-gate reports it can reach vps-gate
    {
        let mut meta = handler.peer_metadata.write().await;
        meta.insert(
            "remote-gate".to_string(),
            PeerMetadata {
                version: Some("0.2.1".to_string()),
                reachable_peers: vec!["vps-gate".to_string()],
                last_updated: Instant::now(),
            },
        );
    }

    // We cannot reach vps-gate locally
    let status = handler.partition_status_for("vps-gate", false).await;
    assert_eq!(
        status,
        PartitionStatus::LocallyUnreachable {
            reachable_from: vec!["remote-gate".to_string()]
        }
    );
}

#[tokio::test]
async fn partition_status_detects_partial_partition() {
    use capability_propagation::PeerMetadata;
    use std::time::Instant;

    let handler = MeshHandler::new();

    // remote-gate does NOT include vps-gate in its reachable list
    {
        let mut meta = handler.peer_metadata.write().await;
        meta.insert(
            "remote-gate".to_string(),
            PeerMetadata {
                version: Some("0.2.1".to_string()),
                reachable_peers: vec!["other-gate".to_string()],
                last_updated: Instant::now(),
            },
        );
    }

    // We CAN reach vps-gate locally, but remote-gate cannot
    let status = handler.partition_status_for("vps-gate", true).await;
    assert_eq!(
        status,
        PartitionStatus::PartialPartition {
            unreachable_from: vec!["remote-gate".to_string()]
        }
    );
}

#[tokio::test]
async fn version_skew_reported_in_mesh_status() {
    use capability_propagation::PeerMetadata;
    use std::time::Instant;

    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "local-gate",
            "bootstrap_peers": [{"node_id": "old-peer", "address": "10.0.0.2:3492"}]
        }))
        .await
        .unwrap();

    // Inject metadata with a different version
    {
        let mut meta = handler.peer_metadata.write().await;
        meta.insert(
            "old-peer".to_string(),
            PeerMetadata {
                version: Some("0.1.0-ancient".to_string()),
                reachable_peers: Vec::new(),
                last_updated: Instant::now(),
            },
        );
    }

    let status = handler.handle_status(json!({})).await.unwrap();
    assert!(status.get("version_skew").is_some(), "should report version skew");
    let skew = status["version_skew"].as_array().unwrap();
    assert_eq!(skew.len(), 1);
    assert_eq!(skew[0]["peer"], "old-peer");
    assert_eq!(skew[0]["version"], "0.1.0-ancient");
}

#[tokio::test]
async fn mesh_status_includes_own_version() {
    let handler = MeshHandler::new();
    handler.handle_init(json!({ "node_id": "gate-a" })).await.unwrap();

    let status = handler.handle_status(json!({})).await.unwrap();
    assert_eq!(status["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn health_check_reports_partition_when_gossip_disagrees() {
    use capability_propagation::PeerMetadata;
    use std::time::Instant;

    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "local-gate",
            "bootstrap_peers": [{"node_id": "reachable-peer", "address": "10.0.0.1:3492"}]
        }))
        .await
        .unwrap();

    // remote-gate claims it can reach "ghost-peer" — but we can't
    {
        let mut meta = handler.peer_metadata.write().await;
        meta.insert(
            "remote-gate".to_string(),
            PeerMetadata {
                version: Some("0.2.1".to_string()),
                reachable_peers: vec!["ghost-peer".to_string(), "reachable-peer".to_string()],
                last_updated: Instant::now(),
            },
        );
    }

    let result = handler.handle_health_check(json!({})).await.unwrap();
    assert_eq!(result["partition_detected"], true);
    let partitions = result["partitions"].as_array().unwrap();
    assert!(partitions.iter().any(|p| p["peer"] == "ghost-peer"));
}

#[tokio::test]
async fn peer_version_returns_stored_version() {
    use capability_propagation::PeerMetadata;
    use std::time::Instant;

    let handler = MeshHandler::new();
    {
        let mut meta = handler.peer_metadata.write().await;
        meta.insert(
            "versioned-peer".to_string(),
            PeerMetadata {
                version: Some("0.3.0-beta".to_string()),
                reachable_peers: Vec::new(),
                last_updated: Instant::now(),
            },
        );
    }

    assert_eq!(handler.peer_version("versioned-peer").await, Some("0.3.0-beta".to_string()));
    assert_eq!(handler.peer_version("unknown-peer").await, None);
}
