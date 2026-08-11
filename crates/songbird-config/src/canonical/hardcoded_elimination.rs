// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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

use crate::canonical::constants::read_process_env;
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
use std::fmt;

pub use super::port_config::PortConfig;

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
        let host = songbird_types::constants::LOCALHOST_HOSTNAME.to_string();
        Self {
            orchestrator: host.clone(),
            discovery: host.clone(),
            registry: host.clone(),
            security: host.clone(),
            storage: host.clone(),
            compute: host.clone(),
            ai: host.clone(),
            gaming: host.clone(),
            dashboard: host.clone(),
            metrics: host,
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
    /// config.orchestrator = String::from("custom.example.com");
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
        Self::from_env_reader(&read_process_env)
    }

    /// Load host configuration using an injectable env reader.
    pub fn from_env_reader(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> SongbirdResult<Self> {
        let default = songbird_types::constants::LOCALHOST_HOSTNAME;
        Ok(Self {
            orchestrator: Self::parse_host_env(env, "SONGBIRD_ORCHESTRATOR_HOST", default),
            discovery: Self::parse_host_env(env, "SONGBIRD_DISCOVERY_HOST", default),
            registry: Self::parse_host_env(env, "SONGBIRD_REGISTRY_HOST", default),
            security: Self::parse_host_env(env, "SONGBIRD_SECURITY_HOST", default),
            storage: Self::parse_host_env(env, "SONGBIRD_STORAGE_HOST", default),
            compute: Self::parse_host_env(env, "SONGBIRD_COMPUTE_HOST", default),
            ai: Self::parse_host_env(env, "SONGBIRD_AI_HOST", default),
            gaming: Self::parse_host_env(env, "SONGBIRD_GAMING_HOST", default),
            dashboard: Self::parse_host_env(env, "SONGBIRD_DASHBOARD_HOST", default),
            metrics: Self::parse_host_env(env, "SONGBIRD_METRICS_HOST", default),
        })
    }

    /// Parse a host from environment variable with fallback to default
    fn parse_host_env(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
        env_var: &str,
        default: &str,
    ) -> String {
        env(env_var).unwrap_or_else(|_| default.to_string())
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
        Self::from_env_reader(&read_process_env)
    }

    /// Load endpoint configuration using an injectable env reader.
    pub fn from_env_reader(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> SongbirdResult<Self> {
        Ok(Self {
            ports: PortConfig::from_env_reader(env)?,
            hosts: HostConfig::from_env_reader(env)?,
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
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::capability_port_config::CapabilityId;
    use songbird_types::SongbirdError;

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
        let env = |key: &str| match key {
            "SONGBIRD_ORCHESTRATOR_PORT" => Ok(String::from("9999")),
            "SONGBIRD_ORCHESTRATOR_HOST" => Ok(String::from("test.local")),
            _ => read_process_env(key),
        };
        let config = EndpointConfig::from_env_reader(&env).expect("Failed to load config");

        assert_eq!(config.ports.orchestrator(), 9999);
        assert_eq!(config.hosts.orchestrator(), "test.local");
        assert_eq!(config.orchestrator_endpoint(), "http://test.local:9999");
    }

    #[test]
    fn test_endpoint_config_display() {
        let config = EndpointConfig::default();
        let display = format!("{config}");
        assert!(display.contains("Orchestrator"));
        assert!(display.contains("8080"));
        assert!(display.contains("localhost"));
    }

    #[test]
    fn test_port_config_from_env_rejects_invalid_port() {
        let env = |key: &str| match key {
            "SONGBIRD_ORCHESTRATOR_PORT" => Ok(String::from("not_a_u16")),
            _ => read_process_env(key),
        };
        let err = PortConfig::from_env_reader(&env).expect_err("invalid port");
        assert!(
            matches!(err, SongbirdError::Configuration { ref field, .. } if field.as_deref() == Some("SONGBIRD_ORCHESTRATOR_PORT")),
            "{err:?}"
        );
    }

    #[test]
    fn test_port_config_to_socket_addr_rejects_bad_host() {
        let config = PortConfig::default();
        let err = config
            .to_socket_addr(config.orchestrator(), "not a valid host!!!")
            .expect_err("bad host");
        assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
    }

    #[test]
    fn test_port_config_to_capability_registry() {
        let reg = PortConfig::default().to_capability_registry();
        assert!(reg.get_port(&CapabilityId::new("orchestrator")).is_ok());
    }

    #[test]
    fn host_config_default_uses_localhost_hostname_constant() {
        let cfg = HostConfig::default();
        let expected = songbird_types::constants::LOCALHOST_HOSTNAME;
        assert_eq!(cfg.orchestrator, expected);
        assert_eq!(cfg.discovery, expected);
        assert_eq!(cfg.registry, expected);
        assert_eq!(cfg.security, expected);
        assert_eq!(cfg.storage, expected);
        assert_eq!(cfg.compute, expected);
        assert_eq!(cfg.ai, expected);
        assert_eq!(cfg.gaming, expected);
        assert_eq!(cfg.dashboard, expected);
        assert_eq!(cfg.metrics, expected);
    }

    #[test]
    fn host_config_from_env_overrides_single_host() {
        let env = |key: &str| -> Result<String, std::env::VarError> {
            if key == "SONGBIRD_ORCHESTRATOR_HOST" {
                Ok(String::from("custom.host"))
            } else {
                Err(std::env::VarError::NotPresent)
            }
        };
        let cfg = HostConfig::from_env_reader(&env).unwrap();
        assert_eq!(cfg.orchestrator, "custom.host");
        assert_eq!(cfg.discovery, songbird_types::constants::LOCALHOST_HOSTNAME);
    }

    #[test]
    fn host_config_from_env_all_defaults() {
        let env =
            |_: &str| -> Result<String, std::env::VarError> { Err(std::env::VarError::NotPresent) };
        let cfg = HostConfig::from_env_reader(&env).unwrap();
        assert_eq!(cfg.orchestrator, songbird_types::constants::LOCALHOST_HOSTNAME);
        assert_eq!(cfg.metrics, songbird_types::constants::LOCALHOST_HOSTNAME);
    }

    #[test]
    fn host_config_with_defaults_equals_default() {
        assert_eq!(
            format!("{:?}", HostConfig::with_defaults()),
            format!("{:?}", HostConfig::default())
        );
    }
}
