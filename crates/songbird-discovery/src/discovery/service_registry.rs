//! # 🎼 Service Registry - Discovery Service Management
//!
//! **🚀 FOCUSED RESPONSIBILITY**
//!
//! Handles service registration, unregistration, and lifecycle management.
//! Extracted from the large songbird_discovery.rs for better maintainability.

use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::debug;

/// Service registry for managing service lifecycle
pub struct ServiceRegistry {
    /// Registered services storage
    registered_services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    /// Event broadcaster for service changes
    event_sender: broadcast::Sender<ServiceEvent>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(event_sender: broadcast::Sender<ServiceEvent>) -> Self {
        Self {
            registered_services: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    /// Register a new service
    pub async fn register(&self, service: ServiceInfo) -> SongbirdResult<()> {
        let service_id = service.service_id.clone();
        debug!(
            "🎼 Service registry: Registering service '{}' (type: {})",
            service_id, service.service_type
        );

        // Store service using service_id as key
        self.registered_services
            .write()
            .await
            .insert(service.service_id.clone(), service);
        Ok(())
    }

    /// Unregister a service
    pub async fn unregister(&self, service_id: &str) -> SongbirdResult<()> {
        let mut services = self.registered_services.write().await;
        if services.contains_key(service_id) {
            services.remove(service_id);
            Ok(())
        } else {
            Ok(()) // Idempotent operation
        }
    }

    /// Update service health status
    pub async fn update_health(&self, service_id: &str, health: crate::traits::service::ServiceStatus) -> SongbirdResult<()> {
        let mut services = self.registered_services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.status = health;
            Ok(())
        } else {
            Ok(()) // Service not found - could be already unregistered
        }
    }

    /// Get all registered services
    pub async fn get_registered_services(&self) -> HashMap<String, ServiceInfo> {
        self.registered_services.read().await.clone()
    }

    /// Discover all services (simplified version)
    pub async fn discover_all_services(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        let services = self.registered_services.read().await;
        Ok(services.values().cloned().collect())
    }

    /// Find services matching a query
    pub async fn discover(&self, query: ServiceQuery) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!(
            "🎼 Service registry: Discovering services with query: {:?}",
            query
        );

        let services = self.registered_services.read().await;
        let matching_services: Vec<ServiceInfo> = services
            .values()
            .filter(|service| self.matches_service_query(service, &query))
            .cloned()
            .collect();

        debug!(
            "🔍 Found {} services matching query",
            matching_services.len()
        );
        Ok(matching_services)
    }

    /// Query services with filtering capabilities
    pub async fn query_services(&self, query: ServiceQuery) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!("🔍 ServiceRegistry: Querying services: {:?}", query);

        let services = self.registered_services.read().await;
        let mut filtered = Vec::new();

        for service in services.values() {
            let mut matches = true;

            // Filter by name pattern
            if let Some(ref name) = query.name {
                if !service.name.contains(name) {
                    matches = false;
                }
            }

            // Filter by service type
            if let Some(ref service_type) = query.service_type {
                if service.service_type != *service_type {
                    matches = false;
                }
            }

            // Filter by tags
            if !query.tags.is_empty() {
                let has_all_tags = query.tags.iter().all(|tag| service.tags.contains(tag));
                if !has_all_tags {
                    matches = false;
                }
            }

            if matches {
                filtered.push(service.clone());
            }
        }

        Ok(filtered)
    }

    /// Check if a service matches the given query
    fn matches_service_query(&self, service: &ServiceInfo, query: &ServiceQuery) -> bool {
        // Name matching
        if let Some(ref name) = query.name {
            if !service.name.contains(name) {
                return false;
            }
        }

        // Service type matching
        if let Some(ref service_type) = query.service_type {
            if service.service_type != *service_type {
                return false;
            }
        }

        // Tags matching
        if !query.tags.is_empty() {
            let has_all_tags = query.tags.iter().all(|tag| service.tags.contains(tag));
            if !has_all_tags {
                return false;
            }
        }

        true
    }

    /// Get service count
    pub async fn service_count(&self) -> usize {
        self.registered_services.read().await.len()
    }

    /// Check if a service exists
    pub async fn exists(&self, service_id: &str) -> bool {
        self.registered_services
            .read()
            .await
            .contains_key(service_id)
    }

    /// Get a specific service by ID
    pub async fn get_service(&self, service_id: &str) -> Option<ServiceInfo> {
        self.registered_services
            .read()
            .await
            .get(service_id)
            .cloned()
    }

    /// Clear all registered services (for testing)
    #[cfg(test)]
    pub async fn clear_all_services(&self) {
        self.registered_services.write().await.clear();
    }

    /// List services with optional metadata filter
    pub async fn list_all(&self, metadata_filter: Option<&HashMap<String, String>>) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!(
            "🎼 Service registry: Listing services with metadata filter: {:?}",
            metadata_filter
        );

        let filtered: Vec<ServiceInfo> = self
            .registered_services
            .read()
            .await
            .values()
            .filter(|service| {
                if let Some(metadata) = metadata_filter {
                    metadata
                        .iter()
                        .all(|(key, value)| {
                            service.metadata.get(key)
                                .and_then(|v| v.as_str())
                                .map(|v| v == value)
                                .unwrap_or(false)
                        })
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Update service metadata for multiple services
    pub async fn update_metadata(&self, updates: HashMap<String, HashMap<String, String>>) -> SongbirdResult<()> {
        let update_count = updates.len();
        debug!(
            "🎼 Service registry: Updating metadata for {} services",
            update_count
        );

        let mut services = self.registered_services.write().await;
        let mut updated_services = Vec::new();

        for (service_id, metadata_updates) in updates {
            if let Some(service) = services.get_mut(&service_id) {
                debug!(
                    "🎼 Service registry: Updating metadata for service '{}' with {} entries",
                    service_id,
                    metadata_updates.len()
                );

                // Update the metadata by merging new values
                for (key, value) in metadata_updates {
                    service.metadata.insert(key.clone(), serde_json::Value::String(value.clone()));
                    debug!(
                        "🎼 Service registry: Updated metadata key '{}' = '{}' for service '{}'",
                        key, value, service_id
                    );
                }

                updated_services.push(service_id.clone());
            } else {
                debug!(
                    "🎼 Service registry: Service '{}' not found for metadata update",
                    service_id
                );
            }
        }

        // Broadcast metadata update events
        for service_id in updated_services {
            if let Some(service) = services.get(&service_id) {
                let _ = self
                    .event_sender
                    .send(ServiceEvent::ServiceMetadataUpdated {
                        service_id: service_id.clone(),
                        metadata: service.metadata.clone(),
                    });
            }
        }

        debug!(
            "🎼 Service registry: Metadata update completed for {} services",
            update_count
        );
        Ok(())
    }
}

impl std::fmt::Debug for ServiceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceRegistry")
            .field("service_count", &"<async>")
            .finish()
    }
}
