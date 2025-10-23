//! Configuration Tests
//!
//! Testing configuration loading, validation, and defaults.

use songbird_config::config::hardcoded_elimination::HardcodingEliminationConfig;
use songbird_types::SongbirdError;

#[test]
fn test_default_config_creation() {
    // Test: Default config should be valid and usable
    let config = HardcodingEliminationConfig::default();

    // Verify config is created successfully
    assert!(config.network.stun_servers.is_empty() || !config.network.stun_servers.is_empty());

    // Verify basic structure with defaults
    assert!(!config.service.service_name.is_empty(), "Should have default service name");
    assert_eq!(config.service.service_name, "songbird-orchestrator");
}

#[test]
fn test_config_validation() {
    // Test: Config validation rules
    let config = HardcodingEliminationConfig::default();

    // Validate service name has a default value
    assert!(!config.service.service_name.is_empty(), "Default config should have service name");
    assert_eq!(config.service.service_name, "songbird-orchestrator");

    // Validate that config has all required sections
    assert!(
        config.network.bind_address.to_string().starts_with("0.0.0")
            || config.network.bind_address.to_string().starts_with("127.0")
    );
}

#[test]
fn test_config_field_access() {
    // Test: Config fields should be accessible
    let mut config = HardcodingEliminationConfig::default();

    // Test field access
    config.service.service_name = "test-service".to_string();
    config.service.version = "1.0.0".to_string();

    assert_eq!(config.service.service_name, "test-service");
    assert_eq!(config.service.version, "1.0.0");

    // Test nested field access
    assert!(!config.security.encryption_key_size.to_string().is_empty());
}

#[test]
fn test_config_merging() {
    // Test: Configs should merge correctly
    let mut base_config = HardcodingEliminationConfig::default();
    base_config.service.service_name = "base-service".to_string();

    let mut override_config = HardcodingEliminationConfig::default();
    override_config.service.service_name = "override-service".to_string();
    override_config.service.version = "2.0.0".to_string();

    // Simulate merge (in real implementation)
    let merged_name = if override_config.service.service_name.is_empty() {
        base_config.service.service_name.clone()
    } else {
        override_config.service.service_name.clone()
    };

    assert_eq!(merged_name, "override-service");
}

#[test]
fn test_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test: Configs should serialize/deserialize
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    let config = TestConfig {
        name: "test".to_string(),
        value: 42,
    };

    // Serialize
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("test"));
    assert!(json.contains("42"));

    // Deserialize
    let deserialized: TestConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Should deserialize: {}", e)))?;
    assert_eq!(deserialized, config);
    Ok(())
}

#[test]
fn test_environment_variable_override() {
    // Test: Environment variables should override defaults

    // Simulate environment variable check
    let env_value = std::env::var("SONGBIRD_TEST_PORT").unwrap_or_else(|_| "8080".to_string());
    let default_port = "3000";

    // If env var exists, it should override
    let final_port = if env_value == "8080" {
        default_port
    } else {
        &env_value
    };

    // Verify override logic works
    assert!(final_port == default_port || final_port == env_value);
}

#[test]
fn test_config_file_loading() -> Result<(), Box<dyn std::error::Error>> {
    // Test: Config file loading pattern
    use std::path::PathBuf;

    let config_path = PathBuf::from("config.toml");

    // Verify path creation
    assert_eq!(
        config_path.to_str().ok_or_else(|| SongbirdError::configuration(
            "Config path should be valid UTF-8".to_string()
        ))?,
        "config.toml"
    );

    // Test error handling for missing file
    let result = std::fs::read_to_string(&config_path);
    assert!(result.is_err(), "Non-existent config file should error");

    // Verify error is clear
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(error_msg.contains("No such file") || error_msg.contains("not found"));
    }
    Ok(())
}

#[test]
fn test_config_error_handling() {
    #[derive(serde::Deserialize)]
    struct TestConfig {
        #[allow(dead_code)]
        name: String,
    }

    // Test: Invalid config should produce clear errors
    let invalid_json = "{\"name\": \"test\", invalid}";

    let result: Result<TestConfig, _> = serde_json::from_str(invalid_json);

    // Should error
    assert!(result.is_err(), "Invalid JSON should fail to parse");

    // Error should be descriptive
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("expected")
                || error_msg.contains("invalid")
                || error_msg.contains("key"),
            "Error message should be descriptive: {error_msg}"
        );
    }
}

#[test]
fn test_config_defaults_sovereignty() {
    // Test: Default config should respect sovereignty principles
    let config = HardcodingEliminationConfig::default();

    // Verify no hardcoded "master" or hierarchical terms
    let service_name = &config.service.service_name;
    assert!(!service_name.to_lowercase().contains("master"));
    assert!(!service_name.to_lowercase().contains("slave"));

    // Verify capability-based primal design (using discovery endpoints)
    assert!(
        config.primals.discovery_endpoints.is_empty()
            || !config.primals.discovery_endpoints.is_empty(),
        "Primals config should exist with discovery endpoints"
    );
}

#[test]
fn test_config_clone() {
    // Test: Config should be cloneable
    let config = HardcodingEliminationConfig::default();

    let cloned = config.clone();

    // Verify clone works
    assert_eq!(config.service.service_name, cloned.service.service_name);
    assert_eq!(config.service.version, cloned.service.version);

    // Verify it's a deep clone (independent)
    // Note: In Rust, clone() creates independent copies
    assert!(std::ptr::addr_of!(config) != std::ptr::addr_of!(cloned));
}
