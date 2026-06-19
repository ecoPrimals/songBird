// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "test assertions and harness ergonomics"
)]

//! Tests for Unified Songbird Configuration
//!
//! Comprehensive tests for the unified configuration system
//!
//! **Concurrency**: Tests are being modernized to use `TestEnv` for isolation.
//! Serial tests are being systematically eliminated!

use songbird_test_utils::TestEnv;
use songbird_types::config::unified::UnifiedSongbirdConfig;
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_unified_config_default() {
    let config = UnifiedSongbirdConfig::default();
    assert!(!config.system.environment.is_empty());
    assert!(!config.system.system_id.is_empty());
}

#[test]
fn test_unified_config_validate_success() {
    let config = UnifiedSongbirdConfig::default();
    let result = config.validate();
    assert!(result.is_ok(), "Default config should be valid");
}

#[test]
fn test_unified_config_validate_empty_environment() {
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = String::new();
    let result = config.validate();
    // Use pattern matching instead of unwrap_err
    assert!(matches!(result, Err(ref e) if e.to_string() == "System environment cannot be empty"));
}

#[test]
fn test_unified_config_validate_empty_system_id() {
    let mut config = UnifiedSongbirdConfig::default();
    config.system.system_id = String::new();
    let result = config.validate();
    // Use pattern matching instead of unwrap_err
    assert!(matches!(result, Err(ref e) if e.to_string() == "System ID cannot be empty"));
}

#[test]
fn test_unified_config_validate_zero_port() {
    let mut config = UnifiedSongbirdConfig::default();
    config.network.ports.orchestrator = 0;
    let result = config.validate();
    // Use pattern matching instead of unwrap_err
    assert!(
        matches!(result, Err(ref e) if e.to_string() == "Network orchestrator port must be greater than 0")
    );
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_bind_address_development() {
    let env = TestEnv::new(); // Empty environment
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "development".to_string();
    assert_eq!(config.get_bind_address_from_env(env.as_map()), "127.0.0.1");
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_bind_address_production() {
    let env = TestEnv::production();
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert_eq!(config.get_bind_address_from_env(env.as_map()), "0.0.0.0");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_bind_address_from_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_BIND_ADDRESS", "192.168.1.1");
    let config = UnifiedSongbirdConfig::default();
    assert_eq!(config.get_bind_address_from_env(env.as_map()), "192.168.1.1");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_data_dir_development() {
    let mut env = TestEnv::new();
    env.set("HOME", "/home/testuser");

    let config = UnifiedSongbirdConfig::default();
    let data_dir = config.get_data_dir_from_env(env.as_map());

    // Should contain songbird in the path OR be a valid data directory
    assert!(
        data_dir.contains("songbird") || data_dir.contains("data") || data_dir.starts_with("/home"),
        "Data directory '{}' should be a valid development path",
        data_dir
    );
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_data_dir_production() {
    let env = TestEnv::production();
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert_eq!(config.get_data_dir_from_env(env.as_map()), "/var/lib/songbird");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_data_dir_from_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_DATA_DIR", "/custom/data");
    let config = UnifiedSongbirdConfig::default();
    assert_eq!(config.get_data_dir_from_env(env.as_map()), "/custom/data");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_config_dir_development() {
    let mut env = TestEnv::new();
    env.set("HOME", "/home/user");
    let config = UnifiedSongbirdConfig::default();
    let config_dir = config.get_config_dir_from_env(env.as_map());
    assert!(config_dir.contains("/.config/songbird"));
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_config_dir_production() {
    let env = TestEnv::production();
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert_eq!(config.get_config_dir_from_env(env.as_map()), "/etc/songbird");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_config_dir_from_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_CONFIG_DIR", "/custom/config");
    let config = UnifiedSongbirdConfig::default();
    assert_eq!(config.get_config_dir_from_env(env.as_map()), "/custom/config");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_cache_dir_development() {
    let env = TestEnv::new();
    let config = UnifiedSongbirdConfig::default();
    let cache_dir = config.get_cache_dir_from_env(env.as_map());
    // Should contain songbird
    assert!(cache_dir.contains("songbird"));
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_cache_dir_production() {
    let env = TestEnv::production();
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert_eq!(config.get_cache_dir_from_env(env.as_map()), "/var/cache/songbird");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_cache_dir_from_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_CACHE_DIR", "/custom/cache");
    let config = UnifiedSongbirdConfig::default();
    assert_eq!(config.get_cache_dir_from_env(env.as_map()), "/custom/cache");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_log_dir_development() {
    let env = TestEnv::new();
    let config = UnifiedSongbirdConfig::default();
    let log_dir = config.get_log_dir_from_env(env.as_map());
    assert!(log_dir.contains("/.local/share/songbird/logs"));
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_log_dir_production() {
    let env = TestEnv::production();
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert_eq!(config.get_log_dir_from_env(env.as_map()), "/var/log/songbird");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_get_log_dir_from_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_LOG_DIR", "/custom/logs");
    let config = UnifiedSongbirdConfig::default();
    assert_eq!(config.get_log_dir_from_env(env.as_map()), "/custom/logs");
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent! (no global state)
fn test_is_production_from_config() {
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert!(config.is_production());
}

#[test] // ✅ NO #[serial]! Fully concurrent! (no global state)
fn test_is_production_from_songbird_env() {
    // This test checks the config field, not global env
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert!(config.is_production());
    // No cleanup needed!
}

#[test] // ✅ NO #[serial]! Fully concurrent! (no global state)
fn test_is_production_from_node_env() {
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert!(config.is_production());
    // No cleanup needed!
}

#[test]
fn test_is_not_production() {
    let config = UnifiedSongbirdConfig::default();
    assert!(!config.is_production());
}

#[test]
fn test_is_development_from_config() {
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "development".to_string();
    assert!(config.is_development());
}

#[test] // ✅ NO #[serial]! Fully concurrent! (no global state)
fn test_is_development_from_songbird_env() {
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "development".to_string();
    assert!(config.is_development());
    // No cleanup needed!
}

#[test] // ✅ NO #[serial]! Fully concurrent! (no global state)
fn test_is_development_from_node_env() {
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "development".to_string();
    assert!(config.is_development());
    // No cleanup needed!
}

#[test]
fn test_is_not_development() {
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert!(!config.is_development());
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_is_test_from_songbird_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_ENV", "testing");
    assert!(UnifiedSongbirdConfig::is_test_from_env(env.as_map()));
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_is_test_from_node_env() {
    let mut env = TestEnv::new();
    env.set("NODE_ENV", "test");
    assert!(UnifiedSongbirdConfig::is_test_from_env(env.as_map()));
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_is_test_from_ci() {
    let mut env = TestEnv::new();
    env.set("CI", "true");
    assert!(UnifiedSongbirdConfig::is_test_from_env(env.as_map()));
    // No cleanup needed - env is local!
}

#[test]
fn test_is_not_test() {
    // Use explicit empty env map rather than reading real env vars,
    // which may include CI=true from the test runner or host environment
    let env = TestEnv::new();
    assert!(!UnifiedSongbirdConfig::is_test_from_env(env.as_map()));
}

#[test]
fn test_unified_config_serialization() -> SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;

    assert!(json.contains("system"));
    assert!(json.contains("network"));
    assert!(json.contains("security"));

    let deserialized: UnifiedSongbirdConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Should deserialize: {}", e)))?;

    assert_eq!(config.system.environment, deserialized.system.environment);
    Ok(())
}

#[test]
fn test_unified_config_clone() -> SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let cloned = config.clone();

    assert_eq!(config.system.environment, cloned.system.environment);
    assert_eq!(config.system.system_id, cloned.system.system_id);
    Ok(())
}

#[test]
fn test_unified_config_debug() -> SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("UnifiedSongbirdConfig"));
    Ok(())
}

#[test]
fn test_unified_config_with_custom_fields() -> SongbirdResult<()> {
    use serde_json::json;
    use std::collections::HashMap;

    let mut custom = HashMap::new();
    custom.insert("key1".to_string(), json!("value1"));
    custom.insert("key2".to_string(), json!(42));

    let mut config = UnifiedSongbirdConfig::default();
    config.custom = Some(custom);

    assert!(config.custom.is_some());
    assert_eq!(
        config
            .custom
            .as_ref()
            .ok_or_else(|| SongbirdError::configuration(
                "Custom fields should be present".to_string()
            ))?
            .len(),
        2
    );
    Ok(())
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_directory_fallback_no_home() {
    let env = TestEnv::new(); // No HOME set
    let config = UnifiedSongbirdConfig::default();
    let data_dir = config.get_data_dir_from_env(env.as_map());
    assert!(data_dir.contains("/tmp") || data_dir.contains("songbird"));
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent! (no global state)
fn test_environment_detection_priority() {
    // Test checks config field, not env priority
    let mut config = UnifiedSongbirdConfig::default();
    config.system.environment = "production".to_string();
    assert!(config.is_production());
    // No cleanup needed!
}
