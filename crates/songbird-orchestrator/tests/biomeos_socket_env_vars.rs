// BiomeOS Neural API Socket Environment Variable Compatibility Test
// January 15, 2026
//
// Validates that Songbird honors BiomeOS Neural API environment variable standards
// as documented in the upstream handoff from BiomeOS team.

use songbird_orchestrator::ipc::UnixSocketServer;
use std::env;
use std::path::PathBuf;

/// Test socket path derivation with BiomeOS Neural API environment variables
#[test]
fn test_biomeos_neural_api_socket_path_priority() {
    // Save original env state
    let original_orchestrator_socket = env::var("SONGBIRD_ORCHESTRATOR_SOCKET").ok();
    let original_socket = env::var("SONGBIRD_SOCKET").ok();
    let original_biomeos_path = env::var("BIOMEOS_SOCKET_PATH").ok();
    let original_family_id = env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID").ok();
    let original_biomeos_family = env::var("BIOMEOS_FAMILY_ID").ok();

    // Clear all env vars to start clean
    env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
    env::remove_var("SONGBIRD_SOCKET");
    env::remove_var("BIOMEOS_SOCKET_PATH");
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    env::remove_var("BIOMEOS_FAMILY_ID");
    env::remove_var("SONGBIRD_FAMILY_ID");

    // Test 1: SONGBIRD_ORCHESTRATOR_SOCKET (highest priority - Neural API standard)
    env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/songbird-nat0.sock");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-nat0.sock"),
        "SONGBIRD_ORCHESTRATOR_SOCKET should be highest priority"
    );
    env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");

    // Test 2: SONGBIRD_SOCKET (alternative naming)
    env::set_var("SONGBIRD_SOCKET", "/tmp/songbird-custom.sock");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-custom.sock"),
        "SONGBIRD_SOCKET should be second priority"
    );
    env::remove_var("SONGBIRD_SOCKET");

    // Test 3: BIOMEOS_SOCKET_PATH (generic orchestrator)
    env::set_var("BIOMEOS_SOCKET_PATH", "/tmp/songbird-biomeos.sock");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-biomeos.sock"),
        "BIOMEOS_SOCKET_PATH should be third priority"
    );
    env::remove_var("BIOMEOS_SOCKET_PATH");

    // Test 4: Family ID from SONGBIRD_ORCHESTRATOR_FAMILY_ID (default path construction)
    env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "nat0");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-nat0.sock"),
        "Should construct path from SONGBIRD_ORCHESTRATOR_FAMILY_ID"
    );
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");

    // Test 5: Family ID from BIOMEOS_FAMILY_ID (generic orchestrator)
    env::set_var("BIOMEOS_FAMILY_ID", "nat0");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-nat0.sock"),
        "Should construct path from BIOMEOS_FAMILY_ID"
    );
    env::remove_var("BIOMEOS_FAMILY_ID");

    // Test 6: Default behavior (no env vars) - ensure all vars are cleared
    env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
    env::remove_var("SONGBIRD_SOCKET");
    env::remove_var("BIOMEOS_SOCKET_PATH");
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    env::remove_var("BIOMEOS_FAMILY_ID");
    env::remove_var("SONGBIRD_FAMILY_ID");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-default.sock"),
        "Should use default family 'default' when no env vars set"
    );

    // Test 7: Neural API standard deployment (full environment)
    env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/songbird-nat0.sock");
    env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "nat0");
    env::set_var("BIOMEOS_FAMILY_ID", "nat0");
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

/// Test that socket path defaults to /tmp/ (NOT /run/user/{uid}/)
/// This is critical for BiomeOS Neural API compatibility
#[test]
fn test_default_socket_directory_is_tmp() {
    // Clear all env vars
    env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
    env::remove_var("SONGBIRD_SOCKET");
    env::remove_var("BIOMEOS_SOCKET_PATH");
    env::remove_var("SONGBIRD_FAMILY_ID");

    let path = UnixSocketServer::socket_path_from_env();
    
    // Should default to /tmp/songbird-default.sock
    // NOT /run/user/1000/songbird-default.sock
    assert!(
        path.starts_with("/tmp/"),
        "Default socket directory must be /tmp/ for BiomeOS compatibility, got: {}",
        path.display()
    );
}

fn restore_env_var(key: &str, value: Option<String>) {
    match value {
        Some(v) => env::set_var(key, v),
        None => env::remove_var(key),
    }
}

#[test]
fn test_family_id_priority_order() {
    // Clear all env vars
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    env::remove_var("BIOMEOS_FAMILY_ID");
    env::remove_var("SONGBIRD_FAMILY_ID");

    // Test 1: SONGBIRD_ORCHESTRATOR_FAMILY_ID (highest priority)
    env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "nat0");
    env::set_var("BIOMEOS_FAMILY_ID", "wrong");
    env::set_var("SONGBIRD_FAMILY_ID", "also-wrong");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-nat0.sock"),
        "SONGBIRD_ORCHESTRATOR_FAMILY_ID should be highest priority for family ID"
    );

    // Test 2: BIOMEOS_FAMILY_ID (when SONGBIRD_ORCHESTRATOR_FAMILY_ID not set)
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    env::set_var("BIOMEOS_FAMILY_ID", "nat1");
    env::set_var("SONGBIRD_FAMILY_ID", "wrong");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-nat1.sock"),
        "BIOMEOS_FAMILY_ID should be used when SONGBIRD_ORCHESTRATOR_FAMILY_ID not set"
    );

    // Test 3: SONGBIRD_FAMILY_ID (legacy fallback)
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    env::remove_var("BIOMEOS_FAMILY_ID");
    env::set_var("SONGBIRD_FAMILY_ID", "legacy");
    let path = UnixSocketServer::socket_path_from_env();
    assert_eq!(
        path,
        PathBuf::from("/tmp/songbird-legacy.sock"),
        "SONGBIRD_FAMILY_ID should be used as legacy fallback"
    );

    // Cleanup
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    env::remove_var("BIOMEOS_FAMILY_ID");
    env::remove_var("SONGBIRD_FAMILY_ID");
}

