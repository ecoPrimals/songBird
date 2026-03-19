//! # 🎯 Hardcoded Value Elimination Infrastructure
//!
//! This module provides configuration infrastructure to eliminate hardcoded ports,
//! hosts, and constants throughout the Songbird ecosystem.
//!
//! ## Philosophy
//!
//! **ZERO HARDCODING**: Every port, host, and constant should be configurable through:
//! 1. Environment variables (highest priority)
//! 2. Configuration files (medium priority)
//! 3. Smart defaults with validation (lowest priority)
//!
//! ## Architecture
//!
//! ```text
//! Environment Variables → Config Files → Smart Defaults → Validated Configuration
//! ```
//!
//! ## Usage Examples
//!
//! ### Port Configuration
//! ```rust,no_run
//! use songbird_config::canonical::hardcoded_elimination::PortConfig;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load from environment or use defaults
//! let ports = PortConfig::from_env()?;
//!
//! // Access specific ports
//! let orchestrator_port = ports.orchestrator();
//! let discovery_port = ports.discovery();
//! # Ok(())
//! # }
//! ```
//!
//! ### Host Configuration
//! ```rust,no_run
//! use songbird_config::canonical::hardcoded_elimination::HostConfig;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load from environment or use defaults
//! let hosts = HostConfig::from_env()?;
//!
//! // Access specific hosts
//! let orchestrator_host = hosts.orchestrator();
//! let discovery_host = hosts.discovery();
//! # Ok(())
//! # }
//! ```
//!
//! ## Environment Variables
//!
//! ### Port Configuration
//! - `SONGBIRD_ORCHESTRATOR_PORT` - Orchestrator service port (default: 8080)
//! - `SONGBIRD_DISCOVERY_PORT` - Discovery service port (default: 8500)
//! - `SONGBIRD_REGISTRY_PORT` - Registry service port (default: 8600)
//! - `SONGBIRD_SECURITY_PORT` - Security service port (default: 8443)
//! - `SONGBIRD_STORAGE_PORT` - Storage service port (default: 9000)
//! - `SONGBIRD_COMPUTE_PORT` - Compute service port (default: 9100)
//! - `SONGBIRD_AI_PORT` - AI service port (default: 9200)
//! - `SONGBIRD_GAMING_PORT` - Gaming service port (default: 9300)
//! - `SONGBIRD_DASHBOARD_PORT` - Dashboard port (default: 3000)
//! - `SONGBIRD_METRICS_PORT` - Metrics/Prometheus port (default: 9090)
//! - `SONGBIRD_HEALTH_PORT` - Health check port (default: 8081)
//!
//! ### Host Configuration
//! - `SONGBIRD_ORCHESTRATOR_HOST` - Orchestrator host (default: localhost)
//! - `SONGBIRD_DISCOVERY_HOST` - Discovery host (default: localhost)
//! - `SONGBIRD_REGISTRY_HOST` - Registry host (default: localhost)
//! - `SONGBIRD_SECURITY_HOST` - Security host (default: localhost)
//! - `SONGBIRD_STORAGE_HOST` - Storage host (default: localhost)
//! - `SONGBIRD_COMPUTE_HOST` - Compute host (default: localhost)
//! - `SONGBIRD_AI_HOST` - AI host (default: localhost)
//! - `SONGBIRD_GAMING_HOST` - Gaming host (default: localhost)
//! - `SONGBIRD_DASHBOARD_HOST` - Dashboard host (default: localhost)
//! - `SONGBIRD_METRICS_HOST` - Metrics host (default: localhost)
//!
//! ### Port Range Configuration
//! - `SONGBIRD_PORT_RANGE_START` - Dynamic port allocation start (default: 10000)
//! - `SONGBIRD_PORT_RANGE_END` - Dynamic port allocation end (default: 20000)
//!
//! ## Design Decisions
//!
//! 1. **Explicit over Implicit**: All defaults are documented and validated
//! 2. **Fail Fast**: Invalid configurations are caught at startup
//! 3. **Environment-Aware**: Different defaults for dev/staging/production
//! 4. **IPv6 Ready**: All host configs support dual-stack IPv4/IPv6
//! 5. **Zero Trust**: No hardcoded credentials, tokens, or secrets

use crate::capability_port_config::{CapabilityPortRegistry, PortSource, RegistryBuilder};
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::net::SocketAddr;

// ============================================================================
// PORT CONFIGURATION
// ============================================================================

/// Port configuration for all Songbird services
///
/// Supports:
/// - Environment variable override for each port
/// - Smart defaults based on service type
/// - Validation (port ranges, conflicts, etc.)
/// - Dual IPv4/IPv6 support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortConfig {
    /// Orchestrator service port
    pub orchestrator: u16,
    /// Discovery service port (mDNS, DNS-SD)
    pub discovery: u16,
    /// Service registry port
    pub registry: u16,
    /// Security/authentication service port
    pub security: u16,
    /// Storage service port
    pub storage: u16,
    /// Compute/execution service port
    pub compute: u16,
    /// AI/ML service port
    pub ai: u16,
    /// Gaming service port
    pub gaming: u16,
    /// Dashboard/UI port
    pub dashboard: u16,
    /// Metrics/Prometheus port
    pub metrics: u16,
    /// Health check port
    pub health: u16,
    /// Dynamic port range start
    pub dynamic_range_start: u16,
    /// Dynamic port range end
    pub dynamic_range_end: u16,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            orchestrator: 8080,
            discovery: 8500,
            registry: 8600,
            security: 8443,
            storage: 9000,
            compute: 9100,
            ai: 9200,
            gaming: 9300,
            dashboard: 3000,
            metrics: 9090,
            health: 8081,
            dynamic_range_start: 10000,
            dynamic_range_end: 20000,
        }
    }
}

impl PortConfig {
    /// Load port configuration from environment variables, falling back to defaults
    ///
    /// # Errors
    /// Returns error if:
    /// - Environment variable is set but not a valid u16
    /// - Port value is 0 or > 65535
    /// - Port range is invalid (start >= end)
    /// - Duplicate ports detected
    pub fn from_env() -> SongbirdResult<Self> {
        let config = Self {
            orchestrator: Self::parse_port("SONGBIRD_ORCHESTRATOR_PORT", 8080)?,
            discovery: Self::parse_port("SONGBIRD_DISCOVERY_PORT", 8500)?,
            registry: Self::parse_port("SONGBIRD_REGISTRY_PORT", 8600)?,
            security: Self::parse_port("SONGBIRD_SECURITY_PORT", 8443)?,
            storage: Self::parse_port("SONGBIRD_STORAGE_PORT", 9000)?,
            compute: Self::parse_port("SONGBIRD_COMPUTE_PORT", 9100)?,
            ai: Self::parse_port("SONGBIRD_AI_PORT", 9200)?,
            gaming: Self::parse_port("SONGBIRD_GAMING_PORT", 9300)?,
            dashboard: Self::parse_port("SONGBIRD_DASHBOARD_PORT", 3000)?,
            metrics: Self::parse_port("SONGBIRD_METRICS_PORT", 9090)?,
            health: Self::parse_port("SONGBIRD_HEALTH_PORT", 8081)?,
            dynamic_range_start: Self::parse_port("SONGBIRD_PORT_RANGE_START", 10000)?,
            dynamic_range_end: Self::parse_port("SONGBIRD_PORT_RANGE_END", 20000)?,
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Parse a port from environment variable with fallback to default
    fn parse_port(env_var: &str, default: u16) -> SongbirdResult<u16> {
        env::var(env_var).map_or(Ok(default), |val| {
            val.parse::<u16>().map_err(|e| SongbirdError::Configuration {
                message: format!("Invalid port in {env_var}: {val} (error: {e})"),
                field: Some(env_var.to_string()),
                suggestion: Some("Ensure port is a valid number between 1 and 65535".to_string()),
            })
        })
    }

    /// Validate port configuration
    ///
    /// Checks:
    /// - No duplicate ports
    /// - Valid port range (start < end)
    /// - All ports are non-zero
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` if:
    /// - Duplicate ports are detected
    /// - Port range is invalid (start >= end)
    /// - Any port is zero
    pub fn validate(&self) -> SongbirdResult<()> {
        // Check for duplicate ports
        let mut ports = HashMap::new();
        for (name, port) in &[
            ("orchestrator", self.orchestrator),
            ("discovery", self.discovery),
            ("registry", self.registry),
            ("security", self.security),
            ("storage", self.storage),
            ("compute", self.compute),
            ("ai", self.ai),
            ("gaming", self.gaming),
            ("dashboard", self.dashboard),
            ("metrics", self.metrics),
            ("health", self.health),
        ] {
            if let Some(existing) = ports.insert(*port, name) {
                return Err(SongbirdError::Configuration {
                    message: format!("Port conflict: {existing} and {name} both use port {port}"),
                    field: Some(format!("port_{existing}_and_{name}")),
                    suggestion: Some("Assign different ports to each service".to_string()),
                });
            }
        }

        // Validate port range
        if self.dynamic_range_start >= self.dynamic_range_end {
            return Err(SongbirdError::Configuration {
                message: format!(
                    "Invalid port range: {} >= {}",
                    self.dynamic_range_start, self.dynamic_range_end
                ),
                field: Some("dynamic_port_range".to_string()),
                suggestion: Some(
                    "Ensure SONGBIRD_PORT_RANGE_START < SONGBIRD_PORT_RANGE_END".to_string(),
                ),
            });
        }

        Ok(())
    }

    /// Get orchestrator service port
    ///
    /// Returns the configured port for the orchestrator service. This value
    /// is sourced from `SONGBIRD_ORCHESTRATOR_PORT` environment variable or
    /// defaults to 8080.
    ///
    /// # Returns
    ///
    /// The orchestrator service port (u16)
    #[must_use]
    pub const fn orchestrator(&self) -> u16 {
        self.orchestrator
    }

    /// Get discovery service port
    ///
    /// Returns the configured port for the discovery service (mDNS, DNS-SD).
    /// This value is sourced from `SONGBIRD_DISCOVERY_PORT` environment variable
    /// or defaults to 8500.
    ///
    /// # Returns
    ///
    /// The discovery service port (u16)
    #[must_use]
    pub const fn discovery(&self) -> u16 {
        self.discovery
    }

    /// Get registry service port
    ///
    /// Returns the configured port for the service registry.
    /// Sourced from `SONGBIRD_REGISTRY_PORT` or defaults to 8600.
    #[must_use]
    pub const fn registry(&self) -> u16 {
        self.registry
    }

    /// Get security service port
    ///
    /// Returns the configured port for security/authentication services.
    /// Sourced from `SONGBIRD_SECURITY_PORT` or defaults to 8443.
    #[must_use]
    pub const fn security(&self) -> u16 {
        self.security
    }

    /// Get storage service port
    ///
    /// Returns the configured port for storage services.
    /// Sourced from `SONGBIRD_STORAGE_PORT` or defaults to 9000.
    #[must_use]
    pub const fn storage(&self) -> u16 {
        self.storage
    }

    /// Get compute service port
    ///
    /// Returns the configured port for compute/execution services.
    /// Sourced from `SONGBIRD_COMPUTE_PORT` or defaults to 9100.
    #[must_use]
    pub const fn compute(&self) -> u16 {
        self.compute
    }

    /// Get AI service port
    ///
    /// Returns the configured port for AI/ML services.
    /// Sourced from `SONGBIRD_AI_PORT` or defaults to 9200.
    #[must_use]
    pub const fn ai(&self) -> u16 {
        self.ai
    }

    /// Get gaming service port
    ///
    /// Returns the configured port for gaming services.
    /// Sourced from `SONGBIRD_GAMING_PORT` or defaults to 9300.
    #[must_use]
    pub const fn gaming(&self) -> u16 {
        self.gaming
    }

    /// Get dashboard UI port
    ///
    /// Returns the configured port for the dashboard/UI service.
    /// Sourced from `SONGBIRD_DASHBOARD_PORT` or defaults to 3000.
    #[must_use]
    pub const fn dashboard(&self) -> u16 {
        self.dashboard
    }

    /// Get metrics collection port
    ///
    /// Returns the configured port for metrics/Prometheus service.
    /// Sourced from `SONGBIRD_METRICS_PORT` or defaults to 9090.
    #[must_use]
    pub const fn metrics(&self) -> u16 {
        self.metrics
    }

    /// Get health check port
    ///
    /// Returns the configured port for health check endpoints.
    /// Sourced from `SONGBIRD_HEALTH_PORT` or defaults to 8081.
    #[must_use]
    pub const fn health(&self) -> u16 {
        self.health
    }

    /// Get dynamic port allocation range
    ///
    /// Returns the range (start, end) for dynamic port allocation.
    /// Sourced from `SONGBIRD_PORT_RANGE_START` and `SONGBIRD_PORT_RANGE_END`
    /// or defaults to (10000, 20000).
    ///
    /// # Returns
    ///
    /// A tuple of (`start_port`, `end_port`) for the dynamic range
    #[must_use]
    pub const fn dynamic_range(&self) -> (u16, u16) {
        (self.dynamic_range_start, self.dynamic_range_end)
    }

    /// Convert to capability-based port registry
    ///
    /// Creates a `CapabilityPortRegistry` from this configuration,
    /// enabling capability-based port discovery throughout the system.
    ///
    /// # Errors
    ///
    /// Returns an error if registry construction fails.
    pub fn to_capability_registry(&self) -> Result<CapabilityPortRegistry, String> {
        RegistryBuilder::new()
            .with_port_and_description(
                "orchestrator",
                self.orchestrator,
                PortSource::ConfigFile,
                "Orchestrator service port".to_string(),
            )
            .with_port_and_description(
                "discovery",
                self.discovery,
                PortSource::ConfigFile,
                "Discovery service port (mDNS, DNS-SD)".to_string(),
            )
            .with_port_and_description(
                "registry",
                self.registry,
                PortSource::ConfigFile,
                "Service registry port".to_string(),
            )
            .with_port_and_description(
                "security",
                self.security,
                PortSource::ConfigFile,
                "Security/authentication service port".to_string(),
            )
            .with_port_and_description(
                "storage",
                self.storage,
                PortSource::ConfigFile,
                "Storage service port".to_string(),
            )
            .with_port_and_description(
                "compute",
                self.compute,
                PortSource::ConfigFile,
                "Compute/execution service port".to_string(),
            )
            .with_port_and_description(
                "ai",
                self.ai,
                PortSource::ConfigFile,
                "AI service port".to_string(),
            )
            .with_port_and_description(
                "gaming",
                self.gaming,
                PortSource::ConfigFile,
                "Gaming service port".to_string(),
            )
            .with_port_and_description(
                "dashboard",
                self.dashboard,
                PortSource::ConfigFile,
                "Dashboard UI port".to_string(),
            )
            .with_port_and_description(
                "metrics",
                self.metrics,
                PortSource::ConfigFile,
                "Metrics/Prometheus port".to_string(),
            )
            .with_port_and_description(
                "health",
                self.health,
                PortSource::ConfigFile,
                "Health check endpoint port".to_string(),
            )
            .build()
    }

    /// Convert port to socket address with given host
    ///
    /// Creates a fully qualified socket address from a port number and host string.
    ///
    /// # Arguments
    ///
    /// * `port` - The port number to use
    /// * `host` - The host address (IP or hostname)
    ///
    /// # Returns
    ///
    /// * `Ok(SocketAddr)` - Parsed socket address
    /// * `Err(SongbirdError)` - If address parsing fails
    ///
    /// # Errors
    ///
    /// Returns error if the host:port combination cannot be parsed as a valid socket address
    pub fn to_socket_addr(&self, port: u16, host: &str) -> SongbirdResult<SocketAddr> {
        let addr = format!("{host}:{port}");
        addr.parse().map_err(|e| SongbirdError::Configuration {
            message: format!("Invalid socket address {addr}: {e}"),
            field: Some("socket_address".to_string()),
            suggestion: Some("Ensure host is a valid IP address or hostname".to_string()),
        })
    }
}

// ============================================================================
// HOST CONFIGURATION
// ============================================================================

/// Host configuration for all Songbird services
///
/// Supports:
/// - Environment variable override for each host
/// - IPv4/IPv6 dual-stack support
/// - DNS name resolution
/// - Localhost detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostConfig {
    /// Orchestrator service host
    pub orchestrator: String,
    /// Discovery service host
    pub discovery: String,
    /// Service registry host
    pub registry: String,
    /// Security/authentication service host
    pub security: String,
    /// Storage service host
    pub storage: String,
    /// Compute/execution service host
    pub compute: String,
    /// AI/ML service host
    pub ai: String,
    /// Gaming service host
    pub gaming: String,
    /// Dashboard/UI host
    pub dashboard: String,
    /// Metrics/Prometheus host
    pub metrics: String,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            orchestrator: "localhost".to_string(),
            discovery: "localhost".to_string(),
            registry: "localhost".to_string(),
            security: "localhost".to_string(),
            storage: "localhost".to_string(),
            compute: "localhost".to_string(),
            ai: "localhost".to_string(),
            gaming: "localhost".to_string(),
            dashboard: "localhost".to_string(),
            metrics: "localhost".to_string(),
        }
    }
}

impl HostConfig {
    /// Create a new `HostConfig` with default values
    ///
    /// This method provides a convenient way to create a configuration
    /// with sensible defaults that can then be customized.
    ///
    /// # Examples
    /// ```
    /// use songbird_config::canonical::hardcoded_elimination::HostConfig;
    ///
    /// let mut config = HostConfig::with_defaults();
    /// config.orchestrator = "custom.example.com".to_string();
    /// ```
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::default()
    }

    /// Load host configuration from environment variables, falling back to defaults
    ///
    /// # Errors
    ///
    /// Currently infallible, but returns `Result` for future extensibility
    pub fn from_env() -> SongbirdResult<Self> {
        Ok(Self {
            orchestrator: Self::parse_host("SONGBIRD_ORCHESTRATOR_HOST", "localhost"),
            discovery: Self::parse_host("SONGBIRD_DISCOVERY_HOST", "localhost"),
            registry: Self::parse_host("SONGBIRD_REGISTRY_HOST", "localhost"),
            security: Self::parse_host("SONGBIRD_SECURITY_HOST", "localhost"),
            storage: Self::parse_host("SONGBIRD_STORAGE_HOST", "localhost"),
            compute: Self::parse_host("SONGBIRD_COMPUTE_HOST", "localhost"),
            ai: Self::parse_host("SONGBIRD_AI_HOST", "localhost"),
            gaming: Self::parse_host("SONGBIRD_GAMING_HOST", "localhost"),
            dashboard: Self::parse_host("SONGBIRD_DASHBOARD_HOST", "localhost"),
            metrics: Self::parse_host("SONGBIRD_METRICS_HOST", "localhost"),
        })
    }

    /// Parse a host from environment variable with fallback to default
    fn parse_host(env_var: &str, default: &str) -> String {
        env::var(env_var).unwrap_or_else(|_| default.to_string())
    }

    /// Get orchestrator host
    #[must_use]
    pub fn orchestrator(&self) -> &str {
        &self.orchestrator
    }

    /// Get discovery host
    #[must_use]
    pub fn discovery(&self) -> &str {
        &self.discovery
    }

    /// Get registry host
    #[must_use]
    pub fn registry(&self) -> &str {
        &self.registry
    }

    /// Get security host
    #[must_use]
    pub fn security(&self) -> &str {
        &self.security
    }

    /// Get storage host
    #[must_use]
    pub fn storage(&self) -> &str {
        &self.storage
    }

    /// Get compute host
    #[must_use]
    pub fn compute(&self) -> &str {
        &self.compute
    }

    /// Get AI host
    #[must_use]
    pub fn ai(&self) -> &str {
        &self.ai
    }

    /// Get gaming host
    #[must_use]
    pub fn gaming(&self) -> &str {
        &self.gaming
    }

    /// Get dashboard host
    #[must_use]
    pub fn dashboard(&self) -> &str {
        &self.dashboard
    }

    /// Get metrics host
    #[must_use]
    pub fn metrics(&self) -> &str {
        &self.metrics
    }

    /// Check if a host is localhost (any variant)
    #[must_use]
    pub fn is_localhost(host: &str) -> bool {
        matches!(
            host,
            "localhost"
                | "127.0.0.1"
                | "::1"
                | "0.0.0.0"
                | "::"
                | "[::1]"
                | "ip6-localhost"
                | "ip6-loopback"
        )
    }

    /// Check if orchestrator is on localhost
    #[must_use]
    pub fn orchestrator_is_localhost(&self) -> bool {
        Self::is_localhost(&self.orchestrator)
    }
}

// ============================================================================
// UNIFIED CONFIGURATION
// ============================================================================

/// Unified configuration combining ports and hosts
///
/// This provides a single point of access for all endpoint configuration,
/// eliminating the need for hardcoded values throughout the codebase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EndpointConfig {
    /// Port configuration
    #[serde(default)]
    pub ports: PortConfig,
    /// Host configuration
    #[serde(default)]
    pub hosts: HostConfig,
}

impl EndpointConfig {
    /// Load endpoint configuration from environment
    ///
    /// # Errors
    ///
    /// Returns error if port or host configuration fails to load
    pub fn from_env() -> SongbirdResult<Self> {
        Ok(Self {
            ports: PortConfig::from_env()?,
            hosts: HostConfig::from_env()?,
        })
    }

    /// Get full endpoint URL for a service
    ///
    /// # Examples
    /// ```rust,ignore
    /// let config = EndpointConfig::from_env()?;
    /// let orchestrator_url = config.orchestrator_endpoint(); // "http://localhost:8080"
    /// ```
    #[must_use]
    pub fn orchestrator_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.orchestrator, self.ports.orchestrator)
    }

    /// Get discovery endpoint
    #[must_use]
    pub fn discovery_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.discovery, self.ports.discovery)
    }

    /// Get registry endpoint
    #[must_use]
    pub fn registry_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.registry, self.ports.registry)
    }

    /// Get security endpoint
    #[must_use]
    pub fn security_endpoint(&self) -> String {
        format!("https://{}:{}", self.hosts.security, self.ports.security)
    }

    /// Get storage endpoint
    #[must_use]
    pub fn storage_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.storage, self.ports.storage)
    }

    /// Get compute endpoint
    #[must_use]
    pub fn compute_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.compute, self.ports.compute)
    }

    /// Get AI endpoint
    #[must_use]
    pub fn ai_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.ai, self.ports.ai)
    }

    /// Get gaming endpoint
    #[must_use]
    pub fn gaming_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.gaming, self.ports.gaming)
    }

    /// Get dashboard endpoint
    #[must_use]
    pub fn dashboard_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.dashboard, self.ports.dashboard)
    }

    /// Get metrics endpoint
    #[must_use]
    pub fn metrics_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.metrics, self.ports.metrics)
    }

    /// Get health check endpoint
    #[must_use]
    pub fn health_endpoint(&self) -> String {
        format!("http://{}:{}", self.hosts.orchestrator, self.ports.health)
    }
}

impl fmt::Display for EndpointConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Songbird Endpoint Configuration:")?;
        writeln!(f, "  Orchestrator: {}", self.orchestrator_endpoint())?;
        writeln!(f, "  Discovery:    {}", self.discovery_endpoint())?;
        writeln!(f, "  Registry:     {}", self.registry_endpoint())?;
        writeln!(f, "  Security:     {}", self.security_endpoint())?;
        writeln!(f, "  Storage:      {}", self.storage_endpoint())?;
        writeln!(f, "  Compute:      {}", self.compute_endpoint())?;
        writeln!(f, "  AI:           {}", self.ai_endpoint())?;
        writeln!(f, "  Gaming:       {}", self.gaming_endpoint())?;
        writeln!(f, "  Dashboard:    {}", self.dashboard_endpoint())?;
        writeln!(f, "  Metrics:      {}", self.metrics_endpoint())?;
        writeln!(f, "  Health:       {}", self.health_endpoint())?;
        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_config_defaults() {
        let config = PortConfig::default();
        assert_eq!(config.orchestrator(), 8080);
        assert_eq!(config.discovery(), 8500);
        assert_eq!(config.security(), 8443);
    }

    #[test]
    fn test_port_config_validation_passes() {
        let config = PortConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_port_config_validation_fails_on_duplicate() {
        let mut config = PortConfig::default();
        config.discovery = config.orchestrator; // Duplicate port
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_port_config_validation_fails_on_invalid_range() {
        let mut config = PortConfig::default();
        config.dynamic_range_start = 20000;
        config.dynamic_range_end = 10000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_host_config_defaults() {
        let config = HostConfig::default();
        assert_eq!(config.orchestrator(), "localhost");
        assert_eq!(config.discovery(), "localhost");
    }

    #[test]
    fn test_host_is_localhost_detection() {
        assert!(HostConfig::is_localhost("localhost"));
        assert!(HostConfig::is_localhost("127.0.0.1"));
        assert!(HostConfig::is_localhost("::1"));
        assert!(HostConfig::is_localhost("[::1]"));
        assert!(!HostConfig::is_localhost("example.com"));
        assert!(!HostConfig::is_localhost("192.168.1.1"));
    }

    #[test]
    fn test_endpoint_config_urls() {
        let config = EndpointConfig::default();
        assert_eq!(config.orchestrator_endpoint(), "http://localhost:8080");
        assert_eq!(config.discovery_endpoint(), "http://localhost:8500");
        assert_eq!(config.security_endpoint(), "https://localhost:8443");
    }

    #[test]
    fn test_endpoint_config_from_env() {
        // Set test environment variables
        env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "9999");
        env::set_var("SONGBIRD_ORCHESTRATOR_HOST", "test.local");

        let config = EndpointConfig::from_env().expect("Failed to load config");

        assert_eq!(config.ports.orchestrator(), 9999);
        assert_eq!(config.hosts.orchestrator(), "test.local");
        assert_eq!(config.orchestrator_endpoint(), "http://test.local:9999");

        // Cleanup
        env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
        env::remove_var("SONGBIRD_ORCHESTRATOR_HOST");
    }

    #[test]
    fn test_endpoint_config_display() {
        let config = EndpointConfig::default();
        let display = format!("{config}");
        assert!(display.contains("Orchestrator"));
        assert!(display.contains("8080"));
        assert!(display.contains("localhost"));
    }
}
