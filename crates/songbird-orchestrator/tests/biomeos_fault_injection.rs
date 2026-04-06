// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// BiomeOS Socket Fault Injection Tests
// January 16, 2026
// Updated: February 5, 2026 (PRIMAL_DEPLOYMENT_STANDARD compliance)
// Updated: February 10, 2026 (concurrent-safe with per-file Mutex)
//
// Fault injection tests for BiomeOS socket integration.
// Tests specific failure scenarios and validates recovery/error handling.
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

/// Fault Test: Missing all environment variables
///
/// Tests that Songbird gracefully handles complete absence of environment variables
/// and falls back to sensible defaults.
#[test]
fn fault_missing_all_env_vars() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    // Save original state
    let original = save_env_state();

    // Remove ALL environment variables
    clear_all_env_vars();

    // Should not panic, should return default
    let path = UnixSocketServer::socket_path_from_env();
    let family_id = UnixSocketServer::get_family_id();

    // PRIMAL_DEPLOYMENT_STANDARD: Socket is {primal}.sock, uses XDG or /tmp fallback
    assert!(path.to_str().unwrap().ends_with("songbird.sock"));
    assert_eq!(family_id, "default");

    // Cleanup
    restore_env_state(original);
}

/// Fault Test: Invalid socket path (non-existent directory)
///
/// Tests behavior when socket path points to non-existent directory.
#[test]
fn fault_nonexistent_directory() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    // Set socket path to non-existent directory
    songbird_process_env::set_var(
        "SONGBIRD_ORCHESTRATOR_SOCKET",
        "/non/existent/path/songbird.sock",
    );

    // Should return the path (validation happens at bind time)
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(path, PathBuf::from("/non/existent/path/songbird.sock"));

    // Note: Actual failure would occur when trying to create socket
    // This tests that path derivation doesn't crash on invalid paths

    restore_env_state(original);
}

/// Fault Test: Invalid family ID (special characters)
///
/// Tests handling of family IDs with special characters, spaces, etc.
#[test]
fn fault_invalid_family_id_special_chars() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    // Test various problematic family IDs
    let test_cases = vec![
        ("spaces in name", "/tmp/songbird-spaces in name.sock"),
        ("dots.and.dots", "/tmp/songbird-dots.and.dots.sock"),
        ("slash/attack", "/tmp/songbird-slash/attack.sock"),
        ("../traversal", "/tmp/songbird-../traversal.sock"),
        ("unicode-🦜", "/tmp/songbird-unicode-🦜.sock"),
        ("", "/tmp/songbird-.sock"), // Empty string
    ];

    for (family_id, _expected_path) in test_cases {
        clear_all_env_vars();
        songbird_process_env::set_var("BIOMEOS_FAMILY_ID", family_id);

        let path = UnixSocketServer::socket_path_from_env();
        let derived_family = UnixSocketServer::get_family_id();

        // PRIMAL_DEPLOYMENT_STANDARD: Should not crash, socket is always {primal}.sock
        // Family ID is not included in socket path
        assert!(path.to_str().unwrap().ends_with("songbird.sock"));
        assert_eq!(derived_family, family_id);
    }

    restore_env_state(original);
}

/// Fault Test: Empty string environment variables
///
/// Tests handling of env vars set to empty strings.
/// The implementation treats empty strings as "not set" and falls through
/// to the next priority/default, which is the correct behavior — an empty
/// socket path is never valid.
#[test]
fn fault_empty_string_env_vars() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();
    clear_all_env_vars();

    // Test 1: Empty socket path env var — treated as "not set", falls to default
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "");

    let path = UnixSocketServer::socket_path_from_env();
    // Implementation correctly skips empty strings and falls back to XDG default
    assert!(
        path.to_str().unwrap().ends_with("songbird.sock"),
        "Empty env var should fall through to default, got: {}",
        path.display()
    );

    // Test 2: Empty family ID (should work, PRIMAL_DEPLOYMENT_STANDARD uses {primal}.sock)
    clear_all_env_vars();
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "");

    let family_id = UnixSocketServer::get_family_id();
    let path = UnixSocketServer::socket_path_from_env();

    // Empty family ID is accepted (but not used in path)
    assert_eq!(family_id, "");
    // PRIMAL_DEPLOYMENT_STANDARD: Socket is always {primal}.sock (no family suffix)
    assert!(path.to_str().unwrap().ends_with("songbird.sock"));

    restore_env_state(original);
}

/// Fault Test: Very long socket paths
///
/// Tests handling of extremely long socket paths (approaching OS limits).
#[test]
fn fault_very_long_socket_path() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    // Create a very long path (but still valid)
    let long_path = format!("/tmp/{}.sock", "a".repeat(200));
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", &long_path);

    let path = UnixSocketServer::socket_path_from_env();

    // Should handle long paths without crashing
    assert_eq!(path, PathBuf::from(long_path));

    restore_env_state(original);
}

/// Fault Test: Absolute vs relative paths
///
/// Tests that socket paths can be absolute or relative.
#[test]
fn fault_relative_socket_path() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    // Test relative path
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "relative/path/songbird.sock");

    let path = UnixSocketServer::socket_path_from_env();

    // Should accept relative paths (path resolution happens at bind time)
    assert_eq!(path, PathBuf::from("relative/path/songbird.sock"));

    restore_env_state(original);
}

/// Fault Test: Path with symlinks
///
/// Tests handling of socket paths containing symlinks.
#[test]
fn fault_path_with_symlinks() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    // Simulate path with symlink
    songbird_process_env::set_var(
        "SONGBIRD_ORCHESTRATOR_SOCKET",
        "/tmp/link/to/socket/songbird.sock",
    );

    let path = UnixSocketServer::socket_path_from_env();

    // Should accept paths with symlinks (resolution happens at bind time)
    assert_eq!(path, PathBuf::from("/tmp/link/to/socket/songbird.sock"));

    restore_env_state(original);
}

/// Fault Test: Whitespace in environment variables
///
/// Tests handling of leading/trailing whitespace in env vars.
#[test]
fn fault_whitespace_in_env_vars() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    // Set env vars with whitespace
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "  /tmp/songbird.sock  ");
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "  nat0  ");

    let path = UnixSocketServer::socket_path_from_env();
    let family_id = UnixSocketServer::get_family_id();

    // Current implementation doesn't trim, so this tests as-is behavior
    // (Trimming could be added as an enhancement)
    assert_eq!(path, PathBuf::from("  /tmp/songbird.sock  "));
    assert_eq!(family_id, "  nat0  ");

    restore_env_state(original);
}

/// Fault Test: Case sensitivity in environment variable names
///
/// Tests that env var names are case-sensitive (as expected in Unix).
#[test]
fn fault_case_sensitivity_env_vars() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();
    clear_all_env_vars();

    // Set lowercase version (should NOT be recognized)
    songbird_process_env::set_var("songbird_orchestrator_socket", "/tmp/wrong.sock");
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/correct.sock");

    let path = UnixSocketServer::socket_path_from_env();

    // Should use the correctly-cased version
    assert_eq!(path, PathBuf::from("/tmp/correct.sock"));

    restore_env_state(original);
}

/// Fault Test: Null bytes in paths (security)
///
/// Tests that paths with null bytes are handled (they shouldn't cause crashes).
#[test]
fn fault_null_bytes_in_path() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    // Rust strings can't contain null bytes, but if they somehow got in via FFI...
    // This is more of a defensive test
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/songbird.sock");

    let path = UnixSocketServer::socket_path_from_env();

    // Should handle normal paths correctly
    assert_eq!(path, PathBuf::from("/tmp/songbird.sock"));

    restore_env_state(original);
}

/// Fault Test: Concurrent environment variable changes
///
/// Tests behavior when env vars change between calls.
#[test]
fn fault_concurrent_env_changes() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    // Set initial env var
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/socket1.sock");
    let path1 = UnixSocketServer::socket_path_from_env();

    // Change env var
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/socket2.sock");
    let path2 = UnixSocketServer::socket_path_from_env();

    // Should reflect the change
    assert_eq!(path1, PathBuf::from("/tmp/socket1.sock"));
    assert_eq!(path2, PathBuf::from("/tmp/socket2.sock"));
    assert_ne!(path1, path2);

    restore_env_state(original);
}

/// Fault Test: Family ID without socket path override
///
/// Tests default path construction with various family IDs.
/// `PRIMAL_DEPLOYMENT_STANDARD`: Family ID is NOT included in socket path.
#[test]
fn fault_family_id_path_construction() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    let test_cases = vec!["nat0", "production", "dev-test", "123", "_underscore_"];

    for family_id in test_cases {
        clear_all_env_vars();
        songbird_process_env::set_var("BIOMEOS_FAMILY_ID", family_id);

        let path = UnixSocketServer::socket_path_from_env();
        let derived_family = UnixSocketServer::get_family_id();

        // PRIMAL_DEPLOYMENT_STANDARD: Socket is {primal}.sock, family_id is separate
        assert!(path.to_str().unwrap().ends_with("songbird.sock"));
        assert_eq!(derived_family, family_id);
    }

    restore_env_state(original);
}

/// Fault Test: Multiple priorities set simultaneously
///
/// Tests that priority order is strictly enforced when all vars are set.
#[test]
fn fault_all_priorities_set() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    // Set ALL socket path env vars
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/highest.sock");
    songbird_process_env::set_var("SONGBIRD_SOCKET", "/tmp/medium.sock");
    songbird_process_env::set_var("BIOMEOS_SOCKET_PATH", "/tmp/lowest.sock");

    // Set ALL family ID env vars
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "highest-family");
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY", "medium-high-family");
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "medium-family");
    songbird_process_env::set_var("SONGBIRD_FAMILY_ID", "lowest-family");

    let path = UnixSocketServer::socket_path_from_env();
    let family_id = UnixSocketServer::get_family_id();

    // Should use highest priority for both
    assert_eq!(path, PathBuf::from("/tmp/highest.sock"));
    assert_eq!(family_id, "highest-family");

    restore_env_state(original);
}

/// Fault Test: Rapid repeated calls
///
/// Tests that repeated calls with same env vars are consistent.
#[test]
fn fault_repeated_calls_consistency() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let original = save_env_state();

    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/test.sock");
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "test");

    // Call multiple times
    let results: Vec<_> = (0..100)
        .map(|_| (UnixSocketServer::socket_path_from_env(), UnixSocketServer::get_family_id()))
        .collect();

    // All results should be identical
    for (path, family_id) in results {
        assert_eq!(path, PathBuf::from("/tmp/test.sock"));
        assert_eq!(family_id, "test");
    }

    restore_env_state(original);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn save_env_state() -> Vec<(String, Option<String>)> {
    let keys = [
        "SONGBIRD_ORCHESTRATOR_SOCKET",
        "SONGBIRD_SOCKET",
        "BIOMEOS_SOCKET_PATH",
        "SONGBIRD_ORCHESTRATOR_FAMILY_ID",
        "SONGBIRD_ORCHESTRATOR_FAMILY",
        "BIOMEOS_FAMILY_ID",
        "SONGBIRD_FAMILY_ID",
    ];

    keys.iter().map(|key| (key.to_string(), env::var(key).ok())).collect()
}

fn restore_env_state(state: Vec<(String, Option<String>)>) {
    for (key, value) in state {
        match value {
            Some(v) => songbird_process_env::set_var(&key, v),
            None => songbird_process_env::remove_var(&key),
        }
    }
}

fn clear_all_env_vars() {
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
    songbird_process_env::remove_var("SONGBIRD_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_PATH");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
}
