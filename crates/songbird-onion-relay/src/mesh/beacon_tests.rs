// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::super::types::{EndpointType, RelayEndpoint};
use super::*;
use crate::signaling::SignalingMessage;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_mesh_creation() {
    let mesh = BeaconMesh::new(String::from("tower"), vec![String::from("abc123.onion")]);

    assert_eq!(mesh.my_node_id, "tower");
    assert_eq!(mesh.bootstrap_onions.len(), 1);
}

#[tokio::test]
async fn test_endpoint_priority() {
    assert!(
        EndpointType::Local {
            addr: "127.0.0.1:1234".parse().unwrap()
        }
        .priority()
            < EndpointType::Direct {
                addr: "1.2.3.4:1234".parse().unwrap()
            }
            .priority()
    );

    assert!(
        EndpointType::Direct {
            addr: "1.2.3.4:1234".parse().unwrap()
        }
        .priority()
            < EndpointType::FamilyRelay {
                relay_node_id: String::from("relay")
            }
            .priority()
    );

    assert!(
        EndpointType::FamilyRelay {
            relay_node_id: String::from("relay")
        }
        .priority()
            < EndpointType::TorOnion {
                onion_addr: String::from("abc.onion")
            }
            .priority()
    );
}

#[tokio::test]
async fn test_add_and_find_path() {
    let mesh = BeaconMesh::new(String::from("tower"), vec![]);

    mesh.record_direct_connection(
        String::from("pixel"),
        "1.2.3.4:5678".parse().unwrap(),
        Duration::from_millis(50),
    )
    .await;

    let path = mesh.get_best_path("pixel").await;
    assert!(path.is_some());
    assert!(matches!(path.unwrap().endpoint_type, EndpointType::Direct { .. }));
}

#[tokio::test]
async fn test_overlay_preferred_over_direct() {
    let mesh = BeaconMesh::new(String::from("east-gate"), vec![]);

    let direct_ep = RelayEndpoint {
        node_id: "flock-gate".into(),
        endpoint_type: EndpointType::Direct {
            addr: "203.0.113.50:7700".parse().unwrap(),
        },
        latency: Some(Duration::from_millis(25)),
        last_seen: Instant::now(),
        reachable: true,
    };
    mesh.add_endpoint("flock-gate".into(), direct_ep).await;

    let overlay_ep = RelayEndpoint {
        node_id: "flock-gate".into(),
        endpoint_type: EndpointType::Overlay {
            addr: "10.13.37.6:7700".parse().unwrap(),
            overlay_name: "wireguard".into(),
        },
        latency: Some(Duration::from_millis(5)),
        last_seen: Instant::now(),
        reachable: true,
    };
    mesh.add_endpoint("flock-gate".into(), overlay_ep).await;

    let path = mesh.get_best_path("flock-gate").await;
    assert!(path.is_some());
    let best = path.unwrap();
    assert!(
        matches!(best.endpoint_type, EndpointType::Overlay { .. }),
        "Expected Overlay, got {:?}",
        best.endpoint_type
    );
    assert_eq!(best.endpoint_type.socket_addr().unwrap(), "10.13.37.6:7700".parse().unwrap());
}

#[tokio::test]
async fn test_overlay_fallback_to_direct_when_unreachable() {
    let mesh = BeaconMesh::new(String::from("east-gate"), vec![]);

    let overlay_ep = RelayEndpoint {
        node_id: "flock-gate".into(),
        endpoint_type: EndpointType::Overlay {
            addr: "10.13.37.6:7700".parse().unwrap(),
            overlay_name: "wireguard".into(),
        },
        latency: None,
        last_seen: Instant::now(),
        reachable: false,
    };
    mesh.add_endpoint("flock-gate".into(), overlay_ep).await;

    let direct_ep = RelayEndpoint {
        node_id: "flock-gate".into(),
        endpoint_type: EndpointType::Direct {
            addr: "203.0.113.50:7700".parse().unwrap(),
        },
        latency: Some(Duration::from_millis(40)),
        last_seen: Instant::now(),
        reachable: true,
    };
    mesh.add_endpoint("flock-gate".into(), direct_ep).await;

    let path = mesh.get_best_path("flock-gate").await;
    assert!(path.is_some());
    let best = path.unwrap();
    assert!(
        matches!(best.endpoint_type, EndpointType::Direct { .. }),
        "Expected Direct fallback, got {:?}",
        best.endpoint_type
    );
}

#[tokio::test]
async fn record_overlay_connection_updates_latency_and_type() {
    let mesh = BeaconMesh::new(String::from("local"), vec![]);

    mesh.record_overlay_connection(
        String::from("peer-gate"),
        "10.13.37.5:7700".parse().unwrap(),
        "wireguard",
        Duration::from_millis(3),
    )
    .await;

    let path = mesh.get_best_path("peer-gate").await;
    assert!(path.is_some());
    let best = path.unwrap();
    assert!(
        matches!(best.endpoint_type, EndpointType::Overlay { .. }),
        "Expected Overlay type from record_overlay_connection, got {:?}",
        best.endpoint_type
    );
    assert_eq!(best.latency, Some(Duration::from_millis(3)));
    assert!(best.reachable);
}

#[tokio::test]
async fn record_overlay_connection_preferred_over_direct_same_peer() {
    let mesh = BeaconMesh::new(String::from("hub"), vec![]);

    mesh.record_direct_connection(
        String::from("remote"),
        "198.51.100.1:7700".parse().unwrap(),
        Duration::from_millis(50),
    )
    .await;

    mesh.record_overlay_connection(
        String::from("remote"),
        "10.13.37.2:7700".parse().unwrap(),
        "wireguard",
        Duration::from_millis(1),
    )
    .await;

    let best = mesh.get_best_path("remote").await.unwrap();
    assert!(
        matches!(best.endpoint_type, EndpointType::Overlay { .. }),
        "Overlay should be preferred over Direct even with higher direct latency"
    );
    assert_eq!(best.latency, Some(Duration::from_millis(1)));
}

#[tokio::test]
async fn test_relay_fallback() {
    let mesh = BeaconMesh::new(String::from("laptop"), vec![String::from("bootstrap.onion")]);

    mesh.record_direct_connection(
        String::from("tower"),
        "1.2.3.4:5678".parse().unwrap(),
        Duration::from_millis(30),
    )
    .await;

    let path = mesh.find_relay_for("phone").await;
    assert!(path.is_some());

    let ep = path.unwrap();
    assert!(
        matches!(ep.endpoint_type, EndpointType::FamilyRelay { .. })
            || matches!(ep.endpoint_type, EndpointType::TorOnion { .. })
    );
}

#[tokio::test]
async fn set_my_onion_and_announce_register_shape() {
    let mesh = BeaconMesh::new("me".into(), vec![]);
    mesh.set_my_onion("abcd1234efgh5678ijkl9012mnop3456qrst7890uvwx.onion".into()).await;
    let msg = mesh.announce_as_relay().await;
    match msg {
        SignalingMessage::Register {
            peer_info,
            encrypted_beacon,
        } => {
            assert_eq!(peer_info.node_id, "me");
            assert!(peer_info.capabilities.iter().any(|c| c.starts_with("can_reach:")));
            assert_eq!(
                encrypted_beacon,
                Some("abcd1234efgh5678ijkl9012mnop3456qrst7890uvwx.onion".into())
            );
        }
        other => panic!("expected Register, got {other:?}"),
    }
}

#[tokio::test]
async fn get_all_paths_and_best_prefers_lower_priority() {
    let mesh = BeaconMesh::new("hub".into(), vec![]);
    let addr = "10.0.0.5:9000".parse().unwrap();
    mesh.record_direct_connection("peer".into(), addr, Duration::from_millis(10)).await;
    mesh.record_relay_path("peer".into(), "via".into(), Duration::from_millis(5)).await;

    let paths = mesh.get_all_paths("peer").await;
    assert_eq!(paths.len(), 2, "both endpoints recorded");

    let best = mesh.get_best_path("peer").await.expect("best path");
    assert!(
        matches!(best.endpoint_type, EndpointType::Direct { .. }),
        "direct should beat family relay: {:?}",
        best.endpoint_type
    );
}

#[tokio::test]
async fn find_relay_for_unknown_peer_without_bootstrap_returns_none() {
    let mesh = BeaconMesh::new("solo".into(), vec![]);
    assert!(mesh.find_relay_for("nobody").await.is_none(), "no relays and no bootstrap → None");
}

#[tokio::test]
async fn find_relay_for_prefers_reachable_lower_priority_endpoint() {
    let mesh = BeaconMesh::new("me".into(), vec![]);
    mesh.record_relay_path("target".into(), "r1".into(), Duration::from_millis(100)).await;
    mesh.record_direct_connection(
        "helper".into(),
        "1.1.1.1:1".parse().unwrap(),
        Duration::from_millis(20),
    )
    .await;

    let path = mesh.find_relay_for("target").await.expect("helper or bootstrap path");
    assert!(
        matches!(path.endpoint_type, EndpointType::FamilyRelay { .. }),
        "expected family relay toward target, got {:?}",
        path.endpoint_type
    );
}

#[tokio::test]
async fn handle_relay_request_ok_and_peer_not_found() {
    let mesh = BeaconMesh::new("relay".into(), vec![]);
    mesh.record_direct_connection("dest".into(), "8.8.8.8:53".parse().unwrap(), Duration::ZERO)
        .await;

    mesh.handle_relay_request("src", "dest", vec![1, 2, 3]).await.expect("path to dest exists");

    let err = mesh.handle_relay_request("src", "missing", vec![]).await.expect_err("no path");
    assert!(matches!(err, crate::OnionRelayError::PeerNotFound(_)));
}

#[tokio::test(start_paused = true)]
async fn health_check_marks_stale_unreachable() {
    let mesh = BeaconMesh::new("n".into(), vec![]);
    let ep = RelayEndpoint {
        node_id: "p".into(),
        endpoint_type: EndpointType::Direct {
            addr: "1.1.1.1:1".parse().unwrap(),
        },
        latency: None,
        last_seen: Instant::now()
            .checked_sub(Duration::from_secs(120))
            .expect("instant far enough after epoch for subtraction"),
        reachable: true,
    };
    {
        let mut map = mesh.endpoints.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        map.insert("p".into(), vec![ep.clone()]);
    }

    mesh.health_check().await;

    let eps = mesh.get_all_paths("p").await;
    assert_eq!(eps.len(), 1);
    assert!(!eps[0].reachable, "endpoint older than 60s should be marked unreachable");
}

#[tokio::test]
async fn get_reachable_nodes_filters_unreachable() {
    let mesh = BeaconMesh::new("n".into(), vec![]);
    let mut ep = RelayEndpoint {
        node_id: "up".into(),
        endpoint_type: EndpointType::Direct {
            addr: "2.2.2.2:2".parse().unwrap(),
        },
        latency: Some(Duration::from_millis(1)),
        last_seen: Instant::now(),
        reachable: true,
    };
    {
        let mut map = mesh.endpoints.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        map.insert("up".into(), vec![ep.clone()]);
        ep.reachable = false;
        map.insert("down".into(), vec![ep]);
    }

    let nodes = mesh.get_reachable_nodes().await;
    assert_eq!(nodes, vec![String::from("up")]);
}

#[tokio::test]
async fn endpoint_type_priority_ordering() {
    assert_eq!(
        EndpointType::Local {
            addr: "127.0.0.1:1".parse().unwrap()
        }
        .priority(),
        0
    );
    assert_eq!(
        EndpointType::Overlay {
            addr: "10.13.37.1:7700".parse().unwrap(),
            overlay_name: "wireguard".into()
        }
        .priority(),
        1
    );
    assert_eq!(
        EndpointType::Direct {
            addr: "1.1.1.1:1".parse().unwrap()
        }
        .priority(),
        2
    );
    assert_eq!(
        EndpointType::FamilyRelay {
            relay_node_id: "r".into()
        }
        .priority(),
        3
    );
    assert_eq!(
        EndpointType::TorOnion {
            onion_addr: "x.onion".into()
        }
        .priority(),
        4
    );
}

#[tokio::test]
async fn find_relay_for_skips_unreachable_candidate_endpoints() {
    let mesh = BeaconMesh::new("me".into(), vec![]);
    mesh.add_endpoint(
        "fast_but_down".into(),
        RelayEndpoint {
            node_id: "fast_but_down".into(),
            endpoint_type: EndpointType::Direct {
                addr: "9.9.9.9:9".parse().unwrap(),
            },
            latency: Some(Duration::from_millis(1)),
            last_seen: Instant::now(),
            reachable: false,
        },
    )
    .await;
    mesh.record_direct_connection(
        "slow_but_up".into(),
        "8.8.8.8:8".parse().unwrap(),
        Duration::from_millis(80),
    )
    .await;

    let path = mesh.find_relay_for("stranger").await.expect("reachable helper exists");
    assert!(
        matches!(
            &path.endpoint_type,
            EndpointType::FamilyRelay {
                relay_node_id
            } if relay_node_id == "slow_but_up"
        ),
        "unreachable endpoints must not be selected as relay helpers: {:?}",
        path.endpoint_type
    );
}

#[tokio::test]
async fn find_relay_for_prefers_lower_latency_when_priority_matches() {
    let mesh = BeaconMesh::new("me".into(), vec![]);
    mesh.record_direct_connection(
        "higher_latency".into(),
        "1.1.1.1:1".parse().unwrap(),
        Duration::from_millis(90),
    )
    .await;
    mesh.record_direct_connection(
        "lower_latency".into(),
        "2.2.2.2:2".parse().unwrap(),
        Duration::from_millis(12),
    )
    .await;

    let path = mesh.find_relay_for("unknown_peer").await.expect("two helpers registered");
    assert!(
        matches!(
            &path.endpoint_type,
            EndpointType::FamilyRelay {
                relay_node_id
            } if relay_node_id == "lower_latency"
        ),
        "expected lower-latency direct path to win tie-break: {:?}",
        path.endpoint_type
    );
}

#[tokio::test]
async fn find_relay_for_returns_best_path_when_target_has_known_route() {
    let mesh = BeaconMesh::new("me".into(), vec!["boot.onion".into()]);
    mesh.record_direct_connection(
        "pixel".into(),
        "10.0.0.1:9000".parse().unwrap(),
        Duration::from_millis(5),
    )
    .await;

    let direct = mesh.find_relay_for("pixel").await.expect("direct path registered");
    assert!(
        matches!(direct.endpoint_type, EndpointType::Direct { .. }),
        "should return stored best path, got {:?}",
        direct.endpoint_type
    );
}

#[tokio::test]
async fn add_endpoint_family_then_direct_updates_best_to_direct() {
    let mesh = BeaconMesh::new("hub".into(), vec![]);
    mesh.record_relay_path("peer".into(), "via".into(), Duration::from_millis(2)).await;
    mesh.record_direct_connection(
        "peer".into(),
        "192.0.2.1:1".parse().unwrap(),
        Duration::from_millis(40),
    )
    .await;

    let best = mesh.get_best_path("peer").await.expect("best exists");
    assert!(
        matches!(best.endpoint_type, EndpointType::Direct { .. }),
        "direct should replace family relay in best-path table: {:?}",
        best.endpoint_type
    );
}

#[tokio::test]
async fn handle_relay_request_fails_when_only_unreachable_endpoints_exist() {
    let mesh = BeaconMesh::new("relay".into(), vec![]);
    mesh.add_endpoint(
        "gone".into(),
        RelayEndpoint {
            node_id: "gone".into(),
            endpoint_type: EndpointType::Direct {
                addr: "198.51.100.1:1".parse().unwrap(),
            },
            latency: None,
            last_seen: Instant::now(),
            reachable: false,
        },
    )
    .await;

    assert!(
        mesh.get_best_path("gone").await.is_none(),
        "unreachable-only endpoints must not populate best_paths"
    );

    let err =
        mesh.handle_relay_request("src", "gone", vec![9]).await.expect_err("no reachable path");
    assert!(matches!(err, crate::OnionRelayError::PeerNotFound(_)));
}

#[tokio::test]
async fn get_all_paths_returns_empty_for_unknown_peer() {
    let mesh = BeaconMesh::new("solo".into(), vec![]);
    assert!(mesh.get_all_paths("nope").await.is_empty());
}
