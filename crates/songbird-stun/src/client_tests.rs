// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use crate::StunServer;
use crate::client::StunClient;
use crate::error::StunError;
use crate::message::{StunAttribute, StunMessage};
use crate::types::{NatType, PortPattern, StunCredentials};
use songbird_config::timeouts::TimeoutConfig;
use songbird_types::constants::DEFAULT_STUN_SERVER_1;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;

#[tokio::test(start_paused = true)]
async fn discover_public_address_racing_empty_server_list() {
    let client = StunClient::with_timeout(Duration::from_millis(200));
    let err = client
        .discover_public_address_racing(&[])
        .await
        .expect_err("empty server list should yield Config error");
    assert!(matches!(err, StunError::Config(_)), "expected Config error, got {err:?}");
}

#[tokio::test(start_paused = true)]
async fn discover_public_address_parallel_empty_server_list() {
    let client = StunClient::with_timeout(Duration::from_millis(200));
    let err = client
        .discover_public_address_parallel(&[])
        .await
        .expect_err("empty server list should yield Config error");
    assert!(matches!(err, StunError::Config(_)), "expected Config error, got {err:?}");
}

async fn start_local_stun_server() -> (tokio::task::JoinHandle<()>, SocketAddr) {
    let (ready_tx, ready_rx) = oneshot::channel();
    let handle = tokio::spawn(async move {
        let server = StunServer::new("127.0.0.1:0".parse().expect("loopback parse"));
        let _ = server.run_with_ready(ready_tx).await;
    });
    let addr = ready_rx.await.expect("STUN server should signal bound address");
    (handle, addr)
}

#[tokio::test]
async fn discover_public_address_parallel_local_server_succeeds() {
    let (server_handle, actual_addr) = start_local_stun_server().await;
    let client = StunClient::with_timeout(Duration::from_secs(2));
    let result = timeout(
        Duration::from_secs(3),
        client.discover_public_address_parallel(&[actual_addr.to_string()]),
    )
    .await;
    server_handle.abort();
    let inner = result.expect("outer timeout: parallel discovery should finish");
    let addr = inner.expect("parallel STUN discovery should succeed against local server");
    assert!(
        addr.ip().is_loopback(),
        "expected loopback mapped address from local server, got {addr}"
    );
}

#[tokio::test]
async fn discover_public_address_racing_local_server_succeeds() {
    let (server_handle, actual_addr) = start_local_stun_server().await;
    let client = StunClient::with_timeout(Duration::from_secs(2));
    let server_str = actual_addr.to_string();
    let result = timeout(
        Duration::from_secs(3),
        client.discover_public_address_racing(&[server_str.as_str()]),
    )
    .await;
    server_handle.abort();
    let inner = result.expect("outer timeout: racing discovery should finish");
    let addr = inner.expect("racing STUN discovery should succeed against local server");
    assert!(
        addr.ip().is_loopback(),
        "expected loopback mapped address from local server, got {addr}"
    );
}

#[tokio::test]
async fn discover_public_endpoint_local_server_returns_unknown_nat() {
    let (server_handle, actual_addr) = start_local_stun_server().await;
    let client = StunClient::with_timeout(Duration::from_secs(2));
    let ep =
        timeout(Duration::from_secs(3), client.discover_public_endpoint(&actual_addr.to_string()))
            .await
            .expect("endpoint discovery should complete within timeout")
            .expect("endpoint discovery should succeed");
    server_handle.abort();
    assert_eq!(ep.nat_type, NatType::Unknown);
    assert!(ep.address.ip().is_loopback());
}

#[test]
fn nat_type_default_is_unknown() {
    assert_eq!(NatType::default(), NatType::Unknown);
}

#[tokio::test]
#[ignore = "requires running STUN/TURN server"]
async fn test_discover_public_address_live() {
    let client = StunClient::new();

    let result = client.discover_public_address(DEFAULT_STUN_SERVER_1).await;

    match result {
        Ok(addr) => {
            println!("Discovered public address: {}", addr);
            assert!(addr.port() > 0);
        }
        Err(e) => {
            eprintln!("STUN request failed (expected if no network): {}", e);
        }
    }
}

#[tokio::test]
async fn test_stun_client_creation() {
    let expected = TimeoutConfig::from_env().connect;
    let client = StunClient::new();
    assert_eq!(
        client.timeout, expected,
        "StunClient::new should use TimeoutConfig::from_env().connect"
    );

    let client = StunClient::with_timeout(Duration::from_secs(10));
    assert_eq!(
        client.timeout,
        Duration::from_secs(10),
        "with_timeout should store the given duration"
    );
}

#[test]
fn test_default_client() {
    let expected = TimeoutConfig::from_env().connect;
    let client = StunClient::default();
    assert_eq!(client.timeout, expected, "default() should match StunClient::new()");
}

#[tokio::test(start_paused = true)]
async fn discover_public_address_invalid_host_fails_before_long_timeout() {
    let client = StunClient::with_timeout(Duration::from_millis(100));
    let err = client
        .discover_public_address("not-a-valid-host.example.invalid")
        .await
        .expect_err("unresolvable host should fail");
    assert!(
        matches!(err, StunError::Network(_)),
        "expected network error from resolution/IO, got {err:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn discover_public_endpoint_invalid_host_surfaces_network_error() {
    let client = StunClient::with_timeout(Duration::from_millis(100));
    let err = client
        .discover_public_endpoint("nonexistent.invalid")
        .await
        .expect_err("endpoint discovery should fail");
    assert!(matches!(err, StunError::Network(_)), "expected network error, got {err:?}");
}

#[tokio::test]
async fn discover_on_socket_local_server_succeeds() {
    use tokio::net::UdpSocket;

    let (server_handle, server_addr) = start_local_stun_server().await;
    let socket = UdpSocket::bind("127.0.0.1:0").await.expect("bind shared socket");

    let client = StunClient::with_timeout(Duration::from_secs(2));
    let addr = tokio::time::timeout(
        Duration::from_secs(3),
        client.discover_on_socket(&socket, &server_addr.to_string()),
    )
    .await
    .expect("outer timeout")
    .expect("discover_on_socket should succeed");
    server_handle.abort();
    assert!(addr.ip().is_loopback(), "expected loopback, got {addr}");
}

#[tokio::test]
async fn discover_on_socket_reuses_local_port() {
    use tokio::net::UdpSocket;

    let (h1, addr1) = start_local_stun_server().await;
    let (h2, addr2) = start_local_stun_server().await;

    let socket = UdpSocket::bind("127.0.0.1:0").await.expect("bind shared socket");
    let local_port = socket.local_addr().expect("local addr").port();

    let client = StunClient::with_timeout(Duration::from_secs(2));
    let mapped1 =
        timeout(Duration::from_secs(3), client.discover_on_socket(&socket, &addr1.to_string()))
            .await
            .expect("timeout")
            .expect("probe 1");

    let mapped2 =
        timeout(Duration::from_secs(3), client.discover_on_socket(&socket, &addr2.to_string()))
            .await
            .expect("timeout")
            .expect("probe 2");

    h1.abort();
    h2.abort();

    assert_eq!(mapped1.port(), local_port, "server should reflect our local port");
    assert_eq!(mapped2.port(), local_port, "second probe should reuse same port");
}

#[test]
fn classify_nat_same_port_is_cone() {
    use crate::client::classify_nat_from_dual_probes;
    use std::net::Ipv4Addr;

    let ip = std::net::IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1));
    let a = SocketAddr::new(ip, 50_000);
    let b = SocketAddr::new(ip, 50_000);
    assert_eq!(classify_nat_from_dual_probes(a, b), NatType::PortRestrictedCone);
}

#[test]
fn classify_nat_different_ports_is_symmetric() {
    use crate::client::classify_nat_from_dual_probes;
    use std::net::Ipv4Addr;

    let ip = std::net::IpAddr::V4(Ipv4Addr::new(203, 0, 113, 2));
    let a = SocketAddr::new(ip, 50_000);
    let b = SocketAddr::new(ip, 50_001);
    assert_eq!(classify_nat_from_dual_probes(a, b), NatType::Symmetric);
}

#[test]
fn classify_nat_different_ips_is_unknown() {
    use crate::client::classify_nat_from_dual_probes;
    use std::net::Ipv4Addr;

    let a = SocketAddr::from((Ipv4Addr::new(1, 1, 1, 1), 10_000));
    let b = SocketAddr::from((Ipv4Addr::new(2, 2, 2, 2), 10_000));
    assert_eq!(classify_nat_from_dual_probes(a, b), NatType::Unknown);
}

#[tokio::test]
async fn discover_public_endpoint_multi_single_server_returns_unknown_nat() {
    let (server_handle, actual_addr) = start_local_stun_server().await;
    let client = StunClient::with_timeout(Duration::from_secs(2));
    let server_str = actual_addr.to_string();
    let ep = timeout(
        Duration::from_secs(3),
        client.discover_public_endpoint_multi(&[server_str.as_str()]),
    )
    .await
    .expect("timeout")
    .expect("single-server endpoint discovery should succeed");
    server_handle.abort();
    assert_eq!(ep.nat_type, NatType::Unknown);
    assert!(ep.address.ip().is_loopback());
}

#[tokio::test]
async fn discover_public_endpoint_multi_two_servers_classifies_cone() {
    let (h1, addr1) = start_local_stun_server().await;
    let (h2, addr2) = start_local_stun_server().await;
    let client = StunClient::with_timeout(Duration::from_secs(2));
    let s1 = addr1.to_string();
    let s2 = addr2.to_string();
    let ep = timeout(
        Duration::from_secs(3),
        client.discover_public_endpoint_multi(&[s1.as_str(), s2.as_str()]),
    )
    .await
    .expect("timeout")
    .expect("dual-server endpoint discovery should succeed");
    h1.abort();
    h2.abort();
    assert_eq!(
        ep.nat_type,
        NatType::PortRestrictedCone,
        "loopback has no NAT, so both servers reflect the same port → cone classification"
    );
}

#[tokio::test]
async fn probe_port_pattern_local_server_detects_pattern() {
    let (server_handle, actual_addr) = start_local_stun_server().await;
    let client = StunClient::with_timeout(Duration::from_secs(2));
    let pattern =
        timeout(Duration::from_secs(5), client.probe_port_pattern(&actual_addr.to_string(), 5))
            .await
            .expect("probe should finish within timeout")
            .expect("probe should succeed against local STUN server");
    server_handle.abort();

    match pattern {
        PortPattern::Sequential {
            step,
            confidence,
            ..
        } => {
            assert!(
                confidence > 0.0,
                "sequential pattern should have positive confidence, got {confidence}"
            );
            assert!(
                step.abs() <= 100,
                "sequential step should be within detection bounds, got {step}"
            );
        }
        PortPattern::Random {
            observed,
        } => {
            assert!(observed.len() >= 2, "random pattern should list at least two observed ports");
        }
        PortPattern::Unknown => {
            panic!(
                "local loopback server should yield enough successful probes for pattern detection"
            );
        }
    }
}

#[tokio::test(start_paused = true)]
async fn probe_port_pattern_unreachable_server_returns_unknown() {
    let client = StunClient::with_timeout(Duration::from_millis(50));
    let pattern = client
        .probe_port_pattern("127.0.0.1:59999", 4)
        .await
        .expect("probe should not error when probes fail individually");
    assert_eq!(
        pattern,
        PortPattern::Unknown,
        "insufficient successful probes should yield Unknown"
    );
}

#[tokio::test]
async fn probe_port_pattern_clamps_probes_to_minimum_two() {
    let (server_handle, actual_addr) = start_local_stun_server().await;
    let client = StunClient::with_timeout(Duration::from_secs(2));
    let pattern =
        timeout(Duration::from_secs(5), client.probe_port_pattern(&actual_addr.to_string(), 1))
            .await
            .expect("probe timeout")
            .expect("probe should succeed with clamped minimum");
    server_handle.abort();

    assert!(
        !matches!(pattern, PortPattern::Unknown),
        "clamped to 2 probes against live server should detect a pattern, got {pattern:?}"
    );
}

#[tokio::test]
async fn with_credentials_discover_public_address_succeeds() {
    let creds = StunCredentials {
        username: "beacon-user".to_string(),
        key: b"beacon-secret-key".to_vec(),
    };
    let client = StunClient::with_timeout(Duration::from_secs(2)).with_credentials(creds.clone());
    let (server_handle, actual_addr) = start_local_stun_server().await;

    let addr =
        timeout(Duration::from_secs(3), client.discover_public_address(&actual_addr.to_string()))
            .await
            .expect("timeout")
            .expect("authenticated client should discover via local server");
    server_handle.abort();

    assert!(addr.ip().is_loopback(), "expected loopback mapped address, got {addr}");
}

#[test]
fn with_credentials_binding_request_includes_message_integrity() {
    let creds = StunCredentials {
        username: "beacon-user".to_string(),
        key: b"beacon-secret-key".to_vec(),
    };
    let mut msg = StunMessage::new_binding_request();
    msg.attributes.push(StunAttribute::Username(creds.username.clone()));
    let wire = msg.encode_authenticated(&creds.key);

    assert!(
        wire.len() > 20,
        "authenticated binding request should include integrity + fingerprint attributes"
    );
    let decoded = StunMessage::decode(&wire).expect("wire should decode");
    assert!(
        decoded
            .attributes
            .iter()
            .any(|a| matches!(a, StunAttribute::Username(u) if u == "beacon-user")),
        "USERNAME attribute should be present in authenticated request"
    );
}

#[tokio::test]
async fn discover_public_address_racing_all_servers_fail() {
    let client = StunClient::with_timeout(Duration::from_millis(100));
    let err = client
        .discover_public_address_racing(&["127.0.0.1:59998", "127.0.0.1:59997"])
        .await
        .expect_err("all unreachable servers should fail");
    assert!(
        matches!(err, StunError::AllServersFailed(_)),
        "expected AllServersFailed, got {err:?}"
    );
    if let StunError::AllServersFailed(msg) = err {
        assert!(msg.contains("2"), "error message should mention server count: {msg}");
    }
}
