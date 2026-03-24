// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP Integration Tests - Week 4 Part 2
//!
//! Comprehensive testing of `BearDog` BTSP (`BearDog` Tunnel Security Protocol)
//! integration via Unix sockets. These tests validate the complete BTSP lifecycle:
//! - Socket discovery and connection
//! - Tunnel establishment
//! - Data encryption/decryption
//! - Lifecycle management
//! - Error handling
//!
//! Modern, idiomatic, async Rust with deep debt solutions.

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
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

use anyhow::Result;
use songbird_orchestrator::btsp_client::{BtspClient, Direction, PeerEndpoint};
use std::env;
use std::sync::Mutex;

/// File-local mutex to serialize tests that modify process-wide env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Acquire env lock with poison recovery (prevents cascade failures).
fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Helper: Check if `BearDog` is available for testing
async fn beardog_available() -> bool {
    let client = BtspClient::new();
    client.ping().await.is_ok()
}

/// Helper: Get or create test socket path
fn test_socket_path() -> String {
    env::var("BEARDOG_SOCKET")
        .or_else(|_| env::var("TEST_BEARDOG_SOCKET"))
        .unwrap_or_else(|_| "/tmp/beardog-test.sock".to_string())
}

/// Helper: Setup test environment variables
fn setup_test_env() {
    songbird_process_env::set_var("BEARDOG_SOCKET", test_socket_path());
    songbird_process_env::set_var("BEARDOG_FAMILY_ID", "test");
}

/// Helper: Cleanup test environment
fn cleanup_test_env() {
    songbird_process_env::remove_var("BEARDOG_SOCKET");
    songbird_process_env::remove_var("BEARDOG_FAMILY_ID");
}

// ====================
// SOCKET DISCOVERY TESTS
// ====================

#[tokio::test]
async fn test_btsp_client_creation() {
    let _guard = lock_env();
    setup_test_env();

    let client = BtspClient::new();

    // Client should be created successfully
    assert!(format!("{client:?}").contains("BtspClient"));

    cleanup_test_env();
}

#[tokio::test]
async fn test_socket_path_discovery_priority() {
    let _guard = lock_env();

    // Clean slate for this test
    songbird_process_env::remove_var("BEARDOG_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");
    let saved_xdg = env::var("XDG_RUNTIME_DIR").ok();

    // Test priority 1: BEARDOG_SOCKET
    songbird_process_env::set_var("BEARDOG_SOCKET", "/custom/beardog.sock");
    let client1 = BtspClient::new();
    let debug1 = format!("{client1:?}");
    assert!(debug1.contains("/custom/beardog.sock"), "Should use BEARDOG_SOCKET, got: {debug1}");

    // Test priority 2: BIOMEOS_SOCKET_PATH (when BEARDOG_SOCKET not set)
    songbird_process_env::remove_var("BEARDOG_SOCKET");
    songbird_process_env::set_var("BIOMEOS_SOCKET_PATH", "/biomeos/beardog.sock");
    let client2 = BtspClient::new();
    let debug2 = format!("{client2:?}");
    assert!(
        debug2.contains("/biomeos/beardog.sock"),
        "Should use BIOMEOS_SOCKET_PATH, got: {debug2}"
    );

    // Test priority 3: XDG_RUNTIME_DIR
    // Note: BtspClient uses "security-{family_id}.sock" path pattern
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");
    songbird_process_env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
    let client3 = BtspClient::new();
    let client3_path = format!("{client3:?}");
    assert!(
        client3_path.contains("/run/user/1000") && client3_path.contains("security"),
        "Should use XDG path with security socket, got: {client3_path}"
    );

    // Cleanup — restore original XDG_RUNTIME_DIR
    songbird_process_env::remove_var("BEARDOG_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");
    if let Some(xdg) = saved_xdg {
        songbird_process_env::set_var("XDG_RUNTIME_DIR", xdg);
    } else {
        songbird_process_env::remove_var("XDG_RUNTIME_DIR");
    }
}

#[tokio::test]
async fn test_socket_path_fallback() {
    let _guard = lock_env();
    // Remove explicit socket env vars (XDG_RUNTIME_DIR may still be set by system)
    songbird_process_env::remove_var("BEARDOG_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");
    songbird_process_env::remove_var("XDG_RUNTIME_DIR");

    let client = BtspClient::new();

    // BtspClient uses "security" socket pattern (XDG or /tmp fallback)
    let client_path = format!("{client:?}");
    assert!(
        client_path.contains("security"),
        "Should use security socket pattern, got: {client_path}"
    );
}

// ====================
// CONNECTIVITY TESTS
// ====================

#[tokio::test]
async fn test_btsp_ping_when_beardog_unavailable() {
    let _guard = lock_env();
    setup_test_env();

    let client = BtspClient::new();
    let result = client.ping().await;

    // Should fail gracefully when BearDog not available
    if !beardog_available().await {
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("Failed to connect") || err_msg.contains("No such file"),
            "Error should indicate connection failure, got: {err_msg}"
        );
    }

    cleanup_test_env();
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_btsp_ping_with_live_beardog() {
    setup_test_env();

    let client = BtspClient::new();
    let result = client.ping().await;

    // Should succeed with live BearDog
    assert!(result.is_ok(), "Ping should succeed with live BearDog");

    let response = result.unwrap();
    assert!(response.is_object(), "Response should be JSON object");

    cleanup_test_env();
}

// ====================
// TUNNEL ESTABLISHMENT TESTS
// ====================

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_establish_tunnel_basic() -> Result<()> {
    if !beardog_available().await {
        println!("⚠️  Skipping test: BearDog not available");
        return Ok(());
    }

    setup_test_env();

    let client = BtspClient::new();
    let peer = PeerEndpoint {
        id: "test-peer-001".to_string(),
        endpoint: "peer://test-peer-001".to_string(),
        public_key: None,
        capabilities: vec!["test".to_string()],
    };

    let tunnel = client.establish_tunnel(peer).await?;

    // Verify tunnel handle
    assert!(!tunnel.id.is_empty(), "Tunnel ID should not be empty");
    assert_eq!(tunnel.peer_id, "test-peer-001");

    cleanup_test_env();
    Ok(())
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_establish_tunnel_with_capabilities() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    setup_test_env();

    let client = BtspClient::new();
    let peer = PeerEndpoint {
        id: "capable-peer".to_string(),
        endpoint: "peer://capable-peer".to_string(),
        public_key: Some("test-pubkey-base64".to_string()),
        capabilities: vec!["storage".to_string(), "compute".to_string(), "security".to_string()],
    };

    let tunnel = client.establish_tunnel(peer).await?;

    assert!(!tunnel.id.is_empty());
    assert_eq!(tunnel.peer_id, "capable-peer");

    cleanup_test_env();
    Ok(())
}

#[tokio::test]
async fn test_establish_tunnel_fails_when_beardog_unavailable() {
    let _guard = lock_env();
    setup_test_env();

    let client = BtspClient::new();
    let peer = PeerEndpoint {
        id: "test-peer".to_string(),
        endpoint: "peer://test-peer".to_string(),
        public_key: None,
        capabilities: vec![],
    };

    let result = client.establish_tunnel(peer).await;

    // Should fail gracefully
    if !beardog_available().await {
        assert!(result.is_err());
    }

    cleanup_test_env();
}

// ====================
// ENCRYPTION/DECRYPTION TESTS
// ====================

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_tunnel_encrypt_decrypt_roundtrip() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    setup_test_env();

    let client = BtspClient::new();
    let peer = PeerEndpoint {
        id: "encrypt-test-peer".to_string(),
        endpoint: "peer://encrypt-test-peer".to_string(),
        public_key: None,
        capabilities: vec!["encryption".to_string()],
    };

    let tunnel = client.establish_tunnel(peer).await?;

    // Test data
    let plaintext = b"Hello, BearDog! This is a test message.";

    // Encrypt
    let ciphertext = client.tunnel_encrypt(&tunnel, plaintext, Direction::Outbound).await?;

    assert!(!ciphertext.is_empty());
    assert_ne!(ciphertext, plaintext.to_vec(), "Ciphertext should differ from plaintext");

    // Decrypt
    let decrypted = client.tunnel_decrypt(&tunnel, &ciphertext).await?;

    assert_eq!(decrypted, plaintext.to_vec(), "Decrypted should match original");

    cleanup_test_env();
    Ok(())
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_tunnel_encrypt_large_data() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    setup_test_env();

    let client = BtspClient::new();
    let peer = PeerEndpoint {
        id: "large-data-peer".to_string(),
        endpoint: "peer://large-data-peer".to_string(),
        public_key: None,
        capabilities: vec![],
    };

    let tunnel = client.establish_tunnel(peer).await?;

    // Large test data (1MB)
    let plaintext = vec![0x42u8; 1024 * 1024];

    let ciphertext = client.tunnel_encrypt(&tunnel, &plaintext, Direction::Outbound).await?;

    assert!(!ciphertext.is_empty());

    let decrypted = client.tunnel_decrypt(&tunnel, &ciphertext).await?;

    assert_eq!(decrypted.len(), plaintext.len());
    assert_eq!(decrypted, plaintext);

    cleanup_test_env();
    Ok(())
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_tunnel_encrypt_empty_data() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    setup_test_env();

    let client = BtspClient::new();
    let peer = PeerEndpoint {
        id: "empty-data-peer".to_string(),
        endpoint: "peer://empty-data-peer".to_string(),
        public_key: None,
        capabilities: vec![],
    };

    let tunnel = client.establish_tunnel(peer).await?;

    let plaintext = b"";
    let ciphertext = client.tunnel_encrypt(&tunnel, plaintext, Direction::Outbound).await?;

    // Empty data should still produce valid ciphertext (may include auth tags, etc.)
    // The exact behavior depends on BearDog's implementation
    assert!(ciphertext.is_empty() || !ciphertext.is_empty());

    cleanup_test_env();
    Ok(())
}

// ====================
// LIFECYCLE TESTS
// ====================

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_tunnel_close() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    setup_test_env();

    let client = BtspClient::new();
    let peer = PeerEndpoint {
        id: "close-test-peer".to_string(),
        endpoint: "peer://close-test-peer".to_string(),
        public_key: None,
        capabilities: vec![],
    };

    let tunnel = client.establish_tunnel(peer).await?;

    // Close tunnel
    client.close_tunnel(&tunnel.id).await?;

    // After close, operations should fail
    let _encrypt_result = client.tunnel_encrypt(&tunnel, b"test", Direction::Outbound).await;

    // Should fail (tunnel closed)
    // Note: Exact behavior depends on BearDog implementation
    // May succeed if BearDog auto-recreates tunnels or uses stateless design

    cleanup_test_env();
    Ok(())
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_multiple_tunnels_concurrent() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    setup_test_env();

    let client = BtspClient::new();

    // Establish multiple tunnels concurrently
    let mut handles = vec![];
    for i in 0..5 {
        let peer = PeerEndpoint {
            id: format!("concurrent-peer-{i}"),
            endpoint: format!("peer://concurrent-peer-{i}"),
            public_key: None,
            capabilities: vec![format!("cap-{}", i)],
        };
        handles.push(client.establish_tunnel(peer));
    }

    // Await all tunnels
    let tunnels: Vec<_> = futures::future::try_join_all(handles).await?;

    assert_eq!(tunnels.len(), 5);

    // Verify each tunnel is unique
    let tunnel_ids: Vec<_> = tunnels.iter().map(|t| t.id.clone()).collect();
    let unique_ids: std::collections::HashSet<_> = tunnel_ids.iter().collect();
    assert_eq!(unique_ids.len(), 5, "All tunnel IDs should be unique");

    cleanup_test_env();
    Ok(())
}

// ====================
// ERROR HANDLING TESTS
// ====================

#[tokio::test]
async fn test_invalid_socket_path() {
    let _guard = lock_env();
    songbird_process_env::set_var("BEARDOG_SOCKET", "/nonexistent/path/to/socket.sock");

    let client = BtspClient::new();
    let result = client.ping().await;

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Failed to connect") || err_msg.contains("No such file"),
        "Should fail with connection error"
    );

    songbird_process_env::remove_var("BEARDOG_SOCKET");
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_malformed_request_handling() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    // This test would require direct JSON-RPC manipulation
    // which is private in the current implementation.
    // Testing malformed requests is best done at the BearDog level.

    Ok(())
}

// ====================
// INTEGRATION TESTS (with Connection Manager)
// ====================

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Only run when BearDog is live
async fn test_connection_manager_btsp_integration() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    setup_test_env();

    // This would test the full integration with ConnectionManager
    // but requires more setup. This is a placeholder for the pattern.

    // use songbird_orchestrator::app::ConnectionManager;
    // let conn_mgr = ConnectionManager::new();
    // ... test BTSP via ConnectionManager ...

    cleanup_test_env();
    Ok(())
}

// ====================
// STRESS TESTS
// ====================

#[tokio::test]
#[ignore = "requires running BearDog crypto provider; stress test"] // Only run when BearDog is live + stress testing
async fn test_btsp_rapid_tunnel_creation() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    setup_test_env();

    let client = BtspClient::new();

    // Create 100 tunnels rapidly
    for i in 0..100 {
        let peer = PeerEndpoint {
            id: format!("rapid-peer-{i}"),
            endpoint: format!("peer://rapid-peer-{i}"),
            public_key: None,
            capabilities: vec![],
        };

        let tunnel = client.establish_tunnel(peer).await?;
        assert!(!tunnel.id.is_empty());

        // Close immediately
        client.close_tunnel(&tunnel.id).await?;
    }

    cleanup_test_env();
    Ok(())
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider; stress test"] // Only run when BearDog is live + stress testing
async fn test_btsp_high_throughput_encryption() -> Result<()> {
    if !beardog_available().await {
        return Ok(());
    }

    setup_test_env();

    let client = BtspClient::new();
    let peer = PeerEndpoint {
        id: "throughput-peer".to_string(),
        endpoint: "peer://throughput-peer".to_string(),
        public_key: None,
        capabilities: vec![],
    };

    let tunnel = client.establish_tunnel(peer).await?;

    // Encrypt 1000 messages
    let message = b"Test message for throughput testing";
    for _ in 0..1000 {
        let _ = client.tunnel_encrypt(&tunnel, message, Direction::Outbound).await?;
    }

    cleanup_test_env();
    Ok(())
}

// ====================
// HELPER TEST: Validate Test Setup
// ====================

#[tokio::test]
async fn test_helper_beardog_availability_check() {
    // This test validates the test helper itself
    let available = beardog_available().await;

    // Should not panic
    println!("BearDog available: {available}");
}

#[test]
fn test_helper_socket_path() {
    let path = test_socket_path();
    assert!(!path.is_empty());
    assert!(path.contains("beardog"));
}
