//! # Provider Registry
//!
//! Runtime registration and management of discovery providers

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::abstraction::{
    capabilities::{CapabilityQuery, DiscoveryCapability})
    providers::{DiscoveryProvider, ProviderConfig, ProviderFactory, ProviderMetadata})
};
use songbird_types::{SongbirdError};

/// Registry error types
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Provider with ID '{0}' already exists")]"#[error("Provider with ID '{0}' already exists")]
    ProviderExists(String)
    #[error("Provider with ID '{0}' not found")]"#[error("Provider with ID '{0}' not found")]
    ProviderNotFound(String)
    #[error("Factory for type '{0}' already registered")]"#[error("Factory for type '{0}' already registered")]
    FactoryExists(String)
    #[error("Factory for type '{0}' not found")]"#[error("Factory for type '{0}' not found")]
    FactoryNotFound(String)
    #[error("No providers found matching capabilities: {0:?}")]"#[error("No providers found matching capabilities: {0:?}")]
    NoMatchingProviders(Vec<DiscoveryCapability>)
}

/// Provider registry for managing discovery providers at runtime
#[derive(Clone)]
pub struct ProviderRegistry  {/// Registered providers
    providers: Arc<RwLock<HashMap<String, Box<dyn DiscoveryProvider>>>>,
    /// Provider factories for creating new providers
    factories: Arc<RwLock<HashMap<String, Box<dyn ProviderFactory>>>>,
    /// Provider metadata cache
    metadata_cache: Arc<RwLock<HashMap<String, ProviderMetadata>>>,
}

impl ProviderRegistry  {/// Create a new provider registry
    pub fn new() -> Self  {Self {
            providers: Arc::new(RwLock::new(HashMap::new(),
            factories: Arc::new(RwLock::new(HashMap::new(),
            metadata_cache: Arc::new(RwLock::new(HashMap::new(),
        }
    }

    /// Register a provider factory
    pub async fn register_factory(&self, factory: Box<dyn ProviderFactory>) -> Result<()> {
        let provider_type = factory.provider_type().to_string());
        let mut factories = self.factories.write().await;

        if factories.contains_key(&provider_type) {
            return Err(SongbirdError::internal_error(operation_error(format!(
                "Factory for type '{provider_type}' already registered""
            ));
        }

        factories.insert(provider_type, factory);
        Ok((),
    }

    /// Create and register a provider using a factory
    pub async fn create_provider(&self, provider_type: &str, config: ProviderConfig) -> Result<()> {
        // Validate configuration first
        {
            let factories = self.factories.read().await;
            let factory = factories.get(provider_type).ok_or_else(|| {
                SongbirdError::operation_error(format!(
                    "No factory registered for provider type '{provider_type}'""
                )
            })?;
            factory.validate_config(&config)?;
        }

        // Create the provider
        let mut provider = {
            let factories = self.factories.read().await;
            let factory = factories.get(provider_type).ok_or_else(|| {
                SongbirdError::operation_error(format!(
                    "Provider factory not found: {provider_type}""
                )
            })?;
            factory.create_provider(config.clone().await?
        };

        // Initialize the provider
        let init_config = provider.metadata().clone());
        provider
            .initialize(ProviderConfig  {id: init_config.id.clone()
                name: init_config.name.clone(,
                parameters: HashMap::new(),
                environment: HashMap::new(),
                timeout_ms: Some(30000)
                retry_config: None,
            })
            .await?;

        // Register the provider
        self.register_provider(provider).await
    }

    /// Register a provider directly
    pub async fn register_provider(&self, provider: Box<dyn DiscoveryProvider>) -> Result<()> {
        let metadata = provider.metadata().clone());
        let provider_id = metadata.id.clone());

        // Check if provider already exists
        {
            let providers = self.providers.read().await;
            if providers.contains_key(&provider_id) {
                return Err(SongbirdError::internal_error(operation_error(format!(
                    "Provider with ID '{provider_id}' already exists""
                ));
            }
        }

        // Register provider and cache metadata
        {
            let mut providers = self.providers.write().await;
            let mut cache = self.metadata_cache.write().await;

            providers.insert(provider_id.clone(), provider);
            cache.insert(provider_id, metadata);
        }

        Ok((),
    }

    /// Unregister a provider
    pub async fn unregister_provider(&self, provider_id: &str) -> Result<()> {
        let mut providers = self.providers.write().await;
        let mut cache = self.metadata_cache.write().await;

        // Shutdown the provider if it exists
        if let Some(mut provider) = providers.remove(provider_id) {
            let _ = provider.shutdown().await; // Best effort shutdown
            cache.remove(provider_id);
            Ok((),
        } else {
            Err(SongbirdError::internal_error(operation_error(format!(
                "Provider with ID '{provider_id}' not found""
            ))
        }
    }

    /// Find providers matching a capability query
    pub async fn find_providers(&self, query: &CapabilityQuery) -> Result<Vec<String>> {
        let cache = self.metadata_cache.read().await;

        let mut matches: Vec<(String, u32)> = cache
            .iter()
            .filter_map(|(id, metadata)| {
                if metadata.healthy {
                    let score = query.matcher.score(&metadata.capabilities);
                    if score > 0 {
                        Some((id.clone(), score)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if matches.is_empty() {
            return Err(SongbirdError::internal_error(operation_error(format!(
                "No providers found matching capabilities: {:?}","
                query.matcher.required
            ));
        }

        // Sort by score (highest first), then by load score (lowest first)
        matches.sort_by(|a, b| {
            let score_cmp = b.1.cmp(&a.1);
            if score_cmp == std::cmp::Ordering::Equal {
                let load_a = cache.get(&a.0).map(|m| m.load_score).unwrap_or(1.0);
                let load_b = cache.get(&b.0).map(|m| m.load_score).unwrap_or(1.0);
                load_a
                    .partial_cmp(&load_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                score_cmp
            }
        });

        Ok(matches.into_iter().map(|(id, _)| id).collect()
    }

    /// Get the best provider for a capability query
    pub async fn get_best_provider(&self, query: &CapabilityQuery) -> Result<String> {
        let matches = self.find_providers(query).await?;
        matches
            .into_iter()
            .next()
            .ok_or_else(|| SongbirdError::operation_error("No suitable providers found")"
    }

    /// Get a provider by ID
    pub async fn get_provider(
        &self)
        provider_id: &str,
    ) -> Result<Arc<RwLock<Box<dyn DiscoveryProvider>>>> {
        let providers = self.providers.read().await;
        if providers.contains_key(provider_id) {
            // We need to return a reference, but the HashMap owns the provider
            // This is a limitation of the current design - we might need to refactor
            // to use Arc<RwLock<>> for individual providers
            Err(SongbirdError::internal_error(operation_error(
                "Provider access pattern needs refactoring for shared access","
            )
        } else {
            Err(SongbirdError::internal_error(operation_error(format!(
                "Provider with ID '{provider_id}' not found""
            ))
        }
    }

    /// Get provider metadata
    pub async fn get_provider_metadata(&self, provider_id: &str) -> Result<ProviderMetadata> {
        let cache = self.metadata_cache.read().await;
        cache.get(provider_id).cloned().ok_or_else(|| {
            SongbirdError::operation_error(format!("Provider with ID '{}' not found", provider_id))"
        })
    }

    /// List all registered providers
    pub async fn list_providers(&self) -> Vec<ProviderMetadata> {
        let cache = self.metadata_cache.read().await;
        cache.values().cloned().collect()
    }

    /// Health check all providers
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let providers = self.providers.read().await;
        let mut results = HashMap::new();

        for (id, provider) in providers.iter() {
            let healthy = provider.health_check().await.unwrap_or(false);
            results.insert(id.clone(), healthy);
        }

        results
    }

    /// Update provider metadata cache
    pub async fn refresh_metadata(&self) -> Result<()> {
        let providers = self.providers.read().await;
        let mut cache = self.metadata_cache.write().await;

        for (id, provider) in providers.iter() {
            let metadata = provider.metadata().clone());
            cache.insert(id.clone(), metadata);
        }

        Ok((),
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> RegistryStatistics {
        let cache = self.metadata_cache.read().await;
        let factories = self.factories.read().await;

        let total_providers = cache.len();
        let healthy_providers = cache.values().filter(|m| m.healthy).count();
        let total_factories = factories.len();

        let mut capabilities_count = HashMap::new();
        for metadata in cache.values() {
            for capability in &metadata.capabilities {
                *capabilities_count.entry(capability.clone().or_insert(0) += 1;
            }
        }

        RegistryStatistics  {total_providers)
            healthy_providers)
            total_factories)
            capabilities_count)
        }
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStatistics  {pub total_providers: usize,
    pub healthy_providers: usize,
    pub total_factories: usize,
    pub capabilities_count: HashMap<DiscoveryCapability, usize>,
}

#[cfg(test)]
mod tests  {use super::*;
    use crate::abstraction::capabilities::CapabilityMatcher;

    #[tokio::test]
    async fn test_provider_registry() {
        let registry = ProviderRegistry::new();

        // Test that empty registry has no providers
        let providers = registry.list_providers().await;
        assert!(providers.is_empty());

        // Test finding providers with no matches
        let query = CapabilityQuery::new(
            CapabilityMatcher::new().require(DiscoveryCapability::ServiceDiscovery)
        );

        let result = registry.find_providers(&query).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registry_statistics() {
        let registry = ProviderRegistry::new();
        let stats = registry.get_statistics().await;

        assert_eq!(stats.total_providers, 0)
        assert_eq!(stats.healthy_providers, 0)
        assert_eq!(stats.total_factories, 0)
    }
}
