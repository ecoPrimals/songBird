//! # 🎼 Service Registry - Discovery Service Management
//!
//! **🚀 FOCUSED RESPONSIBILITY**
//!
//! Handles service registration, unregistration, and lifecycle management.
//! Extracted from the large songbird_discovery.rs for better maintainability.

use crate::traits::{ServiceEvent, ServiceQuery};
use songbird_errors::{SongbirdError, SongbirdResult, SongbirdResponse, evolved_success};
// use songbird_universal::  // TEMPORARILY DISABLED - {ServiceInfo, UniversalHealthStatus};
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
    pub async fn register_service(&self) -> SongbirdResult<()> {
        let service_id = service.name.clone();
        debug!(
            "🎼 Service registry: Registering service '{}' (type: {:?})",
            service_id, service.primal_type
        );

        // Store service using name as key since ServiceInfo doesn't have id field
        self.registered_services
            .write()
            .await
            .insert(service.name.clone(), service);
        Ok(SongbirdResponse::success(()))
    }

    /// Unregister a service
    pub async fn unregister_service(&self) -> SongbirdResult<()> {
        let mut services = self.registered_services.write().await;
        if services.contains_key(service_id) {
            services.remove(service_id);
            Ok(SongbirdResponse::success(()))
        } else {
            Ok(()) // Idempotent operation
        }
    }

    /// Update service health status
    pub async fn update_service_health(&self) -> SongbirdResult<()> {
        let mut services = self.registered_services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.health = health;
            Ok(SongbirdResponse::success(()))
        } else {
            Ok(()) // Service not found - could be already unregistered
        }
    }

    /// Get all registered services
    pub async fn get_registered_services(&self) -> HashMap<String, ServiceInfo> {
        self.registered_services.read().await.clone()
    }

    /// Discover all services (simplified version)
    pub async fn discover_all_services(&self) -> SongbirdResult<()> {let services = self.registered_services.read().await;
        Ok(songbird_errors::evolved_success(songbird_errors::success(
            services.values()).cloned().collect(),
        ))
    }

    /// Find services matching a query
    pub async fn discover_services(&self) -> SongbirdResult<()> {debug!(
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
        Ok(songbird_errors::evolved_success(success(matching_services)))
    }

    /// Query services with filtering capabilities  
    pub async fn query_services(&self) -> SongbirdResult<()> {debug!("🔍 ServiceRegistry: Querying services: {:?}", query);

        let services = self.registered_services.read().await;
        let mut filtered = Vec::new();

        for service in services.values() {
            let mut matches = true;

            // Filter by name pattern
            if let Some(pattern) = &query.name_pattern {
                if !service.name.contains(pattern) {
                    matches = false;
                }
            }

            // Filter by health status
            if let Some(health) = &query.health_status {
                if service.health != *health {
                    matches = false;
                }
            }

            // Filter by metadata
            for (key, value) in &query.metadata {
                if service.metadata.get(key) != Some(value) {
                    matches = false;
                    break;
                }
            }

            if matches {
                filtered.push(service.clone());
            }
        }

        Ok(songbird_errors::evolved_success(success(filtered)))
    }

    /// Check if a service matches the given query
    fn matches_service_query(&self, service: &ServiceInfo, query: &ServiceQuery) -> bool {
        // Name pattern matching
        if let Some(ref name_pattern) = query.name_pattern {
            if !service.name.contains(name_pattern) {
                return false;
            }
        }

        // Tags matching
        if !query.tags.is_empty() {
            let service_tags: Vec<&str> = service.capabilities.iter().map(|s| s.as_str()).collect();
            let has_required_tags = query
                .tags
                .iter()
                .all(|tag| service_tags.contains(&tag.as_str()));
            if !has_required_tags {
                return false;
            }
        }

        // Health status filtering
        if let Some(ref required_health) = query.health_status {
            if &service.health != required_health {
                return false;
            }
        }

        // Metadata matching
        for (key, value) in &query.metadata {
            if let Some(service_value) = service.metadata.get(key) {
                if service_value != value {
                    return false;
                }
            } else {
                return false; // Required metadata key not found
            }
        }

        true
    }

    /// Get service count
    pub async fn service_count(&self) -> usize {
        self.registered_services.read().await.len()
    }

    /// Check if a service exists
    pub async fn service_exists(&self) -> bool {
        self.registered_services
            .read()
            .await
            .contains_key(service_id)
    }

    /// Get a specific service by ID
    pub async fn get_service(&self) -> Option<ServiceInfo> {
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

    pub async fn list_services(&self) -> SongbirdResult<()> {debug!(
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
                        .all(|(key, value)| service.metadata.get(key) == Some(value))
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        Ok(songbird_errors::evolved_success(success(filtered)))
    }

    /// Update service metadata for multiple services
    pub async fn update_service_metadata(&self) -> SongbirdResult<()> {
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
                    service.metadata.insert(key.clone(), value.clone());
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
        Ok(SongbirdResponse::success(()))
    }
}

impl std::fmt::Debug for ServiceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceRegistry")
            .field("service_count", &"<async>")
            .finish()
    }
}
