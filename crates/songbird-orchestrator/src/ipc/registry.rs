// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Service Registry for tracking registered primals
//!
//! v3.20.0: Central registry for all local primals

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::types::{PrimalEndpoint, system_time_to_iso8601};

/// Registered service information (internal)
#[derive(Debug, Clone)]
pub struct RegisteredService {
    /// Unique service ID
    pub service_id: String,

    /// Primal name (e.g., "`BearDog`")
    pub primal_name: String,

    /// Capabilities provided
    pub capabilities: Vec<String>,

    /// Endpoint (Unix socket or URL)
    pub endpoint: String,

    /// Protocol (json-rpc, tarpc, http)
    pub protocol: String,

    /// Health check interval in seconds
    pub health_check_interval: u64,

    /// When the service was registered
    pub registered_at: SystemTime,

    /// Last health check timestamp
    pub last_health_check: SystemTime,

    /// Current health status
    pub health_status: String,
}

/// Service registry for managing primal registration and discovery
///
/// ## Architecture
///
/// ```text
/// Primal → register_service() → Songbird stores in HashMap
/// Primal → discover_by_capability() → Songbird queries HashMap
/// ```
///
/// ## Thread Safety
///
/// Uses `Arc<RwLock<HashMap>>` for concurrent access from multiple IPC handlers.
pub struct ServiceRegistry {
    /// Registered services (`service_id` → `RegisteredService`)
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new service or update existing
    ///
    /// ## Returns
    ///
    /// Service ID (auto-generated UUID if new, existing ID if update)
    pub async fn register_service(
        &self,
        primal_name: String,
        capabilities: Vec<String>,
        endpoint: String,
        protocol: String,
        health_check_interval: u64,
    ) -> Result<String> {
        let mut services = self.services.write().await;

        // Check if service already exists (by endpoint)
        let existing = services.values().find(|s| s.endpoint == endpoint).cloned();

        let service_id = if let Some(existing_service) = existing {
            // Update existing service
            info!("🔄 Updating existing service: {}", existing_service.service_id);
            existing_service.service_id
        } else {
            // Generate new service ID
            let service_id =
                format!("{}-{}", primal_name.to_lowercase(), &Uuid::new_v4().to_string()[..8]);
            info!("✅ Registering new service: {}", service_id);
            service_id
        };

        let now = SystemTime::now();

        let service = RegisteredService {
            service_id: service_id.clone(),
            primal_name,
            capabilities,
            endpoint,
            protocol,
            health_check_interval,
            registered_at: now,
            last_health_check: now,
            health_status: "unknown".to_string(), // Will be checked after registration
        };

        services.insert(service_id.clone(), service);

        debug!("   Total registered services: {}", services.len());

        Ok(service_id)
    }

    /// Discover services by capability
    ///
    /// ## Capability Matching
    ///
    /// - `"*"` matches all services
    /// - Otherwise, checks if service capabilities contain the requested capability
    ///
    /// ## Protocol Filtering
    ///
    /// If `protocol` is `Some`, only returns services with matching protocol.
    pub async fn discover_by_capability(
        &self,
        capability: &str,
        protocol: Option<&str>,
    ) -> Result<Vec<PrimalEndpoint>> {
        let services = self.services.read().await;

        debug!("🔍 Discovering services with capability: {}", capability);

        let matched: Vec<PrimalEndpoint> = services
            .values()
            .filter(|service| {
                // Filter by capability
                let capability_match = if capability == "*" {
                    true // Wildcard matches all
                } else {
                    service.capabilities.iter().any(|cap| cap == capability)
                };

                // Filter by protocol (if specified)
                let protocol_match = protocol.is_none_or(|p| service.protocol == p);

                capability_match && protocol_match
            })
            .map(|service| PrimalEndpoint {
                service_id: service.service_id.clone(),
                primal_name: service.primal_name.clone(),
                capabilities: service.capabilities.clone(),
                endpoint: service.endpoint.clone(),
                protocol: service.protocol.clone(),
                last_health_check: system_time_to_iso8601(service.last_health_check),
                health_status: service.health_status.clone(),
            })
            .collect();

        info!("   Found {} matching services", matched.len());

        Ok(matched)
    }

    /// Get health status of a specific service
    pub async fn get_service_health(&self, service_id: &str) -> Result<(String, Option<String>)> {
        let services = self.services.read().await;

        if let Some(service) = services.get(service_id) {
            Ok((service.health_status.clone(), None))
        } else {
            Ok(("unknown".to_string(), Some(format!("Service '{service_id}' not found"))))
        }
    }

    /// Update health status of a service
    ///
    /// Used by health check background tasks
    pub async fn update_health(&self, service_id: &str, status: String) -> Result<()> {
        let mut services = self.services.write().await;

        if let Some(service) = services.get_mut(service_id) {
            service.health_status = status;
            service.last_health_check = SystemTime::now();
            debug!("🩺 Updated health for {}: {}", service_id, service.health_status);
        } else {
            warn!("⚠️  Service {} not found for health update", service_id);
        }

        Ok(())
    }

    /// Get all registered services (for debugging/monitoring)
    pub async fn list_all_services(&self) -> Vec<PrimalEndpoint> {
        let services = self.services.read().await;

        services
            .values()
            .map(|service| PrimalEndpoint {
                service_id: service.service_id.clone(),
                primal_name: service.primal_name.clone(),
                capabilities: service.capabilities.clone(),
                endpoint: service.endpoint.clone(),
                protocol: service.protocol.clone(),
                last_health_check: system_time_to_iso8601(service.last_health_check),
                health_status: service.health_status.clone(),
            })
            .collect()
    }

    /// Remove a service (for cleanup)
    pub async fn unregister_service(&self, service_id: &str) -> Result<()> {
        let mut services = self.services.write().await;

        if services.remove(service_id).is_some() {
            info!("🗑️  Unregistered service: {}", service_id);
        } else {
            warn!("⚠️  Service {} not found for unregistration", service_id);
        }

        Ok(())
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_service() {
        let registry = ServiceRegistry::new();

        let service_id = registry
            .register_service(
                "BearDog".to_string(),
                vec!["encryption".to_string(), "identity".to_string()],
                "/tmp/beardog.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        assert!(service_id.starts_with("beardog-"));
    }

    #[tokio::test]
    async fn test_register_same_endpoint_twice() {
        let registry = ServiceRegistry::new();

        let service_id1 = registry
            .register_service(
                "BearDog".to_string(),
                vec!["encryption".to_string()],
                "/tmp/beardog.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        let service_id2 = registry
            .register_service(
                "BearDog".to_string(),
                vec!["encryption".to_string(), "identity".to_string()], // Updated capabilities
                "/tmp/beardog.sock".to_string(),                        // Same endpoint
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Should return same service_id (update, not new)
        assert_eq!(service_id1, service_id2);
    }

    #[tokio::test]
    async fn test_discover_by_capability() {
        let registry = ServiceRegistry::new();

        // Register two services
        registry
            .register_service(
                "BearDog".to_string(),
                vec!["encryption".to_string(), "identity".to_string()],
                "/tmp/beardog.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        registry
            .register_service(
                "ToadStool".to_string(),
                vec!["compute".to_string()],
                "/tmp/toadstool.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Discover encryption
        let encryption_primals = registry.discover_by_capability("encryption", None).await.unwrap();
        assert_eq!(encryption_primals.len(), 1);
        assert_eq!(encryption_primals[0].primal_name, "BearDog");

        // Discover compute
        let compute_primals = registry.discover_by_capability("compute", None).await.unwrap();
        assert_eq!(compute_primals.len(), 1);
        assert_eq!(compute_primals[0].primal_name, "ToadStool");

        // Discover all with wildcard
        let all_primals = registry.discover_by_capability("*", None).await.unwrap();
        assert_eq!(all_primals.len(), 2);
    }

    #[tokio::test]
    async fn test_discover_by_capability_with_protocol_filter() {
        let registry = ServiceRegistry::new();

        // Register services with different protocols
        registry
            .register_service(
                "BearDog".to_string(),
                vec!["encryption".to_string()],
                "/tmp/beardog.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        registry
            .register_service(
                "OldPrimal".to_string(),
                vec!["encryption".to_string()],
                "http://localhost:8080".to_string(),
                "http".to_string(),
                30,
            )
            .await
            .unwrap();

        // Discover encryption with json-rpc protocol
        let primals =
            registry.discover_by_capability("encryption", Some("json-rpc")).await.unwrap();
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].primal_name, "BearDog");
    }

    #[tokio::test]
    async fn test_get_service_health() {
        let registry = ServiceRegistry::new();

        let service_id = registry
            .register_service(
                "BearDog".to_string(),
                vec!["encryption".to_string()],
                "/tmp/beardog.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Initial health is "unknown"
        let (status, _) = registry.get_service_health(&service_id).await.unwrap();
        assert_eq!(status, "unknown");

        // Update health
        registry.update_health(&service_id, "healthy".to_string()).await.unwrap();

        let (status, _) = registry.get_service_health(&service_id).await.unwrap();
        assert_eq!(status, "healthy");
    }

    #[tokio::test]
    async fn test_unregister_service() {
        let registry = ServiceRegistry::new();

        let service_id = registry
            .register_service(
                "BearDog".to_string(),
                vec!["encryption".to_string()],
                "/tmp/beardog.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Service should be discoverable
        let primals = registry.discover_by_capability("encryption", None).await.unwrap();
        assert_eq!(primals.len(), 1);

        // Unregister
        registry.unregister_service(&service_id).await.unwrap();

        // Service should no longer be discoverable
        let primals = registry.discover_by_capability("encryption", None).await.unwrap();
        assert_eq!(primals.len(), 0);
    }
}
