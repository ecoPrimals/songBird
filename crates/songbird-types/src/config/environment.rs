// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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

    /// Create deployment mode from environment variables (for testing)
    ///
    /// This allows passing a custom environment provider for testing without
    /// using global `std::env`, enabling concurrent test execution.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use songbird_types::config::environment::DeploymentMode;
    /// use std::collections::HashMap;
    ///
    /// let mut env = HashMap::new();
    /// env.insert(String::from("SONGBIRD_ENV"), String::from("production"));
    ///
    /// let mode = DeploymentMode::from_env_map(&env);
    /// assert!(matches!(mode, DeploymentMode::Production));
    /// ```
    #[must_use]
    pub fn from_env_map(env: &std::collections::HashMap<String, String>) -> Self {
        let env_str = env.get("SONGBIRD_ENV").map_or("development", String::as_str);
        Self::from_env_string(env_str)
    }
}

impl Default for DeploymentMode {
    fn default() -> Self {
        Self::from_env_string(&SafeEnv::get_or_default("SONGBIRD_ENV", "development"))
    }
}

impl ResourceLimits {
    /// Create from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn from_env_map(env: &std::collections::HashMap<String, String>) -> Self {
        let max_connections =
            env.get("SONGBIRD_MAX_CONNECTIONS").and_then(|s| s.parse::<u32>().ok()).unwrap_or(1000);
        let max_memory_mb =
            env.get("SONGBIRD_MAX_MEMORY_MB").and_then(|s| s.parse::<u64>().ok()).unwrap_or(2048);
        let max_cpu_cores =
            env.get("SONGBIRD_MAX_CPU_CORES").and_then(|s| s.parse::<u32>().ok()).unwrap_or(4);
        let max_file_descriptors =
            env.get("SONGBIRD_MAX_FDS").and_then(|s| s.parse::<u32>().ok()).unwrap_or(1024);
        let max_threads =
            env.get("SONGBIRD_MAX_THREADS").and_then(|s| s.parse::<u32>().ok()).unwrap_or(100);
        let disk_space_gb =
            env.get("SONGBIRD_MAX_DISK_GB").and_then(|s| s.parse::<u64>().ok()).unwrap_or(100);

        Self {
            max_connections,
            max_memory_mb,
            max_cpu_cores,
            max_file_descriptors,
            max_threads,
            disk_space_gb,
            memory_pool: MemoryPoolConfig::from_env_map(env),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_connections: u32::try_from(SafeEnv::get_usize("SONGBIRD_MAX_CONNECTIONS", 1000))
                .unwrap_or(1000),
            max_memory_mb: u64::try_from(SafeEnv::get_usize("SONGBIRD_MAX_MEMORY_MB", 2048))
                .unwrap_or(2048),
            max_cpu_cores: u32::try_from(SafeEnv::get_usize("SONGBIRD_MAX_CPU_CORES", 4))
                .unwrap_or(4),
            max_file_descriptors: u32::try_from(SafeEnv::get_usize("SONGBIRD_MAX_FDS", 1024))
                .unwrap_or(1024),
            max_threads: u32::try_from(SafeEnv::get_usize("SONGBIRD_MAX_THREADS", 100))
                .unwrap_or(100),
            disk_space_gb: u64::try_from(SafeEnv::get_usize("SONGBIRD_MAX_DISK_GB", 100))
                .unwrap_or(100),
            memory_pool: MemoryPoolConfig::default(),
        }
    }
}

impl MemoryPoolConfig {
    /// Create from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn from_env_map(env: &std::collections::HashMap<String, String>) -> Self {
        let enabled = env
            .get("SONGBIRD_MEMORY_POOL_ENABLED")
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);

        Self {
            enabled,
            initial_size_mb: 64,
            max_size_mb: 512,
            growth_increment_mb: 32,
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

impl ServiceDiscoveryConfig {
    /// Create from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn from_env_map(env: &std::collections::HashMap<String, String>) -> Self {
        let auto_discovery =
            env.get("SONGBIRD_AUTO_DISCOVERY").and_then(|s| s.parse::<bool>().ok()).unwrap_or(true);
        let refresh_interval = env
            .get("SONGBIRD_DISCOVERY_REFRESH_INTERVAL")
            .and_then(|s| s.parse::<u64>().ok())
            .map_or(Duration::from_secs(30), Duration::from_secs);

        Self {
            auto_discovery,
            refresh_interval,
            discovery_timeout: Duration::from_secs(10),
            fallback_endpoints: HashMap::new(),
            health_checks: EnvironmentHealthCheckConfig::default(),
        }
    }
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_discovery: SafeEnv::get_bool("SONGBIRD_AUTO_DISCOVERY", true),
            refresh_interval: Duration::from_secs(SafeEnv::get_usize(
                "SONGBIRD_DISCOVERY_REFRESH_INTERVAL",
                30,
            ) as u64),
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
            endpoint_path: String::from("/health"),
        }
    }
}

impl NetworkBindingConfig {
    /// Create from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn from_env_map(env: &std::collections::HashMap<String, String>) -> Self {
        let bind_address = env
            .get("SONGBIRD_BIND_ADDRESS")
            .and_then(|s| s.parse::<std::net::IpAddr>().ok())
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
        let production_bind_address = env
            .get("SONGBIRD_PRODUCTION_BIND_ADDRESS")
            .and_then(|s| s.parse::<std::net::IpAddr>().ok())
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
        let bind_port = env
            .get("SONGBIRD_BIND_PORT")
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(crate::constants::DEFAULT_PORT);

        Self {
            bind_address,
            production_bind_address,
            bind_port,
            port_range: PortRange::default(),
            interface_preferences: vec![String::from("eth0"), String::from("en0")],
        }
    }
}

impl Default for NetworkBindingConfig {
    fn default() -> Self {
        Self {
            bind_address: SafeEnv::get_or_default(
                "SONGBIRD_BIND_ADDRESS",
                crate::constants::PRODUCTION_BIND_ADDRESS,
            )
            .parse()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),
            production_bind_address: SafeEnv::get_or_default(
                "SONGBIRD_PRODUCTION_BIND_ADDRESS",
                crate::constants::DEVELOPMENT_BIND_ADDRESS,
            )
            .parse()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
            bind_port: SafeEnv::get_port("SONGBIRD_BIND_PORT", crate::constants::DEFAULT_PORT),
            port_range: PortRange::default(),
            interface_preferences: vec![String::from("eth0"), String::from("en0")],
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

impl CapabilityEndpoints {
    /// Create from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn from_env_map(env: &std::collections::HashMap<String, String>) -> Self {
        Self {
            storage: env.get("SONGBIRD_STORAGE_ENDPOINT").cloned(),
            compute: env.get("SONGBIRD_COMPUTE_ENDPOINT").cloned(),
            ai: env.get("SONGBIRD_AI_ENDPOINT").cloned(),
            security: env.get("SONGBIRD_SECURITY_ENDPOINT").cloned(),
            orchestration: env.get("SONGBIRD_ORCHESTRATION_ENDPOINT").cloned(),
            custom: HashMap::new(),
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

impl LegacyCompatibilityConfig {
    /// Create from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn from_env_map(env: &std::collections::HashMap<String, String>) -> Self {
        let enable_legacy_primal_names = env
            .get("SONGBIRD_ENABLE_LEGACY_NAMES")
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);

        Self {
            enable_legacy_primal_names,
            legacy_endpoints: HashMap::new(),
            deprecation_warnings: DeprecationWarningsConfig::from_env_map(env),
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

impl DeprecationWarningsConfig {
    /// Create from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn from_env_map(env: &std::collections::HashMap<String, String>) -> Self {
        let enabled = env
            .get("SONGBIRD_DEPRECATION_WARNINGS")
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);

        Self {
            enabled,
            log_level: String::from("warn"),
            suppress_warnings: Vec::new(),
        }
    }
}

impl Default for DeprecationWarningsConfig {
    fn default() -> Self {
        Self {
            enabled: SafeEnv::get_bool("SONGBIRD_DEPRECATION_WARNINGS", true),
            log_level: String::from("warn"),
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
        let std_count = usize::from(self.capability_endpoints.storage.is_some())
            + usize::from(self.capability_endpoints.compute.is_some())
            + usize::from(self.capability_endpoints.ai.is_some())
            + usize::from(self.capability_endpoints.security.is_some())
            + usize::from(self.capability_endpoints.orchestration.is_some());
        let mut endpoints =
            HashMap::with_capacity(std_count + self.capability_endpoints.custom.len());

        if let Some(storage) = &self.capability_endpoints.storage {
            endpoints.insert(String::from("storage"), storage.clone());
        }
        if let Some(compute) = &self.capability_endpoints.compute {
            endpoints.insert(String::from("compute"), compute.clone());
        }
        if let Some(ai) = &self.capability_endpoints.ai {
            endpoints.insert(String::from("ai"), ai.clone());
        }
        if let Some(security) = &self.capability_endpoints.security {
            endpoints.insert(String::from("security"), security.clone());
        }
        if let Some(orchestration) = &self.capability_endpoints.orchestration {
            endpoints.insert(String::from("orchestration"), orchestration.clone());
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
#[path = "environment_tests.rs"]
mod tests;
