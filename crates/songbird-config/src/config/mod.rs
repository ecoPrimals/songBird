//! Configuration management for Songbird components
//!
//! This module provides comprehensive configuration management including
//! network settings, security configurations, discovery mechanisms,
//! and `BearDog` integration settings.

use serde::{Deserialize, Serialize};
use songbird_errors::{Result, SongbirdError};
use std::collections::HashMap;
use std::path::Path;

pub mod constants;
pub mod environment;
pub mod network;
pub mod paths;
pub mod providers;
pub mod universal_primals;
pub mod validation;

// Re-export commonly used configuration types
pub use constants::*;
pub use environment::*;
pub use network::*;
pub use paths::*;
pub use universal_primals::*;

// Alias for backward compatibility
pub type PathsConfig = PathConfig;

/// Quality of service requirements for configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QosRequirements {
    /// Expected latency in milliseconds
    pub latency_ms: Option<f64>,

    /// Throughput in operations per second
    pub throughput_ops_sec: Option<f64>,

    /// Availability percentage (0.0 to 1.0)
    pub availability: Option<f64>,

    /// Reliability score (0.0 to 1.0)
    pub reliability: Option<f64>,

    /// Maximum acceptable error rate (0.0 to 1.0)
    pub max_error_rate: Option<f64>,
}

/// Main configuration structure for Songbird Orchestrator
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SongbirdConfig {
    /// Network configuration
    pub network: NetworkConfig,

    /// Environment configuration
    pub environment: EnvironmentConfig,

    /// Path configuration
    pub paths: PathsConfig,

    /// Universal primal registry (NEW - replaces hardcoded primal configs)
    pub primal_registry: Option<PrimalRegistry>,

    /// BearDog security integration (DEPRECATED - use primal_registry)
    #[serde(default)]
    pub beardog: Option<BearDogConfig>,

    /// Toadstool compute integration (DEPRECATED - use primal_registry)
    #[serde(default)]
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
                primary_url: std::env::var("SONGBIRD_TOADSTOOL_ENDPOINT").unwrap_or_else(|_| {
                    format!(
                        "http://{}:8081",
                        crate::config::environment::get_default_bind_address()
                    )
                }),
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
    /// Get the effective primal registry (migrates from legacy if needed)
    pub fn get_primal_registry(&self) -> PrimalRegistry {
        if let Some(registry) = &self.primal_registry {
            registry.clone()
        } else {
            // Migrate from legacy configuration
            LegacyConfigMigrator::migrate_legacy_config(self)
        }
    }

    /// Set the primal registry
    pub fn set_primal_registry(&mut self, registry: PrimalRegistry) {
        self.primal_registry = Some(registry);
    }

    /// Check if a primal type is enabled (universal method)
    pub fn is_primal_enabled(&self, primal_type: &str) -> bool {
        self.get_primal_registry()
            .get_primal(primal_type)
            .map(|p| p.enabled)
            .unwrap_or(false)
    }

    /// Get primal configuration by type (universal method)
    pub fn get_primal_config(&self, primal_type: &str) -> Option<PrimalConfiguration> {
        self.get_primal_registry().get_primal(primal_type).cloned()
    }

    /// Find primals with specific capability (universal method)
    pub fn find_primals_with_capability(&self, capability_type: &str) -> Vec<PrimalConfiguration> {
        self.get_primal_registry()
            .find_primals_with_capability(capability_type)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Enable a primal with basic configuration (universal method)
    pub fn enable_primal(&mut self, primal_type: &str, endpoint_url: &str) {
        let mut registry = self.get_primal_registry();

        if let Some(existing) = registry.primals.get_mut(primal_type) {
            existing.enabled = true;
            existing.endpoint.primary_url = endpoint_url.to_string();
        } else {
            let mut config = PrimalConfiguration::new_template(primal_type, primal_type);
            config.enabled = true;
            config.endpoint.primary_url = endpoint_url.to_string();

            // Add default capabilities based on known primal types
            config.capabilities = Self::get_default_capabilities_for_primal_type(primal_type);

            registry.register_primal(config);
        }

        self.primal_registry = Some(registry);
    }

    /// Get default capabilities for known primal types
    fn get_default_capabilities_for_primal_type(primal_type: &str) -> Vec<PrimalCapability> {
        match primal_type.to_lowercase().as_str() {
            "beardog" => vec![PrimalCapability {
                capability_type: "security".to_string(),
                version: "1.0".to_string(),
                parameters: std::collections::HashMap::new(),
                qos_metrics: QosMetrics::default(),
            }],
            "toadstool" => vec![PrimalCapability {
                capability_type: "compute".to_string(),
                version: "1.0".to_string(),
                parameters: std::collections::HashMap::new(),
                qos_metrics: QosMetrics::default(),
            }],
            "nestgate" => vec![PrimalCapability {
                capability_type: "storage".to_string(),
                version: "1.0".to_string(),
                parameters: std::collections::HashMap::new(),
                qos_metrics: QosMetrics::default(),
            }],
            "phoenix-ai" | "phoenix_ai" => vec![PrimalCapability {
                capability_type: "ai".to_string(),
                version: "1.0".to_string(),
                parameters: std::collections::HashMap::new(),
                qos_metrics: QosMetrics::default(),
            }],
            "squirrel" => vec![PrimalCapability {
                capability_type: "messaging".to_string(),
                version: "1.0".to_string(),
                parameters: std::collections::HashMap::new(),
                qos_metrics: QosMetrics::default(),
            }],
            // Default: basic capability for unknown primals
            _ => vec![PrimalCapability {
                capability_type: "basic".to_string(),
                version: "1.0".to_string(),
                parameters: std::collections::HashMap::new(),
                qos_metrics: QosMetrics::default(),
            }],
        }
    }

    /// Disable a primal (universal method)
    pub fn disable_primal(&mut self, primal_type: &str) {
        let mut registry = self.get_primal_registry();

        if let Some(existing) = registry.primals.get_mut(primal_type) {
            existing.enabled = false;
        }

        self.primal_registry = Some(registry);
    }

    // ===== BACKWARD COMPATIBILITY METHODS (DEPRECATED) =====

    /// Check if BearDog integration is enabled
    #[deprecated(note = "Use is_primal_enabled(\"beardog\") instead")]
    pub fn is_beardog_enabled(&self) -> bool {
        self.is_primal_enabled("beardog")
    }

    /// Get BearDog configuration (returns default if not configured)
    #[deprecated(note = "Use get_primal_config(\"beardog\") instead")]
    pub fn get_beardog_config(&self) -> BearDogConfig {
        // Return legacy config if it exists, otherwise create from primal registry
        if let Some(config) = &self.beardog {
            config.clone()
        } else {
            // Create legacy config from universal primal registry
            BearDogConfig::default()
        }
    }

    /// Enable BearDog integration with default configuration
    #[deprecated(note = "Use enable_primal(\"beardog\", endpoint_url) instead")]
    pub fn enable_beardog(&mut self) {
        self.enable_primal("beardog", &crate::config::constants::default_beardog_endpoint());
    }

    /// Disable BearDog integration
    #[deprecated(note = "Use disable_primal(\"beardog\") instead")]
    pub fn disable_beardog(&mut self) {
        self.disable_primal("beardog");
    }

    /// Check if Toadstool integration is enabled
    #[deprecated(note = "Use is_primal_enabled(\"toadstool\") instead")]
    pub fn is_toadstool_enabled(&self) -> bool {
        self.is_primal_enabled("toadstool")
    }

    /// Get Toadstool configuration (returns default if not configured)
    #[deprecated(note = "Use get_primal_config(\"toadstool\") instead")]
    pub fn get_toadstool_config(&self) -> ToadstoolConfig {
        // Return legacy config if it exists, otherwise create from primal registry
        if let Some(config) = &self.toadstool {
            config.clone()
        } else {
            // Create legacy config from universal primal registry
            ToadstoolConfig::default()
        }
    }

    /// Enable Toadstool integration with default configuration
    #[deprecated(note = "Use enable_primal(\"toadstool\", endpoint_url) instead")]
    pub fn enable_toadstool(&mut self) {
        self.enable_primal("toadstool", &crate::config::constants::default_toadstool_endpoint());
    }

    /// Disable Toadstool integration
    #[deprecated(note = "Use disable_primal(\"toadstool\") instead")]
    pub fn disable_toadstool(&mut self) {
        self.disable_primal("toadstool");
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
            message: format!("Failed to read config file: {e}"),
            context: None,
            suggestion: Some("Check if the file exists and is readable".to_string()),
        })?;

        toml::from_str(&content).map_err(|e| SongbirdError::Config {
            field: None,
            message: format!("Failed to parse config: {e}"),
            context: None,
            suggestion: Some("Check TOML syntax".to_string()),
        })
    }

    /// Save configuration to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self).map_err(|e| SongbirdError::Config {
            field: None,
            message: format!("Failed to serialize config: {e}"),
            context: None,
            suggestion: Some("Check if the config structure is valid".to_string()),
        })?;

        std::fs::write(path, content).map_err(|e| SongbirdError::Config {
            field: Some("config_file".to_string()),
            message: format!("Failed to write config file: {e}"),
            context: None,
            suggestion: Some("Check if you have write permissions".to_string()),
        })
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<()> {
        let mut validation_errors = Vec::new();

        if self.network.orchestrator_port == 0 {
            validation_errors.push("Network port cannot be zero".to_string());
        }

        // Validate port ranges
        if self.network.orchestrator_port < 1024
            && std::env::var("SONGBIRD_ALLOW_PRIVILEGED_PORTS").is_err()
        {
            validation_errors.push(
                "Port must be >= 1024 unless SONGBIRD_ALLOW_PRIVILEGED_PORTS is set".to_string(),
            );
        }

        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(SongbirdError::Config {
                field: None,
                message: validation_errors.join(", "),
                context: None,
                suggestion: Some("Check your configuration values".to_string()),
            })
        }
    }
}
pub mod hardcoded_elimination;
