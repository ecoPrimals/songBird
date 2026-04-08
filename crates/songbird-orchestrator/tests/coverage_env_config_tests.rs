// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for `env_config` module
//!
//! Tests all environment-based configuration with proper mutex serialization
//! to prevent race conditions on process-wide environment variables.

use songbird_orchestrator::env_config;

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

// ═══════════════════════════════════════════════════════════════════════════
// primal_name() tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_primal_name_default() {
    let _g = lock_env();
    songbird_process_env::remove_var("PRIMAL_NAME");
    assert_eq!(env_config::primal_name(), "songbird");
}

#[test]
fn test_primal_name_from_env() {
    let _g = lock_env();
    songbird_process_env::set_var("PRIMAL_NAME", "custom-primal");
    let name = env_config::primal_name();
    songbird_process_env::remove_var("PRIMAL_NAME");
    assert_eq!(name, "custom-primal");
}

// ═══════════════════════════════════════════════════════════════════════════
// family_id() tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_family_id_default() {
    let _g = lock_env();
    for var in &[
        "SONGBIRD_ORCHESTRATOR_FAMILY_ID",
        "SONGBIRD_ORCHESTRATOR_FAMILY",
        "BIOMEOS_FAMILY_ID",
        "SONGBIRD_FAMILY_ID",
        "FAMILY_ID",
    ] {
        songbird_process_env::remove_var(var);
    }
    assert_eq!(env_config::family_id(), "default");
}

#[test]
fn test_family_id_priority_highest() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "highest");
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY", "lower");
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "lowest");
    songbird_process_env::set_var("SONGBIRD_FAMILY_ID", "legacy");
    songbird_process_env::set_var("FAMILY_ID", "generic");
    let id = env_config::family_id();
    for var in &[
        "SONGBIRD_ORCHESTRATOR_FAMILY_ID",
        "SONGBIRD_ORCHESTRATOR_FAMILY",
        "BIOMEOS_FAMILY_ID",
        "SONGBIRD_FAMILY_ID",
        "FAMILY_ID",
    ] {
        songbird_process_env::remove_var(var);
    }
    assert_eq!(id, "highest");
}

#[test]
fn test_family_id_priority_second() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY", "second");
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "lower");
    let id = env_config::family_id();
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    assert_eq!(id, "second");
}

#[test]
fn test_family_id_priority_biomeos() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::set_var("BIOMEOS_FAMILY_ID", "biome-fam");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID");
    let id = env_config::family_id();
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    assert_eq!(id, "biome-fam");
}

#[test]
fn test_family_id_priority_legacy() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    songbird_process_env::set_var("SONGBIRD_FAMILY_ID", "legacy-id");
    songbird_process_env::remove_var("FAMILY_ID");
    let id = env_config::family_id();
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    assert_eq!(id, "legacy-id");
}

#[test]
fn test_family_id_priority_generic() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::set_var("FAMILY_ID", "generic-id");
    let id = env_config::family_id();
    songbird_process_env::remove_var("FAMILY_ID");
    assert_eq!(id, "generic-id");
}

// ═══════════════════════════════════════════════════════════════════════════
// node_id() tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_node_id_default() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_NODE_ID");
    songbird_process_env::remove_var("NODE_ID");
    assert_eq!(env_config::node_id(), "default");
}

#[test]
fn test_node_id_primary() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_NODE_ID", "node-42");
    songbird_process_env::set_var("NODE_ID", "generic-node");
    let id = env_config::node_id();
    songbird_process_env::remove_var("SONGBIRD_NODE_ID");
    songbird_process_env::remove_var("NODE_ID");
    assert_eq!(id, "node-42");
}

#[test]
fn test_node_id_fallback() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_NODE_ID");
    songbird_process_env::set_var("NODE_ID", "generic-node");
    let id = env_config::node_id();
    songbird_process_env::remove_var("NODE_ID");
    assert_eq!(id, "generic-node");
}

// ═══════════════════════════════════════════════════════════════════════════
// socket_path() tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_socket_path_explicit_override() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_SOCKET", "/custom/path/songbird.sock");
    let path = env_config::socket_path();
    songbird_process_env::remove_var("SONGBIRD_SOCKET");
    assert_eq!(path.to_string_lossy(), "/custom/path/songbird.sock");
}

#[test]
fn test_socket_path_biomeos_socket_dir() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_SOCKET");
    songbird_process_env::remove_var("SONGBIRD_MULTI_FAMILY");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_SOCKET");
    songbird_process_env::set_var("BIOMEOS_SOCKET_DIR", "/tmp/test-biomeos-sockets");
    let path = env_config::socket_path();
    songbird_process_env::remove_var("BIOMEOS_SOCKET_DIR");
    assert!(path.to_string_lossy().contains("test-biomeos-sockets"));
    assert!(path.to_string_lossy().ends_with("network.sock"));
}

#[test]
fn test_socket_path_xdg_runtime_dir() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_DIR");
    songbird_process_env::remove_var("SONGBIRD_MULTI_FAMILY");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_SOCKET");
    songbird_process_env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
    let path = env_config::socket_path();
    songbird_process_env::remove_var("XDG_RUNTIME_DIR");
    let path_str = path.to_string_lossy();
    // Either uses XDG path or falls back to /tmp (depends on dir creation)
    assert!(path_str.ends_with("network.sock"));
}

#[test]
fn test_socket_path_default_ends_with_sock() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_SOCKET");
    songbird_process_env::remove_var("BIOMEOS_SOCKET_DIR");
    let path = env_config::socket_path();
    assert!(path.to_string_lossy().ends_with(".sock"));
}

// ═══════════════════════════════════════════════════════════════════════════
// socket_name() tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_socket_name_domain_default() {
    let _g = lock_env();
    for var in &[
        "SONGBIRD_ORCHESTRATOR_FAMILY_ID",
        "SONGBIRD_ORCHESTRATOR_FAMILY",
        "BIOMEOS_FAMILY_ID",
        "SONGBIRD_FAMILY_ID",
        "FAMILY_ID",
    ] {
        songbird_process_env::remove_var(var);
    }
    assert_eq!(env_config::socket_name(), "network.sock");
}

#[test]
fn test_socket_name_domain_with_family_id() {
    let _g = lock_env();
    songbird_process_env::set_var("FAMILY_ID", "alpha-bravo");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    let name = env_config::socket_name();
    songbird_process_env::remove_var("FAMILY_ID");
    assert_eq!(name, "network-alpha-bravo.sock");
}

#[test]
fn test_socket_name_no_family_returns_domain() {
    let _g = lock_env();
    for var in &[
        "SONGBIRD_ORCHESTRATOR_FAMILY_ID",
        "SONGBIRD_ORCHESTRATOR_FAMILY",
        "BIOMEOS_FAMILY_ID",
        "SONGBIRD_FAMILY_ID",
        "FAMILY_ID",
    ] {
        songbird_process_env::remove_var(var);
    }
    let name = env_config::socket_name();
    assert_eq!(name, "network.sock");
}

// ═══════════════════════════════════════════════════════════════════════════
// data_dir(), deployment_dir(), cache_dir() tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_data_dir_default() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_DATA_DIR");
    songbird_process_env::remove_var("XDG_RUNTIME_DIR");
    songbird_process_env::remove_var("TMPDIR");
    assert_eq!(env_config::data_dir().to_string_lossy(), "/tmp/songbird-data");
}

#[test]
fn test_data_dir_from_env() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_DATA_DIR", "/custom/data");
    let dir = env_config::data_dir();
    songbird_process_env::remove_var("SONGBIRD_DATA_DIR");
    assert_eq!(dir.to_string_lossy(), "/custom/data");
}

#[test]
fn test_deployment_dir_default() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_DEPLOY_DIR");
    songbird_process_env::remove_var("XDG_RUNTIME_DIR");
    songbird_process_env::remove_var("TMPDIR");
    assert_eq!(env_config::deployment_dir().to_string_lossy(), "/tmp/songbird-deployments");
}

#[test]
fn test_deployment_dir_from_env() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_DEPLOY_DIR", "/opt/deployments");
    let dir = env_config::deployment_dir();
    songbird_process_env::remove_var("SONGBIRD_DEPLOY_DIR");
    assert_eq!(dir.to_string_lossy(), "/opt/deployments");
}

#[test]
fn test_cache_dir_default() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_CACHE_DIR");
    songbird_process_env::remove_var("XDG_RUNTIME_DIR");
    songbird_process_env::remove_var("TMPDIR");
    assert_eq!(env_config::cache_dir().to_string_lossy(), "/tmp/songbird-cache");
}

#[test]
fn test_cache_dir_from_env() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_CACHE_DIR", "/var/cache/songbird");
    let dir = env_config::cache_dir();
    songbird_process_env::remove_var("SONGBIRD_CACHE_DIR");
    assert_eq!(dir.to_string_lossy(), "/var/cache/songbird");
}

// ═══════════════════════════════════════════════════════════════════════════
// http_bind_address() & http_port() tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_http_bind_address_default() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
    assert_eq!(env_config::http_bind_address(), "0.0.0.0:8080");
}

#[test]
fn test_http_bind_address_from_env() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_HTTP_ADDR", "127.0.0.1:9090");
    let addr = env_config::http_bind_address();
    songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
    assert_eq!(addr, "127.0.0.1:9090");
}

#[test]
fn test_http_port_default() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
    songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
    assert_eq!(env_config::http_port(), 8080);
}

#[test]
fn test_http_port_from_explicit_env() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_HTTP_PORT", "9999");
    let port = env_config::http_port();
    songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
    assert_eq!(port, 9999);
}

#[test]
fn test_http_port_extracted_from_addr() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
    songbird_process_env::set_var("SONGBIRD_HTTP_ADDR", "0.0.0.0:3000");
    let port = env_config::http_port();
    songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
    assert_eq!(port, 3000);
}

#[test]
fn test_http_port_invalid_env_falls_back() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_HTTP_PORT", "not-a-number");
    let port = env_config::http_port();
    songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
    assert_eq!(port, 8080);
}

// ═══════════════════════════════════════════════════════════════════════════
// is_production() tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_is_production_default_false() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_ENV");
    songbird_process_env::remove_var("RUST_ENV");
    assert!(!env_config::is_production());
}

#[test]
fn test_is_production_songbird_env() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_ENV", "production");
    let prod = env_config::is_production();
    songbird_process_env::remove_var("SONGBIRD_ENV");
    assert!(prod);
}

#[test]
fn test_is_production_rust_env_fallback() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_ENV");
    songbird_process_env::set_var("RUST_ENV", "production");
    let prod = env_config::is_production();
    songbird_process_env::remove_var("RUST_ENV");
    assert!(prod);
}

#[test]
fn test_is_production_non_production_value() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_ENV", "development");
    let prod = env_config::is_production();
    songbird_process_env::remove_var("SONGBIRD_ENV");
    assert!(!prod);
}

// ═══════════════════════════════════════════════════════════════════════════
// log_level() tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_log_level_default() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_LOG");
    songbird_process_env::remove_var("RUST_LOG");
    assert_eq!(env_config::log_level(), "info");
}

#[test]
fn test_log_level_songbird_log() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_LOG", "debug");
    let level = env_config::log_level();
    songbird_process_env::remove_var("SONGBIRD_LOG");
    assert_eq!(level, "debug");
}

#[test]
fn test_log_level_rust_log_fallback() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_LOG");
    songbird_process_env::set_var("RUST_LOG", "trace");
    let level = env_config::log_level();
    songbird_process_env::remove_var("RUST_LOG");
    assert_eq!(level, "trace");
}

// ═══════════════════════════════════════════════════════════════════════════
// Dark Forest config tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_dark_forest_enabled_default_false() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_DARK_FOREST");
    assert!(!env_config::dark_forest_enabled());
}

#[test]
fn test_dark_forest_enabled_true() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_DARK_FOREST", "true");
    let enabled = env_config::dark_forest_enabled();
    songbird_process_env::remove_var("SONGBIRD_DARK_FOREST");
    assert!(enabled);
}

#[test]
fn test_dark_forest_enabled_invalid_value() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_DARK_FOREST", "not-a-bool");
    let enabled = env_config::dark_forest_enabled();
    songbird_process_env::remove_var("SONGBIRD_DARK_FOREST");
    assert!(!enabled);
}

#[test]
fn test_accept_legacy_birdsong_default_true() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_ACCEPT_LEGACY_BIRDSONG");
    assert!(env_config::accept_legacy_birdsong());
}

#[test]
fn test_accept_legacy_birdsong_disabled() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_ACCEPT_LEGACY_BIRDSONG", "false");
    let accept = env_config::accept_legacy_birdsong();
    songbird_process_env::remove_var("SONGBIRD_ACCEPT_LEGACY_BIRDSONG");
    assert!(!accept);
}

#[test]
fn test_dual_broadcast_default_false() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_DUAL_BROADCAST");
    assert!(!env_config::dual_broadcast());
}

#[test]
fn test_dual_broadcast_enabled() {
    let _g = lock_env();
    songbird_process_env::set_var("SONGBIRD_DUAL_BROADCAST", "true");
    let dual = env_config::dual_broadcast();
    songbird_process_env::remove_var("SONGBIRD_DUAL_BROADCAST");
    assert!(dual);
}
