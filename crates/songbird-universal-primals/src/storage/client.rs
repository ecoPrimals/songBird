//! Universal storage client - capability-based storage interface

use super::{cache::StorageCache, config::UniversalStorageConfig, events::UniversalStorageEvent, stats::StorageStats, types::*};
use crate::universal_adapter::UniversalPrimalAdapter;
use base64::{Engine as _, engine::general_purpose};

use songbird_errors::{SongbirdError, SongbirdResult, success};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn};

/// Universal storage client - completely capability-based
pub struct UniversalStorageClient {
    /// Universal primal adapter for capability-based routing
    adapter: Arc<UniversalPrimalAdapter>,

    /// Storage capability requirements
    capability_requirements: Arc<RwLock<StorageCapabilityRequirements>>,

    /// Local cache for performance optimization
    cache: Arc<RwLock<StorageCache>>,

    /// Storage operation statistics
    stats: Arc<RwLock<StorageStats>>,

    /// Event broadcaster
    events_tx: broadcast::Sender<UniversalStorageEvent>,

    /// Configuration
    config: UniversalStorageConfig,
}

impl UniversalStorageClient {
    /// Create a new universal storage client
    pub async fn new(&self) -> SongbirdResult<Self> {
        let capability_requirements =
            Arc::new(RwLock::new(StorageCapabilityRequirements::default()));
        let cache = Arc::new(RwLock::new(StorageCache::new(config.cache_config.clone())));
        let stats = Arc::new(RwLock::new(StorageStats::default()));
        let (events_tx, _) = broadcast::channel(1000);

        let client = Self {
            adapter,
            capability_requirements,
            cache,
            stats,
            events_tx,
            config,
        };

        info!("🏪 Universal Storage Client initialized with capability-based routing");
        Ok(songbird_errors::evolved_success(success(client)))
    }

    /// Store data with capability-based provider selection
    pub async fn store(&self) -> SongbirdResult<()> {
        let start_time = std::time::Instant::now();

        // Check cache first if enabled
        if self.config.cache_config.write_through {
            let mut cache = self.cache.write().await;
            cache.put(key.clone(), data.clone(), None);
        }

        // Get storage capabilities required for this operation
        let requirements = self.capability_requirements.read().await;

        // Find providers with required capabilities
        let providers = self
            .adapter
            .discover_providers_with_capabilities(
                &requirements
                    .required_capabilities
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>(),
            )
            .await?;

        if providers.data.is_empty() {
            let error = SongbirdError::operation_error(
                "No storage providers available with required capabilities".to_string(),
            );
            self.record_operation_failure(&key, "store", &error).await;
            return Err(error);
        }

        // Select optimal provider based on performance requirements
        let selected_provider = self
            .select_optimal_provider(&providers.data, &requirements)
            .await?;

        // Perform storage operation
        match self
            .perform_store_operation(&key, &data, &selected_provider.data, metadata.clone())
            .await
        {
            Ok(songbird_errors::evolved_success(_)) => {
                let duration = start_time.elapsed();
                self.record_operation_success("store", &key, duration, data.len() as u64)
                    .await;

                // Emit success event
                let event = UniversalStorageEvent::DataStored {
                    key,
                    size_bytes: data.len() as u64,
                    provider: selected_provider.data.provider_id.to_string(),
                    timestamp: SystemTime::now(),
                    metadata: metadata.unwrap_or_default(),
                };
                let _ = self.events_tx.send(event);
                Ok(())
            }
            Err(e) => {
                self.record_operation_failure(&key, "store", &e).await;
                Err(e)
            }
        }
    }

    /// Retrieve data with capability-based provider selection
    pub async fn retrieve(&self) -> SongbirdResult<()> {let start_time = std::time::Instant::now();

        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(cached_data) = cache.get(&key) {
                let event = UniversalStorageEvent::DataRetrieved {
                    key,
                    size_bytes: cached_data.len() as u64,
                    provider: "cache".to_string(),
                    cache_hit: true,
                    timestamp: SystemTime::now(),
                };
                let _ = self.events_tx.send(event);

                return Ok(songbird_errors::evolved_success(success(Some(cached_data))));
            }
        }

        // Get storage capabilities required for this operation
        let requirements = self.capability_requirements.read().await;

        // Find providers with required capabilities
        let providers = self
            .adapter
            .discover_providers_with_capabilities(
                &requirements
                    .required_capabilities
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>(),
            )
            .await?;

        if providers.data.is_empty() {
            let error = SongbirdError::operation_error(
                "No storage providers available with required capabilities".to_string(),
            );
            self.record_operation_failure(&key, "retrieve", &error)
                .await;
            return Err(error);
        }

        // Try providers in order of preference
        for provider in &providers.data {
            match self.perform_retrieve_operation(&key, provider).await {
                Ok(songbird_errors::evolved_success(response)) if response.data.is_some() => {
                    let data = response.data.ok_or_else(|| {
                        songbird_errors::SongbirdError::operation_error("No data in response")
                    })?;
                    let duration = start_time.elapsed();
                    self.record_operation_success("retrieve", &key, duration, data.len() as u64)
                        .await;

                    // Cache the retrieved data
                    {
                        let mut cache = self.cache.write().await;
                        cache.put(key.clone(), data.clone(), None);
                    }

                    // Emit success event
                    let event = UniversalStorageEvent::DataRetrieved {
                        key,
                        size_bytes: data.len() as u64,
                        provider: provider.provider_id.to_string(),
                        cache_hit: false,
                        timestamp: SystemTime::now(),
                    };
                    let _ = self.events_tx.send(event);

                    return Ok(songbird_errors::evolved_success(success(Some(data))));
                }
                Ok(songbird_errors::evolved_success(response)) if response.data.is_none() => continue, // Try next provider
                Ok(songbird_errors::evolved_success(_)) => continue, // Catch-all for any other Ok cases
                Err(e) => {
                    warn!(
                        "Retrieve failed from provider {}: {}",
                        provider.provider_id.to_string(),
                        e
                    );
                    continue;
                }
            }
        }

        // No provider had the data
        Ok(songbird_errors::evolved_success(success(None)))
    }

    /// Delete data from all capable providers
    pub async fn delete(&self) -> SongbirdResult<()> {
        let start_time = std::time::Instant::now();

        // Remove from cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(&key);
        }

        // Get storage capabilities required for this operation
        let requirements = self.capability_requirements.read().await;

        // Find providers with required capabilities
        let providers = self
            .adapter
            .discover_providers_with_capabilities(
                &requirements
                    .required_capabilities
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>(),
            )
            .await?;

        let mut success_count = 0;
        let mut errors = Vec::new();

        // Delete from all providers
        for provider in &providers.data {
            match self.perform_delete_operation(&key, provider).await {
                Ok(songbird_errors::evolved_success(_)) => success_count += 1,
                Err(e) => {
                    warn!(
                        "Delete failed from provider {}: {}",
                        provider.provider_id.to_string(),
                        e
                    );
                    errors.push(e);
                }
            }
        }

        if success_count > 0 {
            let duration = start_time.elapsed();
            self.record_operation_success("delete", &key, duration, 0)
                .await;

            // Emit success event
            let event = UniversalStorageEvent::DataDeleted {
                key,
                provider: "multiple".to_string(),
                timestamp: SystemTime::now(),
            };
            let _ = self.events_tx.send(event);
            Ok(())
        } else {
            let error = SongbirdError::operation_error(format!(
                "Delete failed from all providers: {:?}",
                errors
            ));
            self.record_operation_failure(&key, "delete", &error).await;
            Err(error)
        }
    }

    /// List keys matching a pattern
    pub async fn list_keys(&self) -> SongbirdResult<()> {// Get storage capabilities required for this operation
        let requirements = self.capability_requirements.read().await;

        // Find providers with required capabilities
        let providers = self
            .adapter
            .discover_providers_with_capabilities(
                &requirements
                    .required_capabilities
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>(),
            )
            .await?;

        let mut all_keys = Vec::new();

        // Collect keys from all providers
        for provider in &providers.data {
            match self.perform_list_operation(&pattern, provider).await {
                Ok(songbird_errors::evolved_success(keys)) => all_keys.extend(keys.data),
                Err(e) => {
                    warn!(
                        "List failed from provider {}: {}",
                        provider.provider_id.to_string(),
                        e
                    );
                }
            }
        }

        // Remove duplicates and sort
        all_keys.sort();
        all_keys.dedup();

        Ok(songbird_errors::evolved_success(success(all_keys)))
    }

    /// Update storage capability requirements
    pub async fn update_capability_requirements(&self) -> SongbirdResult<()> {
        let mut current_requirements = self.capability_requirements.write().await;
        *current_requirements = requirements;

        info!("🔄 Updated storage capability requirements");
        Ok(())
    }

    /// Get current storage statistics
    pub async fn get_stats(&self) -> StorageStats {
        self.stats.read().await.clone()
    }

    /// Get cache statistics
    pub fn get_cache_stats(super::cache::CacheStats {
        self.cache.read().await.get_stats().clone()
    }

    /// Subscribe to storage events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<UniversalStorageEvent> {
        self.events_tx.subscribe()
    }

    /// Clear cache
    pub async fn clear_cache(&self) -> SongbirdResult<()> {
        let mut cache = self.cache.write().await;
        cache.clear();

        let event = UniversalStorageEvent::CacheEvent {
            event_type: super::events::CacheEventType::Clear,
            key: None,
            hit_ratio: cache.hit_ratio(),
            timestamp: SystemTime::now(),
        };
        let _ = self.events_tx.send(event);
        Ok(())
    }

    /// Perform health check on storage providers
    pub async fn health_check(&self) -> SongbirdResult<()> {let requirements = self.capability_requirements.read().await;
        let providers = self
            .adapter
            .discover_providers_with_capabilities(
                &requirements
                    .required_capabilities
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>(),
            )
            .await?;

        let mut health_status = HashMap::new();

        for provider in &providers.data {
            let is_healthy = self.check_provider_health(provider).await;
            health_status.insert(provider.provider_id.to_string(), is_healthy);
        }

        Ok(songbird_errors::evolved_success(success(health_status)))
    }

    // Private helper methods

    async fn select_optimal_provider<'a>(
        &self,
        providers: &'a [crate::universal_adapter::CapabilityProvider],
        _requirements: &StorageCapabilityRequirements,
    ) -> SongbirdResult<&'a crate::universal_adapter::CapabilityProvider> {
        // Simple selection based on first available for now
        // In production, this would consider performance metrics, load, etc.
        providers
            .first()
            .ok_or_else(|| SongbirdError::operation_error("No providers available".to_string()))
            .map(success)
    }

    async fn perform_store_operation(&self) -> SongbirdResult<()> {
        // Encode data for transmission
        let encoded_data = general_purpose::STANDARD.encode(data);

        // Create storage request
        let request = HashMap::from([
            ("operation".to_string(), "store".to_string()),
            ("key".to_string(), key.to_string()),
            ("data".to_string(), encoded_data),
            (
                "metadata".to_string(),
                serde_json::to_string(&metadata).unwrap_or_default(),
            ),
        ]);

        // Route request through universal adapter
        let response = self
            .adapter
            .route_request(
                &provider.provider_id.to_string(),
                "storage",
                serde_json::to_value(request).map_err(|e| {
                    songbird_errors::SongbirdError::operation_error(format!(
                        "JSON serialization failed: {}",
                        e
                    ))
                })?,
            )
            .await?;

        if response
            .data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            Ok(())
        } else {
            let error_msg = response
                .data
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Store operation failed");
            Err(SongbirdError::internal_error(operation_error(error_msg.to_string()))
        }
    }

    pub async fn perform_retrieve_operation(&self) -> SongbirdResult<()> {// Create retrieval request
        let request = HashMap::from([
            ("operation".to_string(), "retrieve".to_string()),
            ("key".to_string(), key.to_string()),
        ]);

        // Route request through universal adapter
        let response = self
            .adapter
            .route_request(
                &provider.provider_id.to_string(),
                "storage",
                serde_json::to_value(request).map_err(|e| {
                    songbird_errors::SongbirdError::operation_error(format!(
                        "JSON serialization failed: {}",
                        e
                    ))
                })?,
            )
            .await?;

        if response
            .data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            if let Some(encoded_data) = response.data.get("data").and_then(|v| v.as_str()) {
                match general_purpose::STANDARD.decode(encoded_data) {
                    Ok(songbird_errors::evolved_success(data)) => Ok(success(Some(data))),
                    Err(e) => Err(SongbirdError::internal_error(operation_error(format!(
                        "Data decode error: {}",
                        e
                    ))),
                }
            } else {
                Ok(songbird_errors::evolved_success(success(None)))
            }
        } else {
            Ok(songbird_errors::evolved_success(success(None)))
        }
    }

    async fn perform_delete_operation(&self) -> SongbirdResult<()> {
        // Create deletion request
        let request = HashMap::from([
            ("operation".to_string(), "delete".to_string()),
            ("key".to_string(), key.to_string()),
        ]);

        // Route request through universal adapter
        let response = self
            .adapter
            .route_request(
                &provider.provider_id.to_string(),
                "storage",
                serde_json::to_value(request).map_err(|e| {
                    songbird_errors::SongbirdError::operation_error(format!(
                        "JSON serialization failed: {}",
                        e
                    ))
                })?,
            )
            .await?;

        if response
            .data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            Ok(())
        } else {
            let error_msg = response
                .data
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Delete operation failed");
            Err(SongbirdError::internal_error(operation_error(error_msg.to_string()))
        }
    }

    pub async fn perform_list_operation(&self) -> SongbirdResult<()> {// Create list request
        let request = HashMap::from([
            ("operation".to_string(), "list".to_string()),
            ("pattern".to_string(), pattern.to_string()),
        ]);

        // Route request through universal adapter
        let response = self
            .adapter
            .route_request(
                &provider.provider_id.to_string(),
                "storage",
                serde_json::to_value(request).map_err(|e| {
                    songbird_errors::SongbirdError::operation_error(format!(
                        "JSON serialization failed: {}",
                        e
                    ))
                })?,
            )
            .await?;

        if response
            .data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            if let Some(keys_array) = response.data.get("keys").and_then(|v| v.as_array()) {
                let keys: Vec<String> = keys_array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                Ok(songbird_errors::evolved_success(success(keys)))
            } else {
                Ok(songbird_errors::evolved_success(success(vec![])))
            }
        } else {
            Ok(songbird_errors::evolved_success(success(vec![])))
        }
    }

    async fn check_provider_health(&self) -> bool {
        // Simple health check - in production this would be more comprehensive
        match self
            .adapter
            .route_request(
                &provider.provider_id.to_string(),
                "health",
                serde_json::json!({"operation": "ping"}),
            )
            .await
        {
            Ok(songbird_errors::evolved_success(response)) => response
                .data
                .get("healthy")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    async fn record_operation_success(
        &self,
        operation: &str,
        _key: &str,
        duration: Duration,
        bytes: u64,
    ) {
        let mut stats = self.stats.write().await;
        match operation {
            "store" => stats.record_write(duration, bytes),
            "retrieve" => stats.record_read(duration, bytes),
            "delete" => stats.record_delete(duration),
            _ => {}
        }
    }

    async fn record_operation_failure(&self, key: &str, operation: &str, error: &SongbirdError) {
        let mut stats = self.stats.write().await;
        stats.record_failure();

        // Emit failure event
        let event = UniversalStorageEvent::OperationFailed {
            operation: match operation {
                "store" => super::events::StorageOperation::Store,
                "retrieve" => super::events::StorageOperation::Retrieve,
                "delete" => super::events::StorageOperation::Delete,
                "list" => super::events::StorageOperation::List,
                _ => super::events::StorageOperation::Store,
            },
            key: key.to_string(),
            provider: "unknown".to_string(),
            error: error.to_string(),
            timestamp: SystemTime::now(),
        };
        let _ = self.events_tx.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use songbird_network::CommunicationLayer;

    // Helper function to create a test adapter
    pub async fn create_test_adapter(&self) -> SongbirdResult<()> {let unified_config = songbird_config::SongbirdConfig::default();
        let adapter_config = crate::universal_adapter::core::UniversalAdapterConfig::default();
        let adapter = UniversalPrimalAdapter::new(unified_config, adapter_config)
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::operation_error(
                    "operation_failed",
                    format!("Operation failed: {}", e),
                )
            })?;
        Arc::new(adapter.data)
    }

    #[tokio::test]
    async fn test_storage_client_creation() {
        let adapter = create_test_adapter().await;
        let config = UniversalStorageConfig::default();

        let client = UniversalStorageClient::new(adapter, config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_capability_requirements_update() {
        let adapter = create_test_adapter().await;
        let config = UniversalStorageConfig::default();
        let client = UniversalStorageClient::new(adapter, config)
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::operation_error(
                    "operation_failed",
                    format!("Operation failed: {}", e),
                )
            })?
            .data;

        let new_requirements = StorageCapabilityRequirements {
            required_capabilities: vec![StorageCapabilityType::ObjectStorage],
            ..Default::default()
        };

        let result = client
            .update_capability_requirements(new_requirements)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let adapter = create_test_adapter().await;
        let config = UniversalStorageConfig::default();
        let client = UniversalStorageClient::new(adapter, config)
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::operation_error(
                    "operation_failed",
                    format!("Operation failed: {}", e),
                )
            })?
            .data;

        let _stats_before = client.get_cache_stats().await;

        let result = client.clear_cache().await;
        assert!(result.is_ok());

        let stats_after = client.get_cache_stats().await;
        assert_eq!(stats_after.total_entries, 0);
    }
}
