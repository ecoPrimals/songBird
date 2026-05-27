// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core discovery engine: capability-based orchestration, cache, env/mDNS/registry paths.

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::announcement;
use super::constants::MIN_TIMEOUT_FOR_SLOW_DISCOVERY_PATHS;
use super::types::{DiscoveredService, DiscoveryMethod};

/// Runtime service discovery engine
///
/// Discovers services by capability at runtime with zero hardcoded knowledge.
pub struct RuntimeDiscoveryEngine {
    /// Expected capabilities this engine should be able to resolve
    capabilities: Vec<String>,

    /// Discovery timeout (will be used in timeout wrapper around discovery methods)
    timeout: Duration,

    /// Cache of discovered services
    cache: Arc<RwLock<HashMap<String, DiscoveredService>>>,

    /// Cache TTL
    cache_ttl: Duration,
}

impl RuntimeDiscoveryEngine {
    /// Create new discovery engine
    #[must_use]
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(5))
    }

    /// Create engine with a custom discovery timeout (e.g. tests use 1ms).
    #[must_use]
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            capabilities: Vec::new(),
            timeout,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Create with specific capabilities
    #[must_use]
    pub fn with_capabilities(capabilities: Vec<String>) -> Self {
        Self {
            capabilities,
            timeout: Duration::from_secs(5),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300),
        }
    }

    /// Discover service by capability (NO hardcoding)
    pub async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> SongbirdResult<DiscoveredService> {
        if let Some(cached) = self.check_cache(capability).await {
            return Ok(cached);
        }

        if let Ok(service) =
            Self::from_environment_with(capability, &|k| songbird_process_env::var(k))
        {
            self.update_cache(capability, &service).await;
            return Ok(service);
        }

        if let Ok(service) = self.discover_mdns(capability).await {
            self.update_cache(capability, &service).await;
            return Ok(service);
        }

        if let Ok(service) = self.discover_registry(capability).await {
            self.update_cache(capability, &service).await;
            return Ok(service);
        }

        if let Ok(service) = announcement::wait_for_announcement(self.timeout, capability).await {
            self.update_cache(capability, &service).await;
            return Ok(service);
        }

        Err(SongbirdError::discovery(format!(
            "No service found for capability '{capability}' after all discovery methods"
        )))
    }

    /// Get endpoint from environment variable (`{CAPABILITY}_ENDPOINT`).
    pub(crate) fn from_environment_with(
        capability: &str,
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> Result<DiscoveredService, SongbirdError> {
        let env_var = format!("{}_ENDPOINT", capability.to_uppercase());

        let endpoint = env(&env_var).map_err(|_| {
            SongbirdError::configuration(format!("Environment variable {env_var} not set"))
        })?;

        Ok(DiscoveredService {
            capability: capability.to_string(),
            endpoint,
            discovered_via: DiscoveryMethod::Environment,
            health_score: 1.0,
            last_seen: std::time::SystemTime::now(),
        })
    }

    async fn discover_mdns(&self, capability: &str) -> SongbirdResult<DiscoveredService> {
        use tracing::debug;

        debug!("Attempting mDNS discovery for capability '{capability}'");

        if self.timeout < MIN_TIMEOUT_FOR_SLOW_DISCOVERY_PATHS {
            return Err(SongbirdError::discovery(format!(
                "mDNS discovery skipped for capability '{capability}' (timeout below slow-path minimum)"
            )));
        }

        #[cfg(feature = "mdns")]
        {
            use crate::capability_based_runtime_discovery::mdns::MdnsDiscovery;

            let mdns = MdnsDiscovery::new(None);
            match mdns.discover_by_capability(capability, Some(self.timeout)).await {
                Ok(services) => {
                    if let Some(service_info) = services.first() {
                        debug!(
                            "mDNS discovered service at {} for capability '{capability}'",
                            service_info.address
                        );
                        return Ok(DiscoveredService {
                            capability: capability.to_string(),
                            endpoint: format!("http://{}", service_info.address),
                            discovered_via: DiscoveryMethod::MDNS,
                            health_score: 1.0,
                            last_seen: std::time::SystemTime::now(),
                        });
                    }
                }
                Err(e) => {
                    debug!("mDNS discovery failed for '{capability}': {e}");
                }
            }
        }

        #[cfg(not(feature = "mdns"))]
        {
            use tracing::warn;
            warn!("mDNS discovery requested but mdns feature not enabled");
        }

        Err(SongbirdError::discovery(format!(
            "No service found via mDNS for capability '{capability}'"
        )))
    }

    async fn discover_registry(&self, capability: &str) -> SongbirdResult<DiscoveredService> {
        use crate::capability_based_runtime_discovery::CapabilityRequest;
        use crate::capability_based_runtime_discovery::service_registry::ServiceRegistryDiscovery;
        use tracing::{debug, info};

        let Ok(registry_endpoint) = songbird_process_env::var("REGISTRY_ENDPOINT") else {
            return Err(SongbirdError::configuration(
                "No registry endpoint configured (REGISTRY_ENDPOINT not set)",
            ));
        };

        debug!(
            "Querying service registry at '{}' for capability '{}'",
            registry_endpoint, capability
        );

        let discovery = ServiceRegistryDiscovery::new(&registry_endpoint);
        let request = CapabilityRequest {
            capability: capability.to_string(),
            required_features: Vec::new(),
            optional_features: Vec::new(),
            preferences: Vec::new(),
            min_sla: None,
        };

        match discovery.discover(&request).await {
            Ok(provider) => {
                info!(
                    "Discovered provider '{}' for capability '{}' from registry",
                    provider.name, capability
                );

                Ok(DiscoveredService {
                    capability: capability.to_string(),
                    endpoint: provider.endpoint,
                    discovered_via: DiscoveryMethod::Registry,
                    health_score: 1.0,
                    last_seen: std::time::SystemTime::now(),
                })
            }
            Err(e) => {
                debug!("Registry query failed for '{capability}': {e}");
                Err(SongbirdError::discovery(format!(
                    "No service found in registry for capability '{capability}': {e}"
                )))
            }
        }
    }

    pub(crate) async fn check_cache(&self, capability: &str) -> Option<DiscoveredService> {
        if let Some(service) = self.cache.read().await.get(capability)
            && let Ok(elapsed) = service.last_seen.elapsed()
            && elapsed < self.cache_ttl
        {
            return Some(service.clone());
        }

        None
    }

    pub(crate) async fn update_cache(&self, capability: &str, service: &DiscoveredService) {
        let mut cache = self.cache.write().await;
        cache.insert(capability.to_string(), service.clone());
    }

    /// Returns the list of capabilities this engine expects to resolve
    #[must_use]
    pub fn required_capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

impl Default for RuntimeDiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn new_engine_has_5s_timeout() {
        let engine = RuntimeDiscoveryEngine::new();
        assert_eq!(engine.timeout, Duration::from_secs(5));
        assert!(engine.capabilities.is_empty());
    }

    #[test]
    fn with_timeout_sets_custom_timeout() {
        let engine = RuntimeDiscoveryEngine::with_timeout(Duration::from_millis(100));
        assert_eq!(engine.timeout, Duration::from_millis(100));
    }

    #[test]
    fn with_capabilities_stores_capabilities() {
        let caps = vec!["http".into(), "crypto".into()];
        let engine = RuntimeDiscoveryEngine::with_capabilities(caps.clone());
        assert_eq!(engine.required_capabilities(), &caps);
    }

    #[test]
    fn default_engine_matches_new() {
        let engine = RuntimeDiscoveryEngine::default();
        assert_eq!(engine.timeout, Duration::from_secs(5));
        assert!(engine.capabilities.is_empty());
    }

    #[test]
    fn from_environment_with_finds_set_var() {
        let env = |name: &str| -> Result<String, std::env::VarError> {
            if name == "CRYPTO_ENDPOINT" {
                Ok("http://localhost:9999".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        };
        let service = RuntimeDiscoveryEngine::from_environment_with("crypto", &env).unwrap();
        assert_eq!(service.capability, "crypto");
        assert_eq!(service.endpoint, "http://localhost:9999");
        assert_eq!(service.discovered_via, DiscoveryMethod::Environment);
        assert!((service.health_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn from_environment_with_uppercases_capability() {
        let env = |name: &str| -> Result<String, std::env::VarError> {
            if name == "HTTP_REQUEST_ENDPOINT" {
                Ok("http://proxy:8080".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        };
        let service = RuntimeDiscoveryEngine::from_environment_with("http_request", &env).unwrap();
        assert_eq!(service.endpoint, "http://proxy:8080");
    }

    #[test]
    fn from_environment_with_missing_var_errors() {
        let env =
            |_: &str| -> Result<String, std::env::VarError> { Err(std::env::VarError::NotPresent) };
        let result = RuntimeDiscoveryEngine::from_environment_with("storage", &env);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cache_miss_returns_none() {
        let engine = RuntimeDiscoveryEngine::new();
        assert!(engine.check_cache("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn update_and_check_cache_roundtrip() {
        let engine = RuntimeDiscoveryEngine::new();
        let service = DiscoveredService {
            capability: "test".into(),
            endpoint: "http://test:1234".into(),
            discovered_via: DiscoveryMethod::Environment,
            health_score: 0.9,
            last_seen: std::time::SystemTime::now(),
        };
        engine.update_cache("test", &service).await;
        let cached = engine.check_cache("test").await.unwrap();
        assert_eq!(cached.endpoint, "http://test:1234");
        assert_eq!(cached.capability, "test");
    }

    #[tokio::test]
    async fn expired_cache_entry_returns_none() {
        let mut engine = RuntimeDiscoveryEngine::new();
        engine.cache_ttl = Duration::from_millis(1);

        let service = DiscoveredService {
            capability: "expired".into(),
            endpoint: "http://stale:5555".into(),
            discovered_via: DiscoveryMethod::Registry,
            health_score: 1.0,
            last_seen: std::time::SystemTime::now() - Duration::from_secs(60),
        };
        engine.update_cache("expired", &service).await;
        assert!(engine.check_cache("expired").await.is_none());
    }

    #[tokio::test]
    async fn discover_by_capability_finds_from_env() {
        let unique_cap = "songbirdconfigtest_engine_env_discover";
        let env_var = format!("{}_ENDPOINT", unique_cap.to_uppercase());
        songbird_process_env::set_var(&env_var, "http://via-env:7777");
        let engine = RuntimeDiscoveryEngine::with_timeout(Duration::from_millis(1));
        let result = engine.discover_by_capability(unique_cap).await;
        songbird_process_env::remove_var(&env_var);
        let service = result.unwrap();
        assert_eq!(service.endpoint, "http://via-env:7777");
        assert_eq!(service.discovered_via, DiscoveryMethod::Environment);
    }

    #[tokio::test]
    async fn discover_by_capability_returns_cached() {
        let engine = RuntimeDiscoveryEngine::with_timeout(Duration::from_millis(1));
        let service = DiscoveredService {
            capability: "cached_cap".into(),
            endpoint: "http://cached:1111".into(),
            discovered_via: DiscoveryMethod::MDNS,
            health_score: 0.95,
            last_seen: std::time::SystemTime::now(),
        };
        engine.update_cache("cached_cap", &service).await;
        let result = engine.discover_by_capability("cached_cap").await.unwrap();
        assert_eq!(result.endpoint, "http://cached:1111");
    }

    #[tokio::test]
    async fn discover_by_capability_fails_when_nothing_available() {
        let engine = RuntimeDiscoveryEngine::with_timeout(Duration::from_millis(1));
        let result = engine.discover_by_capability("nonexistent_capability_xyz").await;
        assert!(result.is_err());
    }
}
