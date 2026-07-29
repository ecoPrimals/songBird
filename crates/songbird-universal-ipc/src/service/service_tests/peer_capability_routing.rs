// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

/// `find_peer_with_capability` ignores expired entries.
#[tokio::test]
async fn find_peer_with_capability_ignores_expired() {
    use crate::handlers::mesh_handler::MeshHandler;
    use crate::handlers::mesh_handler::capability_propagation::PeerCapabilityEntry;
    use std::time::{Duration, Instant};

    let handler = MeshHandler::new();

    {
        let mut caps = handler.peer_capabilities.write().await;
        caps.insert(
            "stale-gate".to_string(),
            PeerCapabilityEntry {
                capabilities: vec!["stale-cap".to_string()],
                last_seen: Instant::now() - Duration::from_secs(700),
            },
        );
        caps.insert(
            "fresh-gate".to_string(),
            PeerCapabilityEntry {
                capabilities: vec!["fresh-cap".to_string()],
                last_seen: Instant::now(),
            },
        );
    }

    assert!(handler.find_peer_with_capability("stale-cap").await.is_none());

    let found = handler.find_peer_with_capability("fresh-cap").await;
    assert!(found.is_some());
    let (peer, caps) = found.unwrap();
    assert_eq!(peer, "fresh-gate");
    assert!(caps.contains(&"fresh-cap".to_string()));
}

/// `find_peer_with_capability` selects the peer with the lowest-cost path when
/// multiple peers provide the same capability (overlay preferred over direct).
#[tokio::test]
async fn find_peer_with_capability_prefers_lower_cost_path() {
    use crate::handlers::mesh_handler::MeshHandler;
    use crate::handlers::mesh_handler::capability_propagation::PeerCapabilityEntry;
    use songbird_onion_relay::mesh::{BeaconMesh, EndpointType, RelayEndpoint};
    use std::time::{Duration, Instant};

    let handler = MeshHandler::new();

    // Initialize mesh with two peers
    let mesh = BeaconMesh::new(String::from("local-gate"), vec![]);

    // gate-a: direct (priority 1), high latency
    mesh.record_direct_connection(
        String::from("gate-a"),
        "203.0.113.1:7700".parse().unwrap(),
        Duration::from_millis(100),
    )
    .await;

    // gate-b: overlay (priority 0), low latency
    mesh.record_overlay_connection(
        String::from("gate-b"),
        "10.13.37.5:7700".parse().unwrap(),
        "wireguard",
        Duration::from_millis(2),
    )
    .await;

    *handler.mesh.write().await = Some(std::sync::Arc::new(mesh));

    // Both peers advertise the same capability
    {
        let mut caps = handler.peer_capabilities.write().await;
        caps.insert(
            String::from("gate-a"),
            PeerCapabilityEntry {
                capabilities: vec![String::from("shared-cap")],
                last_seen: Instant::now(),
            },
        );
        caps.insert(
            String::from("gate-b"),
            PeerCapabilityEntry {
                capabilities: vec![String::from("shared-cap")],
                last_seen: Instant::now(),
            },
        );
    }

    // Should prefer gate-b (overlay, lower cost)
    let found = handler.find_peer_with_capability("shared-cap").await;
    assert!(found.is_some());
    let (peer, _) = found.unwrap();
    assert_eq!(peer, "gate-b", "should prefer overlay peer with lower cost");
}

/// `find_peer_with_capability` returns the single holder even without mesh initialized.
#[tokio::test]
async fn find_peer_with_capability_single_holder_no_mesh() {
    use crate::handlers::mesh_handler::MeshHandler;
    use crate::handlers::mesh_handler::capability_propagation::PeerCapabilityEntry;
    use std::time::Instant;

    let handler = MeshHandler::new();

    {
        let mut caps = handler.peer_capabilities.write().await;
        caps.insert(
            String::from("only-gate"),
            PeerCapabilityEntry {
                capabilities: vec![String::from("unique-cap")],
                last_seen: Instant::now(),
            },
        );
    }

    let found = handler.find_peer_with_capability("unique-cap").await;
    assert!(found.is_some());
    let (peer, _) = found.unwrap();
    assert_eq!(peer, "only-gate");
}
