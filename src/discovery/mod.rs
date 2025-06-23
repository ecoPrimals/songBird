//! Service Discovery Module
//!
//! Implementation of service discovery for various backends

// Core types and data structures
pub mod types;

// Configuration management
pub mod config;

// Resource detection and monitoring
pub mod resources;
pub mod monitoring;

// Network operations
pub mod network;

// Federation management (placeholder for future expansion)
pub mod federation {
    // Federation-specific logic could go here
    // For now, it's handled in the main discovery service
}

// Trust verification (placeholder for future expansion)
pub mod trust {
    // Trust verification logic could go here
    // For now, it's handled in the main discovery service
}

// Certificate validation (placeholder for future expansion)
pub mod certificate {
    // Certificate validation logic could go here
    // For now, it's handled in the main discovery service
}

// Main discovery service implementation
pub mod songbird_discovery;

// Re-export the main discovery service
pub use songbird_discovery::SongbirdDiscovery;

// Re-export commonly used types
pub use types::{
    NodeId, NodeInfo, LocalNode, NodeType, ComputeResources, ResourceQuery,
    FederationStats, FederationHealth, NetworkTopology, TrustLevel,
    ResourceUsage, FederationMessage, NetworkMeasurement, ResourceUpdate,
    DatasetInfo, StorageInfo, InteractionResult, NetworkPartition,
};

// Re-export configuration types
pub use config::{
    SongbirdDiscoveryConfig, NetworkConfig, MonitoringConfig, TrustConfig,
    NetworkTimingConfig, TrustThresholds, InteractionPenalties,
};

// Re-export utility structs
pub use resources::ResourceDetector;
pub use monitoring::ResourceMonitor;
pub use network::NetworkManager;

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use futures_util::Stream;

use crate::errors::Result;
pub use crate::traits::discovery::*;
use crate::traits::service::ServiceInfo;

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
        self.services.write().insert(service.id.clone(), service);
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

    async fn update_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> Result<()> {
        if let Some(service) = self.services.write().get_mut(service_id) {
            // Update health status in metadata
            service.metadata.insert("health_status".to_string(), format!("{:?}", health).into());
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
        Ok(self.services.read().contains_key(service_id))
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
            if service.id != *service_id {
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
            if !service.name.to_lowercase().contains(&name_filter.to_lowercase()) {
                return false;
            }
        }

        // Check tag filters - service must have all required tags
        for required_tag in &query.tags {
            if !service.tags.contains_key(required_tag) {
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
        if requirement.starts_with(">=") {
            let req_version = &requirement[2..];
            service_version >= req_version
        } else if requirement.starts_with("<=") {
            let req_version = &requirement[2..];
            service_version <= req_version
        } else if requirement.starts_with(">") {
            let req_version = &requirement[1..];
            service_version > req_version
        } else if requirement.starts_with("<") {
            let req_version = &requirement[1..];
            service_version < req_version
        } else if requirement.starts_with("=") {
            let req_version = &requirement[1..];
            service_version == req_version
        } else {
            // Exact match
            service_version == requirement
        }
    }
}
