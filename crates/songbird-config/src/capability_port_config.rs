// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-Based Port Configuration
//!
//! This module provides a dynamic, capability-based approach to port configuration
//! that eliminates hardcoding and enables runtime discovery.
//!
//! ## Design Philosophy
//!
//! - **Zero Hardcoding**: No fixed port numbers in production code
//! - **Capability-Based**: Services discover ports based on capabilities
//! - **Runtime Discovery**: Ports are resolved at runtime via configuration or discovery
//! - **Test Isolation**: Tests can use ephemeral ports without conflicts
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   Capability Port Registry                  │
//! │                                                             │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
//! │  │ Discovery   │  │ Compute     │  │ Storage     │       │
//! │  │ Port: 8080  │  │ Port: 8081  │  │ Port: 8082  │       │
//! │  └─────────────┘  └─────────────┘  └─────────────┘       │
//! │                                                             │
//! │  Sources: Config File → Env Vars → Discovery → Ephemeral  │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, RwLock};

/// Capability identifier for port-based services
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(String);

impl CapabilityId {
    /// Create a new capability ID
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the capability ID as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for CapabilityId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for CapabilityId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Port configuration for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// The port number
    pub port: u16,
    /// Source of the port configuration
    pub source: PortSource,
    /// Optional description
    pub description: Option<String>,
}

/// Source of port configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortSource {
    /// From configuration file
    ConfigFile,
    /// From environment variable
    Environment,
    /// From runtime discovery
    Discovery,
    /// Ephemeral port (OS-assigned)
    Ephemeral,
    /// Default fallback
    Default,
}

/// Capability-based port registry
///
/// This registry maintains a mapping from capabilities to ports,
/// enabling dynamic port resolution without hardcoding.
#[derive(Debug, Clone)]
pub struct CapabilityPortRegistry {
    ports: Arc<RwLock<HashMap<CapabilityId, PortConfig>>>,
}

impl Default for CapabilityPortRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityPortRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            ports: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a port for a capability
    ///
    /// # Errors
    ///
    /// Returns an error if the registry lock is poisoned.
    pub fn register(
        &self,
        capability: CapabilityId,
        port: u16,
        source: PortSource,
        description: Option<String>,
    ) -> Result<(), String> {
        self.ports.write().map_err(|e| format!("Failed to acquire write lock: {e}"))?.insert(
            capability,
            PortConfig {
                port,
                source,
                description,
            },
        );

        Ok(())
    }

    /// Get the port for a capability
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The capability is not registered
    /// - The registry lock is poisoned
    pub fn get_port(&self, capability: &CapabilityId) -> Result<u16, String> {
        let ports = self.ports.read().map_err(|e| format!("Failed to acquire read lock: {e}"))?;

        ports
            .get(capability)
            .map(|config| config.port)
            .ok_or_else(|| format!("Capability '{}' not registered", capability.as_str()))
    }

    /// Get the full port configuration for a capability
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The capability is not registered
    /// - The registry lock is poisoned
    pub fn get_config(&self, capability: &CapabilityId) -> Result<PortConfig, String> {
        let ports = self.ports.read().map_err(|e| format!("Failed to acquire read lock: {e}"))?;

        ports
            .get(capability)
            .cloned()
            .ok_or_else(|| format!("Capability '{}' not registered", capability.as_str()))
    }

    /// Register a capability with an ephemeral (OS-assigned) port
    ///
    /// This is useful for testing and dynamic service allocation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot bind to an ephemeral port
    /// - The registry lock is poisoned
    pub fn register_ephemeral(
        &self,
        capability: CapabilityId,
        description: Option<String>,
    ) -> Result<u16, String> {
        // Bind to port 0 to get an OS-assigned ephemeral port
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("Failed to bind ephemeral port: {e}"))?;

        let port =
            listener.local_addr().map_err(|e| format!("Failed to get local address: {e}"))?.port();

        // Drop the listener to free the port
        drop(listener);

        self.register(capability, port, PortSource::Ephemeral, description)?;

        Ok(port)
    }

    /// Check if a capability is registered
    ///
    /// # Errors
    ///
    /// Returns an error if the registry lock is poisoned.
    pub fn has_capability(&self, capability: &CapabilityId) -> Result<bool, String> {
        let ports = self.ports.read().map_err(|e| format!("Failed to acquire read lock: {e}"))?;

        Ok(ports.contains_key(capability))
    }

    /// List all registered capabilities
    ///
    /// # Errors
    ///
    /// Returns an error if the registry lock is poisoned.
    pub fn list_capabilities(&self) -> Result<Vec<CapabilityId>, String> {
        let ports = self.ports.read().map_err(|e| format!("Failed to acquire read lock: {e}"))?;

        Ok(ports.keys().cloned().collect())
    }

    /// Clear all registrations
    ///
    /// # Errors
    ///
    /// Returns an error if the registry lock is poisoned.
    pub fn clear(&self) -> Result<(), String> {
        self.ports.write().map_err(|e| format!("Failed to acquire write lock: {e}"))?.clear();
        Ok(())
    }
}

/// Builder for creating a pre-configured registry
#[derive(Debug, Default)]
pub struct RegistryBuilder {
    ports: HashMap<CapabilityId, (u16, PortSource, Option<String>)>,
}

impl RegistryBuilder {
    /// Create a new registry builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a capability with a specific port
    #[must_use]
    pub fn with_port(
        mut self,
        capability: impl Into<CapabilityId>,
        port: u16,
        source: PortSource,
    ) -> Self {
        self.ports.insert(capability.into(), (port, source, None));
        self
    }

    /// Add a capability with a specific port and description
    #[must_use]
    pub fn with_port_and_description(
        mut self,
        capability: impl Into<CapabilityId>,
        port: u16,
        source: PortSource,
        description: String,
    ) -> Self {
        self.ports.insert(capability.into(), (port, source, Some(description)));
        self
    }

    /// Build the registry
    ///
    /// # Errors
    ///
    /// Returns an error if registration fails.
    pub fn build(self) -> Result<CapabilityPortRegistry, String> {
        let registry = CapabilityPortRegistry::new();

        for (capability, (port, source, description)) in self.ports {
            registry.register(capability, port, source, description)?;
        }

        Ok(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_registration() {
        let registry = CapabilityPortRegistry::new();
        let cap = CapabilityId::new("test.service");

        registry
            .register(cap.clone(), 8080, PortSource::ConfigFile, None)
            .expect("registration should succeed");

        assert_eq!(registry.get_port(&cap).expect("port should exist"), 8080);
    }

    #[test]
    fn test_ephemeral_port() {
        let registry = CapabilityPortRegistry::new();
        let cap = CapabilityId::new("test.ephemeral");

        let port = registry
            .register_ephemeral(cap.clone(), Some("Test service".to_string()))
            .expect("ephemeral registration should succeed");

        assert!(port > 0);
        assert_eq!(registry.get_port(&cap).expect("port should exist"), port);

        let config = registry.get_config(&cap).expect("config should exist");
        assert_eq!(config.source, PortSource::Ephemeral);
        assert_eq!(config.description, Some("Test service".to_string()));
    }

    #[test]
    fn test_builder() {
        let registry = RegistryBuilder::new()
            .with_port("service.a", 8080, PortSource::ConfigFile)
            .with_port("service.b", 8081, PortSource::Environment)
            .with_port_and_description(
                "service.c",
                8082,
                PortSource::Discovery,
                "Discovered service".to_string(),
            )
            .build()
            .expect("build should succeed");

        assert_eq!(
            registry.get_port(&CapabilityId::new("service.a")).expect("port should exist"),
            8080
        );
        assert_eq!(
            registry.get_port(&CapabilityId::new("service.b")).expect("port should exist"),
            8081
        );
        assert_eq!(
            registry.get_port(&CapabilityId::new("service.c")).expect("port should exist"),
            8082
        );
    }

    #[test]
    fn test_list_capabilities() {
        let registry = RegistryBuilder::new()
            .with_port("service.a", 8080, PortSource::ConfigFile)
            .with_port("service.b", 8081, PortSource::Environment)
            .build()
            .expect("build should succeed");

        let caps = registry.list_capabilities().expect("list should succeed");
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&CapabilityId::new("service.a")));
        assert!(caps.contains(&CapabilityId::new("service.b")));
    }

    #[test]
    fn test_clear() {
        let registry = RegistryBuilder::new()
            .with_port("service.a", 8080, PortSource::ConfigFile)
            .build()
            .expect("build should succeed");

        assert!(
            registry.has_capability(&CapabilityId::new("service.a")).expect("check should succeed")
        );

        registry.clear().expect("clear should succeed");

        assert!(
            !registry
                .has_capability(&CapabilityId::new("service.a"))
                .expect("check should succeed")
        );
    }
}
