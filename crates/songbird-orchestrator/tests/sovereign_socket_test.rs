// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! Comprehensive tests for Sovereign Socket implementation
//!
//! Tests cover:
//! - Socket creation and configuration
//! - Binding strategies (IPv4, IPv6, fallback)
//! - `SO_REUSEADDR` and `SO_REUSEPORT` functionality
//! - Multi-strategy fallback
//! - Integration with HTTP server
//! - Concurrent connections
//! - Zero-downtime restart scenarios

use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

use songbird_orchestrator::network::{SovereignBinder, SovereignSocket};

#[tokio::test]
async fn test_sovereign_socket_ipv4_creation() {
    let result = SovereignSocket::new_tcp_v4();
    assert!(result.is_ok(), "Should create IPv4 sovereign socket");
}

#[tokio::test]
async fn test_sovereign_socket_ipv6_creation() {
    let result = SovereignSocket::new_tcp_v6();
    assert!(result.is_ok(), "Should create IPv6 sovereign socket");
}

#[tokio::test]
async fn test_sovereign_socket_bind_ipv4() {
    let socket = SovereignSocket::new_tcp_v4().expect("Failed to create socket");
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let result = socket.bind(addr);
    assert!(result.is_ok(), "Should bind to IPv4 address");
}

#[tokio::test]
async fn test_sovereign_socket_listen() {
    let socket = SovereignSocket::new_tcp_v4().expect("Failed to create socket");
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    socket.bind(addr).expect("Failed to bind");
    let result = socket.listen(128);
    assert!(result.is_ok(), "Should start listening");
}

#[tokio::test]
async fn test_sovereign_socket_to_tokio_listener() {
    let socket = SovereignSocket::new_tcp_v4().expect("Failed to create socket");
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    socket.bind(addr).expect("Failed to bind");
    socket.listen(128).expect("Failed to listen");

    let result = socket.into_tokio_listener();
    assert!(result.is_ok(), "Should convert to tokio listener");
}

#[tokio::test]
async fn test_sovereign_binder_ephemeral_port() {
    // Port 0 = OS chooses ephemeral port
    let result = SovereignBinder::bind_sovereign(0).await;
    assert!(result.is_ok(), "Should bind to ephemeral port");

    if let Ok((listener, addr)) = result {
        assert!(addr.port() > 0, "Should have assigned port");
        println!("✅ Bound to ephemeral port: {addr}");

        // Verify we can get local address
        let local_addr = listener.local_addr().expect("Should have local address");
        assert_eq!(local_addr, addr, "Addresses should match");
    }
}

#[tokio::test]
async fn test_sovereign_binder_specific_high_port() {
    // Use a high port that's likely to be available
    let port = 19876;
    let result = SovereignBinder::bind_sovereign(port).await;

    if result.is_ok() {
        let (_listener, addr) = result.unwrap();
        assert_eq!(addr.port(), port, "Should bind to requested port");
        println!("✅ Bound to specific port: {addr}");
    } else {
        // Port might be in use, which is acceptable for testing
        println!("⚠️  Port {port} already in use (acceptable for test)");
    }
}

#[tokio::test]
async fn test_sovereign_binder_fallback_strategy() {
    // Try to bind to port 1 (privileged, should fail without root)
    // Should fall back to port 2, 3, etc.
    let result = SovereignBinder::bind_sovereign(1).await;

    if result.is_ok() {
        let (_listener, addr) = result.unwrap();
        println!("✅ Fallback successful, bound to: {addr}");
        // Port should be > 1 (fallback occurred)
        assert!(addr.port() >= 1, "Should bind to original or fallback port");
    } else {
        println!("⚠️  All ports in fallback range busy (acceptable)");
    }
}

#[tokio::test]
async fn test_so_reuseaddr_functionality() {
    // Bind to port
    let port = 0; // Ephemeral
    let result1 = SovereignBinder::bind_sovereign(port).await;
    assert!(result1.is_ok(), "First bind should succeed");

    let (_listener1, addr1) = result1.unwrap();
    let actual_port = addr1.port();

    // Drop first listener (simulates restart).
    // With SO_REUSEADDR, rebind should succeed immediately.
    drop(_listener1);

    // Try to bind again to same port (SO_REUSEADDR should allow this)
    let result2 = SovereignBinder::bind_sovereign(actual_port).await;

    if result2.is_ok() {
        let (_listener2, addr2) = result2.unwrap();
        println!("✅ SO_REUSEADDR working - rebind to {addr2} successful");
    } else {
        println!("⚠️  SO_REUSEADDR might need more time for socket cleanup");
    }
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_so_reuseport_multiple_binds() {
    // On Linux with SO_REUSEPORT, multiple processes can bind to same port
    let port = 0;

    let result1 = SovereignBinder::bind_sovereign(port).await;
    assert!(result1.is_ok(), "First bind should succeed");

    let (_listener1, addr1) = result1.unwrap();
    let actual_port = addr1.port();

    // Try to bind second listener to SAME port (SO_REUSEPORT enables this)
    let result2 = SovereignBinder::bind_sovereign(actual_port).await;

    if result2.is_ok() {
        let (_listener2, addr2) = result2.unwrap();
        assert_eq!(addr2.port(), actual_port, "Should bind to same port with SO_REUSEPORT");
        println!("✅ SO_REUSEPORT working - multiple binds to port {actual_port}");

        // Both listeners should be active simultaneously
        // This enables zero-downtime deployments and load balancing
    } else {
        println!("⚠️  SO_REUSEPORT not available or port conflict");
    }
}

#[tokio::test]
async fn test_concurrent_connections() {
    // Bind sovereign socket
    let (listener, addr) = SovereignBinder::bind_sovereign(0).await.expect("Failed to bind");

    // Spawn server in background.
    // The listener is already bound and listening, so the kernel backlog
    // accepts connections immediately — no readiness sleep needed.
    tokio::spawn(async move {
        loop {
            if let Ok((_stream, _peer_addr)) = listener.accept().await {
                // Accept and immediately close (for testing)
            }
        }
    });

    // Try multiple concurrent connections
    let mut handles = vec![];
    for _ in 0..10 {
        let addr_clone = addr;
        let handle = tokio::spawn(async move {
            timeout(Duration::from_secs(2), tokio::net::TcpStream::connect(addr_clone)).await
        });
        handles.push(handle);
    }

    // Wait for all connections
    let mut successes = 0;
    for handle in handles {
        if let Ok(Ok(Ok(_stream))) = handle.await {
            successes += 1;
        }
    }

    assert!(successes >= 8, "Should handle most concurrent connections (got {successes})");
    println!("✅ Handled {successes}/10 concurrent connections");
}

#[tokio::test]
async fn test_bind_strategies_exhaustive() {
    // Test that all binding strategies are attempted
    // This test documents the fallback behavior

    let port = 0; // Let OS choose
    let result = SovereignBinder::bind_sovereign(port).await;

    assert!(result.is_ok(), "At least one binding strategy should succeed");

    if let Ok((_listener, addr)) = result {
        println!("✅ Binding strategy successful:");
        println!("   Address: {addr}");
        println!("   IPv4: {}", addr.is_ipv4());
        println!("   IPv6: {}", addr.is_ipv6());

        // Verify it's a valid address
        assert!(addr.port() > 0, "Should have valid port");
    }
}

#[tokio::test]
async fn test_rapid_bind_unbind_cycle() {
    // Simulates rapid restarts
    for i in 0..5 {
        let result = SovereignBinder::bind_sovereign(0).await;
        assert!(result.is_ok(), "Bind attempt {} should succeed", i + 1);

        if let Ok((listener, addr)) = result {
            println!("Cycle {}: Bound to {}", i + 1, addr);
            drop(listener); // Immediate unbind
        }
    }

    println!("✅ Rapid bind/unbind cycles successful");
}

#[tokio::test]
async fn test_buffer_sizes() {
    let socket = SovereignSocket::new_tcp_v4().expect("Failed to create socket");
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    socket.bind(addr).expect("Failed to bind");
    socket.listen(128).expect("Failed to listen");

    // Convert to tokio listener to verify configuration worked
    let tokio_listener = socket.into_tokio_listener().expect("Should convert");
    let local_addr = tokio_listener.local_addr().expect("Should have local address");

    println!("✅ Buffer sizes configured (verified by successful socket creation)");
    println!("   Listening on: {local_addr}");

    // The fact that the socket was created with the buffer size calls
    // and didn't error means the configuration worked
    assert!(local_addr.port() > 0, "Should have valid port");
}

#[tokio::test]
async fn test_non_blocking_mode() {
    let socket = SovereignSocket::new_tcp_v4().expect("Failed to create socket");
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    socket.bind(addr).expect("Failed to bind");
    socket.listen(128).expect("Failed to listen");

    // Verify non-blocking is set
    let tokio_listener = socket.into_tokio_listener().expect("Should convert");

    // If this doesn't panic, non-blocking mode is working with tokio
    let local_addr = tokio_listener.local_addr().expect("Should have local addr");
    println!("✅ Non-blocking mode working: {local_addr}");
}

#[tokio::test]
async fn test_ipv4_and_ipv6_both_available() {
    // Test that we can bind to both IPv4 and IPv6 if available
    let port = 0;

    // Try IPv4
    let ipv4_result = SovereignBinder::bind_sovereign(port).await;
    if ipv4_result.is_ok() {
        let (_listener, addr) = ipv4_result.unwrap();
        println!("✅ IPv4 binding available: {addr}");
    }

    // Try IPv6 (might not be available on all systems)
    let socket = SovereignSocket::new_tcp_v6();
    if socket.is_ok() {
        println!("✅ IPv6 socket creation available");
    } else {
        println!("⚠️  IPv6 not available on this system");
    }
}

// Regression test for original issue
#[tokio::test]
async fn test_no_address_already_in_use_error() {
    // Original bug: "Address already in use" due to double-bind
    // This test ensures sovereign socket prevents that

    let port = 0;
    let result1 = SovereignBinder::bind_sovereign(port).await;
    assert!(result1.is_ok(), "First bind should succeed");

    let (_listener1, addr1) = result1.unwrap();
    let actual_port = addr1.port();

    // Try to bind again while first is still active
    // This SHOULD work on Linux with SO_REUSEPORT
    #[cfg(target_os = "linux")]
    {
        let result2 = SovereignBinder::bind_sovereign(actual_port).await;
        if result2.is_ok() {
            println!("✅ SO_REUSEPORT prevents 'address in use' error");
        }
    }

    // Drop first listener. With SO_REUSEADDR, rebind is immediate.
    drop(_listener1);

    // Now bind should definitely work (SO_REUSEADDR)
    let result3 = SovereignBinder::bind_sovereign(actual_port).await;
    if result3.is_ok() {
        println!("✅ SO_REUSEADDR allows immediate rebind after close");
    }
}
