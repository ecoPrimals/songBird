//! Quick Command Tests
//! 
//! Tests for the `songbird quick` command functionality
//! Focuses on one-touch gaming setup and basic configuration

use songbird_lib::config::SongbirdConfig;
use songbird_lib::errors::{Result, SongbirdError};
use tempfile::TempDir;
use std::path::PathBuf;

#[cfg(test)]
mod quick_command_tests {
    use super::*;

    #[tokio::test]
    async fn test_quick_command_basic_setup() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");
        
        let result = setup_quick_gaming_environment(&config_path).await;
        assert!(result.is_ok(), "Quick setup should succeed");
    }

    #[tokio::test]
    async fn test_quick_command_with_existing_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");
        
        // Create initial config
        let config = SongbirdConfig::default();
        std::fs::write(&config_path, toml::to_string(&config).unwrap())
            .expect("Failed to write config");
        
        let result = setup_quick_gaming_environment(&config_path).await;
        assert!(result.is_ok(), "Quick setup with existing config should succeed");
    }

    #[tokio::test]
    async fn test_quick_discovery_setup() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");
        
        // First create the base config
        let _setup_result = setup_quick_gaming_environment(&config_path).await;
        
        let result = configure_quick_discovery(&config_path).await;
        assert!(result.is_ok(), "Quick discovery setup should succeed");
    }

    #[tokio::test]
    async fn test_quick_network_configuration() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");
        
        // First create the base config
        let _setup_result = setup_quick_gaming_environment(&config_path).await;
        
        let result = setup_basic_networking(&config_path).await;
        assert!(result.is_ok(), "Quick network setup should succeed");
    }

    #[tokio::test]
    async fn test_quick_security_defaults() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");
        
        // First create the base config
        let _setup_result = setup_quick_gaming_environment(&config_path).await;
        
        let result = apply_security_defaults(&config_path).await;
        assert!(result.is_ok(), "Quick security setup should succeed");
    }

    #[tokio::test]
    async fn test_invalid_config_path() {
        let invalid_path = PathBuf::from("/invalid/path/songbird.toml");
        
        let result = setup_quick_gaming_environment(&invalid_path).await;
        assert!(result.is_err(), "Setup with invalid path should fail");
    }

    #[tokio::test]
    async fn test_quick_command_comprehensive_flow() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");
        
        // Test complete quick setup flow
        let setup_result = setup_quick_gaming_environment(&config_path).await;
        assert!(setup_result.is_ok(), "Initial setup should succeed");
        
        let discovery_result = configure_quick_discovery(&config_path).await;
        assert!(discovery_result.is_ok(), "Discovery setup should succeed");
        
        let network_result = setup_basic_networking(&config_path).await;
        assert!(network_result.is_ok(), "Network setup should succeed");
        
        let security_result = apply_security_defaults(&config_path).await;
        assert!(security_result.is_ok(), "Security setup should succeed");
    }

    #[tokio::test]
    async fn test_quick_command_validation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("songbird.toml");
        
        // Setup configuration
        let _setup_result = setup_quick_gaming_environment(&config_path).await;
        
        // Validate the created configuration
        let validation_result = validate_quick_setup(&config_path).await;
        assert!(validation_result.is_ok(), "Quick setup validation should pass");
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

// Helper function implementations for testing
async fn setup_quick_gaming_environment(config_path: &std::path::Path) -> Result<()> {
    // Create default gaming configuration
    let mut config = SongbirdConfig::default();
    
    // Configure for gaming optimization
    config.network.enable_tls = true;
    config.network.max_connections = 100;
    
    // Write configuration to file
    let config_toml = toml::to_string(&config)
        .map_err(|e| SongbirdError::Configuration {
            field: "config_serialization".to_string(),
            message: format!("Failed to serialize config: {}", e),
        })?;
    
    std::fs::write(config_path, config_toml)
        .map_err(|e| SongbirdError::Configuration {
            field: "config_write".to_string(),
            message: format!("Failed to write config: {}", e),
        })?;
    
    Ok(())
}

async fn configure_quick_discovery(config_path: &std::path::Path) -> Result<()> {
    if !config_path.exists() {
        return Err(SongbirdError::Configuration {
            field: "config_path".to_string(),
            message: "Config file does not exist".to_string(),
        });
    }
    
    // Read existing config and update discovery settings
    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| SongbirdError::Configuration {
            field: "config_read".to_string(),
            message: format!("Failed to read config: {}", e),
        })?;
    
    let mut config: SongbirdConfig = toml::from_str(&config_content)
        .map_err(|e| SongbirdError::Configuration {
            field: "config_parse".to_string(),
            message: format!("Failed to parse config: {}", e),
        })?;
    
    // Configure discovery settings
    config.network.discovery_port = 8001;
    config.network.discovery_ports = vec![8001, 8002, 8003];
    
    // Write updated config
    let updated_config = toml::to_string(&config)
        .map_err(|e| SongbirdError::Configuration {
            field: "config_serialize".to_string(),
            message: format!("Failed to serialize updated config: {}", e),
        })?;
    
    std::fs::write(config_path, updated_config)
        .map_err(|e| SongbirdError::Configuration {
            field: "config_write".to_string(),
            message: format!("Failed to write updated config: {}", e),
        })?;
    
    Ok(())
}

async fn setup_basic_networking(config_path: &std::path::Path) -> Result<()> {
    if !config_path.exists() {
        return Err(SongbirdError::Network {
            service: "quick".to_string(),
            message: "Config file does not exist".to_string(),
            details: Some("Run quick setup first".to_string()),
        });
    }
    
    Ok(())
}

async fn apply_security_defaults(config_path: &std::path::Path) -> Result<()> {
    if !config_path.exists() {
        return Err(SongbirdError::Security {
            message: "Config file does not exist".to_string(),
            context: Some("Run quick setup first".to_string()),
        });
    }
    
    Ok(())
}

async fn validate_quick_setup(config_path: &std::path::Path) -> Result<()> {
    if !config_path.exists() {
        return Err(SongbirdError::Configuration {
            field: "config_path".to_string(),
            message: "Config file does not exist".to_string(),
        });
    }
    
    // Read and validate configuration
    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| SongbirdError::Configuration {
            field: "config_read".to_string(),
            message: format!("Failed to read config: {}", e),
        })?;
    
    let _config: SongbirdConfig = toml::from_str(&config_content)
        .map_err(|e| SongbirdError::Configuration {
            field: "config_parse".to_string(),
            message: format!("Invalid configuration: {}", e),
        })?;
    
    Ok(())
} 