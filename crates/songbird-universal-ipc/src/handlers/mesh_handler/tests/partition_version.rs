// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::handlers::mesh_handler::MeshHandler;
use crate::handlers::mesh_handler::PartitionStatus;
use crate::handlers::mesh_handler::capability_propagation::PeerMetadata;
use serde_json::json;
use std::time::{Duration, Instant};

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
