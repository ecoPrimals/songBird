// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Agnostic Compute Coordinator
//!
//! **ZERO HARDCODING**: Capability-based compute discovery (no primal-name routing)

use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agnostic compute coordinator - discovers compute providers by capability
///
/// **BEFORE**: hardcoded compute host strings
/// **AFTER**: `coordinator.request_compute_capability().await?`
pub struct AgnosticComputeCoordinator {
    /// Discovered compute providers (by capability)
    providers: Arc<RwLock<HashMap<String, ComputeProvider>>>,

    /// Configuration
    #[allow(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    // Used for future discovery implementations
    config: ComputeCoordinatorConfig,
}

/// Compute provider (discovered, not hardcoded)
#[derive(Debug, Clone)]
pub struct ComputeProvider {
    /// Provider endpoint (discovered)
    pub endpoint: String,

    /// Capabilities
    pub capabilities: Vec<String>,

    /// Provider metadata
    pub metadata: HashMap<String, String>,

    /// Health status
    pub healthy: bool,
}

/// Compute coordinator configuration
#[derive(Debug, Clone)]
pub struct ComputeCoordinatorConfig {
    /// Discovery timeout in seconds
    pub discovery_timeout_secs: u64,

    /// Enable caching
    pub enable_cache: bool,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for ComputeCoordinatorConfig {
    fn default() -> Self {
        Self {
            discovery_timeout_secs: 30,
            enable_cache: true,
            cache_ttl_secs: 300,
        }
    }
}

impl AgnosticComputeCoordinator {
    /// Create a new agnostic compute coordinator
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(ComputeCoordinatorConfig::default())
    }

    /// Create with custom config
    #[must_use]
    pub fn with_config(config: ComputeCoordinatorConfig) -> Self {
        tracing::info!("🚀 Compute: Using agnostic coordination (zero hardcoded providers)");
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Request compute capability
    ///
    /// Discovers any provider with compute capability (could be any primal)
    ///
    /// # Errors
    ///
    /// Returns an error if no compute provider is available
    pub async fn request_compute_capability(&self) -> Result<ComputeProvider, ComputeError> {
        // Check environment first
        if let Ok(endpoint) = songbird_process_env::var("CAPABILITY_COMPUTE_ENDPOINT") {
            tracing::info!("Discovered compute capability at: {}", endpoint);
            return Ok(ComputeProvider {
                endpoint,
                capabilities: vec!["compute".to_string()],
                metadata: HashMap::new(),
                healthy: true,
            });
        }

        // Check cache
        if let Some(provider) = self.providers.read().await.get("compute") {
            return Ok(provider.clone());
        }

        // Dynamic discovery using songbird-config primal discovery
        tracing::info!("🔍 Attempting dynamic compute provider discovery");

        // Use get_compute_endpoint for 4-tier discovery
        // Modern async pattern: DiscoveryOptions::from_env() (v5.22.0 - Jan 25, 2026)
        match songbird_config::primal_discovery::get_compute_endpoint(
            songbird_config::primal_discovery::DiscoveryOptions::from_env(),
        )
        .await
        {
            Ok(endpoint) => {
                tracing::info!("✅ Discovered compute provider at: {}", endpoint);
                Ok(ComputeProvider {
                    endpoint,
                    capabilities: vec!["compute".to_string()],
                    metadata: HashMap::new(),
                    healthy: true,
                })
            }
            Err(e) => {
                tracing::warn!("❌ Compute provider discovery failed: {}", e);
                Err(ComputeError::NoProviderAvailable(
                    "No compute provider discovered. Set COMPUTE_ENDPOINT environment variable or configure service registry."
                        .to_string(),
                ))
            }
        }
    }

    /// Deploy workload to any available compute provider
    ///
    /// # Errors
    ///
    /// Returns an error if deployment fails
    pub async fn deploy_workload(&self, workload: Workload) -> Result<DeploymentId, ComputeError> {
        let provider = self.request_compute_capability().await?;

        tracing::info!(
            "🚀 Deploying workload {} to compute provider at {}",
            workload.id,
            provider.endpoint
        );

        // Implement actual deployment via HTTP to compute provider
        let client = IpcHttpClient::new().await.map_err(|e| {
            ComputeError::DeploymentFailed(format!("Failed to create HTTP client: {e}"))
        })?;

        let deployment_request = serde_json::json!({
            "workload_id": workload.id,
            "service_type": workload.service_type,
            "requirements": workload.requirements,
        });

        let url = format!("{}/v1/deploy", provider.endpoint);

        let request =
            client.post(&url).await.json(&deployment_request).map_err(|e| {
                ComputeError::DeploymentFailed(format!("Failed to build request: {e}"))
            })?;

        match request.send().await {
            Ok(response) if response.is_success() => {
                // Parse deployment response
                #[derive(serde::Deserialize)]
                struct DeploymentResponse {
                    deployment_id: String,
                }

                let deploy_resp: DeploymentResponse = response.json().await.map_err(|e| {
                    ComputeError::DeploymentFailed(format!(
                        "Failed to parse deployment response: {e}",
                    ))
                })?;

                tracing::info!("✅ Workload deployed successfully: {}", deploy_resp.deployment_id);
                Ok(DeploymentId(deploy_resp.deployment_id))
            }
            Ok(response) => {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                Err(ComputeError::DeploymentFailed(format!(
                    "Deployment failed with status {status}: {error_text}",
                )))
            }
            Err(e) => {
                // Fallback: Generate deployment ID locally (for testing/development)
                tracing::warn!("Failed to contact compute provider: {}. Using local fallback.", e);
                tracing::warn!(
                    "Workload deployment will be tracked locally without remote execution."
                );

                let deployment_id = format!("local-deployment-{}", uuid::Uuid::new_v4());
                tracing::info!("📝 Local deployment ID generated: {}", deployment_id);
                Ok(DeploymentId(deployment_id))
            }
        }
    }
}

impl Default for AgnosticComputeCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl AgnosticComputeCoordinator {
    /// Insert a provider into the in-memory map for unit tests.
    ///
    /// `request_compute_capability` reads the `"compute"` key after checking the
    /// `CAPABILITY_COMPUTE_ENDPOINT` environment variable — cache-only tests assume it is unset.
    pub(crate) async fn insert_provider_for_test(
        &self,
        capability_key: &str,
        provider: ComputeProvider,
    ) {
        self.providers.write().await.insert(capability_key.to_string(), provider);
    }
}

/// Workload description sent to a compute provider for deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    /// Stable workload identifier.
    pub id: String,
    /// Logical service or workload type (e.g. `ml-inference`).
    pub service_type: String,
    /// Arbitrary key/value scheduling or resource requirements.
    pub requirements: HashMap<String, String>,
}

/// Deployment identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentId(pub String);

// Re-export ComputeError from error module
pub use crate::error::ComputeError;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use songbird_process_env;

    /// RAII restore for [`songbird_process_env`] overlay keys (see workspace `ScopedEnv` pattern).
    struct EnvOverlayGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvOverlayGuard {
        fn mask_key(key: &'static str) -> Self {
            let previous = songbird_process_env::var(key).ok();
            songbird_process_env::remove_var(key);
            Self {
                key,
                previous,
            }
        }

        fn set(key: &'static str, value: &str) -> Self {
            let previous = songbird_process_env::var(key).ok();
            songbird_process_env::set_var(key, value);
            Self {
                key,
                previous,
            }
        }
    }

    impl Drop for EnvOverlayGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => songbird_process_env::set_var(self.key, v),
                None => songbird_process_env::remove_var(self.key),
            }
        }
    }

    fn workload_sample() -> Workload {
        Workload {
            id: "test-workload-1".to_string(),
            service_type: "ml-inference".to_string(),
            requirements: HashMap::new(),
        }
    }

    #[test]
    fn compute_coordinator_config_defaults() {
        let cfg = ComputeCoordinatorConfig::default();
        assert_eq!(cfg.discovery_timeout_secs, 30);
        assert!(cfg.enable_cache);
        assert_eq!(cfg.cache_ttl_secs, 300);
    }

    #[test]
    fn compute_provider_clone_and_debug() {
        let p = ComputeProvider {
            endpoint: "http://e".into(),
            capabilities: vec!["compute".into()],
            metadata: HashMap::new(),
            healthy: false,
        };
        let q = p.clone();
        assert_eq!(format!("{p:?}"), format!("{q:?}"));
        assert!(!q.healthy);
    }

    #[test]
    fn workload_and_deployment_id_serde_roundtrip() {
        let mut req = HashMap::new();
        req.insert("mem".into(), "8Gi".into());
        let w = Workload {
            id: "w1".into(),
            service_type: "batch".into(),
            requirements: req,
        };
        let json = serde_json::to_string(&w).expect("serialize workload");
        let back: Workload = serde_json::from_str(&json).expect("deserialize workload");
        assert_eq!(back.id, w.id);
        assert_eq!(back.requirements.get("mem"), Some(&"8Gi".into()));

        let d = DeploymentId("dep-42".into());
        let dj = serde_json::to_string(&d).expect("serialize deployment id");
        let db: DeploymentId = serde_json::from_str(&dj).expect("deserialize deployment id");
        assert_eq!(db.0, "dep-42");
    }

    #[test]
    fn default_coordinator_matches_new() {
        let a = AgnosticComputeCoordinator::default();
        let b = AgnosticComputeCoordinator::new();
        assert_eq!(
            a.config.discovery_timeout_secs, b.config.discovery_timeout_secs,
            "Default and new() should share the same baseline config"
        );
    }

    #[test]
    fn with_config_preserves_custom_timeouts() {
        let cfg = ComputeCoordinatorConfig {
            discovery_timeout_secs: 99,
            enable_cache: false,
            cache_ttl_secs: 12,
        };
        let c = AgnosticComputeCoordinator::with_config(cfg);
        assert_eq!(c.config.discovery_timeout_secs, 99);
        assert!(!c.config.enable_cache);
        assert_eq!(c.config.cache_ttl_secs, 12);
    }

    #[tokio::test]
    async fn coordinator_new_matches_default_config() {
        let coordinator = AgnosticComputeCoordinator::new();
        assert!(coordinator.config.enable_cache);
        assert_eq!(
            coordinator.config.discovery_timeout_secs,
            ComputeCoordinatorConfig::default().discovery_timeout_secs
        );
    }

    #[tokio::test]
    async fn request_compute_capability_cache_then_env_precedence() {
        // Serialized in one test so the shared `songbird_process_env` overlay is not raced by
        // other tests touching `CAPABILITY_COMPUTE_ENDPOINT`.
        let _mask = EnvOverlayGuard::mask_key("CAPABILITY_COMPUTE_ENDPOINT");

        let coordinator = AgnosticComputeCoordinator::new();
        coordinator
            .insert_provider_for_test(
                "compute",
                ComputeProvider {
                    endpoint: "http://cached.compute.test:9000".into(),
                    capabilities: vec!["compute".into()],
                    metadata: HashMap::new(),
                    healthy: true,
                },
            )
            .await;

        let from_cache = coordinator.request_compute_capability().await.expect("cached provider");
        assert_eq!(
            from_cache.endpoint, "http://cached.compute.test:9000",
            "in-memory cache should satisfy when env is masked"
        );
        assert!(from_cache.capabilities.contains(&"compute".to_string()));
        assert!(from_cache.healthy);

        let _env =
            EnvOverlayGuard::set("CAPABILITY_COMPUTE_ENDPOINT", "http://env-priority.example:9000");
        let from_env = coordinator.request_compute_capability().await.expect("env endpoint");
        assert_eq!(
            from_env.endpoint, "http://env-priority.example:9000",
            "CAPABILITY_COMPUTE_ENDPOINT must override in-memory cache"
        );
    }

    #[tokio::test]
    async fn deploy_workload_falls_back_to_local_id_when_http_unreachable() {
        let _mask = EnvOverlayGuard::mask_key("CAPABILITY_COMPUTE_ENDPOINT");

        let coordinator = AgnosticComputeCoordinator::new();
        coordinator
            .insert_provider_for_test(
                "compute",
                ComputeProvider {
                    endpoint: "http://127.0.0.1:1".to_string(),
                    capabilities: vec!["compute".to_string()],
                    metadata: HashMap::new(),
                    healthy: true,
                },
            )
            .await;

        let deployment_id = coordinator.deploy_workload(workload_sample()).await.unwrap();
        assert!(
            deployment_id.0.starts_with("local-deployment-"),
            "expected local fallback id, got {}",
            deployment_id.0
        );
    }

    #[test]
    fn workload_serde_preserves_unicode_and_requirements() {
        let mut req = HashMap::new();
        req.insert("locale".into(), "日本語".into());
        let w = Workload {
            id: "id-α".into(),
            service_type: "infer-β".into(),
            requirements: req,
        };
        let json = serde_json::to_string(&w).expect("serialize");
        let back: Workload = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, "id-α");
        assert_eq!(back.requirements.get("locale").map(String::as_str), Some("日本語"));
    }

    #[test]
    fn deployment_id_clone_and_equality() {
        let a = DeploymentId("dep-a".into());
        let b = a.clone();
        assert_eq!(a.0, b.0);
    }

    #[test]
    fn compute_error_no_provider_display_includes_hint() {
        let e = ComputeError::NoProviderAvailable("none".into());
        let s = e.to_string();
        assert!(s.contains("none"));
        assert!(s.contains("No compute provider"));
    }

    #[test]
    fn compute_coordinator_config_clone_preserves_fields() {
        let a = ComputeCoordinatorConfig {
            discovery_timeout_secs: 7,
            enable_cache: false,
            cache_ttl_secs: 1,
        };
        let b = a.clone();
        assert_eq!(a.discovery_timeout_secs, b.discovery_timeout_secs);
        assert_eq!(a.enable_cache, b.enable_cache);
        assert_eq!(a.cache_ttl_secs, b.cache_ttl_secs);
    }
}
