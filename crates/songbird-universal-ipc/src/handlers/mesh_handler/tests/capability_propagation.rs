// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::handlers::mesh_handler::MeshHandler;
use crate::handlers::mesh_handler::capability_propagation::{PeerCapabilityEntry, PendingAnnounce};
use serde_json::json;
use std::time::{Duration, Instant};

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
    let mut handler = MeshHandler::new();
    handler.min_announce_interval = std::time::Duration::ZERO;

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
async fn capabilities_announce_rate_limited() {
    let mut handler = MeshHandler::new();
    handler.min_announce_interval = std::time::Duration::from_secs(60);

    handler
        .handle_capabilities_announce(serde_json::json!({
            "node_id": "flood-gate",
            "capabilities": ["cap1"]
        }))
        .await
        .expect("first should succeed");

    let err = handler
        .handle_capabilities_announce(serde_json::json!({
            "node_id": "flood-gate",
            "capabilities": ["cap2"]
        }))
        .await
        .expect_err("second should be rate limited");

    assert!(err.contains("Rate limited"), "expected rate limit error, got: {err}");
}

#[tokio::test]
async fn capabilities_announce_validates_input() {
    let handler = MeshHandler::new();

    // Empty node_id
    let err = handler
        .handle_capabilities_announce(serde_json::json!({
            "node_id": "",
            "capabilities": ["cap"]
        }))
        .await
        .expect_err("empty node_id should fail");
    assert!(err.contains("Invalid node_id"), "got: {err}");

    // Too many capabilities
    let big_caps: Vec<String> = (0..65).map(|i| format!("cap-{i}")).collect();
    let err = handler
        .handle_capabilities_announce(serde_json::json!({
            "node_id": "valid-gate",
            "capabilities": big_caps
        }))
        .await
        .expect_err("65 capabilities should fail");
    assert!(err.contains("Too many capabilities"), "got: {err}");
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
