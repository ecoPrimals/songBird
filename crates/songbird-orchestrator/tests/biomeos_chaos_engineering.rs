// BiomeOS Socket Chaos Engineering Tests
// January 16, 2026
// Updated: February 5, 2026 (PRIMAL_DEPLOYMENT_STANDARD compliance)
// Updated: February 10, 2026 (concurrent-safe with per-file Mutex)
//
// Chaos tests for BiomeOS socket integration.
// Tests system behavior under random failures, disruptions, and edge cases.
//
// PRIMAL_DEPLOYMENT_STANDARD changes (Feb 5, 2026):
// - Socket names are now {primal}.sock (no family suffix)
// - XDG_RUNTIME_DIR/biomeos/ is the preferred location
// - Family ID is NOT included in socket path
//
// **Concurrency Evolution**: These chaos tests mutate process-wide env vars.
// A static Mutex ensures they don't race with each other within this binary.

use rand::Rng;
use songbird_orchestrator::ipc::UnixSocketServer;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

/// Serialize all env var chaos tests in this file.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Chaos Test: Random environment variable mutations
///
/// Randomly mutates environment variables and ensures no panics.
#[test]
fn chaos_random_env_mutations() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();
    let mut rng = rand::thread_rng();

    let env_vars = vec![
        "SONGBIRD_ORCHESTRATOR_SOCKET",
        "SONGBIRD_SOCKET",
        "BIOMEOS_SOCKET_PATH",
        "SONGBIRD_ORCHESTRATOR_FAMILY_ID",
        "BIOMEOS_FAMILY_ID",
    ];

    let long_string = "x".repeat(500);
    let random_values = vec![
        "/tmp/socket.sock",
        "",
        "/non/existent/path.sock",
        "test-family",
        "../../../etc/passwd",
        long_string.as_str(),
        "nat0",
    ];

    // Perform 100 random mutations
    for _ in 0..100 {
        // Randomly set or unset env vars
        for env_var in &env_vars {
            if rng.gen_bool(0.5) {
                let value = random_values[rng.gen_range(0..random_values.len())];
                env::set_var(env_var, value);
            } else {
                env::remove_var(env_var);
            }
        }

        // Should not panic regardless of env state
        let _path = UnixSocketServer::socket_path_from_env();
        let _family = UnixSocketServer::get_family_id();
    }

    restore_env_state(original);
}

/// Chaos Test: Rapid environment changes
///
/// Rapidly changes environment variables to simulate racing conditions.
#[test]
fn chaos_rapid_env_changes() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();

    let socket_paths =
        vec!["/tmp/socket1.sock", "/tmp/socket2.sock", "/tmp/socket3.sock", "/tmp/socket4.sock"];

    let family_ids = vec!["nat0", "nat1", "nat2", "nat3"];

    // Rapidly change env vars and derive paths
    for i in 0..100 {
        env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", socket_paths[i % socket_paths.len()]);
        env::set_var("BIOMEOS_FAMILY_ID", family_ids[i % family_ids.len()]);

        let path = UnixSocketServer::socket_path_from_env();
        let family = UnixSocketServer::get_family_id();

        // Should always return a valid path and family ID
        assert!(!path.as_os_str().is_empty());
        assert!(!family.is_empty());
    }

    restore_env_state(original);
}

/// Chaos Test: Random priority conflicts
///
/// Sets random combinations of env vars to test priority resolution.
#[test]
fn chaos_random_priority_conflicts() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();
    let mut rng = rand::thread_rng();

    let socket_vars =
        vec!["SONGBIRD_ORCHESTRATOR_SOCKET", "SONGBIRD_SOCKET", "BIOMEOS_SOCKET_PATH"];

    let family_vars = vec![
        "SONGBIRD_ORCHESTRATOR_FAMILY_ID",
        "SONGBIRD_ORCHESTRATOR_FAMILY",
        "BIOMEOS_FAMILY_ID",
        "SONGBIRD_FAMILY_ID",
    ];

    // Test 50 random combinations
    for _ in 0..50 {
        clear_all_env_vars();

        // Randomly set socket vars
        for var in &socket_vars {
            if rng.gen_bool(0.6) {
                env::set_var(var, format!("/tmp/test-{}.sock", var));
            }
        }

        // Randomly set family vars
        for var in &family_vars {
            if rng.gen_bool(0.6) {
                env::set_var(var, format!("family-{}", var));
            }
        }

        // Should always return valid results without panic
        let path = UnixSocketServer::socket_path_from_env();
        let family = UnixSocketServer::get_family_id();

        assert!(!path.as_os_str().is_empty());
        assert!(!family.is_empty());

        // Verify priority order is respected
        if let Ok(expected) = env::var("SONGBIRD_ORCHESTRATOR_SOCKET") {
            // Highest priority socket env var should be used directly
            assert_eq!(
                path,
                PathBuf::from(&expected),
                "SONGBIRD_ORCHESTRATOR_SOCKET should be used when set"
            );
        }

        if let Ok(expected) = env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID") {
            assert_eq!(
                family, expected,
                "SONGBIRD_ORCHESTRATOR_FAMILY_ID should have highest priority"
            );
        }
    }

    restore_env_state(original);
}

/// Chaos Test: Stress test with many rapid calls
///
/// Makes thousands of calls to test performance and memory leaks.
#[test]
fn chaos_stress_many_calls() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();

    env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/stress-test.sock");
    env::set_var("BIOMEOS_FAMILY_ID", "stress");

    // Make 10,000 calls
    for _ in 0..10_000 {
        let _path = UnixSocketServer::socket_path_from_env();
        let _family = UnixSocketServer::get_family_id();
    }

    // If we get here without OOM or panic, test passes
    restore_env_state(original);
}

/// Chaos Test: Random special characters in paths
///
/// Tests handling of various special characters in socket paths.
#[test]
fn chaos_special_characters() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();

    let special_chars = vec![
        "/tmp/socket-with-dash.sock",
        "/tmp/socket_with_underscore.sock",
        "/tmp/socket.with.dots.sock",
        "/tmp/socket:with:colons.sock",
        "/tmp/socket@with@at.sock",
        "/tmp/socket#with#hash.sock",
        "/tmp/socket$with$dollar.sock",
        "/tmp/socket%with%percent.sock",
        "/tmp/socket&with&ampersand.sock",
        "/tmp/socket~with~tilde.sock",
    ];

    for path in special_chars {
        env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", path);
        let derived = UnixSocketServer::socket_path_from_env();

        // Should accept various special characters
        assert_eq!(derived, PathBuf::from(path));
    }

    restore_env_state(original);
}

/// Chaos Test: Random family ID formats
///
/// Tests handling of various family ID formats.
#[test]
fn chaos_family_id_formats() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();

    let family_ids = vec![
        "nat0",
        "NAT0",
        "nat-0",
        "nat_0",
        "nat.0",
        "nat:0",
        "0",
        "1234567890",
        "very-long-family-id-with-many-parts-separated-by-dashes",
        "MixedCase123",
        "with spaces",
        "with\ttabs",
        "with\nnewlines",
    ];

    for family_id in family_ids {
        clear_all_env_vars();
        env::set_var("BIOMEOS_FAMILY_ID", family_id);

        let derived_family = UnixSocketServer::get_family_id();
        let derived_path = UnixSocketServer::socket_path_from_env();

        // Should accept various formats for family ID
        assert_eq!(derived_family, family_id);
        // PRIMAL_DEPLOYMENT_STANDARD: Socket path is {primal}.sock (no family suffix)
        // Family ID is NOT included in socket path per new standard
        assert!(derived_path.to_str().unwrap().ends_with("songbird.sock"));
    }

    restore_env_state(original);
}

/// Chaos Test: Alternating clear and set operations
///
/// Rapidly alternates between clearing and setting env vars.
#[test]
fn chaos_alternating_clear_set() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();

    for i in 0..100 {
        if i % 2 == 0 {
            // Set env vars
            env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/test.sock");
            env::set_var("BIOMEOS_FAMILY_ID", "test");

            let path = UnixSocketServer::socket_path_from_env();
            let family = UnixSocketServer::get_family_id();

            assert_eq!(path, PathBuf::from("/tmp/test.sock"));
            assert_eq!(family, "test");
        } else {
            // Clear env vars
            clear_all_env_vars();

            let path = UnixSocketServer::socket_path_from_env();
            let family = UnixSocketServer::get_family_id();

            // PRIMAL_DEPLOYMENT_STANDARD: Uses XDG or /tmp fallback, socket is songbird.sock
            assert!(path.to_str().unwrap().ends_with("songbird.sock"));
            assert_eq!(family, "default");
        }
    }

    restore_env_state(original);
}

/// Chaos Test: Random path lengths
///
/// Tests handling of paths with various lengths.
#[test]
fn chaos_random_path_lengths() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();
    let mut rng = rand::thread_rng();

    for _ in 0..50 {
        let length = rng.gen_range(1..300);
        let path = format!("/tmp/{}.sock", "x".repeat(length));

        env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", &path);
        let derived = UnixSocketServer::socket_path_from_env();

        assert_eq!(derived, PathBuf::from(path));
    }

    restore_env_state(original);
}

/// Chaos Test: Mixed valid and invalid states
///
/// Randomly mixes valid and invalid environment states.
#[test]
fn chaos_mixed_valid_invalid() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();
    let mut rng = rand::thread_rng();

    let valid_paths = vec!["/tmp/valid1.sock", "/tmp/valid2.sock", "/var/run/valid3.sock"];

    let questionable_paths = vec!["/non/existent/path.sock", "../relative/path.sock", "", "/tmp/"];

    for _ in 0..50 {
        clear_all_env_vars();

        // Randomly choose valid or questionable
        let paths = if rng.gen_bool(0.7) {
            &valid_paths
        } else {
            &questionable_paths
        };
        let path = paths[rng.gen_range(0..paths.len())];

        if !path.is_empty() {
            env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", path);
        }

        // Should not panic
        let _derived = UnixSocketServer::socket_path_from_env();
        let _family = UnixSocketServer::get_family_id();
    }

    restore_env_state(original);
}

/// Chaos Test: Simulated environment pollution
///
/// Sets many unrelated env vars to simulate polluted environment.
#[test]
fn chaos_environment_pollution() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();

    // Pollute environment with unrelated vars
    for i in 0..50 {
        env::set_var(format!("RANDOM_VAR_{}", i), format!("value_{}", i));
        env::set_var(format!("SONGBIRD_UNRELATED_{}", i), format!("value_{}", i));
    }

    // Set correct vars
    env::set_var("SONGBIRD_ORCHESTRATOR_SOCKET", "/tmp/correct.sock");
    env::set_var("BIOMEOS_FAMILY_ID", "correct");

    // Should find correct vars despite pollution
    let path = UnixSocketServer::socket_path_from_env();
    let family = UnixSocketServer::get_family_id();

    assert_eq!(path, PathBuf::from("/tmp/correct.sock"));
    assert_eq!(family, "correct");

    // Cleanup pollution
    for i in 0..50 {
        env::remove_var(format!("RANDOM_VAR_{}", i));
        env::remove_var(format!("SONGBIRD_UNRELATED_{}", i));
    }

    restore_env_state(original);
}

/// Chaos Test: Unicode and international characters
///
/// Tests handling of Unicode and international characters.
#[test]
fn chaos_unicode_international() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let original = save_env_state();

    let test_cases = vec![
        ("家族", "family in Japanese"),
        ("семья", "family in Russian"),
        ("famille", "family in French"),
        ("οικογένεια", "family in Greek"),
        ("משפחה", "family in Hebrew"),
        ("家庭", "family in Chinese"),
        ("가족", "family in Korean"),
        ("परिवार", "family in Hindi"),
        ("🦜-nat0", "emoji prefix"),
        ("nat0-🐦", "emoji suffix"),
    ];

    for (family_id, _description) in test_cases {
        clear_all_env_vars();
        env::set_var("BIOMEOS_FAMILY_ID", family_id);

        // Should handle Unicode without panic
        let derived_family = UnixSocketServer::get_family_id();
        let derived_path = UnixSocketServer::socket_path_from_env();

        assert_eq!(derived_family, family_id);
        assert!(derived_path.to_str().is_some());
    }

    restore_env_state(original);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn save_env_state() -> Vec<(String, Option<String>)> {
    let keys = vec![
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
            Some(v) => env::set_var(&key, v),
            None => env::remove_var(&key),
        }
    }
}

fn clear_all_env_vars() {
    env::remove_var("SONGBIRD_ORCHESTRATOR_SOCKET");
    env::remove_var("SONGBIRD_SOCKET");
    env::remove_var("BIOMEOS_SOCKET_PATH");
    env::remove_var("BIOMEOS_SOCKET_DIR");
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    env::remove_var("BIOMEOS_FAMILY_ID");
    env::remove_var("SONGBIRD_FAMILY_ID");
}
