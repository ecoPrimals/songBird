// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Runtime Endpoint Resolution
//!
//! Modern, capability-based endpoint resolution that replaces hardcoded values
//! with runtime discovery. Each primal only knows itself; all inter-primal
//! communication is discovered dynamically.
//!
//! ## Evolution from Hardcoding
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded
//! let endpoint = "http://localhost:8080";
//!
//! // ✅ NEW: Capability-based discovery
//! let endpoint = resolver.resolve_capability("compute").await?;
//! ```

use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::capability_discovery::{CapabilityDiscovery, ServiceEndpoint};

/// Runtime endpoint resolver using capability-based discovery
///
/// This resolver provides a modern alternative to hardcoded endpoints,
/// supporting multiple discovery strategies with fallback chains.
pub struct RuntimeEndpointResolver {
    /// Capability discovery engine
    discovery: Arc<CapabilityDiscovery>,

    /// Local service registry (for self-knowledge)
    local_services: Arc<RwLock<HashMap<String, String>>>,

    /// Fallback configuration
    fallbacks: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl RuntimeEndpointResolver {
    /// Create new resolver with default discovery
    #[must_use]
    pub fn new() -> Self {
        Self {
            discovery: Arc::new(CapabilityDiscovery::new()),
            local_services: Arc::new(RwLock::new(HashMap::new())),
            fallbacks: Arc::new(RwLock::new(Self::default_fallbacks())),
        }
    }

    /// Create with custom discovery engine
    #[must_use]
    pub fn with_discovery(discovery: CapabilityDiscovery) -> Self {
        Self {
            discovery: Arc::new(discovery),
            local_services: Arc::new(RwLock::new(HashMap::new())),
            fallbacks: Arc::new(RwLock::new(Self::default_fallbacks())),
        }
    }

    /// Register local service (self-knowledge)
    ///
    /// Each primal registers only its own services.
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but returns `Result` for future extensibility.
    pub async fn register_local_service(
        &self,
        capability: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> SongbirdResult<()> {
        let capability = capability.into();
        let endpoint = endpoint.into();

        info!("🔍 Registering local service: {} -> {}", capability, endpoint);
        self.local_services.write().await.insert(capability, endpoint);
        Ok(())
    }

    /// Resolve endpoint by capability
    ///
    /// Discovery order:
    /// 1. Local services (self-knowledge)
    /// 2. Environment variables
    /// 3. Runtime discovery
    /// 4. Configured fallbacks
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let resolver = RuntimeEndpointResolver::new();
    ///
    /// // Resolve by capability, not by primal name
    /// let compute_endpoint = resolver.resolve_capability("compute").await?;
    /// let storage_endpoint = resolver.resolve_capability("storage").await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the capability cannot be resolved through any discovery method
    /// (local services, environment variables, runtime discovery, or fallbacks).
    pub async fn resolve_capability(&self, capability: &str) -> SongbirdResult<String> {
        debug!("🔍 Resolving capability: {}", capability);

        // 1. Check local services first (self-knowledge)
        if let Some(endpoint) = self.local_services.read().await.get(capability) {
            info!("✅ Resolved {} from local services: {}", capability, endpoint);
            return Ok(endpoint.clone());
        }

        // 2. Check environment variables
        if let Some(endpoint) = Self::try_env_resolution(capability) {
            info!("✅ Resolved {} from environment: {}", capability, endpoint);
            return Ok(endpoint);
        }

        // 3. Try runtime discovery
        match self.discovery.find_providers_by_capability(capability).await {
            Ok(endpoints) if !endpoints.is_empty() => {
                let best = Self::select_best_endpoint(&endpoints);
                info!("✅ Resolved {} from discovery: {}", capability, best.url);
                return Ok(best.url.clone());
            }
            Ok(_) => debug!("No endpoints found via discovery for {}", capability),
            Err(e) => debug!("Discovery failed for {}: {}", capability, e),
        }

        // 4. Fall back to configured defaults
        if let Some(endpoint) = self.try_fallback_resolution(capability).await {
            warn!("⚠️  Using fallback for {}: {}", capability, endpoint);
            return Ok(endpoint);
        }

        Err(SongbirdError::discovery(format!(
            "Could not resolve capability '{capability}' through any discovery method"
        )))
    }

    /// Try to resolve from environment variables
    ///
    /// Supports multiple naming conventions:
    /// - `{CAPABILITY}_ENDPOINT` (e.g., `COMPUTE_ENDPOINT`)
    /// - `SONGBIRD_{CAPABILITY}_URL` (e.g., `SONGBIRD_COMPUTE_URL`)
    fn try_env_resolution(capability: &str) -> Option<String> {
        Self::try_env_resolution_with(capability, |k| songbird_process_env::var(k))
    }

    /// Injectable version for testing without global env mutation
    fn try_env_resolution_with(
        capability: &str,
        env: impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> Option<String> {
        let capability_upper = capability.to_uppercase();

        let env_keys = [
            format!("{capability_upper}_ENDPOINT"),
            format!("SONGBIRD_{capability_upper}_URL"),
            format!("SONGBIRD_{capability_upper}_ENDPOINT"),
            format!("{capability_upper}_URL"),
        ];

        for key in env_keys {
            if let Ok(value) = env(&key)
                && !value.is_empty()
            {
                return Some(value);
            }
        }

        None
    }

    /// Try configured fallback endpoints
    async fn try_fallback_resolution(&self, capability: &str) -> Option<String> {
        let fallbacks = self.fallbacks.read().await;
        fallbacks.get(capability).and_then(|endpoints| endpoints.first()).cloned()
    }

    /// Select best endpoint from discovered options
    ///
    /// Criteria:
    /// 1. Health score
    /// 2. Response time (if available)
    /// 3. Recency
    ///
    /// # Panics
    ///
    /// Panics if called with an empty `endpoints` slice. The caller must ensure the slice
    /// is non-empty before calling this function.
    #[expect(clippy::expect_used, reason = "invariant enforced at call site (non-empty endpoints)")]
    fn select_best_endpoint(endpoints: &[ServiceEndpoint]) -> &ServiceEndpoint {
        endpoints
            .iter()
            .max_by(|a, b| {
                a.health_score.partial_cmp(&b.health_score).unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("BUG: select_best_endpoint called with empty endpoints - logic error at call site (line 109)")
    }

    /// Default fallback endpoints
    ///
    /// These are ONLY used as last resort when all discovery methods fail.
    /// They use localhost for development/testing scenarios.
    fn default_fallbacks() -> HashMap<String, Vec<String>> {
        let mut fallbacks = HashMap::new();

        // Development fallbacks (localhost only, for testing)
        fallbacks.insert(
            "orchestrator".to_string(),
            vec![SafeEnv::get_or_default(
                "SONGBIRD_ORCHESTRATOR_FALLBACK",
                "http://localhost:8080",
            )],
        );

        fallbacks.insert(
            "registry".to_string(),
            vec![SafeEnv::get_or_default("SONGBIRD_REGISTRY_FALLBACK", "http://localhost:8081")],
        );

        fallbacks
    }
}

impl Default for RuntimeEndpointResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration helper to evolve from hardcoded patterns
///
/// This struct helps in migrating existing code from hardcoded
/// endpoints to capability-based discovery.
pub struct EndpointMigrationHelper {
    resolver: Arc<RuntimeEndpointResolver>,
}

impl EndpointMigrationHelper {
    /// Create new migration helper
    #[must_use]
    pub fn new(resolver: RuntimeEndpointResolver) -> Self {
        Self {
            resolver: Arc::new(resolver),
        }
    }

    /// Migrate a hardcoded endpoint pattern
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Before: let addr = "http://localhost:8080";
    /// let addr = helper.migrate_endpoint(
    ///     "http://localhost:8080",
    ///     "orchestrator"
    /// ).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the capability cannot be resolved through any discovery method.
    pub async fn migrate_endpoint(
        &self,
        _hardcoded: &str, // For documentation/logging only
        capability: &str,
    ) -> SongbirdResult<String> {
        self.resolver.resolve_capability(capability).await
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_service_resolution() {
        let resolver = RuntimeEndpointResolver::new();

        resolver
            .register_local_service("my-service", "http://my-endpoint:9000")
            .await
            .expect("register local");

        let endpoint = resolver.resolve_capability("my-service").await.expect("resolve");
        assert_eq!(endpoint, "http://my-endpoint:9000");
    }

    #[test]
    fn test_env_resolution_compute_endpoint() {
        let result = RuntimeEndpointResolver::try_env_resolution_with("compute", |key| match key {
            "COMPUTE_ENDPOINT" => Ok("http://env-compute:8080".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert_eq!(result, Some("http://env-compute:8080".to_string()));
    }

    #[test]
    fn test_env_resolution_songbird_url_variant() {
        let result = RuntimeEndpointResolver::try_env_resolution_with("storage", |key| match key {
            "SONGBIRD_STORAGE_URL" => Ok("http://storage-from-songbird:9000".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert_eq!(result, Some("http://storage-from-songbird:9000".to_string()));
    }

    #[test]
    fn test_try_env_resolution_skips_empty_values() {
        let result = RuntimeEndpointResolver::try_env_resolution_with("netsvc", |key| match key {
            "NETSVC_ENDPOINT" => Ok(String::new()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert!(result.is_none());
    }

    #[test]
    fn test_select_best_endpoint_prefers_health_score() {
        let endpoints = vec![
            ServiceEndpoint {
                id: "a".to_string(),
                url: "http://low".to_string(),
                capabilities: vec![],
                health_score: 0.2,
                last_seen: std::time::SystemTime::UNIX_EPOCH,
            },
            ServiceEndpoint {
                id: "b".to_string(),
                url: "http://high".to_string(),
                capabilities: vec![],
                health_score: 0.95,
                last_seen: std::time::SystemTime::UNIX_EPOCH,
            },
        ];
        let best = RuntimeEndpointResolver::select_best_endpoint(&endpoints);
        assert_eq!(best.url, "http://high");
    }

    #[test]
    fn test_fallback_is_configured_for_orchestrator() {
        let fallbacks = RuntimeEndpointResolver::default_fallbacks();
        let orch = fallbacks.get("orchestrator").expect("orchestrator fallback");
        assert!(!orch.is_empty());
        assert!(orch[0].contains("8080"), "url={}", orch[0]);
    }

    #[tokio::test]
    async fn test_migration_helper_delegates_to_resolver() {
        let resolver = RuntimeEndpointResolver::new();
        resolver.register_local_service("legacy", "http://legacy:1").await.expect("register");
        let helper = EndpointMigrationHelper::new(resolver);
        let url = helper.migrate_endpoint("ignored", "legacy").await.expect("migrate");
        assert_eq!(url, "http://legacy:1");
    }

    #[test]
    fn test_fallbacks_configuration() {
        let fallbacks = RuntimeEndpointResolver::default_fallbacks();
        assert!(fallbacks.contains_key("orchestrator"));
        assert!(fallbacks.contains_key("registry"));
    }
}
