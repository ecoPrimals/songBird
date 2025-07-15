//! Service Discovery Module
//!
//! Implementation of service discovery for various backends

use async_trait::async_trait;
use futures_util::Stream;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::errors::Result;
use crate::traits::discovery::*;
use crate::traits::service::ServiceInfo;

// Core types and data structures
pub mod types;
// Configuration management
pub mod config;
// Resource detection and monitoring
pub mod monitoring;
pub mod resources;
// Network operations
pub mod network;

// DISCOVERY ARCHITECTURE NOTE:
// =========================
// Discovery services are now handled through external API integrations:
// - Federation discovery: Managed by crates/songbird-federation
// - Trust verification: Handled by crates/songbird-security via BearDog integration
// - Certificate validation: Managed by crates/songbird-security
// - Service discovery: Supported via songbird-discovery crate with multiple backends
//
// Local discovery modules focus on resource detection and network topology mapping.
// All security-related discovery operations are delegated to the appropriate
// security and federation modules with proper API boundaries.

// Main discovery service implementation
pub mod songbird_discovery;

// Re-export the main discovery service
pub use songbird_discovery::SongbirdDiscovery;

// Re-export commonly used types
pub use types::{
    ComputeResources, DatasetInfo, FederationHealth, FederationMessage, FederationStats,
    InteractionResult, LocalNode, NetworkMeasurement, NetworkPartition, NetworkTopology, NodeId,
    NodeInfo, NodeType, ResourceQuery, ResourceUpdate, ResourceUsage, StorageInfo, TrustLevel,
};

// Re-export configuration types
pub use config::{
    InteractionPenalties, MonitoringConfig, NetworkConfig, NetworkTimingConfig,
    SongbirdDiscoveryConfig, TrustConfig, TrustThresholds,
};

// Re-export utility structs
pub use monitoring::ResourceMonitor;
pub use network::NetworkManager;
pub use resources::ResourceDetector;

/// Static service discovery implementation for development and testing
pub struct StaticServiceDiscovery {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
}

impl StaticServiceDiscovery {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for StaticServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ServiceDiscovery for StaticServiceDiscovery {
    async fn register(&self, service: ServiceInfo) -> Result<()> {
        self.services
            .write()
            .insert(service.service_id.clone(), service);
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        self.services.write().remove(service_id);
        Ok(())
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        let services = self.services.read();
        let mut results = Vec::new();

        for service in services.values() {
            if self.service_matches_query(service, &query) {
                results.push(service.clone());
            }
        }

        Ok(results)
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        // Return empty stream for static discovery
        Ok(Box::pin(futures_util::stream::empty()))
    }

    async fn update_health(&self, service_id: &str, health: ServiceHealthStatus) -> Result<()> {
        if let Some(service) = self.services.write().get_mut(service_id) {
            // Update health status in metadata
            service
                .metadata
                .insert("health_status".to_string(), format!("{health:?}").into());
        }
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        Ok(self.services.read().values().cloned().collect())
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        Ok(self.services.read().contains_key(service_id))
    }

    async fn is_registered(&self, service_id: &str) -> Result<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        if let Some(service) = self.services.write().get_mut(service_id) {
            // Convert String values to serde_json::Value
            for (key, value) in metadata {
                service.metadata.insert(key, value.into());
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl StaticServiceDiscovery {
    /// Check if a service matches the query criteria
    fn service_matches_query(&self, service: &ServiceInfo, query: &ServiceQuery) -> bool {
        // Check service ID filter (exact match)
        if let Some(ref service_id) = query.service_id {
            if service.service_id != *service_id {
                return false;
            }
        }

        // Check service type filter
        if let Some(ref service_type) = query.service_type {
            if service.service_type != *service_type {
                return false;
            }
        }

        // Check name filter (case-insensitive substring match)
        if let Some(ref name_filter) = query.name {
            if !service
                .name
                .to_lowercase()
                .contains(&name_filter.to_lowercase())
            {
                return false;
            }
        }

        // Check tag filters - service must have all required tags
        for required_tag in &query.tags {
            if !service.tags.contains(required_tag) {
                return false;
            }
        }

        // Check version requirements (basic comparison)
        if let Some(ref version_req) = query.version {
            if !self.version_matches_requirement(&service.version, version_req) {
                return false;
            }
        }

        // Check metadata filters
        for (key, expected_value) in &query.metadata {
            if let Some(service_value) = service.metadata.get(key) {
                if service_value != expected_value {
                    return false;
                }
            } else {
                return false; // Required metadata not found
            }
        }

        true
    }

    /// Check if a service version matches a requirement
    fn version_matches_requirement(&self, service_version: &str, requirement: &str) -> bool {
        // Simplified version matching
        if let Some(req_version) = requirement.strip_prefix(">=") {
            service_version >= req_version
        } else if let Some(req_version) = requirement.strip_prefix("<=") {
            service_version <= req_version
        } else if let Some(req_version) = requirement.strip_prefix(">") {
            service_version > req_version
        } else if let Some(req_version) = requirement.strip_prefix("<") {
            service_version < req_version
        } else if let Some(req_version) = requirement.strip_prefix("=") {
            service_version == req_version
        } else {
            // Exact match
            service_version == requirement
        }
    }
}
