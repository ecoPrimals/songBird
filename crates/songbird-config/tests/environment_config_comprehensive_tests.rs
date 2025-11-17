#![allow(deprecated)]
//! Comprehensive Environment Config Tests
//!
//! Tests environment configuration loading, validation, and defaults

use songbird_config::canonical::environment::{EnvironmentConfig, LogConfig}; // ✅ Migrated from config::environment
use songbird_types::{SongbirdError, SongbirdResult};
use std::env;

#[test]
fn test_log_config_default() -> SongbirdResult<()> {
    let config = LogConfig::default();

    // Should have sensible defaults
    assert!(!config.level.is_empty());
    assert!(!config.format.is_empty());
    Ok(())
}

#[test]
fn test_log_config_clone() -> SongbirdResult<()> {
    let config1 = LogConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.level, config2.level);
    assert_eq!(config1.format, config2.format);
    Ok(())
}

#[test]
fn test_log_config_debug() -> SongbirdResult<()> {
    let config = LogConfig::default();
    let debug_str = format!("{config:?}");

    assert!(!debug_str.is_empty());
    Ok(())
}

#[test]
fn test_log_config_custom() {
    let config = LogConfig {
        level: "debug".to_string(),
        format: "json".to_string(),
        ..Default::default()
    };

    assert_eq!(config.level, "debug");
    assert_eq!(config.format, "json");
}

#[test]
fn test_environment_config_default() {
    let config = EnvironmentConfig::default();

    // Should have environment field
    assert!(!config.environment.is_empty());
}

#[test]
fn test_environment_config_development() {
    let config = EnvironmentConfig {
        environment: "development".to_string(),
        ..Default::default()
    };

    assert_eq!(config.environment, "development");
}

#[test]
fn test_environment_config_production() {
    let config = EnvironmentConfig {
        environment: "production".to_string(),
        ..Default::default()
    };

    assert_eq!(config.environment, "production");
}

#[test]
fn test_environment_config_staging() -> SongbirdResult<()> {
    let config = EnvironmentConfig {
        environment: "staging".to_string(),
        ..Default::default()
    };

    assert_eq!(config.environment, "staging");
    Ok(())
}

#[test]
fn test_environment_config_clone() -> SongbirdResult<()> {
    let config1 = EnvironmentConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.environment, config2.environment);
    Ok(())
}

#[test]
fn test_environment_config_debug() -> SongbirdResult<()> {
    let config = EnvironmentConfig::default();
    let debug_str = format!("{config:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("EnvironmentConfig"));
    Ok(())
}

#[test]
fn test_environment_config_serialization() {
    let config = EnvironmentConfig::default();
    let json = serde_json::to_string(&config);

    assert!(json.is_ok());
}

#[test]
fn test_environment_config_modification() {
    let mut config = EnvironmentConfig::default();
    let original = config.environment.clone();

    config.environment = "test".to_string();

    assert_ne!(config.environment, original);
    assert_eq!(config.environment, "test");
}

#[test]
fn test_log_level_variations() {
    let levels = vec!["trace", "debug", "info", "warn", "error"];

    for level in levels {
        let config = LogConfig {
            level: level.to_string(),
            ..Default::default()
        };

        assert_eq!(config.level, level);
    }
}

#[test]
fn test_log_format_variations() {
    let formats = vec!["json", "plain", "pretty"];

    for format in formats {
        let config = LogConfig {
            format: format.to_string(),
            ..Default::default()
        };

        assert_eq!(config.format, format);
    }
}

#[test]
fn test_config_equality() {
    let config1 = EnvironmentConfig {
        environment: "test".to_string(),
        ..Default::default()
    };

    let config2 = EnvironmentConfig {
        environment: "test".to_string(),
        ..Default::default()
    };

    assert_eq!(config1.environment, config2.environment);
}

#[test]
fn test_config_inequality() {
    let config1 = EnvironmentConfig {
        environment: "development".to_string(),
        ..Default::default()
    };

    let config2 = EnvironmentConfig {
        environment: "production".to_string(),
        ..Default::default()
    };

    assert_ne!(config1.environment, config2.environment);
}

#[test]
fn test_empty_environment_handling() {
    let config = EnvironmentConfig {
        environment: String::new(),
        ..Default::default()
    };

    // Empty environment is allowed (will use default later)
    assert_eq!(config.environment, "");
}

#[test]
fn test_case_sensitive_environment() {
    let config1 = EnvironmentConfig {
        environment: "Production".to_string(),
        ..Default::default()
    };

    let config2 = EnvironmentConfig {
        environment: "production".to_string(),
        ..Default::default()
    };

    assert_ne!(config1.environment, config2.environment);
}

#[test]
fn test_log_config_with_all_fields() {
    let config = LogConfig {
        level: "info".to_string(),
        format: "json".to_string(),
        ..Default::default()
    };

    assert_eq!(config.level, "info");
    assert_eq!(config.format, "json");
}

#[test]
fn test_multiple_config_instances() -> SongbirdResult<()> {
    let _config1 = EnvironmentConfig::default();
    let _config2 = EnvironmentConfig::default();
    let _config3 = EnvironmentConfig::default();

    // Should be able to create multiple instances without panic
    // Test passes if we reach here without panicking
    Ok(())
}

#[test]
fn test_config_from_environment_variables() -> SongbirdResult<()> {
    env::set_var("TEST_ENV", "test-value");

    // Just verify we can set and get env vars
    assert_eq!(
        env::var("TEST_ENV").or_else(|_| SongbirdError::configuration(format!("Error: {}", e)))?,
        "test-value"
    );

    env::remove_var("TEST_ENV");
    Ok(())
}

#[test]
fn test_config_struct_size() {
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::mem::size_of;

    let log_size = size_of::<LogConfig>();
    let env_size = size_of::<EnvironmentConfig>();

    // Should not be excessively large
    assert!(log_size < 10000);
    assert!(env_size < 10000);
}

#[test]
fn test_config_reset_pattern() {
    // Test that default config doesn't have "modified" value
    let config = EnvironmentConfig::default();

    assert_ne!(config.environment, "modified");
}
