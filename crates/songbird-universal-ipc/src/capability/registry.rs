//! Unified capability registry

use crate::capability::provider::{HealthStatus, Provider};
use crate::capability::strategy::{DiscoveryStrategy, EnvironmentStrategy, FilesystemStrategy};
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
    strategies: Vec<Box<dyn DiscoveryStrategy>>,

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
            Box::new(EnvironmentStrategy),
            Box::new(FilesystemStrategy::new()),
        ])
    }

    /// Create with custom strategies
    #[must_use]
    pub fn with_strategies(strategies: Vec<Box<dyn DiscoveryStrategy>>) -> Self {
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
    use super::*;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Ensure no env vars set
        std::env::remove_var("NONEXISTENT_PROVIDER_SOCKET");

        let registry = CapabilityRegistry::new();
        let result = registry.discover("nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("TEST_PROVIDER_SOCKET", "/tmp/test.sock");

        let registry = CapabilityRegistry::new().with_cache_ttl(Duration::from_secs(10));

        // First discovery
        let result1 = registry.discover("test").await;
        assert!(result1.is_ok());

        // Second discovery (should use cache)
        let result2 = registry.discover("test").await;
        assert!(result2.is_ok());

        // Clear cache
        registry.clear_cache().await;

        std::env::remove_var("TEST_PROVIDER_SOCKET");
    }

    #[tokio::test]
    async fn test_discover_all() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("CRYPTO_PROVIDER_SOCKET", "/tmp/crypto1.sock");

        let registry = CapabilityRegistry::new();
        let providers = registry.discover_all("crypto").await.unwrap();

        assert!(!providers.is_empty());

        std::env::remove_var("CRYPTO_PROVIDER_SOCKET");
    }
}
