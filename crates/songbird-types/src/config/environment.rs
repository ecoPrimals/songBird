//! **CANONICAL**: Environment Configuration - Single Source of Truth Truth
//!
//! Enhanced with comprehensive features from various environment configuration fragments.

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
            environment_overrides: HashMap::new()),
            capability_endpoints: CapabilityEndpoints::default(),
            legacy_compatibility: LegacyCompatibilityConfig::default(),
        }
    }
}

impl Default for DeploymentMode {
    fn default() -> Self {
        match std::env::var("SONGBIRD_ENV").as_deref() {
            Ok("production") => Self::Production,
            Ok("staging") => Self::Staging,
            Ok("testing") => Self::Testing,
            Ok(custom) => Self::Custom(custom.to_string()),
            _ => Self::Development,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_connections: std::env::var("SONGBIRD_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            max_memory_mb: std::env::var("SONGBIRD_MAX_MEMORY_MB")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2048),
            max_cpu_cores: std::env::var("SONGBIRD_MAX_CPU_CORES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(4),
            max_file_descriptors: std::env::var("SONGBIRD_MAX_FDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1024),
            max_threads: std::env::var("SONGBIRD_MAX_THREADS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            disk_space_gb: std::env::var("SONGBIRD_MAX_DISK_GB")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            memory_pool: MemoryPoolConfig::default(),
        }
    }
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("SONGBIRD_MEMORY_POOL_ENABLED")
                .map(|s| s.parse().unwrap_or(true))
                .unwrap_or(true),
            initial_size_mb: 64,
            max_size_mb: 512,
            growth_increment_mb: 32,
        }
    }
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_discovery: std::env::var("SONGBIRD_AUTO_DISCOVERY")
                .map(|s| s.parse().unwrap_or(true))
                .unwrap_or(true),
            refresh_interval: Duration::from_secs(
                std::env::var("SONGBIRD_DISCOVERY_REFRESH_INTERVAL")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
            ),
            discovery_timeout: Duration::from_secs(10),
            fallback_endpoints: HashMap::new()),
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
            bind_address: std::env::var("SONGBIRD_BIND_ADDRESS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| "0.0.0.0".parse().unwrap()),
            production_bind_address: std::env::var("SONGBIRD_PRODUCTION_BIND_ADDRESS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| "127.0.0.1".parse().unwrap()),
            bind_port: std::env::var("SONGBIRD_BIND_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
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
            storage: std::env::var("SONGBIRD_STORAGE_ENDPOINT").ok(),
            compute: std::env::var("SONGBIRD_COMPUTE_ENDPOINT").ok(),
            ai: std::env::var("SONGBIRD_AI_ENDPOINT").ok(),
            security: std::env::var("SONGBIRD_SECURITY_ENDPOINT").ok(),
            orchestration: std::env::var("SONGBIRD_ORCHESTRATION_ENDPOINT").ok(),
            custom: HashMap::new()),
        }
    }
}

impl Default for LegacyCompatibilityConfig {
    fn default() -> Self {
        Self {
            enable_legacy_primal_names: std::env::var("SONGBIRD_ENABLE_LEGACY_NAMES")
                .map(|s| s.parse().unwrap_or(true))
                .unwrap_or(true),
            legacy_endpoints: HashMap::new()),
            deprecation_warnings: DeprecationWarningsConfig::default(),
        }
    }
}

impl Default for DeprecationWarningsConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("SONGBIRD_DEPRECATION_WARNINGS")
                .map(|s| s.parse().unwrap_or(true))
                .unwrap_or(true),
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
