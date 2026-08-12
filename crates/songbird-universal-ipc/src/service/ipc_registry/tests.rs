// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::super::TransportEndpoint;
use super::register::build_canonical_payload;
use super::transport::transport_endpoint_from_native;
use crate::endpoint::NativeEndpoint;

#[test]
fn canonical_payload_is_deterministic_regardless_of_cap_order() {
    let a = build_canonical_payload(
        "nestgate",
        &["storage".into(), "crypto".into(), "auth".into()],
        "/tmp/nestgate.sock",
        "2026-04-28T12:00:00Z",
    );
    let b = build_canonical_payload(
        "nestgate",
        &["auth".into(), "crypto".into(), "storage".into()],
        "/tmp/nestgate.sock",
        "2026-04-28T12:00:00Z",
    );
    assert_eq!(a, b, "canonical payload must be order-independent");
}

#[test]
fn canonical_payload_contains_all_fields() {
    let payload = build_canonical_payload(
        "beardog",
        &["crypto".into(), "security".into()],
        "/run/user/1000/biomeos/beardog.sock",
        "2026-04-28T14:30:00Z",
    );
    let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(parsed["p"], "beardog");
    assert_eq!(parsed["e"], "/run/user/1000/biomeos/beardog.sock");
    assert_eq!(parsed["t"], "2026-04-28T14:30:00Z");
    let caps = parsed["c"].as_array().unwrap();
    assert_eq!(caps[0], "crypto");
    assert_eq!(caps[1], "security");
}

#[test]
fn canonical_payload_empty_capabilities() {
    let payload =
        build_canonical_payload("minimal", &[], "tcp://127.0.0.1:9000", "2026-01-01T00:00:00Z");
    let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert!(parsed["c"].as_array().unwrap().is_empty());
}

#[test]
fn turn_config_from_env_fails_gracefully_when_not_set() {
    // In CI and dev, SONGBIRD_TURN_SERVER is not set — from_env returns Err
    if songbird_process_env::var("SONGBIRD_TURN_SERVER").is_err() {
        let peer_addr: std::net::SocketAddr = "192.168.1.100:8080".parse().unwrap();
        let result = songbird_turn_client::TurnSessionConfig::from_env(peer_addr);
        assert!(result.is_err(), "Should fail when TURN env vars are absent");
    }
}

#[test]
fn transport_endpoint_from_unix_socket() {
    let ep = NativeEndpoint::UnixSocket("/run/membrane/beardog.sock".into());
    let te = transport_endpoint_from_native(&ep);
    assert_eq!(
        te,
        TransportEndpoint::Uds {
            path: String::from("/run/membrane/beardog.sock")
        }
    );
}

#[test]
fn transport_endpoint_from_abstract_socket() {
    let ep = NativeEndpoint::AbstractSocket("biomeos_security".into());
    let te = transport_endpoint_from_native(&ep);
    assert_eq!(
        te,
        TransportEndpoint::Uds {
            path: String::from("@biomeos_security")
        }
    );
}

#[test]
fn transport_endpoint_from_tcp_local() {
    let ep = NativeEndpoint::TcpLocal(7700);
    let te = transport_endpoint_from_native(&ep);
    assert_eq!(
        te,
        TransportEndpoint::Tcp {
            host: String::from("127.0.0.1"),
            port: 7700
        }
    );
}

#[test]
fn transport_endpoint_from_in_process() {
    let ep = NativeEndpoint::InProcess(42);
    let te = transport_endpoint_from_native(&ep);
    assert_eq!(
        te,
        TransportEndpoint::Tcp {
            host: String::from("127.0.0.1"),
            port: 42
        }
    );
}
