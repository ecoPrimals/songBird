// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability Discovery Module
//!
//! Handles discovery of primals and their capabilities through multiple channels:
//! - Environment variables (CAPABILITY_* endpoints)
//! - Network scanning (when enabled)
//! - Capability inference (pattern-based)
//!
//! Part of the smart refactoring from monolithic adapter.rs (1080 lines) → focused modules

use chrono::Utc;
use songbird_types::SafeEnv;
use songbird_types::constants::LOCALHOST;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::types::ServiceInfo;

use super::super::error::CapabilityError;
use super::super::registry::CapabilityRegistry;
use super::super::types::{Capability, DiscoveryConfig};

/// Capability discovery component
#[derive(Debug, Clone)]
pub struct CapabilityDiscovery {
    /// Shared capability registry
    registry: Arc<RwLock<CapabilityRegistry>>,
    /// Discovery configuration
    config: DiscoveryConfig,
}

impl CapabilityDiscovery {
    /// Create new capability discovery component
    pub const fn new(registry: Arc<RwLock<CapabilityRegistry>>, config: DiscoveryConfig) -> Self {
        Self {
            registry,
            config,
        }
    }

    /// Discover capabilities for a primal by name
    ///
    /// # Errors
    ///
    /// Returns an error if the primal is unreachable or does not respond with valid capabilities
    pub async fn discover_primal_capabilities<F, Fut>(
        &self,
        primal_name: &str,
        query_fn: F,
    ) -> Result<Vec<Capability>, CapabilityError>
    where
        F: Fn(&str) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<Capability>, CapabilityError>> + Send,
    {
        info!("🔍 Discovering capabilities for primal: {}", primal_name);

        // Get primal endpoint from environment
        // NOTE: Defaults to loopback, but should be overridden by environment discovery
        // This is a fallback - real deployment uses capability discovery
        let capability_host = SafeEnv::get_or_default(
            "UNIVERSAL_CAPABILITY_HOST",
            LOCALHOST, // Fallback for local development only
        );
        let capability_port = SafeEnv::get_port(
            "UNIVERSAL_CAPABILITY_PORT",
            songbird_config::defaults::ports::orchestrator_port(),
        );
        let endpoint = format!("http://{capability_host}:{capability_port}/{primal_name}");

        // Attempt capability discovery via HTTP
        match query_fn(&endpoint).await {
            Ok(capabilities) => {
                // Update registry
                let mut registry = self.registry.write().await;
                registry.primal_capabilities.insert(primal_name.to_string(), capabilities.clone());
                registry.last_updated.insert(primal_name.to_string(), Utc::now());

                // Update capability providers index
                for capability in &capabilities {
                    registry
                        .capability_providers
                        .entry(capability.capability_type.clone())
                        .or_insert_with(Vec::new)
                        .push(primal_name.to_string());
                }

                info!("✅ Discovered {} capabilities for {}", capabilities.len(), primal_name);
                Ok(capabilities)
            }
            Err(e) => {
                warn!("❌ Failed to discover capabilities for {}: {}", primal_name, e);
                Err(e)
            }
        }
    }

    /// Find all primals that provide a specific capability
    ///
    /// Merges environment hints, optional HTTP discovery (orchestrator `/capabilities` and
    /// discovery `/services`, same URLs as [`crate::unified_adapter::UnifiedUniversalAdapter`]),
    /// inference patterns, and the in-memory [`CapabilityRegistry`].
    pub async fn find_capability_providers(&self, capability_type: &str) -> Vec<String> {
        debug!("🔍 Finding providers for capability: {}", capability_type);

        let mut providers = self.injected_providers_from_config(capability_type);

        // Check environment variables for capability-based discovery
        let capability_providers = Self::discover_capability_providers_from_env(capability_type);
        providers.extend(capability_providers);

        // Network-based discovery (if enabled)
        if self.config.enable_network_discovery {
            let network_providers =
                self.discover_capability_providers_from_network(capability_type).await;
            providers.extend(network_providers);
        }

        // Capability inference from known patterns
        let inferred_providers = self.infer_capability_providers(capability_type);
        providers.extend(inferred_providers);

        // Primal registry: `discover_primal_capabilities` and peers populate `capability_providers`
        // and `primal_capabilities`. Merge both indexes so env/network inference is not the only source.
        {
            let registry = self.registry.read().await;
            if let Some(names) = registry.capability_providers.get(capability_type) {
                providers.extend(names.iter().cloned());
            }
            for (primal_name, caps) in &registry.primal_capabilities {
                let matches = caps.iter().any(|c| {
                    c.capability_type.eq_ignore_ascii_case(capability_type)
                        || c.name.eq_ignore_ascii_case(capability_type)
                        || Self::primal_provides_capability(&c.capability_type, capability_type)
                        || Self::primal_provides_capability(capability_type, &c.capability_type)
                });
                if matches {
                    providers.push(primal_name.clone());
                }
            }
        }

        // Remove duplicates and return
        providers.sort();
        providers.dedup();

        debug!(
            "✅ Found {} providers for capability {}: {:?}",
            providers.len(),
            capability_type,
            providers
        );

        providers
    }

    /// Provider names derived from [`DiscoveryConfig::provider_endpoints`] (no process env).
    fn injected_providers_from_config(&self, capability_type: &str) -> Vec<String> {
        let mut out = Vec::new();
        for (canonical, url) in &self.config.provider_endpoints {
            if capability_type.eq_ignore_ascii_case(canonical)
                || Self::primal_provides_capability(canonical.as_str(), capability_type)
                || Self::primal_provides_capability(capability_type, canonical.as_str())
            {
                out.push(Self::extract_primal_name_from_endpoint(url));
            }
        }
        out
    }

    /// Discover capability providers from environment variables
    ///
    /// Synchronous function - no I/O, just env var reading.
    /// Future: May become async if env discovery involves remote lookups.
    fn discover_capability_providers_from_env(capability_type: &str) -> Vec<String> {
        let mut providers = Vec::new();

        // Check for generic capability environment variables
        let capability_env = format!("{}_PROVIDERS", capability_type.to_uppercase());
        if let Ok(provider_list) = SafeEnv::get_required(&capability_env) {
            providers.extend(provider_list.split(',').map(|s| s.trim().to_string()));
        }

        // Check for capability-based environment variables (zero hardcoding)
        let capability_endpoints = [
            ("SECURITY_PROVIDER_ENDPOINT", "security"),
            ("COMPUTE_PROVIDER_ENDPOINT", "compute"),
            ("STORAGE_PROVIDER_ENDPOINT", "storage"),
            ("AI_PROVIDER_ENDPOINT", "ai"),
        ];
        for (env_var, cap_type) in &capability_endpoints {
            if (capability_type == *cap_type
                || Self::primal_provides_capability(cap_type, capability_type))
                && let Ok(endpoint) = SafeEnv::get_required(env_var)
            {
                let provider_name = Self::extract_primal_name_from_endpoint(&endpoint);
                providers.push(provider_name);
            }
        }

        providers
    }

    /// Discover capability providers over HTTP via the IPC-backed client
    ///
    /// Queries the same discovery endpoints as the unified adapter (`/capabilities` on the
    /// orchestrator port, `/services` on the discovery port), parses `Vec<ServiceInfo>`, and
    /// returns service names whose capabilities or [`ServiceInfo::primal_type`] category match
    /// `capability_type` (via [`Self::primal_provides_capability`]).
    ///
    /// When the IPC HTTP stack is unavailable or endpoints are down, returns an empty list
    /// (callers still get env + registry + inference from [`Self::find_capability_providers`]).
    async fn discover_capability_providers_from_network(
        &self,
        capability_type: &str,
    ) -> Vec<String> {
        let timeout = self.config.discovery_timeout;
        let capability_type = capability_type.to_string();

        let outcome = tokio::time::timeout(timeout, async {
            let client = match songbird_http_client::IpcHttpClient::new().await {
                Ok(c) => c,
                Err(e) => {
                    warn!("Network capability discovery: IPC HTTP client unavailable: {e}");
                    return Vec::new();
                }
            };

            let mut out = Vec::new();
            for url in Self::orchestrator_discovery_urls() {
                match client.get(&url).await {
                    Ok(resp) if resp.is_success() => match resp.json::<Vec<ServiceInfo>>().await {
                        Ok(services) => {
                            out.extend(Self::collect_matching_service_names(
                                &services,
                                &capability_type,
                            ));
                        }
                        Err(e) => debug!("Network discovery: JSON parse failed for {url}: {e}"),
                    },
                    Ok(resp) => {
                        debug!(
                            "Network discovery: {url} returned non-success status {}",
                            resp.status()
                        );
                    }
                    Err(e) => debug!("Network discovery: GET {url} failed: {e}"),
                }
            }
            out
        })
        .await;

        outcome.unwrap_or_else(|_| {
            warn!(
                "Network capability discovery timed out after {timeout:?} for capability {capability_type}",
            );
            Vec::new()
        })
    }

    /// Build orchestrator + discovery HTTP URLs (aligned with `UnifiedAdapterConfig` defaults).
    fn orchestrator_discovery_urls() -> Vec<String> {
        let host = SafeEnv::get_or_default(
            "ADAPTER_DISCOVERY_HOST",
            songbird_config::canonical::constants::get_bind_address(),
        );
        let capabilities_port = SafeEnv::get_port(
            "ADAPTER_CAPABILITIES_PORT",
            songbird_config::canonical::constants::network::default_orchestrator_port(),
        );
        let services_port = SafeEnv::get_port(
            "ADAPTER_SERVICES_PORT",
            songbird_config::defaults::ports::discovery_port(),
        );
        vec![
            format!("http://{host}:{capabilities_port}/capabilities"),
            format!("http://{host}:{services_port}/services"),
        ]
    }

    /// Service names from a discovery payload that advertise `capability_type`.
    fn collect_matching_service_names(
        services: &[ServiceInfo],
        capability_type: &str,
    ) -> Vec<String> {
        services
            .iter()
            .filter(|s| Self::service_matches_capability(s, capability_type))
            .map(|s| s.name.clone())
            .collect()
    }

    fn service_matches_capability(service: &ServiceInfo, capability_type: &str) -> bool {
        if service.capabilities.iter().any(|c| {
            c.name.eq_ignore_ascii_case(capability_type)
                || Self::primal_provides_capability(&c.name, capability_type)
                || Self::primal_provides_capability(capability_type, &c.name)
        }) {
            return true;
        }

        let coarse = service.primal_type.as_str();
        if coarse.is_empty() || coarse.eq_ignore_ascii_case("unknown") {
            return false;
        }
        Self::primal_provides_capability(coarse, capability_type)
    }

    /// Infer capability providers based on known patterns
    ///
    /// Pure function - could be static but kept as method for potential future
    /// use of discovery config or primal registry state.
    #[allow(
        clippy::unused_self,
        reason = "method on self for API consistency with other discovery helpers"
    )]
    fn infer_capability_providers(&self, capability_type: &str) -> Vec<String> {
        let mut providers = Vec::new();

        // Infer providers based on capability type patterns
        match capability_type {
            "security" | "encryption" | "authentication" => {
                // Look for security capability providers (zero hardcoding)
                if let Ok(endpoint) = SafeEnv::get_required("SECURITY_PROVIDER_ENDPOINT") {
                    let provider = Self::extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
                // Check for custom security services
                for i in 1..=10 {
                    let primal_env = format!("PRIMAL_{i}_NAME");
                    let endpoint_env = format!("PRIMAL_{i}_ENDPOINT");
                    if let (Ok(name), Ok(_)) =
                        (SafeEnv::get_required(&primal_env), SafeEnv::get_required(&endpoint_env))
                        && (name.contains("security")
                            || name.contains("auth")
                            || name.contains("crypto"))
                    {
                        providers.push(name);
                    }
                }
            }
            "compute" | "processing" | "workers" => {
                // Look for compute providers
                if let Ok(endpoint) = SafeEnv::get_required("COMPUTE_PROVIDER_ENDPOINT") {
                    let provider = Self::extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
            }
            "storage" | "persistence" | "database" => {
                // Look for storage providers
                if let Ok(endpoint) = SafeEnv::get_required("STORAGE_PROVIDER_ENDPOINT") {
                    let provider = Self::extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
            }
            "ai" | "ml" | "inference" => {
                // Look for AI providers
                if let Ok(endpoint) = SafeEnv::get_required("AI_PROVIDER_ENDPOINT") {
                    let provider = Self::extract_primal_name_from_endpoint(&endpoint);
                    providers.push(provider);
                }
            }
            _ => {
                // Generic discovery for unknown capability types
                debug!("No inference pattern for capability type: {}", capability_type);
            }
        }

        providers
    }

    /// Check if category `primal_cap` subsumes `target_cap` for env/inference crosswalk.
    ///
    /// Registry-backed provider lists are built in [`Self::find_capability_providers`]
    /// (`capability_providers` + scan of `primal_capabilities`). This helper stays synchronous
    /// for use from env-only code paths.
    fn primal_provides_capability(primal_cap: &str, target_cap: &str) -> bool {
        // Exact match: primal providing X also provides X
        if primal_cap == target_cap {
            return true;
        }
        // Capability hierarchy: primal_cap subsumes target_cap
        matches!(
            (primal_cap, target_cap),
            ("security", "encryption" | "authentication" | "signing" | "crypto")
                | ("compute", "processing" | "workers")
                | ("storage", "persistence" | "database")
                | ("ai", "ml" | "inference")
        )
    }

    /// Extract primal name from endpoint URL
    fn extract_primal_name_from_endpoint(endpoint: &str) -> String {
        endpoint
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split(':')
            .next()
            .unwrap_or("unknown")
            .split('.')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::capabilities::registry::CapabilityRegistry;
    use crate::capabilities::types::{Capability, QoSMetrics};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_find_capability_providers_includes_registry() {
        let registry = Arc::new(RwLock::new(CapabilityRegistry::default()));
        {
            let mut r = registry.write().await;
            r.capability_providers
                .insert("compute".to_string(), vec!["tower-a".to_string(), "tower-b".to_string()]);
            r.primal_capabilities.insert(
                "cipher-primal".to_string(),
                vec![Capability {
                    capability_type: "security".to_string(),
                    name: "vault".to_string(),
                    version: "1.0.0".to_string(),
                    parameters: HashMap::new(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                }],
            );
        }
        let config = DiscoveryConfig::default();
        let discovery = CapabilityDiscovery::new(registry, config);
        let providers = discovery.find_capability_providers("compute").await;
        assert!(providers.contains(&"tower-a".to_string()));
        assert!(providers.contains(&"tower-b".to_string()));

        let sec = discovery.find_capability_providers("encryption").await;
        assert!(sec.contains(&"cipher-primal".to_string()));
    }

    #[test]
    fn test_discover_from_config_injection() {
        let registry = Arc::new(RwLock::new(CapabilityRegistry::default()));
        let mut config = DiscoveryConfig::default();
        config.provider_endpoints.insert("security".to_string(), "http://beardog:8443".to_string());
        let discovery = CapabilityDiscovery::new(registry, config);

        let providers = discovery.injected_providers_from_config("security");

        assert!(!providers.is_empty(), "Should find at least one security provider");
        assert_eq!(providers[0], "beardog");
    }

    #[tokio::test]
    async fn test_extract_primal_name() {
        let name = CapabilityDiscovery::extract_primal_name_from_endpoint("http://beardog:8443");
        assert_eq!(name, "beardog");

        let name2 =
            CapabilityDiscovery::extract_primal_name_from_endpoint("https://toadstool.local:9000");
        assert_eq!(name2, "toadstool");
    }

    #[tokio::test]
    async fn test_find_capability_providers_hierarchy_workers_under_compute() {
        let registry = Arc::new(RwLock::new(CapabilityRegistry::default()));
        {
            let mut r = registry.write().await;
            r.primal_capabilities.insert(
                "worker-node".to_string(),
                vec![Capability {
                    capability_type: "compute".to_string(),
                    name: "batch".to_string(),
                    version: "1.0.0".to_string(),
                    parameters: HashMap::new(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                }],
            );
        }
        let config = DiscoveryConfig::default();
        let discovery = CapabilityDiscovery::new(registry, config);
        let providers = discovery.find_capability_providers("workers").await;
        assert!(providers.contains(&"worker-node".to_string()));
    }
}
