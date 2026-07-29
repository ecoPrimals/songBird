// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::handlers::mesh_handler::MeshHandler;
use songbird_onion_relay::mesh::{EndpointType, RelayEndpoint};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

#[test]
fn endpoint_to_strings_local_and_direct() {
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
