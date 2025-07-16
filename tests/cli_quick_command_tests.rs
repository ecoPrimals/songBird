//! Quick Command Tests
//!
//! Tests for the `songbird quick` command functionality
//! Focuses on one-touch gaming setup and basic configuration

use songbird_lib::config::SongbirdConfig;
use songbird_lib::errors::{Result, SongbirdError};
use std::path::PathBuf;
use tempfile::TempDir;

// Helper functions for tests
async fn setup_quick_gaming_environment(config_path: &std::path::Path) -> Result<()> {
    let mut config = SongbirdConfig::default();

    // Set up gaming-specific configuration
    config.network.bind_port = 8080;
    config.network.discovery_port = 8081;
    config.network.federation_port = 8082;
    config.network.health_port = 8083;
    config.network.dashboard_port = 8084;

    // Serialize configuration
    let config_toml = toml::to_string(&config).map_err(|e| SongbirdError::Configuration {
        field: "config_serialization".to_string(),
        message: format!("Failed to serialize config: {e}"),
    })?;

    std::fs::write(config_path, config_toml).map_err(|e| SongbirdError::Configuration {
        field: "config_write".to_string(),
        message: format!("Failed to write config: {e}"),
    })?;

    Ok(())
}

async fn configure_quick_discovery(config_path: &std::path::Path) -> Result<()> {
    // Check if config file exists
    if !config_path.exists() {
        return Err(SongbirdError::Configuration {
            field: "config_path".to_string(),
            message: "Config file does not exist".to_string(),
        });
    }

    // Read existing config and update discovery settings
    let config_content =
        std::fs::read_to_string(config_path).map_err(|e| SongbirdError::Configuration {
            field: "config_read".to_string(),
            message: format!("Failed to read config: {e}"),
        })?;

    let mut config: SongbirdConfig =
        toml::from_str(&config_content).map_err(|e| SongbirdError::Configuration {
            field: "config_parse".to_string(),
            message: format!("Failed to parse config: {e}"),
        })?;

    // Configure discovery settings
    config.network.discovery_port = 8081;

    // Serialize updated configuration
    let updated_config = toml::to_string(&config).map_err(|e| SongbirdError::Configuration {
        field: "config_serialize".to_string(),
        message: format!("Failed to serialize updated config: {e}"),
    })?;

    std::fs::write(config_path, updated_config).map_err(|e| SongbirdError::Configuration {
        field: "config_write".to_string(),
        message: format!("Failed to write updated config: {e}"),
    })?;

    Ok(())
}

async fn setup_basic_networking(config_path: &std::path::Path) -> Result<()> {
    let mut config = SongbirdConfig::default();

    // Basic networking configuration
    config.network.bind_port = 8080;
    config.network.bind_address = "127.0.0.1".parse().unwrap();

    let config_toml = toml::to_string(&config).unwrap();
    std::fs::write(config_path, config_toml).unwrap();

    Ok(())
}

async fn apply_security_defaults(config_path: &std::path::Path) -> Result<()> {
    let mut config = SongbirdConfig::default();

    // Apply security defaults
    config.security.tls_enabled = true;
    config.security.cert_path = Some("/etc/songbird/certs/server.crt".to_string());
    config.security.key_path = Some("/etc/songbird/certs/server.key".to_string());

    let config_toml = toml::to_string(&config).unwrap();
    std::fs::write(config_path, config_toml).unwrap();

    Ok(())
}

async fn validate_quick_setup(config_path: &std::path::Path) -> Result<()> {
    let config_content =
        std::fs::read_to_string(config_path).map_err(|e| SongbirdError::Configuration {
            field: "config_read".to_string(),
            message: format!("Failed to read config: {e}"),
        })?;

    let _config: SongbirdConfig =
        toml::from_str(&config_content).map_err(|e| SongbirdError::Configuration {
            field: "config_parse".to_string(),
            message: format!("Invalid configuration: {e}"),
        })?;

    Ok(())
}

#[cfg(test)]
mod quick_command_tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_command_basic_setup() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");

        let result = setup_quick_gaming_environment(&config_path).await;
        assert!(result.is_ok(), "Quick setup should succeed");
        assert!(config_path.exists(), "Config file should be created");
    }

    #[tokio::test]
    async fn test_quick_command_with_existing_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");

        // Create initial config
        let result = setup_quick_gaming_environment(&config_path).await;
        assert!(result.is_ok(), "Initial setup should succeed");

        // Update discovery settings
        let result = configure_quick_discovery(&config_path).await;
        assert!(result.is_ok(), "Discovery update should succeed");

        // Validate final config
        let result = validate_quick_setup(&config_path).await;
        assert!(result.is_ok(), "Final validation should succeed");
    }

    #[tokio::test]
    async fn test_quick_discovery_setup() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");

        // Set up base config first
        let result = setup_basic_networking(&config_path).await;
        assert!(result.is_ok(), "Base networking setup should succeed");

        // Configure discovery
        let result = configure_quick_discovery(&config_path).await;
        assert!(result.is_ok(), "Discovery configuration should succeed");
    }

    #[tokio::test]
    async fn test_quick_network_configuration() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");

        let result = setup_basic_networking(&config_path).await;
        assert!(result.is_ok(), "Network configuration should succeed");

        let result = validate_quick_setup(&config_path).await;
        assert!(result.is_ok(), "Network config validation should succeed");
    }

    #[tokio::test]
    async fn test_quick_security_defaults() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");

        let result = apply_security_defaults(&config_path).await;
        assert!(result.is_ok(), "Security defaults should be applied");

        let result = validate_quick_setup(&config_path).await;
        assert!(result.is_ok(), "Security config validation should succeed");
    }

    #[tokio::test]
    async fn test_invalid_config_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/songbird.toml");
        let result = configure_quick_discovery(&nonexistent_path).await;
        assert!(result.is_err(), "Config with invalid path should fail");
    }

    #[tokio::test]
    async fn test_quick_command_comprehensive_flow() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");

        // Full workflow test
        let result = setup_quick_gaming_environment(&config_path).await;
        assert!(result.is_ok(), "Gaming setup should succeed");

        let result = configure_quick_discovery(&config_path).await;
        assert!(result.is_ok(), "Discovery config should succeed");

        let result = apply_security_defaults(&config_path).await;
        assert!(result.is_ok(), "Security defaults should be applied");

        let result = validate_quick_setup(&config_path).await;
        assert!(result.is_ok(), "Final validation should succeed");
    }

    #[tokio::test]
    async fn test_quick_command_validation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");

        // Test validation with basic config
        let result = setup_basic_networking(&config_path).await;
        assert!(result.is_ok(), "Basic networking should work");

        let result = validate_quick_setup(&config_path).await;
        assert!(result.is_ok(), "Validation should pass for basic config");
    }

    #[tokio::test]
    async fn test_quick_command_error_handling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Test with directory instead of file
        let dir_path = temp_dir.path().to_path_buf();
        let result = setup_quick_gaming_environment(&dir_path).await;
        assert!(result.is_err(), "Setup with directory path should fail");
    }
}
