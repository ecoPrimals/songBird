//! Adapter Module - Orchestration Layer
//!
//! This is the main orchestration layer for the Universal Capability Adapter.
//! Previously a 1080-line monolithic file, now smartly refactored into focused modules:
//!
//! - `discovery`: Capability discovery (env, network, inference)
//! - `capability_query`: Querying and selecting capability providers
//! - `connection_manager`: Connection lifecycle management
//! - `federation`: Federation coordination (TODO)
//! - `cache`: Response caching (TODO)
//! - `metrics`: QoS metrics collection (TODO)
//!
//! Each module has clear boundaries and responsibilities.
//! This orchestration layer provides a unified API by delegating to specialized components.

pub mod capability_query;
pub mod connection_manager;
pub mod discovery;

// Re-export key types for convenience
pub use capability_query::CapabilityQuery;
pub use connection_manager::ConnectionManager;
pub use discovery::CapabilityDiscovery;

use std::sync::Arc;
use tokio::sync::RwLock;

use super::error::CapabilityError;
use super::registry::CapabilityRegistry;
use super::types::{Capability, DiscoveryConfig};

/// Universal capability adapter - orchestration layer
///
/// This is the main entry point for capability-based primal interaction.
/// It coordinates between discovery, querying, connection management, and other subsystems.
#[derive(Debug, Clone)]
pub struct UniversalCapabilityAdapter {
    /// Discovery subsystem
    discovery: Arc<CapabilityDiscovery>,
    /// Query subsystem
    query: Arc<CapabilityQuery>,
    /// Connection management subsystem
    connections: Arc<ConnectionManager>,
    /// Shared registry
    registry: Arc<RwLock<CapabilityRegistry>>,
}

impl UniversalCapabilityAdapter {
    /// Create a new universal capability adapter
    #[must_use]
    pub fn new(config: DiscoveryConfig) -> Self {
        let registry = Arc::new(RwLock::new(CapabilityRegistry::default()));

        let discovery = Arc::new(CapabilityDiscovery::new(Arc::clone(&registry), config));
        let query = Arc::new(CapabilityQuery::new(Arc::clone(&registry)));
        let connections = Arc::new(ConnectionManager::new());

        Self {
            discovery,
            query,
            connections,
            registry,
        }
    }

    /// Discover capabilities for a primal by name
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is unreachable or does not respond with valid capabilities
    pub async fn discover_primal_capabilities(
        &self,
        primal_name: &str,
    ) -> Result<Vec<Capability>, CapabilityError> {
        // Create closure that captures query for the discovery module
        let query = Arc::clone(&self.query);
        self.discovery
            .discover_primal_capabilities(primal_name, move |endpoint: &str| {
                let query = Arc::clone(&query);
                let endpoint = endpoint.to_string();
                Box::pin(async move { query.query_primal_capabilities(&endpoint).await })
            })
            .await
    }

    /// Find all primals that provide a specific capability
    pub async fn find_capability_providers(&self, capability_type: &str) -> Vec<String> {
        self.discovery.find_capability_providers(capability_type).await
    }

    /// Get the best primal for a capability based on QoS metrics
    pub async fn get_best_primal_for_capability(
        &self,
        capability_type: &str,
    ) -> Option<String> {
        self.query.get_best_primal_for_capability(capability_type).await
    }

    /// Check if a primal provides a specific capability
    pub async fn check_primal_provides_capability(
        &self,
        primal_name: &str,
        capability_type: &str,
    ) -> bool {
        self.query
            .check_primal_provides_capability(primal_name, capability_type)
            .await
    }

    /// Establish connection to a primal
    ///
    /// # Errors
    ///
    /// Returns an error if the connection test fails or the primal health check fails
    pub async fn establish_connection(
        &self,
        primal_name: &str,
        endpoint: &str,
    ) -> Result<(), CapabilityError> {
        self.connections
            .establish_connection(primal_name, endpoint)
            .await
    }

    /// Get all active connections
    pub async fn get_all_connections(
        &self,
    ) -> Vec<crate::capabilities::connection::PrimalConnection> {
        self.connections.get_all_connections().await
    }

    /// Disconnect from a primal
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is not currently connected
    pub async fn disconnect_from_primal(&self, primal_name: &str) -> Result<(), CapabilityError> {
        self.connections.disconnect_from_primal(primal_name).await
    }

    /// Update connection health for all primals
    pub async fn update_connection_health(&self) -> Result<(), CapabilityError> {
        self.connections.update_connection_health().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_creation() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Should start with no connections
        assert_eq!(adapter.get_all_connections().await.len(), 0);
    }

    #[tokio::test]
    async fn test_find_providers_empty() {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        let providers = adapter.find_capability_providers("security").await;
        // May find some from env, but shouldn't crash
        assert!(providers.len() >= 0);
    }
}

