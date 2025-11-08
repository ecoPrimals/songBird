//! **CANONICAL**: Environment Configuration - Single Source of Truth Truth
//!
//! Enhanced with comprehensive features from various environment configuration fragments.

use crate::SafeEnv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

/// **CANONICAL**: Environment Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEnvironmentConfig {
    /// Deployment mode (development, staging, production)
    /// Deployment Mode field
    pub deployment_mode: DeploymentMode,
    /// Resource limits and constraints
    /// Resource limitation configurations
    pub resource_limits: ResourceLimits,
    /// Service endpoints and discovery
    pub service_discovery: ServiceDiscoveryConfig,
    /// Network binding and addresses
    /// Network Binding field
    pub network_binding: NetworkBindingConfig,
    /// Environment variables and overrides
    pub environment_overrides: HashMap<String, String>,
    /// Capability-based service endpoints
    /// Capability Endpoints field
    pub capability_endpoints: CapabilityEndpoints,
    /// Legacy compatibility settings
    pub legacy_compatibility: LegacyCompatibilityConfig,
}

/// Deployment modes for environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentMode {
    /// Development environment with debug features enabled
    Development,
    /// Testing environment for automated testing
    Testing,
    /// Staging environment for pre-production testing
    Staging,
    /// Production environment with optimizations
    Production,
    /// Custom deployment mode with user-defined settings
    Custom(String),
}

/// Resource limits and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum number of concurrent connections
    /// Max Connections field
    pub max_connections: u32,
    /// Maximum memory usage in
    pub max_memory_mb: u64,
    /// Maximum CPU cores to use
    pub max_cpu_cores: u32,
    /// Maximum file descriptors
    pub max_file_descriptors: u32,
    /// Maximum number of threads
    pub max_threads: u32,
    /// Maximum disk space in
    pub disk_space_gb: u64,
    /// Memory pool configuration
    pub memory_pool: MemoryPoolConfig,
}

/// Memory pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolConfig {
    /// Enable memory pooling
    /// Enabled field
    pub enabled: bool,
    /// Initial pool size in /// MB
    /// Initial Size Mb field
    pub initial_size_mb: u64,
    /// Maximum pool size in
    pub max_size_mb: u64,
    /// Pool growth increment in /// MB
    /// Growth Increment Mb field
    pub growth_increment_mb: u64,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// Enable automatic service discovery
    pub auto_discovery: bool,
    /// Discovery refresh interval
    /// Refresh Interval field
    pub refresh_interval: Duration,
    /// Discovery timeout
    pub discovery_timeout: Duration,
    /// Fallback endpoints when discovery fails
    pub fallback_endpoints: HashMap<String, String>,
    /// Health check configuration
    /// Whether health checking is enabled
    pub health_checks: EnvironmentHealthCheckConfig,
}

/// Health check configuration for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentHealthCheckConfig {
    /// Enable health checks
    /// Enabled field
    pub enabled: bool,
    /// Health check interval
    /// Interval field
    pub interval: Duration,
    /// Health check timeout
    pub timeout: Duration,
    /// Number of retries before marking unhealthy
    pub max_retries: u32,
    /// Health check endpoint path
    /// Endpoint Path field
    pub endpoint_path: String,
}

/// Network binding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBindingConfig {
    /// Primary bind address
    /// Bind Address field
    pub bind_address: IpAddr,
    /// Production bind address
    /// Production Bind Address field
    pub production_bind_address: IpAddr,
    /// Default bind port
    /// Bind Port field
    pub bind_port: u16,
    /// Port range for dynamic allocation
    /// Port Range field
    pub port_range: PortRange,
    /// Network interface preferences
    /// Interface Preferences field
    pub interface_preferences: Vec<String>,
}

/// Port range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    /// Starting port number
    pub start: u16,
    /// Ending port number
    /// End field
    pub end: u16,
    /// Reserved ports to avoid
    pub reserved: Vec<u16>,
}

/// Capability-based service endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEndpoints {
    /// Storage capability endpoint
    pub storage: Option<String>,
    /// Compute capability endpoint
    pub compute: Option<String>,
    /// AI capability endpoint
    pub ai: Option<String>,
    /// Security capability endpoint
    pub security: Option<String>,
    /// Orchestration capability endpoint
    /// Orchestration field
    pub orchestration: Option<String>,
    /// Custom capability endpoints
    pub custom: HashMap<String, String>,
}

/// Legacy compatibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyCompatibilityConfig {
    /// Enable legacy primal name support
    /// Enable Legacy Primal Names field
    pub enable_legacy_primal_names: bool,
    /// Legacy endpoint mappings
    pub legacy_endpoints: HashMap<String, String>,
    /// Deprecation warnings configuration
    /// Deprecation Warnings field
    pub deprecation_warnings: DeprecationWarningsConfig,
}

/// Deprecation warnings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationWarningsConfig {
    /// Enable deprecation warnings
    /// Enabled field
    pub enabled: bool,
    /// Log level for warnings
    pub log_level: String,
    /// Suppress specific warnings
    /// Suppress Warnings field
    pub suppress_warnings: Vec<String>,
}

impl Default for CanonicalEnvironmentConfig {
    fn default() -> Self {
        Self {
            deployment_mode: DeploymentMode::Development,
            resource_limits: ResourceLimits::default(),
            service_discovery: ServiceDiscoveryConfig::default(),
            network_binding: NetworkBindingConfig::default(),
            environment_overrides: HashMap::new(),
            capability_endpoints: CapabilityEndpoints::default(),
            legacy_compatibility: LegacyCompatibilityConfig::default(),
        }
    }
}

impl DeploymentMode {
    /// Create deployment mode from environment string
    #[must_use]
    pub fn from_env_string(env_str: &str) -> Self {
        match env_str {
            "production" => Self::Production,
            "staging" => Self::Staging,
            "testing" => Self::Testing,
            "development" => Self::Development,
            custom => Self::Custom(custom.to_string()),
        }
    }
}

impl Default for DeploymentMode {
    fn default() -> Self {
        Self::from_env_string(&SafeEnv::get_or_default("SONGBIRD_ENV", "development"))
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_connections: SafeEnv::get_usize("SONGBIRD_MAX_CONNECTIONS", 1000) as u32,
            max_memory_mb: SafeEnv::get_usize("SONGBIRD_MAX_MEMORY_MB", 2048) as u64,
            max_cpu_cores: SafeEnv::get_usize("SONGBIRD_MAX_CPU_CORES", 4) as u32,
            max_file_descriptors: SafeEnv::get_usize("SONGBIRD_MAX_FDS", 1024) as u32,
            max_threads: SafeEnv::get_usize("SONGBIRD_MAX_THREADS", 100) as u32,
            disk_space_gb: SafeEnv::get_usize("SONGBIRD_MAX_DISK_GB", 100) as u64,
            memory_pool: MemoryPoolConfig::default(),
        }
    }
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            enabled: SafeEnv::get_bool("SONGBIRD_MEMORY_POOL_ENABLED", true),
            initial_size_mb: 64,
            max_size_mb: 512,
            growth_increment_mb: 32,
        }
    }
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_discovery: SafeEnv::get_bool("SONGBIRD_AUTO_DISCOVERY", true),
            refresh_interval: Duration::from_secs(
                SafeEnv::get_usize("SONGBIRD_DISCOVERY_REFRESH_INTERVAL", 30) as u64,
            ),
            discovery_timeout: Duration::from_secs(10),
            fallback_endpoints: HashMap::new(),
            health_checks: EnvironmentHealthCheckConfig::default(),
        }
    }
}

impl Default for EnvironmentHealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            max_retries: 3,
            endpoint_path: "/health".to_string(),
        }
    }
}

impl Default for NetworkBindingConfig {
    fn default() -> Self {
        Self {
            bind_address: SafeEnv::get_or_default("SONGBIRD_BIND_ADDRESS", "0.0.0.0")
                .parse()
                .unwrap_or_else(|_| std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),
            production_bind_address: SafeEnv::get_or_default("SONGBIRD_PRODUCTION_BIND_ADDRESS", "127.0.0.1")
                .parse()
                .unwrap_or_else(|_| std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
            bind_port: SafeEnv::get_port("SONGBIRD_BIND_PORT", crate::constants::DEFAULT_PORT),
            port_range: PortRange::default(),
            interface_preferences: vec!["eth0".to_string(), "en0".to_string()],
        }
    }
}

impl Default for PortRange {
    fn default() -> Self {
        Self {
            start: 8000,
            end: 9000,
            reserved: vec![8080, 8443, 8888],
        }
    }
}

impl Default for CapabilityEndpoints {
    fn default() -> Self {
        Self {
            storage: SafeEnv::get_required("SONGBIRD_STORAGE_ENDPOINT").ok(),
            compute: SafeEnv::get_required("SONGBIRD_COMPUTE_ENDPOINT").ok(),
            ai: SafeEnv::get_required("SONGBIRD_AI_ENDPOINT").ok(),
            security: SafeEnv::get_required("SONGBIRD_SECURITY_ENDPOINT").ok(),
            orchestration: SafeEnv::get_required("SONGBIRD_ORCHESTRATION_ENDPOINT").ok(),
            custom: HashMap::new(),
        }
    }
}

impl Default for LegacyCompatibilityConfig {
    fn default() -> Self {
        Self {
            enable_legacy_primal_names: SafeEnv::get_bool("SONGBIRD_ENABLE_LEGACY_NAMES", true),
            legacy_endpoints: HashMap::new(),
            deprecation_warnings: DeprecationWarningsConfig::default(),
        }
    }
}

impl Default for DeprecationWarningsConfig {
    fn default() -> Self {
        Self {
            enabled: SafeEnv::get_bool("SONGBIRD_DEPRECATION_WARNINGS", true),
            log_level: "warn".to_string(),
            suppress_warnings: Vec::new(),
        }
    }
}

impl CanonicalEnvironmentConfig {
    /// Get service endpoint by capability type
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]
    pub fn get_capability_endpoint(&self, capability: &str) -> Option<String> {
        match capability {
            "storage" => self.capability_endpoints.storage.clone(),
            "compute" => self.capability_endpoints.compute.clone(),
            "ai" => self.capability_endpoints.ai.clone(),
            "security" => self.capability_endpoints.security.clone(),
            "orchestration" => self.capability_endpoints.orchestration.clone(),
            custom => self.capability_endpoints.custom.get(custom).cloned(),
        }
    }
    /// Get all configured endpoints
    #[must_use]
    pub fn get_all_endpoints(&self) -> HashMap<String, String> {
        let mut endpoints = HashMap::new();

        if let Some(storage) = &self.capability_endpoints.storage {
            endpoints.insert("storage".to_string(), storage.clone());
        }
        if let Some(compute) = &self.capability_endpoints.compute {
            endpoints.insert("compute".to_string(), compute.clone());
        }
        if let Some(ai) = &self.capability_endpoints.ai {
            endpoints.insert("ai".to_string(), ai.clone());
        }
        if let Some(security) = &self.capability_endpoints.security {
            endpoints.insert("security".to_string(), security.clone());
        }
        if let Some(orchestration) = &self.capability_endpoints.orchestration {
            endpoints.insert("orchestration".to_string(), orchestration.clone());
        }

        endpoints.extend(self.capability_endpoints.custom.clone());
        endpoints
    }

    /// Check if running in production mode
    #[must_use]
    pub const fn is_production(&self) -> bool {
        matches!(self.deployment_mode, DeploymentMode::Production)
    }

    /// Check if running in development mode
    #[must_use]
    pub const fn is_development(&self) -> bool {
        matches!(self.deployment_mode, DeploymentMode::Development)
    }

    /// Get the appropriate bind address based on deployment mode
    #[must_use]
    pub const fn get_bind_address(&self) -> IpAddr {
        match self.deployment_mode {
            DeploymentMode::Production => IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            _ => IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SongbirdError;
    use std::env;

    #[test]
    fn test_canonical_environment_config_default() {
        let config = CanonicalEnvironmentConfig::default();
        assert!(matches!(config.deployment_mode, DeploymentMode::Development));
        assert_eq!(config.environment_overrides.len(), 0);
    }

    #[test]
    fn test_deployment_mode_from_string() {
        // Test standard modes
        assert!(matches!(
            DeploymentMode::from_env_string("production"),
            DeploymentMode::Production
        ));
        assert!(matches!(DeploymentMode::from_env_string("staging"), DeploymentMode::Staging));
        assert!(matches!(DeploymentMode::from_env_string("testing"), DeploymentMode::Testing));
        assert!(matches!(
            DeploymentMode::from_env_string("development"),
            DeploymentMode::Development
        ));
    }

    #[test]
    fn test_deployment_mode_custom() {
        let mode = DeploymentMode::from_env_string("custom-env");
        assert!(matches!(mode, DeploymentMode::Custom(_)));
        if let DeploymentMode::Custom(name) = mode {
            assert_eq!(name, "custom-env");
        }
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_connections, 1000);
        assert_eq!(limits.max_memory_mb, 2048);
        assert_eq!(limits.max_cpu_cores, 4);
        assert_eq!(limits.max_file_descriptors, 1024);
        assert_eq!(limits.max_threads, 100);
        assert_eq!(limits.disk_space_gb, 100);
    }

    #[test]
    fn test_resource_limits_from_env() {
        env::set_var("SONGBIRD_MAX_CONNECTIONS", "5000");
        env::set_var("SONGBIRD_MAX_MEMORY_MB", "4096");
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_connections, 5000);
        assert_eq!(limits.max_memory_mb, 4096);
        env::remove_var("SONGBIRD_MAX_CONNECTIONS");
        env::remove_var("SONGBIRD_MAX_MEMORY_MB");
    }

    #[test]
    fn test_memory_pool_config_default() {
        let config = MemoryPoolConfig::default();
        assert!(config.enabled);
        assert_eq!(config.initial_size_mb, 64);
        assert_eq!(config.max_size_mb, 512);
        assert_eq!(config.growth_increment_mb, 32);
    }

    #[test]
    fn test_memory_pool_config_sizes_valid() {
        let config = MemoryPoolConfig::default();
        assert!(config.initial_size_mb <= config.max_size_mb);
        assert!(config.growth_increment_mb > 0);
    }

    #[test]
    fn test_service_discovery_config_default() {
        let config = ServiceDiscoveryConfig::default();
        assert!(config.auto_discovery);
        assert!(config.refresh_interval.as_secs() > 0);
        assert!(config.discovery_timeout.as_secs() > 0);
        assert_eq!(config.fallback_endpoints.len(), 0);
    }

    #[test]
    fn test_health_check_config_default() {
        let config = EnvironmentHealthCheckConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval.as_secs(), 30);
        assert_eq!(config.timeout.as_secs(), 5);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.endpoint_path, "/health");
    }

    #[test]
    fn test_network_binding_config_default() {
        let config = NetworkBindingConfig::default();
        assert_eq!(config.bind_port, 8080);
        assert_eq!(config.interface_preferences.len(), 2);
    }

    #[test]
    fn test_port_range_default() {
        let range = PortRange::default();
        assert_eq!(range.start, 8000);
        assert_eq!(range.end, 9000);
        assert!(range.end > range.start);
        assert_eq!(range.reserved.len(), 3);
    }

    #[test]
    #[serial_test::serial]
    fn test_capability_endpoints_default() {
        // Clear any environment variables that might interfere with default test
        env::remove_var("SONGBIRD_STORAGE_ENDPOINT");
        env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
        env::remove_var("SONGBIRD_AI_ENDPOINT");
        env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
        env::remove_var("SONGBIRD_ORCHESTRATION_ENDPOINT");

        let endpoints = CapabilityEndpoints::default();
        assert!(endpoints.storage.is_none());
        assert!(endpoints.compute.is_none());
        assert!(endpoints.ai.is_none());
        assert!(endpoints.security.is_none());
        assert!(endpoints.orchestration.is_none());
        assert_eq!(endpoints.custom.len(), 0);
    }

    #[test]
    #[serial_test::serial]
    fn test_capability_endpoints_from_env() {
        env::set_var("SONGBIRD_STORAGE_ENDPOINT", "http://storage:8001");
        env::set_var("SONGBIRD_AI_ENDPOINT", "http://ai:8002");
        let endpoints = CapabilityEndpoints::default();
        assert_eq!(endpoints.storage, Some("http://storage:8001".to_string()));
        assert_eq!(endpoints.ai, Some("http://ai:8002".to_string()));
        env::remove_var("SONGBIRD_STORAGE_ENDPOINT");
        env::remove_var("SONGBIRD_AI_ENDPOINT");
    }

    #[test]
    fn test_legacy_compatibility_config_default() {
        let config = LegacyCompatibilityConfig::default();
        assert!(config.enable_legacy_primal_names);
        assert_eq!(config.legacy_endpoints.len(), 0);
    }

    #[test]
    fn test_deprecation_warnings_config_default() {
        let config = DeprecationWarningsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.log_level, "warn");
        assert_eq!(config.suppress_warnings.len(), 0);
    }

    #[test]
    fn test_get_capability_endpoint_storage() {
        let mut config = CanonicalEnvironmentConfig::default();
        config.capability_endpoints.storage = Some("http://storage:8001".to_string());
        let endpoint = config.get_capability_endpoint("storage");
        assert_eq!(endpoint, Some("http://storage:8001".to_string()));
    }

    #[test]
    fn test_get_capability_endpoint_compute() {
        let mut config = CanonicalEnvironmentConfig::default();
        config.capability_endpoints.compute = Some("http://compute:8002".to_string());
        let endpoint = config.get_capability_endpoint("compute");
        assert_eq!(endpoint, Some("http://compute:8002".to_string()));
    }

    #[test]
    fn test_get_capability_endpoint_custom() {
        let mut config = CanonicalEnvironmentConfig::default();
        config
            .capability_endpoints
            .custom
            .insert("custom".to_string(), "http://custom:9000".to_string());
        let endpoint = config.get_capability_endpoint("custom");
        assert_eq!(endpoint, Some("http://custom:9000".to_string()));
    }

    #[test]
    #[serial_test::serial]
    fn test_get_capability_endpoint_none() {
        env::remove_var("SONGBIRD_STORAGE_ENDPOINT");
        let config = CanonicalEnvironmentConfig::default();
        let endpoint = config.get_capability_endpoint("storage");
        assert_eq!(endpoint, None);
    }

    #[test]
    #[serial_test::serial]
    fn test_get_all_endpoints_empty() {
        env::remove_var("SONGBIRD_STORAGE_ENDPOINT");
        env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
        env::remove_var("SONGBIRD_AI_ENDPOINT");
        env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
        env::remove_var("SONGBIRD_ORCHESTRATION_ENDPOINT");
        let config = CanonicalEnvironmentConfig::default();
        let endpoints = config.get_all_endpoints();
        assert_eq!(endpoints.len(), 0);
    }

    #[test]
    fn test_get_all_endpoints_with_values() {
        let mut config = CanonicalEnvironmentConfig::default();
        config.capability_endpoints.storage = Some("http://storage:8001".to_string());
        config.capability_endpoints.ai = Some("http://ai:8002".to_string());
        config
            .capability_endpoints
            .custom
            .insert("metrics".to_string(), "http://metrics:9090".to_string());

        let endpoints = config.get_all_endpoints();
        assert_eq!(endpoints.len(), 3);
        assert_eq!(endpoints.get("storage"), Some(&"http://storage:8001".to_string()));
        assert_eq!(endpoints.get("ai"), Some(&"http://ai:8002".to_string()));
        assert_eq!(endpoints.get("metrics"), Some(&"http://metrics:9090".to_string()));
    }

    #[test]
    fn test_is_production() {
        let config = CanonicalEnvironmentConfig {
            deployment_mode: DeploymentMode::Production,
            ..Default::default()
        };
        assert!(config.is_production());
        assert!(!config.is_development());
    }

    #[test]
    fn test_is_development() {
        let config = CanonicalEnvironmentConfig {
            deployment_mode: DeploymentMode::Development,
            ..Default::default()
        };
        assert!(config.is_development());
        assert!(!config.is_production());
    }

    #[test]
    fn test_get_bind_address_production() {
        let config = CanonicalEnvironmentConfig {
            deployment_mode: DeploymentMode::Production,
            ..Default::default()
        };
        let addr = config.get_bind_address();
        assert_eq!(addr, IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
    }

    #[test]
    fn test_get_bind_address_development() {
        let config = CanonicalEnvironmentConfig {
            deployment_mode: DeploymentMode::Development,
            ..Default::default()
        };
        let addr = config.get_bind_address();
        assert_eq!(addr, IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn test_serialization_canonical_config() -> Result<(), Box<dyn std::error::Error>> {
        let config = CanonicalEnvironmentConfig::default();
        let json = serde_json::to_string(&config)
            .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
        let deserialized: CanonicalEnvironmentConfig =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Failed to deserialize: {}", e),
                debug_info: None,
            })?;
        assert!(matches!(deserialized.deployment_mode, DeploymentMode::Development));
        Ok(())
    }

    #[test]
    fn test_port_range_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let range = PortRange::default();
        let json = serde_json::to_string(&range)
            .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
        let deserialized: PortRange =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Failed to deserialize: {}", e),
                debug_info: None,
            })?;
        assert_eq!(deserialized.start, range.start);
        assert_eq!(deserialized.end, range.end);
        Ok(())
    }
}
