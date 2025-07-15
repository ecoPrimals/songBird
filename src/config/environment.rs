//! Environment Variable Configuration Support
//!
//! Universal environment variable integration for all configuration values

use crate::config::constants;
use crate::errors::{Result, SongbirdError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::str::FromStr;
use std::time::Duration;
/// Environment variable configuration provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Prefix for all environment variables
    pub prefix: String,

    /// Whether to fallback to defaults when env vars are not set
    pub use_defaults: bool,
    /// Custom environment variable mappings
    pub custom_mappings: HashMap<String, String>,

    // Network Security Configuration
    pub bind_address: String,
    pub bind_port: u16,
    pub discovery_ports: Vec<u16>,
    pub gaming_port_range: (u16, u16),
    pub metrics_port: u16,
    pub dashboard_port: u16,
    pub websocket_port: u16,

    // Service Endpoints (no hardcoding!)
    pub beardog_endpoint: String,
    pub federation_endpoints: Vec<String>,
    pub stun_servers: Vec<String>,

    // Timeout Configuration (all configurable)
    pub connection_timeout_secs: u64,
    pub request_timeout_secs: u64,
    pub health_check_timeout_secs: u64,
    pub discovery_timeout_secs: u64,
    pub session_timeout_secs: u64,

    // File System Configuration (security critical)
    pub data_dir: String,
    pub config_dir: String,
    pub log_dir: String,
    pub cache_dir: String,
    pub runtime_dir: String,

    // Security Configuration
    pub enable_encryption: bool,
    pub require_tls: bool,
    pub allowed_networks: Vec<String>,
    pub max_connections: u32,

    // Performance Configuration
    pub max_memory_mb: u64,
    pub max_bandwidth_mbps: u64,
    pub worker_threads: usize,

    // Monitoring Configuration
    pub metrics_interval_secs: u64,
    pub health_check_interval_secs: u64,
    pub log_level: String,
}
impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            prefix: "SONGBIRD_".to_string(),
            use_defaults: true,
            custom_mappings: HashMap::new(),

            // Secure network defaults (localhost-only by default)
            bind_address: env_or_default("SONGBIRD_BIND_ADDRESS", "127.0.0.1"),
            bind_port: env_or_parse("SONGBIRD_BIND_PORT", 8080),
            discovery_ports: parse_port_list(&env_or_default(
                "SONGBIRD_DISCOVERY_PORTS",
                "6112,6113,6114",
            )),
            gaming_port_range: parse_port_range(&env_or_default(
                "SONGBIRD_GAMING_PORT_RANGE",
                "7000-8000",
            )),
            metrics_port: env_or_parse("SONGBIRD_METRICS_PORT", 9090),
            dashboard_port: env_or_parse("SONGBIRD_DASHBOARD_PORT", 3000),
            websocket_port: env_or_parse("SONGBIRD_WEBSOCKET_PORT", 8081),

            // Service endpoints (all configurable)
            beardog_endpoint: env_or_default(
                "SONGBIRD_BEARDOG_ENDPOINT",
                "https://beardog.internal:8443",
            ),
            federation_endpoints: parse_endpoint_list(&env_or_default(
                "SONGBIRD_FEDERATION_ENDPOINTS",
                "",
            )),
            stun_servers: parse_endpoint_list(&env_or_default(
                "SONGBIRD_STUN_SERVERS",
                "stun.l.google.com:19302,stun1.l.google.com:19302,stun.stunprotocol.org:3478",
            )),

            // Configurable timeouts (network conditions vary)
            connection_timeout_secs: env_or_parse("SONGBIRD_CONNECTION_TIMEOUT", 30),
            request_timeout_secs: env_or_parse("SONGBIRD_REQUEST_TIMEOUT", 60),
            health_check_timeout_secs: env_or_parse("SONGBIRD_HEALTH_CHECK_TIMEOUT", 10),
            discovery_timeout_secs: env_or_parse("SONGBIRD_DISCOVERY_TIMEOUT", 5),
            session_timeout_secs: env_or_parse("SONGBIRD_SESSION_TIMEOUT", 3600),

            // File system paths (platform/deployment specific)
            data_dir: env_or_default("SONGBIRD_DATA_DIR", &default_data_dir()),
            config_dir: env_or_default("SONGBIRD_CONFIG_DIR", &default_config_dir()),
            log_dir: env_or_default("SONGBIRD_LOG_DIR", &default_log_dir()),
            cache_dir: env_or_default("SONGBIRD_CACHE_DIR", &default_cache_dir()),
            runtime_dir: env_or_default("SONGBIRD_RUNTIME_DIR", &default_runtime_dir()),

            // Security configuration
            enable_encryption: env_or_parse("SONGBIRD_ENABLE_ENCRYPTION", true),
            require_tls: env_or_parse("SONGBIRD_REQUIRE_TLS", false),
            allowed_networks: parse_network_list(&env_or_default(
                "SONGBIRD_ALLOWED_NETWORKS",
                "127.0.0.0/8,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16",
            )),
            max_connections: env_or_parse("SONGBIRD_MAX_CONNECTIONS", 1000),

            // Performance configuration (hardware specific)
            max_memory_mb: env_or_parse("SONGBIRD_MAX_MEMORY_MB", 2048),
            max_bandwidth_mbps: env_or_parse("SONGBIRD_MAX_BANDWIDTH_MBPS", 1000),
            worker_threads: env_or_parse("SONGBIRD_WORKER_THREADS", num_cpus::get()),

            // Monitoring configuration
            metrics_interval_secs: env_or_parse("SONGBIRD_METRICS_INTERVAL", 60),
            health_check_interval_secs: env_or_parse("SONGBIRD_HEALTH_CHECK_INTERVAL", 30),
            log_level: env_or_default("SONGBIRD_LOG_LEVEL", "info"),
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
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on"))
            .unwrap_or(default)
    }

    /// Create configuration with complete environment variable support
    pub fn from_env() -> Result<Self> {
        let config = Self::default();

        // Validate critical security settings
        config.validate_security_settings()?;

        Ok(config)
    }

    /// Validate security-critical configuration
    fn validate_security_settings(&self) -> Result<()> {
        // Validate bind address is not dangerous in production
        if std::env::var("SONGBIRD_ENV").unwrap_or_default() == "production"
            && self.bind_address == "0.0.0.0"
        {
            return Err(SongbirdError::Config {
                field: Some("bind_address".to_string()),
                message: "Production environments should not bind to 0.0.0.0 without explicit configuration".to_string(),
            });
        }

        // Validate port ranges
        if self.gaming_port_range.0 >= self.gaming_port_range.1 {
            return Err(SongbirdError::Config {
                field: Some("gaming_port_range".to_string()),
                message: "Invalid gaming port range".to_string(),
            });
        }

        // Validate directories exist or can be created
        for (name, path) in [
            ("data_dir", &self.data_dir),
            ("config_dir", &self.config_dir),
            ("log_dir", &self.log_dir),
        ] {
            if let Err(e) = std::fs::create_dir_all(path) {
                return Err(SongbirdError::Config {
                    field: Some(name.to_string()),
                    message: format!("Cannot create directory {path}: {e}"),
                });
            }
        }

        Ok(())
    }

    /// Get full socket address for binding
    pub fn socket_addr(&self) -> Result<std::net::SocketAddr> {
        format!("{}:{}", self.bind_address, self.bind_port)
            .parse()
            .map_err(|e| SongbirdError::Config {
                field: Some("socket_addr".to_string()),
                message: format!("Invalid socket address: {e}"),
            })
    }

    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.connection_timeout_secs)
    }

    /// Get request timeout as Duration  
    pub fn request_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.request_timeout_secs)
    }
}

// Helper functions for parsing environment variables

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_or_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_port_list(s: &str) -> Vec<u16> {
    if s.is_empty() {
        return vec![];
    }
    s.split(',').filter_map(|p| p.trim().parse().ok()).collect()
}

fn parse_port_range(s: &str) -> (u16, u16) {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() == 2 {
        let start = parts[0].parse().unwrap_or(7000);
        let end = parts[1].parse().unwrap_or(8000);
        (start, end)
    } else {
        (7000, 8000)
    }
}

fn parse_endpoint_list(s: &str) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
    s.split(',')
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect()
}

fn parse_network_list(s: &str) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }
    s.split(',')
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .collect()
}

// Platform-specific default paths (no hardcoding!)
fn default_data_dir() -> String {
    if let Ok(home) = std::env::var("HOME") {
        format!("{home}/.local/share/songbird")
    } else {
        "/var/lib/songbird".to_string()
    }
}

fn default_config_dir() -> String {
    if let Ok(home) = std::env::var("HOME") {
        format!("{home}/.config/songbird")
    } else {
        "/etc/songbird".to_string()
    }
}

fn default_log_dir() -> String {
    if let Ok(home) = std::env::var("HOME") {
        format!("{home}/.local/share/songbird/logs")
    } else {
        "/var/log/songbird".to_string()
    }
}

fn default_cache_dir() -> String {
    if let Ok(home) = std::env::var("HOME") {
        format!("{home}/.cache/songbird")
    } else {
        "/var/cache/songbird".to_string()
    }
}

fn default_runtime_dir() -> String {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        format!("{runtime_dir}/songbird")
    } else if let Ok(home) = std::env::var("HOME") {
        format!("{home}/.local/run/songbird")
    } else {
        "/tmp/songbird".to_string()
    }
}

/// Environment-aware configuration trait
pub trait EnvironmentAware {
    /// Load configuration from environment variables
    fn from_env() -> Self;
    /// Load configuration from environment with custom config
    fn from_env_with_config(env_config: &EnvironmentConfig) -> Self;
}
// Environment configuration helper macros removed - using direct implementations instead
/// Environment variable mappings for core configuration
pub struct EnvMappings;
impl EnvMappings {
    /// Get all standard environment variable mappings
    pub fn get_standard_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        // Core orchestrator mappings
        mappings.insert("id".to_string(), "SONGBIRD_ORCHESTRATOR_ID".to_string());
        mappings.insert(
            "bind_address".to_string(),
            "SONGBIRD_BIND_ADDRESS".to_string(),
        );
        mappings.insert("port".to_string(), "SONGBIRD_PORT".to_string());
        mappings.insert(
            "max_services".to_string(),
            "SONGBIRD_MAX_SERVICES".to_string(),
        );
        mappings.insert("log_level".to_string(), "SONGBIRD_LOG_LEVEL".to_string());
        // Network mappings
        mappings.insert(
            "interface".to_string(),
            "SONGBIRD_NETWORK_INTERFACE".to_string(),
        );
        mappings.insert("enable_tls".to_string(), "SONGBIRD_ENABLE_TLS".to_string());
        mappings.insert(
            "tls_cert_path".to_string(),
            "SONGBIRD_TLS_CERT_PATH".to_string(),
        );
        mappings.insert(
            "tls_key_path".to_string(),
            "SONGBIRD_TLS_KEY_PATH".to_string(),
        );
        // Security mappings
        mappings.insert(
            "enable_auth".to_string(),
            "SONGBIRD_ENABLE_AUTH".to_string(),
        );
        mappings.insert("api_key".to_string(), "SONGBIRD_API_KEY".to_string());
        // Monitoring mappings
        mappings.insert(
            "enable_prometheus".to_string(),
            "SONGBIRD_ENABLE_PROMETHEUS".to_string(),
        );
        mappings.insert(
            "prometheus_endpoint".to_string(),
            "SONGBIRD_PROMETHEUS_ENDPOINT".to_string(),
        );
        // Discovery mappings
        mappings.insert(
            "discovery_backend".to_string(),
            "SONGBIRD_DISCOVERY_BACKEND".to_string(),
        );
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
        mappings.insert(
            "discovery_backend".to_string(),
            "DISCOVERY_BACKEND".to_string(),
        );
        mappings.insert(
            "k8s_namespace".to_string(),
            "KUBERNETES_NAMESPACE".to_string(),
        );
        mappings
    }
}
/// Configuration builder with environment support
#[derive(Debug)]
pub struct ConfigBuilder<T> {
    #[allow(dead_code)]
    base_config: T,
    env_config: EnvironmentConfig,
    file_path: Option<String>,
}

impl<T: Default + EnvironmentAware> Default for ConfigBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn with_env_mapping(mut self, message: &str, env_var: &str) -> Self {
        self.env_config
            .custom_mappings
            .insert(message.to_string(), env_var.to_string());
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
        // File configuration override not implemented in this version
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
            return Err(SongbirdError::Config {
                field: Some("environment_variables".to_string()),
                message: format!(
                    "Missing required environment variables: {}",
                    missing_vars.join(", ")
                ),
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
                        return Err(SongbirdError::Config {
                            field: Some(var_name.to_string()),
                            message: format!("{var_name} must be a valid URL"),
                        });
                    }
                }
                "port" => {
                    if value.parse::<u16>().is_err() {
                        return Err(SongbirdError::Config {
                            field: Some(var_name.to_string()),
                            message: format!("{var_name} must be a valid port number"),
                        });
                    }
                }
                "ip" => {
                    if value.parse::<std::net::IpAddr>().is_err() {
                        return Err(SongbirdError::Config {
                            field: Some(var_name.to_string()),
                            message: format!("{var_name} must be a valid IP address"),
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

        docs.insert(
            "SONGBIRD_BIND_ADDRESS".to_string(),
            EnvVarDoc {
                description: "IP address to bind the orchestrator API".to_string(),
                default_value: Some("crate::config::constants::default_bind_address()".to_string()),
                required: false,
                example: Some("0.0.0.0".to_string()),
                format: Some("ip".to_string()),
            },
        );

        docs.insert(
            "SONGBIRD_PORT".to_string(),
            EnvVarDoc {
                description: "Port for the orchestrator API".to_string(),
                default_value: Some("8080".to_string()),
                required: false,
                example: Some("3000".to_string()),
                format: Some("port".to_string()),
            },
        );

        docs.insert(
            "SONGBIRD_LOG_LEVEL".to_string(),
            EnvVarDoc {
                description: "Logging level (trace, debug, info, warn, error)".to_string(),
                default_value: Some("info".to_string()),
                required: false,
                example: Some("debug".to_string()),
                format: None,
            },
        );

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
        }
        Ok("staging") | Ok("stage") => {
            tracing::info!("Staging environment detected, using 0.0.0.0 for external access");
            constants::network::PRODUCTION_BIND_ADDRESS.to_string()
        }
        Ok("development") | Ok("dev") | Ok("local") => {
            tracing::debug!("Development environment detected, using default localhost binding");
            constants::network::DEFAULT_BIND_ADDRESS.to_string()
        }
        _ => {
            tracing::debug!("Unknown or unset environment, using default localhost binding");
            constants::network::DEFAULT_BIND_ADDRESS.to_string()
        }
    }
}

/// Check if we're running in a container environment
pub fn is_container_environment() -> bool {
    // Check common container indicators
    env::var("KUBERNETES_SERVICE_HOST").is_ok()
        || env::var("DOCKER_CONTAINER").is_ok()
        || std::path::Path::new("/.dockerenv").exists()
        || env::var("container").is_ok()
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
