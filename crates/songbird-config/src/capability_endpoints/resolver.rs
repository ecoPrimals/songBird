// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Resolver state, cache, and orchestrated discovery.

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::remote_probes;
use super::types::{CapabilityEndpoint, CapabilityType, DiscoveryMethod};

/// Capability endpoint resolver
#[derive(Debug, Clone)]
pub struct CapabilityEndpointResolver {
    /// Cached endpoints
    cache: Arc<RwLock<HashMap<CapabilityType, CapabilityEndpoint>>>,
    /// Cache TTL
    cache_ttl: Duration,
    /// Injected endpoints (tests, explicit wiring) — consulted before environment discovery.
    static_overrides: Option<Arc<HashMap<CapabilityType, String>>>,
}

impl CapabilityEndpointResolver {
    /// Create new resolver
    #[must_use]
    pub fn new() -> Self {
        let cache_ttl_secs = songbird_process_env::var("DISCOVERY_CACHE_TTL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);

        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            static_overrides: None,
        }
    }

    /// Resolver with fixed capability → endpoint URLs (checked before any environment lookup).
    ///
    /// Use for tests and embedders that pass configuration in-process instead of mutating
    /// process-global environment variables.
    #[must_use]
    pub fn with_endpoint_overrides(overrides: HashMap<CapabilityType, String>) -> Self {
        let mut base = Self::new();
        base.static_overrides = Some(Arc::new(overrides));
        base
    }

    /// Get endpoint for a capability
    ///
    /// # Errors
    /// Returns error if no endpoint can be discovered for the capability
    pub async fn get_endpoint(&self, capability: CapabilityType) -> SongbirdResult<String> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&capability)
                && let Ok(elapsed) = cached.discovered_at.elapsed()
                && elapsed < self.cache_ttl
            {
                debug!("Using cached endpoint for {:?}", capability);
                return Ok(cached.endpoint.clone());
            }
        }

        let endpoint = self.discover_endpoint(&capability).await?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(capability.clone(), endpoint.clone());
        }

        Ok(endpoint.endpoint)
    }

    /// Discover endpoint for a capability
    async fn discover_endpoint(
        &self,
        capability: &CapabilityType,
    ) -> SongbirdResult<CapabilityEndpoint> {
        debug!("Discovering endpoint for capability: {:?}", capability);

        if let Some(overrides) = &self.static_overrides
            && let Some(endpoint) = overrides.get(capability)
        {
            return Ok(CapabilityEndpoint {
                capability: capability.clone(),
                endpoint: endpoint.clone(),
                provider_id: None,
                discovery_method: DiscoveryMethod::ConfigFile,
                confidence: 1.0,
                discovered_at: std::time::SystemTime::now(),
            });
        }

        if let Ok(endpoint) = songbird_process_env::var(capability.env_var_name()) {
            info!("Found {} endpoint in environment: {}", capability.as_str(), endpoint);
            return Ok(CapabilityEndpoint {
                capability: capability.clone(),
                endpoint,
                provider_id: None,
                discovery_method: DiscoveryMethod::Environment,
                confidence: 1.0,
                discovered_at: std::time::SystemTime::now(),
            });
        }

        if let Some(endpoint) = remote_probes::discover_from_registry(capability).await? {
            return Ok(endpoint);
        }

        if let Some(endpoint) = remote_probes::discover_from_container_metadata(capability).await? {
            return Ok(endpoint);
        }

        if let Some(endpoint) = remote_probes::discover_from_dns(capability).await? {
            return Ok(endpoint);
        }

        Err(SongbirdError::Configuration {
            message: format!(
                "No endpoint found for capability: {}. Set {} environment variable or enable discovery.",
                capability.as_str(),
                capability.env_var_name()
            ),
            field: Some(capability.env_var_name()),
            suggestion: Some(format!(
                "Set {}=http://your-provider:port or enable discovery with SERVICE_REGISTRY_ENDPOINT",
                capability.env_var_name()
            )),
        })
    }

    /// Clear cache (force re-discovery)
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
        info!("Capability endpoint cache cleared");
    }

    /// Get all cached endpoints
    pub async fn get_all_cached(&self) -> HashMap<CapabilityType, CapabilityEndpoint> {
        let cache = self.cache.read().await;
        cache.clone()
    }
}

impl Default for CapabilityEndpointResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn new_resolver_has_empty_cache() {
        let resolver = CapabilityEndpointResolver::new();
        assert!(resolver.static_overrides.is_none());
    }

    #[test]
    fn with_endpoint_overrides_stores_overrides() {
        let mut overrides = HashMap::new();
        overrides.insert(CapabilityType::Networking, "http://mock:8080".into());
        let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
        assert!(resolver.static_overrides.is_some());
        let map = resolver.static_overrides.unwrap();
        assert_eq!(map.get(&CapabilityType::Networking).unwrap(), "http://mock:8080");
    }

    #[tokio::test]
    async fn get_endpoint_uses_static_override() {
        let mut overrides = HashMap::new();
        overrides.insert(CapabilityType::Storage, "http://storage:9000".into());
        let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
        let endpoint = resolver.get_endpoint(CapabilityType::Storage).await.unwrap();
        assert_eq!(endpoint, "http://storage:9000");
    }

    #[tokio::test]
    async fn get_endpoint_caches_result() {
        let mut overrides = HashMap::new();
        overrides.insert(CapabilityType::Compute, "http://compute:7000".into());
        let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);

        let _ = resolver.get_endpoint(CapabilityType::Compute).await.unwrap();
        let cached = resolver.get_all_cached().await;
        assert!(cached.contains_key(&CapabilityType::Compute));
        assert_eq!(cached[&CapabilityType::Compute].endpoint, "http://compute:7000");
    }

    #[tokio::test]
    async fn clear_cache_empties_state() {
        let mut overrides = HashMap::new();
        overrides.insert(CapabilityType::Ai, "http://ai:5000".into());
        let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);

        let _ = resolver.get_endpoint(CapabilityType::Ai).await.unwrap();
        assert!(!resolver.get_all_cached().await.is_empty());

        resolver.clear_cache().await;
        assert!(resolver.get_all_cached().await.is_empty());
    }

    #[tokio::test]
    async fn get_endpoint_fails_without_any_source() {
        let resolver = CapabilityEndpointResolver::with_endpoint_overrides(HashMap::new());
        let result = resolver.get_endpoint(CapabilityType::Observability).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cached_endpoint_returned_on_second_call() {
        let mut overrides = HashMap::new();
        overrides.insert(CapabilityType::Security, "http://cached-test:1234".into());
        let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);

        let first = resolver.get_endpoint(CapabilityType::Security).await.unwrap();
        let second = resolver.get_endpoint(CapabilityType::Security).await.unwrap();
        assert_eq!(first, second);
        assert_eq!(first, "http://cached-test:1234");
    }

    #[test]
    fn default_resolver_matches_new() {
        let resolver = CapabilityEndpointResolver::default();
        assert!(resolver.static_overrides.is_none());
    }
}
