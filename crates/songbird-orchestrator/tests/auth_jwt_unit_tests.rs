// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unit Tests for `security provider` JWT Delegation
//!
//! Tests the JWT provisioning and capability discovery in isolation.
//!
//! **Concurrency Evolution**: Tests that mutate env vars use a static Mutex
//! to prevent race conditions. This is the correct pattern for env var tests.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use songbird_orchestrator::auth::{
    discover_security_socket, discover_security_socket_for_family, get_security_socket_for_jwt,
    provision_jwt_secret,
};
use std::sync::Mutex;

/// Serialize env var tests — process env vars are global mutable state.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_capability_discovery_with_security_provider() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Clean environment
    songbird_process_env::remove_var("SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_PROVIDER_SOCKET");
    songbird_process_env::remove_var("BEARDOG_SOCKET");

    // Set SECURITY_PROVIDER
    songbird_process_env::set_var("SECURITY_PROVIDER", "/tmp/test-security-provider-unit.sock");

    let socket = discover_security_socket();
    assert!(socket.is_some());
    assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/test-security-provider-unit.sock");

    // Cleanup
    songbird_process_env::remove_var("SECURITY_PROVIDER");
}

#[test]
fn test_capability_discovery_with_legacy_env_socket() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Clean environment
    songbird_process_env::remove_var("SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_PROVIDER_SOCKET");
    songbird_process_env::remove_var("BEARDOG_SOCKET");

    // Set BEARDOG_SOCKET (legacy explicit override; prefer SECURITY_PROVIDER_SOCKET)
    songbird_process_env::set_var("BEARDOG_SOCKET", "/tmp/test-legacy-socket-override.sock");

    let socket = discover_security_socket();
    assert!(socket.is_some());
    assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/test-legacy-socket-override.sock");

    // Cleanup
    songbird_process_env::remove_var("BEARDOG_SOCKET");
}

#[test]
fn test_capability_discovery_priority() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Clean environment
    songbird_process_env::remove_var("SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_PROVIDER_SOCKET");
    songbird_process_env::remove_var("BEARDOG_SOCKET");

    // Set both (SECURITY_PROVIDER should win over SECURITY_PROVIDER_SOCKET and BEARDOG_SOCKET)
    songbird_process_env::set_var("SECURITY_PROVIDER", "/tmp/security-provider.sock");
    songbird_process_env::set_var("SECURITY_PROVIDER_SOCKET", "/tmp/security-provider-socket.sock");
    songbird_process_env::set_var("BEARDOG_SOCKET", "/tmp/legacy-socket.sock");

    let socket = discover_security_socket();
    assert!(socket.is_some());
    assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/security-provider.sock");

    // Cleanup
    songbird_process_env::remove_var("SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_PROVIDER_SOCKET");
    songbird_process_env::remove_var("BEARDOG_SOCKET");
}

#[test]
fn test_family_specific_discovery() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Clean environment
    songbird_process_env::remove_var("SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_PROVIDER_SOCKET");
    songbird_process_env::remove_var("BEARDOG_SOCKET");

    // Family-specific discovery should fall back to generic
    let socket = discover_security_socket_for_family("nat0");

    // May or may not find a socket depending on system state
    // Just verify it doesn't panic
    if let Some(path) = socket {
        println!("Found socket: {}", path.display());
    }
}

#[test]
fn test_get_security_socket_for_jwt() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Clean environment
    songbird_process_env::remove_var("SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_PROVIDER_SOCKET");
    songbird_process_env::remove_var("BEARDOG_SOCKET");

    songbird_process_env::set_var("SECURITY_PROVIDER", "/tmp/jwt-test-unit.sock");

    let socket = get_security_socket_for_jwt();
    assert!(socket.is_some());
    assert_eq!(socket.unwrap(), "/tmp/jwt-test-unit.sock");

    songbird_process_env::remove_var("SECURITY_PROVIDER");
}

#[tokio::test]
async fn test_jwt_provisioning_fallback_secure_random() {
    // No env vars needed — tests pure crypto, fully concurrent-safe
    let secret = provision_jwt_secret(None, "test_unit_purpose").await.unwrap();

    // Should be base64-encoded (64 bytes → ~88 characters)
    assert!(secret.len() >= 85);
    assert!(secret.len() <= 90);
    assert!(!secret.is_empty());

    // Should be different each time
    let secret2 = provision_jwt_secret(None, "test_unit_purpose2").await.unwrap();
    assert_ne!(secret, secret2);
}

#[tokio::test]
async fn test_jwt_provisioning_deterministic_length() {
    // No env vars needed — fully concurrent-safe
    let secret1 = provision_jwt_secret(None, "test1").await.unwrap();
    let secret2 = provision_jwt_secret(None, "test2").await.unwrap();
    let secret3 = provision_jwt_secret(None, "test3").await.unwrap();

    assert_eq!(secret1.len(), secret2.len());
    assert_eq!(secret2.len(), secret3.len());
}

#[tokio::test]
async fn test_jwt_provisioning_base64_validity() {
    // No env vars needed — fully concurrent-safe
    use base64::Engine;

    let secret = provision_jwt_secret(None, "test_base64").await.unwrap();

    // Should be valid base64
    let decoded =
        base64::engine::general_purpose::STANDARD.decode(&secret).expect("Should be valid base64");

    // Should be 64 bytes (512 bits)
    assert_eq!(decoded.len(), 64);
}

#[test]
fn test_capability_discovery_no_env_vars() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Clean environment
    songbird_process_env::remove_var("SECURITY_PROVIDER");
    songbird_process_env::remove_var("SECURITY_PROVIDER_SOCKET");
    songbird_process_env::remove_var("BEARDOG_SOCKET");

    let socket = discover_security_socket();

    // May or may not find a socket depending on system state
    // Just verify it doesn't panic and returns Option
    match socket {
        Some(path) => println!("Found socket at: {}", path.display()),
        None => println!("No socket found (expected without env vars)"),
    }
}
