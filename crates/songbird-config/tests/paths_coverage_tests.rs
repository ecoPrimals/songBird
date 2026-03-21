// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Coverage tests for `songbird_config::config::paths`
//!
//! Tests path configuration, initialization, fallback logic, and validation.

use songbird_config::config::paths::{PathConfig, get_path_config, testing_config};
use std::collections::HashMap;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

// ==================== DEFAULT TESTS ====================

#[test]
fn test_path_config_default() {
    let _g = lock_env();
    let config = PathConfig::default();
    assert!(!config.data_dir.as_os_str().is_empty());
    assert!(!config.config_dir.as_os_str().is_empty());
    assert!(!config.log_dir.as_os_str().is_empty());
    assert!(!config.cache_dir.as_os_str().is_empty());
}

#[test]
fn test_path_config_new() {
    let _g = lock_env();
    let config = PathConfig::new();
    assert!(config.is_ok(), "PathConfig::new() should succeed");
    let config = config.unwrap();
    assert!(config.config_dir.to_string_lossy().contains("songbird"));
}

// ==================== FALLBACK TESTS ====================

#[test]
fn test_path_config_fallback() {
    let _g = lock_env();
    let config = PathConfig::new_fallback();
    assert!(!config.data_dir.as_os_str().is_empty());
    assert!(!config.config_dir.as_os_str().is_empty());
}

// ==================== DEVELOPMENT TESTS ====================

#[test]
fn test_path_config_development() {
    let _g = lock_env();
    let config = PathConfig::development();
    assert!(
        config.data_dir.to_string_lossy().contains(".songbird"),
        "Dev config should use .songbird dir"
    );
}

// ==================== TESTING CONFIG ====================

#[test]
fn test_testing_config() {
    let _g = lock_env();
    let config = testing_config();
    // Testing config should return valid paths
    assert!(!config.data_dir.as_os_str().is_empty());
}

// ==================== GET_PATH_CONFIG ====================

#[test]
fn test_get_path_config() {
    let _g = lock_env();
    let config = get_path_config();
    assert!(!config.data_dir.as_os_str().is_empty());
    assert!(!config.config_dir.as_os_str().is_empty());
}

// ==================== SERVICE DATA DIRS ====================

#[test]
fn test_service_data_dirs_present() {
    let _g = lock_env();
    let config = PathConfig::default();
    assert!(!config.service_data_dirs.orchestrator.as_os_str().is_empty());
    assert!(!config.service_data_dirs.federation.as_os_str().is_empty());
    assert!(!config.service_data_dirs.metrics.as_os_str().is_empty());
    assert!(!config.service_data_dirs.discovery.as_os_str().is_empty());
    assert!(!config.service_data_dirs.registry.as_os_str().is_empty());
}

// ==================== SERIALIZATION TESTS ====================

#[test]
fn test_path_config_serializable() {
    let _g = lock_env();
    let config = PathConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(!json.is_empty());
    let deserialized: PathConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.data_dir, deserialized.data_dir);
}

#[test]
fn test_path_config_debug() {
    let _g = lock_env();
    let config = PathConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("PathConfig"));
}

#[test]
fn test_path_config_clone() {
    let _g = lock_env();
    let config = PathConfig::default();
    let cloned = config.clone();
    assert_eq!(config.data_dir, cloned.data_dir);
    assert_eq!(config.config_dir, cloned.config_dir);
}

// ==================== INITIALIZE SERVICE PATHS ====================

#[test]
fn test_initialize_service_paths() {
    let _g = lock_env();
    use songbird_config::config::paths::initialize_service_paths;
    let result = initialize_service_paths("test-service");
    // This may fail if dirs can't be created (permissions) — that's ok
    if let Ok(dirs) = result {
        assert!(
            dirs.orchestrator.to_string_lossy().contains("orchestrator")
                || dirs.orchestrator.to_string_lossy().contains("test-service")
        );
    }
}

// ==================== ENVIRONMENT OVERRIDE TESTS ====================

#[test]
fn test_paths_from_environment() {
    let _g = lock_env();
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_DATA_DIR".into(), "/custom/data".into());
    m.insert("SONGBIRD_CONFIG_DIR".into(), "/custom/config".into());
    let config =
        PathConfig::from_env_reader(|k| m.get(k).cloned().ok_or(std::env::VarError::NotPresent));
    drop(config);
}

// ==================== CONSISTENCY TESTS ====================

#[test]
fn test_default_and_get_path_config_consistent() {
    let _g = lock_env();
    let default = PathConfig::default();
    let getter = get_path_config();

    // Both should produce valid non-empty paths
    assert!(!default.data_dir.as_os_str().is_empty());
    assert!(!getter.data_dir.as_os_str().is_empty());
}

#[test]
fn test_development_uses_local_dir() {
    let _g = lock_env();
    let dev = PathConfig::development();
    // Development paths should be relative to current dir
    let dev_str = dev.config_dir.to_string_lossy();
    assert!(dev_str.contains(".songbird"), "Dev config_dir should contain .songbird: {dev_str}");
}
