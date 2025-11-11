//! Capability Provider Registry
//!
//! Manages external compute providers that register their capabilities with Songbird,
//! enabling dynamic task routing based on provider capabilities and health status.

pub mod types;

use crate::core::registry::types::{
    CapabilityRegistrationRequest, HealthStatus, ProviderHealth, RegisteredProvider,
    ResourceUsage,
};
use chrono::{Duration as ChronoDuration, Utc};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Central registry for managing capability providers
#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    /// Map of provider_id -> RegisteredProvider
    providers: Arc<RwLock<HashMap<String, RegisteredProvider>>>,

    /// Heartbeat configuration
    config: HeartbeatConfig,
}

/// Configuration for heartbeat monitoring
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// Expected interval between heartbeats (milliseconds)
    pub interval_ms: u64,

    /// Number of missed heartbeats before marking provider unhealthy (seconds)
    pub unhealthy_threshold_secs: i64,

    /// Number of missed heartbeats before removing provider (seconds)
    pub removal_threshold_secs: i64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 5000,            // 5 seconds between heartbeats
            unhealthy_threshold_secs: 15, // 15 seconds = 3 missed heartbeats
            removal_threshold_secs: 60,   // 60 seconds = 12 missed heartbeats
        }
    }
}

impl CapabilityRegistry {
    /// Create a new capability registry with default configuration
    pub fn new() -> Self {
        Self::with_config(HeartbeatConfig::default())
    }

    /// Create a new capability registry with custom configuration
    pub fn with_config(config: HeartbeatConfig) -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register a new capability provider
    ///
    /// # Arguments
    /// * `request` - Registration details including capabilities and endpoints
    ///
    /// # Returns
    /// * `Ok(registration_id)` - Unique registration ID on success
    /// * `Err(SongbirdError)` - If provider already registered or validation fails
    pub async fn register(
        &self,
        request: CapabilityRegistrationRequest,
    ) -> SongbirdResult<String> {
        let mut providers = self.providers.write().await;

        // Check for duplicate provider ID
        if providers.contains_key(&request.provider_id) {
            return Err(SongbirdError::Registry {
                message: format!("Provider '{}' is already registered", request.provider_id),
                service_name: Some(request.provider_id),
                operation: "register".to_string(),
            });
        }

        // Validate required fields
        if request.provider_id.is_empty() {
            return Err(SongbirdError::Validation {
                message: "provider_id cannot be empty".to_string(),
                field: Some("provider_id".to_string()),
                suggestion: Some("Provide a unique identifier for this provider".to_string()),
            });
        }

        if request.endpoint.is_empty() {
            return Err(SongbirdError::Validation {
                message: "endpoint cannot be empty".to_string(),
                field: Some("endpoint".to_string()),
                suggestion: Some("Provide the base HTTP endpoint".to_string()),
            });
        }

        if request.capabilities.is_empty() {
            return Err(SongbirdError::Validation {
                message: "capabilities list cannot be empty".to_string(),
                field: Some("capabilities".to_string()),
                suggestion: Some("Provide at least one capability".to_string()),
            });
        }

        // Generate unique registration ID
        let registration_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Create registered provider entry
        let registered_provider = RegisteredProvider {
            registration: request.clone(),
            registration_id: registration_id.clone(),
            health: ProviderHealth {
                status: HealthStatus::Healthy,
                available_capacity: request
                    .metadata
                    .get("max_concurrent_tasks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize,
                resource_usage: ResourceUsage {
                    cpu_percent: 0.0,
                    memory_percent: 0.0,
                    gpu_utilization: vec![],
                },
            },
            registered_at: now,
            last_heartbeat: now,
            active_tasks: 0,
        };

        providers.insert(request.provider_id.clone(), registered_provider);

        info!(
            provider_id = %request.provider_id,
            registration_id = %registration_id,
            capabilities = ?request.capabilities.iter().map(|c| &c.name).collect::<Vec<_>>(),
            "Provider registered successfully"
        );

        Ok(registration_id)
    }

    /// Update provider health via heartbeat
    ///
    /// # Arguments
    /// * `provider_id` - Provider identifier
    /// * `registration_id` - Registration ID for verification
    /// * `health` - Current health status and resource usage
    ///
    /// # Returns
    /// * `Ok(())` - Heartbeat processed successfully
    /// * `Err(SongbirdError)` - If provider not found or registration_id mismatch
    pub async fn update_heartbeat(
        &self,
        provider_id: &str,
        registration_id: &str,
        health: Option<ProviderHealth>,
    ) -> SongbirdResult<()> {
        let mut providers = self.providers.write().await;

        let provider = providers.get_mut(provider_id).ok_or_else(|| {
            SongbirdError::Registry {
                message: format!("Provider '{}' not found", provider_id),
                service_name: Some(provider_id.to_string()),
                operation: "heartbeat".to_string(),
            }
        })?;

        // Verify registration ID
        if provider.registration_id != registration_id {
            return Err(SongbirdError::Security(songbird_types::SecurityError {
                message: format!(
                    "Registration ID mismatch for provider '{}'",
                    provider_id
                ),
                operation: Some("heartbeat".to_string()),
                required_permission: Some(format!(
                    "Valid registration_id for provider '{}'",
                    provider_id
                )),
                context: Some("capability_provider_heartbeat".to_string()),
                remediation: Some("Use the registration_id returned during registration".to_string()),
            }));
        }

        // Update last heartbeat timestamp
        provider.last_heartbeat = Utc::now();

        // Update health status if provided
        if let Some(new_health) = health {
            provider.health = new_health;
            // Ensure status is at least Healthy if we're receiving heartbeats
            if provider.health.status == HealthStatus::Offline {
                provider.health.status = HealthStatus::Healthy;
            }
        } else {
            // If no health provided but heartbeat received, mark as Healthy
            provider.health.status = HealthStatus::Healthy;
        }

        debug!(
            provider_id = %provider_id,
            status = ?provider.health.status,
            active_tasks = provider.active_tasks,
            "Heartbeat received"
        );

        Ok(())
    }

    /// Unregister a provider
    ///
    /// # Arguments
    /// * `provider_id` - Provider identifier to remove
    ///
    /// # Returns
    /// * `Ok(())` - Provider removed successfully
    /// * `Err(SongbirdError)` - If provider not found
    pub async fn unregister(&self, provider_id: &str) -> SongbirdResult<()> {
        let mut providers = self.providers.write().await;

        providers.remove(provider_id).ok_or_else(|| {
            SongbirdError::Registry {
                message: format!("Provider '{}' not found", provider_id),
                service_name: Some(provider_id.to_string()),
                operation: "unregister".to_string(),
            }
        })?;

        info!(
            provider_id = %provider_id,
            "Provider unregistered"
        );

        Ok(())
    }

    /// Find providers that support a specific capability
    ///
    /// # Arguments
    /// * `capability` - Capability name to search for
    ///
    /// # Returns
    /// * Vector of providers that support the capability and are healthy
    pub async fn find_providers_with_capability(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<RegisteredProvider>> {
        let providers = self.providers.read().await;

        let matching_providers: Vec<RegisteredProvider> = providers
            .values()
            .filter(|p| {
                // Only include healthy or degraded providers
                matches!(
                    p.health.status,
                    HealthStatus::Healthy | HealthStatus::Degraded
                ) && p
                    .registration
                    .capabilities
                    .iter()
                    .any(|c| c.name == capability)
            })
            .cloned()
            .collect();

        debug!(
            capability = %capability,
            count = matching_providers.len(),
            "Found providers with capability"
        );

        Ok(matching_providers)
    }

    /// Get a specific provider by ID
    pub async fn get_provider(&self, provider_id: &str) -> SongbirdResult<RegisteredProvider> {
        let providers = self.providers.read().await;

        providers
            .get(provider_id)
            .cloned()
            .ok_or_else(|| SongbirdError::Registry {
                message: format!("Provider '{}' not found", provider_id),
                service_name: Some(provider_id.to_string()),
                operation: "get".to_string(),
            })
    }

    /// List all registered providers
    pub async fn list_providers(&self) -> Vec<RegisteredProvider> {
        let providers = self.providers.read().await;
        providers.values().cloned().collect()
    }

    /// Start background health monitoring task
    ///
    /// This spawns a tokio task that periodically checks provider health
    /// and removes offline providers.
    pub fn start_health_monitor(self: Arc<Self>) {
        let check_interval_ms = self.config.interval_ms;

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_millis(check_interval_ms));

            loop {
                interval.tick().await;
                if let Err(e) = self.check_provider_health().await {
                    warn!(error = %e, "Error checking provider health");
                }
            }
        });

        info!("Health monitor started");
    }

    /// Check health of all providers and update status
    async fn check_provider_health(&self) -> SongbirdResult<()> {
        let mut providers = self.providers.write().await;
        let now = Utc::now();

        let mut to_remove = Vec::new();

        for (provider_id, provider) in providers.iter_mut() {
            let elapsed = now - provider.last_heartbeat;

            // Check if provider should be marked unhealthy
            if elapsed > ChronoDuration::seconds(self.config.unhealthy_threshold_secs) {
                if provider.health.status != HealthStatus::Unhealthy
                    && provider.health.status != HealthStatus::Offline
                {
                    warn!(
                        provider_id = %provider_id,
                        elapsed_seconds = elapsed.num_seconds(),
                        "Provider unhealthy - missing heartbeats"
                    );
                    provider.health.status = HealthStatus::Unhealthy;
                }
            }

            // Check if provider should be removed
            if elapsed > ChronoDuration::seconds(self.config.removal_threshold_secs) {
                warn!(
                    provider_id = %provider_id,
                    elapsed_seconds = elapsed.num_seconds(),
                    "Provider offline - removing from registry"
                );
                provider.health.status = HealthStatus::Offline;
                to_remove.push(provider_id.clone());
            }
        }

        // Remove offline providers
        for provider_id in to_remove {
            providers.remove(&provider_id);
            info!(provider_id = %provider_id, "Removed offline provider");
        }

        Ok(())
    }

    /// Get current heartbeat configuration
    pub fn config(&self) -> &HeartbeatConfig {
        &self.config
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::registry::types::CapabilityDescriptor;

    fn create_test_registration() -> CapabilityRegistrationRequest {
        let mut metadata = HashMap::new();
        metadata.insert("max_concurrent_tasks".to_string(), serde_json::json!(10));

        CapabilityRegistrationRequest {
            provider_id: "test-provider".to_string(),
            provider_name: "Test Provider".to_string(),
            provider_type: "compute".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:9000".to_string(),
            capabilities: vec![CapabilityDescriptor {
                name: "compute_gpu".to_string(),
                description: "GPU compute".to_string(),
                metadata: HashMap::new(),
            }],
            workload_endpoint: "/execute".to_string(),
            health_endpoint: "/health".to_string(),
            metadata,
        }
    }

    #[tokio::test]
    async fn test_register_provider() {
        let registry = CapabilityRegistry::new();
        let request = create_test_registration();

        let result = registry.register(request).await;
        assert!(result.is_ok());
        let registration_id = result.unwrap();
        assert!(!registration_id.is_empty());
    }

    #[tokio::test]
    async fn test_duplicate_registration_fails() {
        let registry = CapabilityRegistry::new();
        let request = create_test_registration();

        // First registration should succeed
        assert!(registry.register(request.clone()).await.is_ok());

        // Second registration with same provider_id should fail
        let result = registry.register(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_providers_with_capability() {
        let registry = CapabilityRegistry::new();
        let request = create_test_registration();

        registry.register(request).await.unwrap();

        let providers = registry
            .find_providers_with_capability("compute_gpu")
            .await
            .unwrap();
        assert_eq!(providers.len(), 1);

        let no_providers = registry
            .find_providers_with_capability("nonexistent")
            .await
            .unwrap();
        assert_eq!(no_providers.len(), 0);
    }

    #[tokio::test]
    async fn test_heartbeat_updates() {
        let registry = CapabilityRegistry::new();
        let request = create_test_registration();

        let registration_id = registry.register(request.clone()).await.unwrap();

        // Update heartbeat
        let result = registry
            .update_heartbeat(&request.provider_id, &registration_id, None)
            .await;
        assert!(result.is_ok());

        // Wrong registration_id should fail
        let result = registry
            .update_heartbeat(&request.provider_id, "wrong-id", None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unregister_provider() {
        let registry = CapabilityRegistry::new();
        let request = create_test_registration();

        registry.register(request.clone()).await.unwrap();

        // Unregister should succeed
        let result = registry.unregister(&request.provider_id).await;
        assert!(result.is_ok());

        // Unregistering again should fail
        let result = registry.unregister(&request.provider_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_providers() {
        let registry = CapabilityRegistry::new();

        // Initially empty
        let providers = registry.list_providers().await;
        assert_eq!(providers.len(), 0);

        // Add provider
        let request = create_test_registration();
        registry.register(request).await.unwrap();

        // Should have one provider
        let providers = registry.list_providers().await;
        assert_eq!(providers.len(), 1);
    }
}

