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
