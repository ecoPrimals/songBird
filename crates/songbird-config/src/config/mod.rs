//! Configuration management for the Songbird Universal Orchestrator.
//!
//! This module provides comprehensive configuration management including
//! network settings, security configurations, discovery mechanisms,
//! and `BearDog` integration settings.
//!
//! # Examples
//!
//! ```
//! use songbird_config::config::SongbirdConfig;
//!
//! let config = SongbirdConfig::new();
//! println!("Config loaded: {:?}", config);
//! ```

use serde::{Deserialize, Serialize};
use songbird_errors::{Result, SongbirdError};
use std::collections::HashMap;
use std::path::Path;

// Helper function to create config errors with proper context
pub fn config_error(
    message: &str,
    field: Option<&str>,
    context: Option<&str>,
    suggestion: Option<&str>,
) -> SongbirdError {
    SongbirdError::Config {
        message: message.to_string(),
        field: field.map(|f| f.to_string()),
        context: context.map(|c| c.to_string()),
        suggestion: suggestion.map(|s| s.to_string()),
    }
}

pub mod constants;
pub mod environment;
pub mod network;
pub mod paths;
pub mod providers;
pub mod validation;

// Re-export commonly used configuration types
pub use constants::*;
pub use environment::*;
pub use network::*;
pub use paths::*;

// Alias for backward compatibility
pub type PathsConfig = PathConfig;

/// Main configuration structure for Songbird Orchestrator
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SongbirdConfig {
    /// Network configuration
    pub network: NetworkConfig,

    /// Environment configuration
    pub environment: EnvironmentConfig,

    /// Path configuration
    pub paths: PathsConfig,

    /// `BearDog` security integration (optional)
    pub beardog: Option<BearDogConfig>,

    /// Security configuration
    pub security: SecurityConfig,

    /// Additional custom configuration
    pub custom: HashMap<String, serde_json::Value>,
}

/// Orchestrator configuration (alias for compatibility)
pub type OrchestratorConfig = SongbirdConfig;

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub tls_enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub ca_path: Option<String>,

    /// JWT secret for authentication
    pub jwt_secret: Option<String>,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub dashboard_enabled: bool,
    pub dashboard_port: Option<u16>,
}

/// Gaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingConfig {
    pub enabled: bool,
    pub auto_detect: bool,
    pub supported_protocols: Vec<String>,
    pub bridge_timeout_secs: u64,
}

/// `BearDog` security module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogConfig {
    /// Enable `BearDog` integration
    pub enabled: bool,

    /// `BearDog` service endpoint configuration
    pub endpoint: BearDogEndpointConfig,

    /// Authentication configuration for `BearDog`
    pub authentication: BearDogAuthConfig,

    /// Default security settings
    pub security: BearDogSecurityConfig,
}

/// `BearDog` service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogEndpointConfig {
    /// Primary `BearDog` service URL
    pub primary_url: String,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Enable TLS verification
    pub verify_tls: bool,
}

/// `BearDog` authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogAuthConfig {
    /// Authentication method
    pub auth_method: BearDogAuthMethod,

    /// API key (if using API key auth)
    pub api_key: Option<String>,
}

/// `BearDog` authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BearDogAuthMethod {
    /// API key authentication
    ApiKey,
    /// Mutual TLS authentication
    MutualTls,
}

/// `BearDog` security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogSecurityConfig {
    /// Default security level for operations
    pub default_security_level: String,

    /// Enable automatic key rotation
    pub auto_key_rotation: bool,
}

impl Default for BearDogConfig {
    fn default() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();

        Self {
            enabled: false,
            endpoint: BearDogEndpointConfig {
                primary_url: env_config.beardog_endpoint,
                connection_timeout_secs: env_config.connection_timeout_secs,
                verify_tls: env_config.require_tls,
            },
            authentication: BearDogAuthConfig {
                auth_method: BearDogAuthMethod::ApiKey,
                api_key: None,
            },
            security: BearDogSecurityConfig {
                default_security_level: std::env::var("SONGBIRD_BEARDOG_SECURITY_LEVEL")
                    .unwrap_or_else(|_| "confidential".to_string()),
                auto_key_rotation: std::env::var("SONGBIRD_BEARDOG_AUTO_KEY_ROTATION")
                    .map(|v| v.parse().unwrap_or(false)) // Security: Default to false for safety
                    .unwrap_or(true),
            },
        }
    }
}

impl SongbirdConfig {
    /// Check if `BearDog` integration is enabled
    #[must_use]
    pub fn is_beardog_enabled(&self) -> bool {
        self.beardog.as_ref().is_some_and(|b| b.enabled)
    }

    /// Get `BearDog` configuration (returns default if not configured)
    #[must_use]
    pub fn get_beardog_config(&self) -> BearDogConfig {
        self.beardog.clone().unwrap_or_default()
    }

    /// Enable `BearDog` integration with default configuration
    pub fn enable_beardog(&mut self) {
        let beardog_config = BearDogConfig {
            enabled: true,
            ..BearDogConfig::default()
        };
        self.beardog = Some(beardog_config);
    }

    /// Disable `BearDog` integration
    pub fn disable_beardog(&mut self) {
        self.beardog = None;
    }
}

/// Configuration file formats
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            tls_enabled: false,
            cert_path: None,
            key_path: None,
            ca_path: None,

            jwt_secret: None,
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            tracing_enabled: true,
            dashboard_enabled: false,
            dashboard_port: Some(3000),
        }
    }
}

impl Default for GamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_detect: true,
            supported_protocols: vec![
                "IPX".to_string(),
                "DirectPlay".to_string(),
                "NetBIOS".to_string(),
                "UDP".to_string(),
                "TCP".to_string(),
            ],
            bridge_timeout_secs: 300,
        }
    }
}

impl SongbirdConfig {
    /// Load configuration from file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| SongbirdError::Config {
            field: Some("config_file".to_string()),
            message: format!("Failed to read config file: {e}"),
            context: Some("Configuration file reading".to_string()),
            suggestion: Some("Check file path and permissions".to_string()),
        })?;

        serde_yaml::from_str(&content).map_err(|e| SongbirdError::Config {
            field: Some("config_parse".to_string()),
            message: format!("Failed to parse config: {e}"),
            context: Some("Configuration parsing".to_string()),
            suggestion: Some("Check YAML syntax and format".to_string()),
        })
    }

    /// Save configuration to file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or serialized
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self).map_err(|e| SongbirdError::Config {
            field: Some("config_serialize".to_string()),
            message: format!("Failed to serialize config: {e}"),
            context: Some("Configuration serialization".to_string()),
            suggestion: Some("Check configuration data for serialization issues".to_string()),
        })?;

        std::fs::write(path, content).map_err(|e| SongbirdError::Config {
            field: Some("config_write".to_string()),
            message: format!("Failed to write config file: {e}"),
            context: Some("Configuration file writing".to_string()),
            suggestion: Some("Check file path permissions and available disk space".to_string()),
        })
    }

    /// Validate configuration security and completeness
    ///
    /// # Errors
    ///
    /// Returns an error if any validation checks fail, including security misconfigurations
    pub fn validate_basic(&self) -> Result<()> {
        self.validate_security()?;
        Ok(())
    }
}
pub mod hardcoded_elimination;
