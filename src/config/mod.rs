//! Configuration Module
//!
//! Provides configuration management for Songbird Orchestrator
//! including environment-specific settings, security configuration,
//! and BearDog integration settings.

use crate::errors::{Result, SongbirdError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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

    /// BearDog security integration (optional)
    pub beardog: Option<BearDogConfig>,

    /// Toadstool compute integration (optional)
    pub toadstool: Option<ToadstoolConfig>,

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

/// BearDog security module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogConfig {
    /// Enable BearDog integration
    pub enabled: bool,

    /// BearDog service endpoint configuration
    pub endpoint: BearDogEndpointConfig,

    /// Authentication configuration for BearDog
    pub authentication: BearDogAuthConfig,

    /// Default security settings
    pub security: BearDogSecurityConfig,
}

/// BearDog service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogEndpointConfig {
    /// Primary BearDog service URL
    pub primary_url: String,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Enable TLS verification
    pub verify_tls: bool,
}

/// BearDog authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogAuthConfig {
    /// Authentication method
    pub auth_method: BearDogAuthMethod,

    /// API key (if using API key auth)
    pub api_key: Option<String>,
}

/// BearDog authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BearDogAuthMethod {
    /// API key authentication
    ApiKey,
    /// Mutual TLS authentication
    MutualTls,
}

/// BearDog security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogSecurityConfig {
    /// Default security level for operations
    pub default_security_level: String,

    /// Enable automatic key rotation
    pub auto_key_rotation: bool,
}

/// Toadstool compute integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolConfig {
    /// Enable Toadstool integration
    pub enabled: bool,

    /// Toadstool service endpoint configuration
    pub endpoint: ToadstoolEndpointConfig,

    /// Authentication configuration for Toadstool
    pub authentication: ToadstoolAuthConfig,

    /// Default compute settings
    pub compute: ToadstoolComputeConfig,
}

/// Toadstool service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolEndpointConfig {
    /// Primary Toadstool service URL
    pub primary_url: String,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Enable TLS verification
    pub verify_tls: bool,
}

/// Toadstool authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolAuthConfig {
    /// Authentication method
    pub auth_method: ToadstoolAuthMethod,

    /// API key (if using API key auth)
    pub api_key: Option<String>,
}

/// Toadstool authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToadstoolAuthMethod {
    /// API key authentication
    ApiKey,
    /// Mutual TLS authentication
    MutualTls,
    /// No authentication (for development)
    None,
}

/// Toadstool compute configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolComputeConfig {
    /// Default container runtime
    pub default_runtime: String,

    /// Enable GPU support
    pub enable_gpu: bool,

    /// Default resource limits
    pub default_resource_limits: ToadstoolResourceLimits,
}

/// Toadstool resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolResourceLimits {
    /// Max CPU cores per deployment
    pub max_cpu_cores: f64,

    /// Max memory bytes per deployment
    pub max_memory_bytes: u64,

    /// Max storage bytes per deployment
    pub max_storage_bytes: u64,

    /// Max GPU count per deployment
    pub max_gpu_count: u32,
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
                    .map(|v| v.parse().unwrap_or(true))
                    .unwrap_or(true),
            },
        }
    }
}

impl Default for ToadstoolConfig {
    fn default() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        
        Self {
            enabled: false,
            endpoint: ToadstoolEndpointConfig {
                primary_url: std::env::var("SONGBIRD_TOADSTOOL_ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string()),
                connection_timeout_secs: env_config.connection_timeout_secs,
                verify_tls: env_config.require_tls,
            },
            authentication: ToadstoolAuthConfig {
                auth_method: ToadstoolAuthMethod::None,
                api_key: None,
            },
            compute: ToadstoolComputeConfig {
                default_runtime: "docker".to_string(),
                enable_gpu: false,
                default_resource_limits: ToadstoolResourceLimits {
                    max_cpu_cores: 16.0,
                    max_memory_bytes: 32 * 1024 * 1024 * 1024, // 32GB
                    max_storage_bytes: 100 * 1024 * 1024 * 1024, // 100GB
                    max_gpu_count: 4,
                },
            },
        }
    }
}

impl SongbirdConfig {
    /// Check if BearDog integration is enabled
    pub fn is_beardog_enabled(&self) -> bool {
        self.beardog.as_ref().map(|b| b.enabled).unwrap_or(false)
    }

    /// Get BearDog configuration (returns default if not configured)
    pub fn get_beardog_config(&self) -> BearDogConfig {
        self.beardog.clone().unwrap_or_default()
    }

    /// Enable BearDog integration with default configuration
    pub fn enable_beardog(&mut self) {
        let beardog_config = BearDogConfig { enabled: true, ..BearDogConfig::default() };
        self.beardog = Some(beardog_config);
    }

    /// Disable BearDog integration
    pub fn disable_beardog(&mut self) {
        self.beardog = None;
    }

    /// Check if Toadstool integration is enabled
    pub fn is_toadstool_enabled(&self) -> bool {
        self.toadstool.as_ref().map(|t| t.enabled).unwrap_or(false)
    }

    /// Get Toadstool configuration (returns default if not configured)
    pub fn get_toadstool_config(&self) -> ToadstoolConfig {
        self.toadstool.clone().unwrap_or_default()
    }

    /// Enable Toadstool integration with default configuration
    pub fn enable_toadstool(&mut self) {
        let toadstool_config = ToadstoolConfig { enabled: true, ..ToadstoolConfig::default() };
        self.toadstool = Some(toadstool_config);
    }

    /// Disable Toadstool integration
    pub fn disable_toadstool(&mut self) {
        self.toadstool = None;
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| SongbirdError::Config {
            field: Some("config_file".to_string()),
            message: format!("Failed to read config file: {}", e),
        })?;

        toml::from_str(&content).map_err(|e| SongbirdError::Config {
            field: None,
            message: format!("Failed to parse config: {}", e),
        })
    }

    /// Save configuration to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| SongbirdError::Config {
            field: None,
            message: format!("Failed to serialize config: {}", e),
        })?;

        std::fs::write(path, content).map_err(|e| SongbirdError::Config {
            field: Some("config_file".to_string()),
            message: format!("Failed to write config file: {}", e),
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.network.orchestrator_port == 0 {
            return Err(SongbirdError::Config {
                field: Some("node_id".to_string()),
                message: "Node ID cannot be empty".to_string(),
            });
        }

        if self.network.orchestrator_port == 0 {
            return Err(SongbirdError::Config {
                field: Some("port".to_string()),
                message: "Port cannot be zero".to_string(),
            });
        }

        Ok(())
    }
}
pub mod hardcoded_elimination;
