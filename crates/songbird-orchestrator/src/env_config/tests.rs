// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::collections::HashMap;
use std::env::VarError;
use std::path::PathBuf;
use std::sync::Mutex;

use songbird_process_env;
use songbird_types::defaults::ports::DEFAULT_HTTP_PORT;

use super::*;

/// Injectable env map for [`super::*_with`] tests (no shared process env).
fn env_map(pairs: Vec<(&'static str, &'static str)>) -> impl Fn(&str) -> Result<String, VarError> {
    let map: HashMap<String, String> =
        pairs.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    move |key: &str| map.get(key).cloned().ok_or(VarError::NotPresent)
}

static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

// Note: These tests validate default behavior when env vars are NOT set.
// We avoid set_var/remove_var where possible to prevent concurrent test pollution.
// Functions like primal_name() and family_id() have stable defaults that are
// testable without env manipulation.

#[test]
fn test_primal_name_returns_string() {
    // primal_name() always returns a value (either env or default)
    let name = primal_name();
    assert!(!name.is_empty());
}

#[test]
fn test_family_id_returns_string() {
    // family_id() always returns a value (either env or default "default")
    let fid = family_id();
    assert!(!fid.is_empty());
}

#[test]
fn test_socket_path_returns_valid_path() {
    let path = socket_path();
    let path_str = path.to_string_lossy();
    // Should end with .sock
    assert!(path_str.ends_with(".sock"), "Expected .sock extension, got: {path_str}");
}

#[test]
fn test_socket_name_domain_based() {
    let name = socket_name();
    assert!(name.ends_with(".sock"));
    assert!(name.starts_with("network"), "Expected domain-based name, got: {name}");
}

#[test]
fn test_data_dir_returns_valid_path() {
    let dir = data_dir();
    assert!(!dir.to_string_lossy().is_empty());
}

#[test]
fn test_http_port_returns_valid_port() {
    let port = http_port();
    assert!(port > 0);
}

#[test]
fn test_log_level_returns_string() {
    let level = log_level();
    assert!(!level.is_empty());
}

#[test]
fn test_dark_forest_config() {
    // These functions always return a bool
    let _dark = dark_forest_enabled();
    let _legacy = accept_legacy_birdsong();
    let _dual = dual_broadcast();
}

#[test]
fn http_port_reads_songbird_http_port() {
    let _g = ENV_TEST_LOCK.lock().unwrap();
    songbird_process_env::set_var("SONGBIRD_HTTP_PORT", "9443");
    songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
    assert_eq!(http_port(), 9443);
    songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
}

#[test]
fn http_port_invalid_falls_back_to_default() {
    let _g = ENV_TEST_LOCK.lock().unwrap();
    songbird_process_env::set_var("SONGBIRD_HTTP_PORT", "not-a-number");
    songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
    assert_eq!(http_port(), DEFAULT_HTTP_PORT);
    songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
}

#[test]
fn http_port_parsed_from_bind_addr_when_port_env_unset() {
    let _g = ENV_TEST_LOCK.lock().unwrap();
    songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
    songbird_process_env::set_var("SONGBIRD_HTTP_ADDR", "0.0.0.0:18080");
    assert_eq!(http_port(), 18080);
    songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
}

#[test]
fn http_bind_address_respects_override() {
    let _g = ENV_TEST_LOCK.lock().unwrap();
    songbird_process_env::set_var("SONGBIRD_HTTP_ADDR", "10.0.0.2:9000");
    assert_eq!(http_bind_address(), "10.0.0.2:9000");
    songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
}

#[test]
fn is_production_true_when_songbird_env_set() {
    let _g = ENV_TEST_LOCK.lock().unwrap();
    songbird_process_env::set_var("SONGBIRD_ENV", "production");
    songbird_process_env::remove_var("RUST_ENV");
    assert!(is_production());
    songbird_process_env::remove_var("SONGBIRD_ENV");
}

#[test]
fn is_production_checks_rust_env_when_songbird_unset() {
    let _g = ENV_TEST_LOCK.lock().unwrap();
    songbird_process_env::remove_var("SONGBIRD_ENV");
    songbird_process_env::set_var("RUST_ENV", "production");
    assert!(is_production());
    songbird_process_env::remove_var("RUST_ENV");
}

#[test]
fn primal_name_env_override() {
    let _g = ENV_TEST_LOCK.lock().unwrap();
    songbird_process_env::set_var("PRIMAL_NAME", "custom-primal");
    assert_eq!(primal_name(), "custom-primal");
    songbird_process_env::remove_var("PRIMAL_NAME");
}

#[test]
fn family_id_prefers_songbird_orchestrator_family_id() {
    let _g = ENV_TEST_LOCK.lock().unwrap();
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID");
    songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "orch-family");
    assert_eq!(family_id(), "orch-family");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
}

#[test]
fn socket_name_with_no_family_returns_domain_sock() {
    assert_eq!(socket_name_with(env_map(vec![])), "network.sock");
}

#[test]
fn socket_name_with_family_id_returns_domain_scoped() {
    let n = socket_name_with(env_map(vec![("FAMILY_ID", "fam-a")]));
    assert_eq!(n, "network-fam-a.sock");
}

#[test]
fn socket_name_with_family_respects_priority_chain() {
    let n = socket_name_with(env_map(vec![
        ("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "orch-fam"),
        ("FAMILY_ID", "ignored"),
    ]));
    assert_eq!(n, "network-orch-fam.sock");
}

#[test]
fn legacy_socket_name_with_no_family() {
    assert_eq!(legacy_socket_name_with(env_map(vec![])), "songbird.sock");
}

#[test]
fn legacy_socket_name_with_family_id() {
    let n = legacy_socket_name_with(env_map(vec![("FAMILY_ID", "edge")]));
    assert_eq!(n, "songbird-edge.sock");
}

#[test]
fn btsp_insecure_guard_ok_when_no_conflict() {
    assert!(validate_btsp_insecure_guard_with(env_map(vec![])).is_ok());
    assert!(validate_btsp_insecure_guard_with(env_map(vec![("FAMILY_ID", "fam")])).is_ok());
    assert!(validate_btsp_insecure_guard_with(env_map(vec![("BIOMEOS_INSECURE", "1")])).is_ok());
}

#[test]
fn btsp_insecure_guard_rejects_family_plus_insecure() {
    let result = validate_btsp_insecure_guard_with(env_map(vec![
        ("FAMILY_ID", "production-fam"),
        ("BIOMEOS_INSECURE", "1"),
    ]));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("BTSP_PROTOCOL_STANDARD"), "{msg}");
}

#[test]
fn btsp_insecure_guard_allows_default_family_with_insecure() {
    assert!(
        validate_btsp_insecure_guard_with(env_map(vec![
            ("FAMILY_ID", "default"),
            ("BIOMEOS_INSECURE", "1"),
        ]))
        .is_ok()
    );
}

#[test]
fn family_id_with_priority_chain() {
    assert_eq!(family_id_with(env_map(vec![("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "p1")])), "p1");
    assert_eq!(
        family_id_with(env_map(vec![("SONGBIRD_ORCHESTRATOR_FAMILY", "p2"), ("FAMILY_ID", "x"),])),
        "p2"
    );
    assert_eq!(
        family_id_with(env_map(vec![("BIOMEOS_FAMILY_ID", "p3"), ("FAMILY_ID", "x")])),
        "p3"
    );
    assert_eq!(
        family_id_with(env_map(vec![("SONGBIRD_FAMILY_ID", "p4"), ("FAMILY_ID", "x")])),
        "p4"
    );
    assert_eq!(family_id_with(env_map(vec![("FAMILY_ID", "p5")])), "p5");
    assert_eq!(family_id_with(env_map(vec![])), "default");
}

#[test]
fn data_dir_with_explicit_override() {
    let p = data_dir_with(env_map(vec![("SONGBIRD_DATA_DIR", "/var/sb/data")]));
    assert_eq!(p, PathBuf::from("/var/sb/data"));
}

#[test]
fn data_dir_with_xdg_data_home() {
    let p = data_dir_with(env_map(vec![("XDG_DATA_HOME", "/home/user/.local/share")]));
    assert_eq!(p, PathBuf::from("/home/user/.local/share/songbird"));
}

#[test]
fn data_dir_with_home_fallback() {
    let p = data_dir_with(env_map(vec![("HOME", "/home/user")]));
    assert_eq!(p, PathBuf::from("/home/user/.local/share/songbird"));
}

#[test]
fn data_dir_with_vps_fallback() {
    let p = data_dir_with(env_map(vec![]));
    assert_eq!(p, PathBuf::from("/var/lib/songbird"));
}

#[test]
fn deployment_dir_with_explicit_and_defaults() {
    assert_eq!(
        deployment_dir_with(env_map(vec![("SONGBIRD_DEPLOY_DIR", "/deploy")])),
        PathBuf::from("/deploy")
    );
    assert_eq!(
        deployment_dir_with(env_map(vec![("XDG_DATA_HOME", "/xdg/data")])),
        PathBuf::from("/xdg/data/songbird/deployments")
    );
    assert_eq!(
        deployment_dir_with(env_map(vec![("HOME", "/home/user")])),
        PathBuf::from("/home/user/.local/share/songbird/deployments")
    );
    assert_eq!(
        deployment_dir_with(env_map(vec![])),
        PathBuf::from("/var/lib/songbird/deployments")
    );
}

#[cfg(unix)]
#[test]
fn domain_socket_symlink_create_and_remove() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bound = tmp.path().join("songbird.sock");
    std::fs::write(&bound, "").expect("touch bound path");
    super::create_domain_socket_symlink(&bound);
    let domain = tmp.path().join("network.sock");
    assert!(
        std::fs::symlink_metadata(&domain).expect("symlink metadata").file_type().is_symlink(),
        "domain path should be a symlink"
    );
    assert_eq!(std::fs::read_link(&domain).expect("readlink"), bound);
    super::remove_domain_socket_symlink_if_matches(&bound);
    assert!(!domain.exists(), "domain symlink should be removed");
}

#[test]
fn cache_dir_with_explicit_and_defaults() {
    assert_eq!(
        cache_dir_with(env_map(vec![("SONGBIRD_CACHE_DIR", "/cache")])),
        PathBuf::from("/cache")
    );
    assert_eq!(
        cache_dir_with(env_map(vec![("XDG_CACHE_HOME", "/home/user/.cache")])),
        PathBuf::from("/home/user/.cache/songbird")
    );
    assert_eq!(
        cache_dir_with(env_map(vec![("HOME", "/home/user")])),
        PathBuf::from("/home/user/.cache/songbird")
    );
    assert_eq!(cache_dir_with(env_map(vec![])), PathBuf::from("/var/cache/songbird"));
}
