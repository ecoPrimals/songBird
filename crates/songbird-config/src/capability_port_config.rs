// SPDX-License-Identifier: AGPL-3.0-or-later
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
use songbird_types::{SongbirdError, SongbirdResult};
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
    pub fn register(
        &self,
        capability: CapabilityId,
        port: u16,
        source: PortSource,
        description: Option<String>,
    ) {
        self.ports
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                capability,
                PortConfig {
                    port,
                    source,
                    description,
                },
            );
    }

    /// Get the port for a capability
    ///
    /// # Errors
    ///
    /// Returns an error if the capability is not registered.
    pub fn get_port(&self, capability: &CapabilityId) -> SongbirdResult<u16> {
        let ports = self.ports.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        ports
            .get(capability)
            .map(|config| config.port)
            .ok_or_else(|| SongbirdError::Configuration {
                message: format!("Capability '{}' not registered", capability.as_str()),
                field: Some(String::from("capability")),
                suggestion: Some(String::from("Register the capability before querying its port")),
            })
    }

    /// Get the full port configuration for a capability
    ///
    /// # Errors
    ///
    /// Returns an error if the capability is not registered.
    pub fn get_config(&self, capability: &CapabilityId) -> SongbirdResult<PortConfig> {
        let ports = self.ports.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        ports.get(capability).cloned().ok_or_else(|| SongbirdError::Configuration {
            message: format!("Capability '{}' not registered", capability.as_str()),
            field: Some(String::from("capability")),
            suggestion: Some(String::from("Register the capability before querying its config")),
        })
    }

    /// Register a capability with an ephemeral (OS-assigned) port
    ///
    /// This is useful for testing and dynamic service allocation.
    ///
    /// # Errors
    ///
    /// Returns an error if the OS cannot bind an ephemeral port.
    pub fn register_ephemeral(
        &self,
        capability: CapabilityId,
        description: Option<String>,
    ) -> SongbirdResult<u16> {
        let listener = TcpListener::bind(songbird_types::defaults::ports::EPHEMERAL_BIND_ADDR)
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to bind ephemeral port: {e}"),
                field: None,
                suggestion: Some(String::from("Check that port range is available")),
            })?;
        let port = listener
            .local_addr()
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to get local address: {e}"),
                field: None,
                suggestion: None,
            })?
            .port();
        drop(listener);
        self.register(capability, port, PortSource::Ephemeral, description);
        Ok(port)
    }

    /// Check if a capability is registered
    #[must_use]
    pub fn has_capability(&self, capability: &CapabilityId) -> bool {
        let ports = self.ports.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        ports.contains_key(capability)
    }

    /// List all registered capabilities
    #[must_use]
    pub fn list_capabilities(&self) -> Vec<CapabilityId> {
        let ports = self.ports.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        ports.keys().cloned().collect()
    }

    /// Clear all registrations
    pub fn clear(&self) {
        self.ports
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
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
    #[must_use]
    pub fn build(self) -> CapabilityPortRegistry {
        let registry = CapabilityPortRegistry::new();

        for (capability, (port, source, description)) in self.ports {
            registry.register(capability, port, source, description);
        }

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_registration() {
        let registry = CapabilityPortRegistry::new();
        let cap = CapabilityId::new("test.service");

        registry.register(cap.clone(), 8080, PortSource::ConfigFile, None);

        assert_eq!(registry.get_port(&cap).expect("port should exist"), 8080);
    }

    #[test]
    fn test_ephemeral_port() {
        let registry = CapabilityPortRegistry::new();
        let cap = CapabilityId::new("test.ephemeral");

        let port = registry
            .register_ephemeral(cap.clone(), Some(String::from("Test service")))
            .expect("ephemeral registration should succeed");

        assert!(port > 0);
        assert_eq!(registry.get_port(&cap).expect("port should exist"), port);

        let config = registry.get_config(&cap).expect("config should exist");
        assert_eq!(config.source, PortSource::Ephemeral);
        assert_eq!(config.description, Some(String::from("Test service")));
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
                String::from("Discovered service"),
            )
            .build();

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
            .build();

        let caps = registry.list_capabilities();
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&CapabilityId::new("service.a")));
        assert!(caps.contains(&CapabilityId::new("service.b")));
    }

    #[test]
    fn test_clear() {
        let registry = RegistryBuilder::new()
            .with_port("service.a", 8080, PortSource::ConfigFile)
            .build();

        assert!(registry.has_capability(&CapabilityId::new("service.a")));

        registry.clear();

        assert!(!registry.has_capability(&CapabilityId::new("service.a")));
    }
}
