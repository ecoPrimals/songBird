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
