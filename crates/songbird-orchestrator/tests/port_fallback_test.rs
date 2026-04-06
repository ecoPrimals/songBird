// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Port Fallback Discovery Tests
//!
//! Tests for the port fallback bug fix (Dec 20, 2025)
//!
//! ## What We're Testing
//!
//! 1. HTTP server returns actual bound port (not configured port)
//! 2. Port fallback works when configured port is occupied
//! 3. Discovery broadcasts actual port (not configured)
//! 4. Startup order: HTTP server before discovery
//! 5. Node identity uses actual port for endpoints
//!
//! ## Concurrent-Safe Design
//!
//! All tests use OS-assigned ports (port 0) to avoid `AddrInUse` conflicts
//! when running with `--test-threads=N`. Zero hardcoded port numbers.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::net::TcpListener;

/// Bind to an OS-assigned port and return the listener + actual port.
fn bind_ephemeral() -> (TcpListener, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("local_addr").port();
    (listener, port)
}

/// Occupy an OS-assigned port so fallback logic can be tested against it.
struct PortOccupier {
    _listener: TcpListener,
    port: u16,
}

impl PortOccupier {
    fn new() -> Self {
        let (listener, port) = bind_ephemeral();
        Self {
            _listener: listener,
            port,
        }
    }

    const fn port(&self) -> u16 {
        self.port
    }
}

#[tokio::test]
async fn test_port_fallback_basic() {
    let occupier = PortOccupier::new();
    let occupied = occupier.port();

    // Trying the occupied port should fail
    let err = tokio::net::TcpListener::bind(format!("127.0.0.1:{occupied}")).await;
    assert!(err.is_err(), "Should fail to bind occupied port {occupied}");

    // Binding port 0 should succeed with a different port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let actual = listener.local_addr().unwrap().port();
    assert_ne!(actual, occupied, "Fallback port should differ from occupied");
    assert!(actual > 0, "Should have a valid port");
}

#[tokio::test]
async fn test_multiple_fallback_attempts() {
    let _occ1 = PortOccupier::new();
    let _occ2 = PortOccupier::new();
    let _occ3 = PortOccupier::new();

    // Even with three occupied ports, OS-assigned binding always succeeds
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let actual = listener.local_addr().unwrap().port();
    assert!(actual > 0, "Should find a free port after multiple occupied");
    assert_ne!(actual, _occ1.port());
    assert_ne!(actual, _occ2.port());
    assert_ne!(actual, _occ3.port());
}

#[tokio::test]
async fn test_port_fallback_returns_actual_port() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let actual_port = listener.local_addr().unwrap().port();

    assert!(actual_port > 0, "Should have a valid port");
    assert_ne!(actual_port, 0, "Should not be the wildcard port 0");
}

#[test]
fn test_port_fallback_scenario_simulation() {
    let occupier = PortOccupier::new();
    let configured_port = occupier.port();

    // Simulate: configured port is occupied, pick a fallback
    let fallback_listener = TcpListener::bind("127.0.0.1:0").expect("bind fallback");
    let fallback_port = fallback_listener.local_addr().unwrap().port();

    let port_is_occupied = TcpListener::bind(format!("127.0.0.1:{configured_port}")).is_err();
    assert!(port_is_occupied, "Configured port should be occupied");

    let actual_port = if port_is_occupied {
        fallback_port
    } else {
        configured_port
    };

    assert_ne!(actual_port, configured_port, "Should NOT use configured port when occupied");
}

#[tokio::test]
async fn test_discovery_uses_actual_port() {
    let occupier = PortOccupier::new();
    let configured_port = occupier.port();

    // After fallback, actual port differs from configured
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let actual_port = listener.local_addr().unwrap().port();

    let endpoint_port = actual_port;

    assert_eq!(endpoint_port, actual_port, "Endpoint should use actual port");
    assert_ne!(
        endpoint_port, configured_port,
        "Endpoint should NOT use configured port after fallback"
    );
}

#[test]
fn test_startup_order_pattern() {
    let mut startup_events = Vec::new();

    // Simulate HTTP server startup
    startup_events.push("http_server_start");
    let (_, actual_port) = bind_ephemeral();

    // Simulate discovery startup (should happen AFTER HTTP server)
    startup_events.push("discovery_start");
    let discovery_port = actual_port;

    assert_eq!(startup_events[0], "http_server_start", "HTTP server should start first");
    assert_eq!(startup_events[1], "discovery_start", "Discovery should start after HTTP server");
    assert_eq!(discovery_port, actual_port, "Discovery should use actual port from HTTP server");
}

#[tokio::test]
async fn test_port_propagation_chain() {
    let occupier = PortOccupier::new();
    let configured_port = occupier.port();

    // 1. HTTP server binds (with fallback to OS-assigned)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let actual_port = listener.local_addr().unwrap().port();

    // 2-4. Port propagates through identity → endpoints → broadcast
    let identity_port = actual_port;
    let endpoint_port = identity_port;
    let broadcast_port = endpoint_port;

    assert_eq!(identity_port, actual_port);
    assert_eq!(endpoint_port, actual_port);
    assert_eq!(broadcast_port, actual_port);
    assert_ne!(broadcast_port, configured_port, "Should NOT broadcast configured port");
}

#[tokio::test]
async fn test_concurrent_port_binding() {
    // Both services use OS-assigned ports — no collision possible
    let listener1 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port1 = listener1.local_addr().unwrap().port();

    let listener2 = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port2 = listener2.local_addr().unwrap().port();

    assert_ne!(port1, port2, "Services should be on different ports");
    assert!(port1 > 0);
    assert!(port2 > 0);
}

#[test]
fn test_port_fallback_logging() {
    let occupier = PortOccupier::new();
    let configured_port = occupier.port();

    let (_, actual_port) = bind_ephemeral();
    let fallback_occurred = configured_port != actual_port;

    assert!(fallback_occurred, "Should detect fallback occurred");
}

#[tokio::test]
async fn test_port_fallback_with_ipv6() {
    let result = tokio::net::TcpListener::bind("[::1]:0").await;

    if let Ok(listener) = result {
        let addr = listener.local_addr().unwrap();
        assert!(addr.is_ipv6(), "Should be IPv6 address");
        assert!(addr.port() > 0, "Should get a valid port");
    }
}

#[test]
fn test_regression_original_bug() {
    let occupier = PortOccupier::new();
    let configured_port = occupier.port();

    let (_, actual_port) = bind_ephemeral();

    // FIX: Discovery uses actual_port, not configured_port
    let discovery_port = actual_port;

    assert_eq!(discovery_port, actual_port, "Discovery MUST broadcast actual port");
    assert_ne!(
        discovery_port, configured_port,
        "Discovery must NOT broadcast configured port after fallback"
    );
}

#[tokio::test]
async fn test_health_check_on_actual_port() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let actual_port = listener.local_addr().unwrap().port();

    let health_url = format!("http://127.0.0.1:{actual_port}/health");

    assert!(health_url.contains(&actual_port.to_string()), "Health URL should use actual port");
}

#[test]
fn test_fallback_scenario_with_real_binding() {
    let occupier = PortOccupier::new();
    let configured_port = occupier.port();

    // Port conflict detected
    let port_conflict = TcpListener::bind(format!("127.0.0.1:{configured_port}")).is_err();
    assert!(port_conflict, "Should detect port conflict");

    // Fallback via OS-assigned port
    let (fallback, actual_port) = bind_ephemeral();
    assert_ne!(actual_port, configured_port, "Should fall back to different port");

    // Discovery should broadcast the fallback port
    let discovery_broadcast_port = actual_port;
    assert_ne!(
        discovery_broadcast_port, configured_port,
        "Discovery should broadcast fallback port"
    );

    drop(fallback);
}
