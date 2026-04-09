// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// BiomeOS Neural API Socket Environment Variable Compatibility Test
// January 15, 2026
// Updated: February 10, 2026 (concurrent-safe with per-file Mutex)
//
// Validates that Songbird honors BiomeOS Neural API environment variable standards
// as documented in the upstream handoff from BiomeOS team.
//
// PRIMAL_DEPLOYMENT_STANDARD changes (Feb 5, 2026):
// - Socket names are now {primal}.sock (no family suffix)
// - XDG_RUNTIME_DIR/biomeos/ is the preferred location
// - Family ID is NOT included in socket path
//
// **Concurrency Evolution**: These tests mutate process-wide env vars.
// A static Mutex ensures they don't race with each other within this binary.
// This is NOT a production concern — production reads env vars once at startup.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use songbird_orchestrator::ipc::UnixSocketServer;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

/// Serialize all env var tests in this file.
/// Process env vars are global state — there is no way around serialization here.
/// This is the correct pattern: env var tests serialize, everything else runs concurrent.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Test socket path derivation with `BiomeOS` Neural API environment variables
#[test]
fn test_biomeos_neural_api_socket_path_priority() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Save original env state
    let original_orchestrator_socket = env::var("SONGBIRD_ORCHESTRATOR_SOCKET").ok();
    let original_socket = env::var("SONGBIRD_SOCKET").ok();
    let original_biomeos_path = env::var("BIOMEOS_SOCKET_PATH").ok();
    let original_family_id = env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID").ok();
    let original_biomeos_family = env::var("BIOMEOS_FAMILY_ID").ok();

    // Clear all env vars to start clean (including env_config vars)
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
    songbird_process_env::remove_var("SONGBIRD_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID"); // env_config also reads this!

    // Test 1: SONGBIRD_ORCHESTRATOR_SOCKET (highest priority - Neural API standard)
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/songbird-nat0.sock");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-nat0.sock"),
        "SONGBIRD_ORCHESTRATOR_SOCKET should be highest priority"
    );
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");

    // Test 2: SONGBIRD_SOCKET (alternative naming)
    songbird_process_env::set_var("SONGBIRD_SOCKET", "/tmp/songbird-custom.sock");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-custom.sock"),
        "SONGBIRD_SOCKET should be second priority"
    );
    songbird_process_env::remove_var("SONGBIRD_SOCKET");

    // Test 3: BIOMEOS_SOCKET_PATH (generic orchestrator)
    songbird_process_env::set_var("BIOMEOS_SOCKET_PATH", "/tmp/songbird-biomeos.sock");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-biomeos.sock"),
        "BIOMEOS_SOCKET_PATH should be third priority"
    );
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");

    // Test 4: Family ID from SONGBIRD_ORCHESTRATOR_FAMILY_ID
    // PRIMAL_SELF_KNOWLEDGE_STANDARD v1.1: Family ID IS in socket path (production mode)
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "nat0");
    let path = UnixSocketServer::socket_path_from_env();
    let family = UnixSocketServer::get_family_id();
    assert!(
        path.to_str().unwrap().ends_with("songbird-nat0.sock"),
        "Socket path should be domain-fid.sock per PRIMAL_SELF_KNOWLEDGE_STANDARD v1.1"
    );
    assert_eq!(
        family, "nat0",
        "Family ID should be extracted from SONGBIRD_ORCHESTRATOR_FAMILY_ID"
    );
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");

    // Test 5: Family ID from BIOMEOS_FAMILY_ID (generic orchestrator)
    // PRIMAL_SELF_KNOWLEDGE_STANDARD v1.1: Family-scoped socket
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "nat0");
    let path = UnixSocketServer::socket_path_from_env();
    let family = UnixSocketServer::get_family_id();
    assert!(
        path.to_str().unwrap().ends_with("songbird-nat0.sock"),
        "Socket path should be domain-fid.sock per PRIMAL_SELF_KNOWLEDGE_STANDARD v1.1"
    );
    assert_eq!(family, "nat0", "Family ID should be extracted from BIOMEOS_FAMILY_ID");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");

    // Test 6: Default behavior (no env vars) - ensure all vars are cleared
    // PRIMAL_DEPLOYMENT_STANDARD: Socket is {primal}.sock, uses XDG or /tmp
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
    songbird_process_env::remove_var("SONGBIRD_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_DIR");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID");
    let path = UnixSocketServer::socket_path_from_env();
    assert!(
        path.to_str().unwrap().ends_with("songbird.sock"),
        "Socket should be primal-named bind path; domain is network.sock symlink"
    );

    // Test 7: Neural API standard deployment (full environment)
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/songbird-nat0.sock");
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "nat0");
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "nat0");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-nat0.sock"),
        "Neural API deployment should honor SONGBIRD_ORCHESTRATOR_SOCKET"
    );

    // Restore original env state
    restore_env_var("SONGBIRD_ORCHESTRATOR_SOCKET", original_orchestrator_socket);
    restore_env_var("SONGBIRD_SOCKET", original_socket);
    restore_env_var("BIOMEOS_SOCKET_PATH", original_biomeos_path);
    restore_env_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", original_family_id);
    restore_env_var("BIOMEOS_FAMILY_ID", original_biomeos_family);
}

/// Test that socket path follows XDG or /tmp fallback
/// `PRIMAL_DEPLOYMENT_STANDARD`: `XDG_RUNTIME_DIR/biomeos`/ is preferred, /tmp is fallback
#[test]
fn test_default_socket_directory_is_tmp() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Clear all explicit socket env vars (but NOT XDG_RUNTIME_DIR)
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
    songbird_process_env::remove_var("SONGBIRD_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_DIR");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");

    let path = UnixSocketServer::socket_path_from_env();
    let path_str = path.to_str().unwrap();

    // PRIMAL_DEPLOYMENT_STANDARD: Socket is {primal}.sock
    // Should be in XDG_RUNTIME_DIR/biomeos/ or /tmp/ fallback
    assert!(
        path_str.starts_with("/run/user/") || path_str.starts_with("/tmp/"),
        "Default socket should be in XDG or /tmp, got: {path_str}"
    );
    assert!(
        path_str.ends_with("songbird.sock"),
        "Socket should be primal-named bind path per PRIMAL_SELF_KNOWLEDGE_STANDARD v1.1, got: {path_str}"
    );
}

fn restore_env_var(key: &str, value: Option<String>) {
    match value {
        Some(v) => songbird_process_env::set_var(key, v),
        None => songbird_process_env::remove_var(key),
    }
}

#[test]
fn test_family_id_priority_order() {
    let _guard = ENV_LOCK.lock().unwrap();

    // Clear all env vars (including env_config vars)
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
    songbird_process_env::remove_var("SONGBIRD_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_DIR");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID");

    // Test 1: Explicit socket path (bypasses family ID)
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/songbird-explicit.sock");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-explicit.sock"),
        "Explicit socket path should bypass family ID resolution"
    );
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");

    // Test 2: SONGBIRD_FAMILY_ID priority for family ID extraction
    // PRIMAL_SELF_KNOWLEDGE_STANDARD v1.1: Family ID IS in socket path (production mode)
    songbird_process_env::set_var("SONGBIRD_FAMILY_ID", "custom");
    songbird_process_env::set_var("FAMILY_ID", "wrong");
    let path = UnixSocketServer::socket_path_from_env();
    let family = UnixSocketServer::get_family_id();
    assert!(
        path.to_str().unwrap().ends_with("songbird-custom.sock"),
        "Socket should be domain-fid.sock per PRIMAL_SELF_KNOWLEDGE_STANDARD v1.1"
    );
    assert_eq!(family, "custom", "SONGBIRD_FAMILY_ID should be correctly extracted");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");

    // Test 3: BIOMEOS_FAMILY_ID (fallback when SONGBIRD_FAMILY_ID not set)
    // PRIMAL_SELF_KNOWLEDGE_STANDARD v1.1: Family-scoped socket
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "generic");
    let path = UnixSocketServer::socket_path_from_env();
    let family = UnixSocketServer::get_family_id();
    assert!(
        path.to_str().unwrap().ends_with("songbird-generic.sock"),
        "Socket should be domain-fid.sock per PRIMAL_SELF_KNOWLEDGE_STANDARD v1.1"
    );
    assert_eq!(family, "generic", "BIOMEOS_FAMILY_ID should be used as fallback");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID");

    // Test 4: Default (no env vars) - socket is domain.sock
    let path = UnixSocketServer::socket_path_from_env();
    let family = UnixSocketServer::get_family_id();
    assert!(
        path.to_str().unwrap().ends_with("songbird.sock"),
        "Socket should be primal-named bind path; domain is network.sock symlink"
    );
    assert_eq!(family, "default", "Should default when no env vars set");
}
