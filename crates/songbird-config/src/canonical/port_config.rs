// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Port configuration for all Songbird services
//!
//! Environment-driven port allocation with validation, conflict detection,
//! and capability-registry bridging.

use crate::canonical::constants::read_process_env;
use crate::capability_port_config::{CapabilityPortRegistry, PortSource, RegistryBuilder};
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::SocketAddr;

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
        Self::from_env_reader(&read_process_env)
    }

    /// Load port configuration using an injectable env reader (tests avoid mutating process env).
    pub fn from_env_reader(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> SongbirdResult<Self> {
        let config = Self {
            orchestrator: Self::parse_port_env(env, "SONGBIRD_ORCHESTRATOR_PORT", 8080)?,
            discovery: Self::parse_port_env(env, "SONGBIRD_DISCOVERY_PORT", 8500)?,
            registry: Self::parse_port_env(env, "SONGBIRD_REGISTRY_PORT", 8600)?,
            security: Self::parse_port_env(env, "SONGBIRD_SECURITY_PORT", 8443)?,
            storage: Self::parse_port_env(env, "SONGBIRD_STORAGE_PORT", 9000)?,
            compute: Self::parse_port_env(env, "SONGBIRD_COMPUTE_PORT", 9100)?,
            ai: Self::parse_port_env(env, "SONGBIRD_AI_PORT", 9200)?,
            gaming: Self::parse_port_env(env, "SONGBIRD_GAMING_PORT", 9300)?,
            dashboard: Self::parse_port_env(env, "SONGBIRD_DASHBOARD_PORT", 3000)?,
            metrics: Self::parse_port_env(env, "SONGBIRD_METRICS_PORT", 9090)?,
            health: Self::parse_port_env(env, "SONGBIRD_HEALTH_PORT", 8081)?,
            dynamic_range_start: Self::parse_port_env(env, "SONGBIRD_PORT_RANGE_START", 10000)?,
            dynamic_range_end: Self::parse_port_env(env, "SONGBIRD_PORT_RANGE_END", 20000)?,
        };

        config.validate()?;

        Ok(config)
    }

    /// Parse a port from environment variable with fallback to default
    fn parse_port_env(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
        env_var: &str,
        default: u16,
    ) -> SongbirdResult<u16> {
        env(env_var).map_or(Ok(default), |val| {
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
    #[must_use]
    pub const fn orchestrator(&self) -> u16 {
        self.orchestrator
    }

    /// Get discovery service port
    #[must_use]
    pub const fn discovery(&self) -> u16 {
        self.discovery
    }

    /// Get registry service port
    #[must_use]
    pub const fn registry(&self) -> u16 {
        self.registry
    }

    /// Get security service port
    #[must_use]
    pub const fn security(&self) -> u16 {
        self.security
    }

    /// Get storage service port
    #[must_use]
    pub const fn storage(&self) -> u16 {
        self.storage
    }

    /// Get compute service port
    #[must_use]
    pub const fn compute(&self) -> u16 {
        self.compute
    }

    /// Get AI service port
    #[must_use]
    pub const fn ai(&self) -> u16 {
        self.ai
    }

    /// Get gaming service port
    #[must_use]
    pub const fn gaming(&self) -> u16 {
        self.gaming
    }

    /// Get dashboard UI port
    #[must_use]
    pub const fn dashboard(&self) -> u16 {
        self.dashboard
    }

    /// Get metrics collection port
    #[must_use]
    pub const fn metrics(&self) -> u16 {
        self.metrics
    }

    /// Get health check port
    #[must_use]
    pub const fn health(&self) -> u16 {
        self.health
    }

    /// Get dynamic port allocation range
    #[must_use]
    pub const fn dynamic_range(&self) -> (u16, u16) {
        (self.dynamic_range_start, self.dynamic_range_end)
    }

    /// Convert to capability-based port registry
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    fn empty_env(_key: &str) -> Result<String, std::env::VarError> {
        Err(std::env::VarError::NotPresent)
    }

    fn env_with(overrides: &[(&str, &str)]) -> impl Fn(&str) -> Result<String, std::env::VarError> {
        let map: HashMap<String, String> =
            overrides.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect();
        move |key: &str| map.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn default_config_has_expected_ports() {
        let config = PortConfig::default();
        assert_eq!(config.orchestrator(), 8080);
        assert_eq!(config.discovery(), 8500);
        assert_eq!(config.registry(), 8600);
        assert_eq!(config.security(), 8443);
        assert_eq!(config.storage(), 9000);
        assert_eq!(config.compute(), 9100);
        assert_eq!(config.ai(), 9200);
        assert_eq!(config.gaming(), 9300);
        assert_eq!(config.dashboard(), 3000);
        assert_eq!(config.metrics(), 9090);
        assert_eq!(config.health(), 8081);
        assert_eq!(config.dynamic_range(), (10000, 20000));
    }

    #[test]
    fn default_config_validates_cleanly() {
        let config = PortConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn from_env_reader_uses_defaults_when_no_env() {
        let config = PortConfig::from_env_reader(&empty_env).unwrap();
        assert_eq!(config, PortConfig::default());
    }

    #[test]
    fn from_env_reader_overrides_individual_ports() {
        let env = env_with(&[("SONGBIRD_ORCHESTRATOR_PORT", "4000"), ("SONGBIRD_AI_PORT", "5555")]);
        let config = PortConfig::from_env_reader(&env).unwrap();
        assert_eq!(config.orchestrator(), 4000);
        assert_eq!(config.ai(), 5555);
        assert_eq!(config.discovery(), 8500);
    }

    #[test]
    fn from_env_reader_rejects_non_numeric_port() {
        let env = env_with(&[("SONGBIRD_ORCHESTRATOR_PORT", "not_a_port")]);
        let err = PortConfig::from_env_reader(&env).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Invalid port"), "got: {msg}");
    }

    #[test]
    fn from_env_reader_rejects_port_overflow() {
        let env = env_with(&[("SONGBIRD_STORAGE_PORT", "99999")]);
        let err = PortConfig::from_env_reader(&env).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Invalid port"), "got: {msg}");
    }

    #[test]
    fn validate_detects_duplicate_ports() {
        let mut config = PortConfig::default();
        config.orchestrator = 8443;
        config.security = 8443;
        let err = config.validate().unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Port conflict"), "got: {msg}");
        assert!(msg.contains("8443"), "got: {msg}");
    }

    #[test]
    fn validate_detects_invalid_dynamic_range() {
        let mut config = PortConfig::default();
        config.dynamic_range_start = 20000;
        config.dynamic_range_end = 10000;
        let err = config.validate().unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Invalid port range"), "got: {msg}");
    }

    #[test]
    fn validate_detects_equal_dynamic_range() {
        let mut config = PortConfig::default();
        config.dynamic_range_start = 15000;
        config.dynamic_range_end = 15000;
        let err = config.validate().unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Invalid port range"), "got: {msg}");
    }

    #[test]
    fn to_socket_addr_produces_valid_address() {
        let config = PortConfig::default();
        let addr = config.to_socket_addr(8080, "127.0.0.1").unwrap();
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[test]
    fn to_socket_addr_supports_ipv6() {
        let config = PortConfig::default();
        let addr = config.to_socket_addr(9090, "[::1]").unwrap();
        assert_eq!(addr.port(), 9090);
    }

    #[test]
    fn to_socket_addr_rejects_invalid_host() {
        let config = PortConfig::default();
        let err = config.to_socket_addr(8080, "not-an-ip").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Invalid socket address"), "got: {msg}");
    }

    #[test]
    fn to_capability_registry_builds_all_services() {
        use crate::capability_port_config::CapabilityId;
        let config = PortConfig::default();
        let registry = config.to_capability_registry().unwrap();
        assert!(registry.get_port(&CapabilityId::new("orchestrator")).is_ok());
        assert!(registry.get_port(&CapabilityId::new("discovery")).is_ok());
        assert!(registry.get_port(&CapabilityId::new("security")).is_ok());
        assert!(registry.get_port(&CapabilityId::new("health")).is_ok());
    }

    #[test]
    fn serde_roundtrip() {
        let config = PortConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let back: PortConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, back);
    }

    #[test]
    fn env_overrides_for_all_ports() {
        let env = env_with(&[
            ("SONGBIRD_ORCHESTRATOR_PORT", "1001"),
            ("SONGBIRD_DISCOVERY_PORT", "1002"),
            ("SONGBIRD_REGISTRY_PORT", "1003"),
            ("SONGBIRD_SECURITY_PORT", "1004"),
            ("SONGBIRD_STORAGE_PORT", "1005"),
            ("SONGBIRD_COMPUTE_PORT", "1006"),
            ("SONGBIRD_AI_PORT", "1007"),
            ("SONGBIRD_GAMING_PORT", "1008"),
            ("SONGBIRD_DASHBOARD_PORT", "1009"),
            ("SONGBIRD_METRICS_PORT", "1010"),
            ("SONGBIRD_HEALTH_PORT", "1011"),
            ("SONGBIRD_PORT_RANGE_START", "2000"),
            ("SONGBIRD_PORT_RANGE_END", "3000"),
        ]);
        let config = PortConfig::from_env_reader(&env).unwrap();
        assert_eq!(config.orchestrator(), 1001);
        assert_eq!(config.discovery(), 1002);
        assert_eq!(config.registry(), 1003);
        assert_eq!(config.security(), 1004);
        assert_eq!(config.storage(), 1005);
        assert_eq!(config.compute(), 1006);
        assert_eq!(config.ai(), 1007);
        assert_eq!(config.gaming(), 1008);
        assert_eq!(config.dashboard(), 1009);
        assert_eq!(config.metrics(), 1010);
        assert_eq!(config.health(), 1011);
        assert_eq!(config.dynamic_range(), (2000, 3000));
    }
}
