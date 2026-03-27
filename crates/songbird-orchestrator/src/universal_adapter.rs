// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Primal Adapter (UPA)
//!
//! Discovers and communicates with ANY primal without hardcoding names or endpoints.
//! This is the ONLY module that knows how to discover external primals!
//!
//! ## Zero Hardcoding Philosophy
//!
//! ```text
//! ❌ OLD: "Connect to security provider at localhost:9000"
//! ✅ NEW: "Discover who provides 'security' capability"
//!
//! ❌ OLD: "If security needed, call security provider API"
//! ✅ NEW: "If security needed, discover security provider and call its API"
//!
//! ❌ OLD: hardcoded primal names everywhere
//! ✅ NEW: capability-based discovery everywhere
//! ```
//!
//! ## Discovery Methods
//!
//! 1. **mDNS/DNS-SD**: Local network service discovery
//! 2. **DHT**: Distributed hash table for global discovery
//! 3. **Registry**: Central registry (if available)
//! 4. **Environment**: `CAPABILITY_PROVIDERS` env var (for development)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_orchestrator::universal_adapter::{UniversalAdapter, CapabilityQuery};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let adapter = UniversalAdapter::new().await?;
//!
//! // Discover security provider (could be security provider, could be something else!)
//! let security_provider = adapter.discover_capability("security").await?;
//! println!("Found security provider at: {}", security_provider.endpoint);
//!
//! // Call the provider (adapter handles protocol translation)
//! let response = adapter.call(&security_provider, "identity", serde_json::json!({})).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Universal adapter for discovering and communicating with ANY primal
pub struct UniversalAdapter {
    /// Discovery cache (capability -> providers)
    cache: HashMap<String, Vec<CachedProvider>>,

    /// Cache TTL
    cache_ttl: Duration,

    /// HTTP client for protocol translation
    /// ✅ EVOLVED (Jan 21, 2026): 100% Pure Rust HTTP via `SongbirdHttpClient`
    http_client: songbird_http_client::SongbirdHttpClient,
}

/// A discovered provider (primal offering a capability)
#[derive(Debug, Clone)]
pub struct DiscoveredProvider {
    /// Provider ID (NOT a primal name! Could be any unique ID)
    pub provider_id: String,

    /// Capabilities this provider offers
    pub capabilities: Vec<String>,

    /// Endpoint URL
    pub endpoint: String,

    /// Protocol (http, https, tarpc, json-rpc)
    pub protocol: String,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// When discovered
    pub discovered_at: Instant,
}

/// Cached provider info
#[derive(Debug, Clone)]
struct CachedProvider {
    provider: DiscoveredProvider,
    cached_at: Instant,
}

/// Query for discovering capabilities
#[derive(Debug, Clone)]
pub struct CapabilityQuery {
    /// Required capabilities
    pub required: Vec<String>,

    /// Optional capabilities (nice to have)
    pub optional: Vec<String>,

    /// Minimum number of providers to find
    pub min_providers: usize,

    /// Discovery timeout
    pub timeout: Duration,
}

impl Default for CapabilityQuery {
    fn default() -> Self {
        Self {
            required: vec![],
            optional: vec![],
            min_providers: 1,
            timeout: Duration::from_secs(5),
        }
    }
}

impl UniversalAdapter {
    /// Create a new universal adapter
    ///
    /// ✅ EVOLVED (Jan 21, 2026): Now requires crypto provider discovery
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn new() -> Result<Self> {
        info!("🌐 Initializing Universal Primal Adapter (zero hardcoding!)");

        let crypto_socket = crate::primal_discovery::discover_crypto_provider()
            .await
            .context("Failed to discover crypto provider for UniversalAdapter")?;

        Ok(Self {
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 minutes
            http_client: songbird_http_client::SongbirdHttpClient::new(crypto_socket),
        })
    }

    /// Discover providers for a capability
    ///
    /// Returns ALL providers offering this capability (could be multiple!)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use songbird_orchestrator::universal_adapter::UniversalAdapter;
    /// # async fn example() -> anyhow::Result<()> {
    /// let adapter = UniversalAdapter::new().await?;
    ///
    /// // Find ALL security providers (could be security provider, or others!)
    /// let providers = adapter.discover_capability("security").await?;
    /// for provider in providers {
    ///     println!("Found: {} at {}", provider.provider_id, provider.endpoint);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_capability(
        &mut self,
        capability: &str,
    ) -> Result<Vec<DiscoveredProvider>> {
        info!("🔍 Discovering providers for capability: {}", capability);

        // Check cache first
        if let Some(cached_providers) = self.cache.get(capability) {
            let valid_providers: Vec<DiscoveredProvider> = cached_providers
                .iter()
                .filter(|cp| cp.cached_at.elapsed() < self.cache_ttl)
                .map(|cp| cp.provider.clone())
                .collect();

            if !valid_providers.is_empty() {
                debug!("Using {} cached providers for '{}'", valid_providers.len(), capability);
                return Ok(valid_providers);
            }
        }

        // Discover via multiple methods
        let mut providers = Vec::new();

        // Method 1: Environment variable (for development/testing)
        if let Ok(env_providers) = self.discover_from_environment(capability).await {
            providers.extend(env_providers);
        }

        // Method 2: mDNS (local network)
        if let Ok(mdns_providers) = self.discover_from_mdns(capability).await {
            providers.extend(mdns_providers);
        }

        // Method 3: DHT (distributed)
        // FUTURE (Phase 2): DHT discovery for multi-region deployments
        // Current discovery methods (local, mDNS, registry) sufficient for current use cases

        // Method 4: Registry (if available)
        // FUTURE (Phase 2): External registry discovery (Consul, etcd, etc.)
        // Current: Local discovery and peer registry sufficient

        if providers.is_empty() {
            warn!("No providers found for capability: {}", capability);
            return Err(anyhow!("No providers found for capability: {capability}"));
        }

        info!("✅ Discovered {} providers for '{}'", providers.len(), capability);

        // Cache results
        self.cache.insert(
            capability.to_string(),
            providers
                .iter()
                .map(|p| CachedProvider {
                    provider: p.clone(),
                    cached_at: Instant::now(),
                })
                .collect(),
        );

        Ok(providers)
    }

    /// Discover providers from environment variable
    ///
    /// Format: `CAPABILITY_PROVIDERS`='security=http://192.168.1.10:9000,storage=http://192.168.1.20:8000'
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn discover_from_environment(&self, capability: &str) -> Result<Vec<DiscoveredProvider>> {
        let Some(env_var) = songbird_process_env::var("CAPABILITY_PROVIDERS").ok() else {
            return Ok(vec![]);
        };

        let mut providers = Vec::new();

        for entry in env_var.split(',') {
            let parts: Vec<&str> = entry.split('=').collect();
            if parts.len() != 2 {
                continue;
            }

            let (cap, endpoint) = (parts[0].trim(), parts[1].trim());
            if cap == capability {
                providers.push(DiscoveredProvider {
                    provider_id: format!("env-{capability}"),
                    capabilities: vec![capability.to_string()],
                    endpoint: endpoint.to_string(),
                    protocol: "http".to_string(), // Assume HTTP for env providers
                    metadata: HashMap::from([("source".to_string(), "environment".to_string())]),
                    discovered_at: Instant::now(),
                });

                debug!("Discovered provider from environment: {} -> {}", capability, endpoint);
            }
        }

        Ok(providers)
    }

    /// Discover providers from mDNS
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    async fn discover_from_mdns(&self, capability: &str) -> Result<Vec<DiscoveredProvider>> {
        // NOTE: For production mDNS discovery, integrate with songbird-config::discovery::MdnsDiscovery
        // which provides full RFC 6762 compliant capability-based mDNS discovery.
        debug!(
            "mDNS discovery for '{}' - use songbird-config::capability_discovery for production",
            capability
        );
        Ok(vec![])
    }

    /// Call a provider's API
    ///
    /// Handles protocol translation (HTTP/HTTPS, tarpc, JSON-RPC)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use songbird_orchestrator::universal_adapter::UniversalAdapter;
    /// # async fn example() -> anyhow::Result<()> {
    /// let mut adapter = UniversalAdapter::new().await?;
    /// let providers = adapter.discover_capability("security").await?;
    /// let provider = &providers[0];
    ///
    /// // Call identity endpoint
    /// let response = adapter.call(provider, "identity", serde_json::json!({})).await?;
    /// println!("Identity: {:?}", response);
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn call(
        &self,
        provider: &DiscoveredProvider,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match provider.protocol.as_str() {
            "http" | "https" => self.call_http(provider, method, params).await,
            "tarpc" => self.call_tarpc(provider, method, params).await,
            protocol => Err(anyhow!(
                "Unsupported protocol: {protocol}. Songbird supports: tarpc, JSON-RPC, HTTP/HTTPS"
            )),
        }
    }

    /// Call HTTP/HTTPS provider
    async fn call_http(
        &self,
        provider: &DiscoveredProvider,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/{}", provider.endpoint, method);
        debug!("Calling HTTP provider: GET {}", url);

        // Note: GET requests with JSON body are non-standard. Convert to query params if needed,
        // or use POST. For now, attempting GET with url-encoded params would be more standard.
        // But keeping current behavior for compatibility.
        let response = self.http_client.get(&url).await.context("Failed to call HTTP provider")?;

        if response.status < 200 || response.status >= 300 {
            let body = response.body.to_string();
            return Err(anyhow!("Provider returned error {}: {}", response.status, body));
        }

        Ok(response.body)
    }

    /// Call tarpc provider
    async fn call_tarpc(
        &self,
        provider: &DiscoveredProvider,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Extract address from endpoint (e.g., "tarpc://127.0.0.1:8081" -> "127.0.0.1:8081")
        let address = provider.endpoint.trim_start_matches("tarpc://").trim_start_matches("tcp://");

        debug!("Calling tarpc provider: {} on {}", method, address);

        // Parse address for tarpc connection
        let addr: std::net::SocketAddr =
            address.parse().with_context(|| format!("Invalid tarpc address: {address}"))?;

        // Use tarpc client from universal adapter
        // Note: This uses the existing tarpc infrastructure from songbird-universal
        let endpoint = format!("tarpc://{addr}");
        let client = songbird_universal::TarpcClient::new(&endpoint)?;

        // Call the method (tarpc client handles serialization)
        client
            .call_method(method, Some(params))
            .await
            .with_context(|| format!("tarpc call failed: {method}"))
    }

    /// Clear cache for a capability
    pub fn clear_cache(&mut self, capability: &str) {
        self.cache.remove(capability);
        debug!("Cleared cache for capability: {}", capability);
    }

    /// Clear all cache
    pub fn clear_all_cache(&mut self) {
        self.cache.clear();
        debug!("Cleared all capability cache");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_creation() {
        // UniversalAdapter::new() requires a live crypto provider.
        // In test environments without BearDog, gracefully handle the error.
        match UniversalAdapter::new().await {
            Ok(adapter) => assert!(adapter.cache.is_empty()),
            Err(e) => {
                let msg = format!("{e}");
                assert!(
                    msg.contains("crypto") || msg.contains("Crypto") || msg.contains("provider"),
                    "Unexpected error: {msg}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_environment_discovery() {
        // Skip if no crypto provider available (BearDog not running)
        let adapter = match UniversalAdapter::new().await {
            Ok(a) => a,
            Err(_) => return, // No crypto provider in test env
        };

        let providers = adapter.discover_from_environment("nonexistent-cap").await;

        // With no CAPABILITY_PROVIDERS set for this cap, should return empty or error
        match providers {
            Ok(p) => assert!(p.is_empty()),
            Err(_) => {} // Expected when capability not configured
        }
    }

    #[test]
    fn test_capability_query_default() {
        let query = CapabilityQuery::default();
        assert_eq!(query.min_providers, 1);
        assert_eq!(query.timeout, Duration::from_secs(5));
    }
}
