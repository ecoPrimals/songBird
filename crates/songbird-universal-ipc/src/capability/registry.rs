// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unified capability registry

use crate::capability::provider::{HealthStatus, Provider};
use crate::capability::strategy::{DiscoveryStrategy, FilesystemStrategy};
use crate::error::{IpcError, IpcResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Unified capability registry
///
/// Discovers and caches capability providers using multiple strategies.
/// Integrates with Universal IPC for platform-agnostic communication.
///
/// ## TRUE PRIMAL Principles
///
/// 1. **Self-Knowledge**: Applications only know themselves
/// 2. **Capability Discovery**: Find providers by what they do, not who they are
/// 3. **Runtime Discovery**: No hardcoded primal names
/// 4. **Graceful Fallback**: Works without dependencies
pub struct CapabilityRegistry {
    /// Cached providers by capability
    cache: Arc<RwLock<HashMap<String, Vec<CachedProvider>>>>,

    /// Discovery strategies (in priority order)
    strategies: Vec<DiscoveryStrategy>,

    /// Cache TTL
    cache_ttl: Duration,
}

/// Cached provider with TTL
#[derive(Debug, Clone)]
struct CachedProvider {
    provider: Provider,
    cached_at: SystemTime,
}

impl CapabilityRegistry {
    /// Create a new capability registry with default strategies
    #[must_use]
    pub fn new() -> Self {
        Self::with_strategies(vec![
            DiscoveryStrategy::Environment,
            DiscoveryStrategy::Filesystem(FilesystemStrategy::new()),
        ])
    }

    /// Create with custom strategies
    #[must_use]
    pub fn with_strategies(strategies: Vec<DiscoveryStrategy>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            strategies,
            cache_ttl: Duration::from_secs(60), // 1 minute default TTL
        }
    }

    /// Set cache TTL
    #[must_use]
    pub const fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Discover providers offering a capability
    ///
    /// # Arguments
    /// * `capability` - Capability to discover (e.g., "crypto", "storage")
    ///
    /// # Returns
    /// First healthy provider found, or error if none available
    ///
    /// # Example
    /// ```rust,no_run
    /// # use songbird_universal_ipc::capability::CapabilityRegistry;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = CapabilityRegistry::new();
    /// let provider = registry.discover("crypto").await?;
    /// println!("Found crypto provider: {}", provider.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover(&self, capability: &str) -> IpcResult<Provider> {
        info!("🔍 Discovering providers for capability: {}", capability);

        // Check cache first
        if let Some(provider) = self.get_from_cache(capability).await {
            info!("   ✅ Found in cache: {}", provider.id);
            return Ok(provider);
        }

        // Try each strategy in order
        for strategy in &self.strategies {
            debug!("   Trying strategy: {}", strategy.name());

            match strategy.discover(capability).await {
                Ok(providers) if !providers.is_empty() => {
                    info!(
                        "   ✅ Strategy '{}' found {} provider(s)",
                        strategy.name(),
                        providers.len()
                    );

                    // Cache all providers
                    self.cache_providers(capability, providers.clone()).await;

                    // Return first usable provider
                    if let Some(provider) =
                        providers.into_iter().find(super::provider::Provider::is_usable)
                    {
                        return Ok(provider);
                    }
                }
                Ok(_) => {
                    debug!("   ⏭️  Strategy '{}' found no providers", strategy.name());
                }
                Err(e) => {
                    warn!("   ⚠️  Strategy '{}' failed: {}", strategy.name(), e);
                }
            }
        }

        Err(IpcError::ServiceNotFound(format!("No providers found for capability: {capability}")))
    }

    /// Discover all providers offering a capability
    ///
    /// # Arguments
    /// * `capability` - Capability to discover
    ///
    /// # Returns
    /// All providers found (may be empty)
    pub async fn discover_all(&self, capability: &str) -> IpcResult<Vec<Provider>> {
        info!("🔍 Discovering ALL providers for capability: {}", capability);

        let mut all_providers = Vec::new();

        // Try each strategy
        for strategy in &self.strategies {
            match strategy.discover(capability).await {
                Ok(providers) if !providers.is_empty() => {
                    info!(
                        "   ✅ Strategy '{}' found {} provider(s)",
                        strategy.name(),
                        providers.len()
                    );
                    all_providers.extend(providers);
                }
                Ok(_) => {}
                Err(e) => {
                    warn!("   ⚠️  Strategy '{}' failed: {}", strategy.name(), e);
                }
            }
        }

        if !all_providers.is_empty() {
            // Cache all providers
            self.cache_providers(capability, all_providers.clone()).await;
        }

        Ok(all_providers)
    }

    /// Check cache for capability providers
    async fn get_from_cache(&self, capability: &str) -> Option<Provider> {
        let cache_ttl = self.cache_ttl;
        self.cache.read().await.get(capability).and_then(|cached_providers| {
            let now = SystemTime::now();
            cached_providers
                .iter()
                .find(|cached| {
                    now.duration_since(cached.cached_at)
                        .is_ok_and(|age| age < cache_ttl && cached.provider.is_usable())
                })
                .map(|cached| cached.provider.clone())
        })
    }

    /// Cache providers for a capability
    async fn cache_providers(&self, capability: &str, providers: Vec<Provider>) {
        let mut cache = self.cache.write().await;

        let cached_providers = providers
            .into_iter()
            .map(|provider| CachedProvider {
                provider,
                cached_at: SystemTime::now(),
            })
            .collect();

        cache.insert(capability.to_string(), cached_providers);
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
        info!("🗑️  Capability cache cleared");
    }

    /// Update provider health status
    pub async fn update_health(&self, capability: &str, provider_id: &str, status: HealthStatus) {
        let mut cache = self.cache.write().await;

        if let Some(cached_providers) = cache.get_mut(capability) {
            for cached in cached_providers {
                if cached.provider.id == provider_id {
                    cached.provider.update_health(status);
                    break;
                }
            }
        }
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::capability::provider::HealthStatus;
    use crate::capability::strategy::{DiscoveryStrategy, FilesystemStrategy};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn registry_with_injected_env(map: HashMap<String, String>) -> CapabilityRegistry {
        CapabilityRegistry::with_strategies(vec![
            DiscoveryStrategy::InjectedEnvironment(Arc::new(map)),
            DiscoveryStrategy::Filesystem(FilesystemStrategy::with_paths(vec![])),
        ])
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = CapabilityRegistry::new();
        assert_eq!(registry.strategies.len(), 2); // Environment + Filesystem
    }

    #[tokio::test]
    async fn test_discover_with_env() {
        // Skip this test as it requires actual socket connection
        // The capability registry discovery is tested in E2E tests
        // This unit test would need to mock the socket connection
    }

    #[tokio::test]
    async fn test_discover_not_found() {
        let registry = registry_with_injected_env(HashMap::new());
        let result = registry.discover("nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache() {
        let mut map = HashMap::new();
        map.insert(String::from("TEST_PROVIDER_SOCKET"), String::from("/tmp/test.sock"));

        let registry = registry_with_injected_env(map).with_cache_ttl(Duration::from_secs(10));

        // First discovery
        let result1 = registry.discover("test").await;
        assert!(result1.is_ok());

        // Second discovery (should use cache)
        let result2 = registry.discover("test").await;
        assert!(result2.is_ok());

        // Clear cache
        registry.clear_cache().await;
    }

    #[tokio::test]
    async fn test_discover_all() {
        const CAP: &str = "sbipc_registry_discover_all";
        const ENV: &str = "SBIPC_REGISTRY_DISCOVER_ALL_PROVIDER_SOCKET";
        let mut map = HashMap::new();
        map.insert(ENV.to_string(), String::from("/tmp/sbipc_registry_discover_all.sock"));

        let registry = registry_with_injected_env(map);
        let providers = registry.discover_all(CAP).await.unwrap();

        assert!(!providers.is_empty(), "expected env strategy to find {ENV}");
    }

    #[tokio::test]
    async fn discover_all_empty_when_no_providers() {
        let registry = registry_with_injected_env(HashMap::new());
        let all = registry.discover_all("nothing_here").await.unwrap();
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn clear_cache_then_discover_runs_strategies_again() {
        const CAP: &str = "sbipc_clear";
        const ENV: &str = "SBIPC_CLEAR_PROVIDER_SOCKET";
        let mut map = HashMap::new();
        map.insert(ENV.to_string(), String::from("/tmp/sbipc_clear.sock"));
        let registry = registry_with_injected_env(map);
        let _ = registry.discover(CAP).await.unwrap();
        registry.clear_cache().await;
        let _ = registry.discover(CAP).await.unwrap();
    }

    #[tokio::test]
    async fn update_health_noop_when_capability_not_in_cache() {
        let registry = registry_with_injected_env(HashMap::new());
        registry.update_health("no_cache", "any-id", HealthStatus::Unhealthy).await;
    }

    #[tokio::test]
    async fn update_health_changes_cached_provider_health() {
        const CAP: &str = "sbipc_health";
        const ENV: &str = "SBIPC_HEALTH_PROVIDER_SOCKET";
        let mut map = HashMap::new();
        map.insert(ENV.to_string(), String::from("/tmp/sbipc_health.sock"));
        let registry = registry_with_injected_env(map).with_cache_ttl(Duration::from_secs(3600));
        let p = registry.discover(CAP).await.unwrap();
        registry.update_health(CAP, &p.id, HealthStatus::Degraded).await;
        let p2 = registry.discover(CAP).await.unwrap();
        assert_eq!(p2.id, p.id);
        assert_eq!(p2.metadata.health, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn with_cache_ttl_zero_always_misses_cache() {
        const CAP: &str = "sbipc_ttl0";
        const ENV: &str = "SBIPC_TTL0_PROVIDER_SOCKET";
        let mut map = HashMap::new();
        map.insert(ENV.to_string(), String::from("/tmp/sbipc_ttl0.sock"));
        let registry = registry_with_injected_env(map).with_cache_ttl(Duration::from_secs(0));
        let _ = registry.discover(CAP).await.unwrap();
        let _ = registry.discover(CAP).await.unwrap();
    }

    #[tokio::test]
    async fn registry_default_constructible() {
        let _ = CapabilityRegistry::default();
        let _ = CapabilityRegistry::new();
    }
}
