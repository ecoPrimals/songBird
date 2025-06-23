//! Environment Variable Configuration Support
//!
//! Universal environment variable integration for all configuration values

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::time::Duration;

use crate::errors::{Result, SongbirdError};
use crate::config::constants;

/// Environment variable configuration provider
#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    /// Prefix for all environment variables
    pub prefix: String,
    
    /// Whether to fallback to defaults when env vars are not set
    pub use_defaults: bool,
    
    /// Custom environment variable mappings
    pub custom_mappings: HashMap<String, String>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            prefix: "SONGBIRD_".to_string(),
            use_defaults: true,
            custom_mappings: HashMap::new(),
        }
    }
}

impl EnvironmentConfig {
    /// Create new environment config with custom prefix
    pub fn with_prefix(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            ..Default::default()
        }
    }
    
    /// Get environment variable with prefix
    pub fn get_env(&self, key: &str) -> Option<String> {
        let env_key = if let Some(custom) = self.custom_mappings.get(key) {
            custom.clone()
        } else {
            format!("{}{}", self.prefix, key.to_uppercase())
        };
        
        env::var(env_key).ok()
    }
    
    /// Get environment variable or default
    pub fn get_env_or<T>(&self, key: &str, default: T) -> T 
    where 
        T: FromStr + Clone,
        T::Err: std::fmt::Debug,
    {
        self.get_env(key)
            .and_then(|v| v.parse::<T>().ok())
            .unwrap_or(default)
    }
    
    /// Get duration from environment variable
    pub fn get_duration_env(&self, key: &str, default: Duration) -> Duration {
        self.get_env(key)
            .and_then(|v| {
                if let Ok(secs) = v.parse::<u64>() {
                    Some(Duration::from_secs(secs))
                } else {
                    None
                }
            })
            .unwrap_or(default)
    }
    
    /// Get boolean from environment variable
    pub fn get_bool_env(&self, key: &str, default: bool) -> bool {
        self.get_env(key)
            .map(|v| {
                matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on")
            })
            .unwrap_or(default)
    }
}

/// Environment-aware configuration trait
pub trait EnvironmentAware {
    /// Load configuration from environment variables
    fn from_env() -> Self;
    
    /// Load configuration from environment with custom config
    fn from_env_with_config(env_config: &EnvironmentConfig) -> Self;
}

/// Macro to implement environment variable support for configuration structs
macro_rules! impl_env_config {
    ($struct_name:ident {
        $(
            $field:ident: $env_key:literal => $field_type:ty
        ),* $(,)?
    }) => {
        impl EnvironmentAware for $struct_name {
            fn from_env() -> Self {
                Self::from_env_with_config(&EnvironmentConfig::default())
            }
            
            fn from_env_with_config(env_config: &EnvironmentConfig) -> Self {
                let defaults = Self::default();
                Self {
                    $(
                        $field: env_config.get_env_or($env_key, defaults.$field.clone()),
                    )*
                }
            }
        }
    };
}

/// Environment variable mappings for core configuration
pub struct EnvMappings;

impl EnvMappings {
    /// Get all standard environment variable mappings
    pub fn get_standard_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        
        // Core orchestrator mappings
        mappings.insert("id".to_string(), "SONGBIRD_ORCHESTRATOR_ID".to_string());
        mappings.insert("bind_address".to_string(), "SONGBIRD_BIND_ADDRESS".to_string());
        mappings.insert("port".to_string(), "SONGBIRD_PORT".to_string());
        mappings.insert("max_services".to_string(), "SONGBIRD_MAX_SERVICES".to_string());
        mappings.insert("log_level".to_string(), "SONGBIRD_LOG_LEVEL".to_string());
        
        // Network mappings
        mappings.insert("interface".to_string(), "SONGBIRD_NETWORK_INTERFACE".to_string());
        mappings.insert("enable_tls".to_string(), "SONGBIRD_ENABLE_TLS".to_string());
        mappings.insert("tls_cert_path".to_string(), "SONGBIRD_TLS_CERT_PATH".to_string());
        mappings.insert("tls_key_path".to_string(), "SONGBIRD_TLS_KEY_PATH".to_string());
        
        // Security mappings
        mappings.insert("enable_auth".to_string(), "SONGBIRD_ENABLE_AUTH".to_string());
        mappings.insert("api_key".to_string(), "SONGBIRD_API_KEY".to_string());
        
        // Monitoring mappings
        mappings.insert("enable_prometheus".to_string(), "SONGBIRD_ENABLE_PROMETHEUS".to_string());
        mappings.insert("prometheus_endpoint".to_string(), "SONGBIRD_PROMETHEUS_ENDPOINT".to_string());
        
        // Discovery mappings
        mappings.insert("discovery_backend".to_string(), "SONGBIRD_DISCOVERY_BACKEND".to_string());
        
        mappings
    }
    
    /// Get all Docker/Kubernetes standard mappings
    pub fn get_container_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        
        // Standard container environment variables
        mappings.insert("bind_address".to_string(), "BIND_ADDRESS".to_string());
        mappings.insert("port".to_string(), "PORT".to_string());
        mappings.insert("log_level".to_string(), "LOG_LEVEL".to_string());
        
        // Kubernetes service discovery
        mappings.insert("discovery_backend".to_string(), "DISCOVERY_BACKEND".to_string());
        mappings.insert("k8s_namespace".to_string(), "KUBERNETES_NAMESPACE".to_string());
        
        mappings
    }
}

/// Configuration builder with environment support
#[derive(Debug)]
pub struct ConfigBuilder<T> {
    base_config: T,
    env_config: EnvironmentConfig,
    file_path: Option<String>,
}

impl<T: Default + EnvironmentAware> ConfigBuilder<T> {
    /// Create new builder with defaults
    pub fn new() -> Self {
        Self {
            base_config: T::default(),
            env_config: EnvironmentConfig::default(),
            file_path: None,
        }
    }
    
    /// Set custom environment prefix
    pub fn with_env_prefix(mut self, prefix: &str) -> Self {
        self.env_config.prefix = prefix.to_string();
        self
    }
    
    /// Add custom environment variable mapping
    pub fn with_env_mapping(mut self, field: &str, env_var: &str) -> Self {
        self.env_config.custom_mappings.insert(field.to_string(), env_var.to_string());
        self
    }
    
    /// Set configuration file path
    pub fn with_file(mut self, path: &str) -> Self {
        self.file_path = Some(path.to_string());
        self
    }
    
    /// Build final configuration
    pub fn build(self) -> Result<T> {
        // Start with environment-based configuration
        let config = T::from_env_with_config(&self.env_config);
        
        // TODO: Override with file configuration if provided
        if let Some(_file_path) = self.file_path {
            // File loading would go here
        }
        
        Ok(config)
    }
}

/// Environment variable validation
pub struct EnvValidator;

impl EnvValidator {
    /// Validate that required environment variables are set
    pub fn validate_required_env_vars(required_vars: &[&str]) -> Result<()> {
        let missing_vars: Vec<&str> = required_vars
            .iter()
            .filter(|&&var| env::var(var).is_err())
            .copied()
            .collect();
        
        if !missing_vars.is_empty() {
            return Err(SongbirdError::Configuration { 
                field: "environment_variables".to_string(),
                message: format!("Missing required environment variables: {}", missing_vars.join(", "))
            });
        }
        
        Ok(())
    }
    
    /// Validate environment variable format
    pub fn validate_env_format(var_name: &str, expected_format: &str) -> Result<()> {
        if let Ok(value) = env::var(var_name) {
            match expected_format {
                "url" => {
                    if !value.starts_with("http://") && !value.starts_with("https://") {
                        return Err(SongbirdError::Configuration {
                            field: var_name.to_string(),
                            message: format!("{} must be a valid URL", var_name)
                        });
                    }
                }
                "port" => {
                    if value.parse::<u16>().is_err() {
                        return Err(SongbirdError::Configuration {
                            field: var_name.to_string(),
                            message: format!("{} must be a valid port number", var_name)
                        });
                    }
                }
                "ip" => {
                    if value.parse::<std::net::IpAddr>().is_err() {
                        return Err(SongbirdError::Configuration {
                            field: var_name.to_string(),
                            message: format!("{} must be a valid IP address", var_name)
                        });
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Get environment variable documentation
    pub fn get_env_documentation() -> HashMap<String, EnvVarDoc> {
        let mut docs = HashMap::new();
        
        docs.insert("SONGBIRD_BIND_ADDRESS".to_string(), EnvVarDoc {
            description: "IP address to bind the orchestrator API".to_string(),
            default_value: Some("127.0.0.1".to_string()),
            required: false,
            example: Some("0.0.0.0".to_string()),
            format: Some("ip".to_string()),
        });
        
        docs.insert("SONGBIRD_PORT".to_string(), EnvVarDoc {
            description: "Port for the orchestrator API".to_string(),
            default_value: Some("8080".to_string()),
            required: false,
            example: Some("3000".to_string()),
            format: Some("port".to_string()),
        });
        
        docs.insert("SONGBIRD_LOG_LEVEL".to_string(), EnvVarDoc {
            description: "Logging level (trace, debug, info, warn, error)".to_string(),
            default_value: Some("info".to_string()),
            required: false,
            example: Some("debug".to_string()),
            format: None,
        });
        
        // Add more documentation as needed...
        
        docs
    }
}

/// Environment variable documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarDoc {
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
    pub example: Option<String>,
    pub format: Option<String>,
}

/// Get environment-aware default bind address
pub fn get_default_bind_address() -> String {
    // Check explicit environment variable first
    if let Ok(addr) = env::var("SONGBIRD_BIND_ADDRESS") {
        return addr;
    }
    
    // Check deployment environment
    match env::var("SONGBIRD_ENVIRONMENT").as_deref() {
        Ok("production") | Ok("prod") => {
            tracing::info!("Production environment detected, using 0.0.0.0 for external access");
            constants::network::PRODUCTION_BIND_ADDRESS.to_string()
        },
        Ok("staging") | Ok("stage") => {
            tracing::info!("Staging environment detected, using 0.0.0.0 for external access");
            constants::network::PRODUCTION_BIND_ADDRESS.to_string()
        },
        Ok("development") | Ok("dev") | Ok("local") | _ => {
            tracing::debug!("Development environment detected, using 127.0.0.1 for localhost only");
            constants::network::DEFAULT_BIND_ADDRESS.to_string()
        }
    }
}

/// Check if we're running in a container environment
pub fn is_container_environment() -> bool {
    // Check common container indicators
    env::var("KUBERNETES_SERVICE_HOST").is_ok() ||
    env::var("DOCKER_CONTAINER").is_ok() ||
    std::path::Path::new("/.dockerenv").exists() ||
    env::var("container").is_ok()
}

/// Get production-safe default bind address for containers
pub fn get_container_bind_address() -> String {
    if is_container_environment() {
        tracing::info!("Container environment detected, using 0.0.0.0 for container networking");
        constants::network::PRODUCTION_BIND_ADDRESS.to_string()
    } else {
        get_default_bind_address()
    }
} 