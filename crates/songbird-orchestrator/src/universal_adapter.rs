//! Universal Primal Adapter (UPA)
//!
//! Discovers and communicates with ANY primal without hardcoding names or endpoints.
//! This is the ONLY module that knows how to discover external primals!
//!
//! ## Zero Hardcoding Philosophy
//!
//! ```text
//! ❌ OLD: "Connect to BearDog at localhost:9000"
//! ✅ NEW: "Discover who provides 'security' capability"
//!
//! ❌ OLD: "If security needed, call BearDog API"
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
//! 4. **Environment**: CAPABILITY_PROVIDERS env var (for development)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_orchestrator::universal_adapter::{UniversalAdapter, CapabilityQuery};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let adapter = UniversalAdapter::new().await?;
//!
//! // Discover security provider (could be BearDog, could be something else!)
//! let security_provider = adapter.discover_capability("security").await?;
//! println!("Found security provider at: {}", security_provider.endpoint);
//!
//! // Call the provider (adapter handles protocol translation)
//! let response = adapter.call(&security_provider, "identity", serde_json::json!({})).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{anyhow, Context, Result};
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
    http_client: reqwest::Client,
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
    
    /// Protocol (http, tarpc, grpc, etc.)
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
    pub async fn new() -> Result<Self> {
        info!("🌐 Initializing Universal Primal Adapter (zero hardcoding!)");
        
        Ok(Self {
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 minutes
            http_client: reqwest::Client::new(),
        })
    }
    
    /// Discover providers for a capability
    ///
    /// Returns ALL providers offering this capability (could be multiple!)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::universal_adapter::UniversalAdapter;
    /// # async fn example() -> anyhow::Result<()> {
    /// let adapter = UniversalAdapter::new().await?;
    ///
    /// // Find ALL security providers (could be BearDog, or others!)
    /// let providers = adapter.discover_capability("security").await?;
    /// for provider in providers {
    ///     println!("Found: {} at {}", provider.provider_id, provider.endpoint);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_capability(&mut self, capability: &str) -> Result<Vec<DiscoveredProvider>> {
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
        // TODO: Implement DHT discovery
        
        // Method 4: Registry (if available)
        // TODO: Implement registry discovery
        
        if providers.is_empty() {
            warn!("No providers found for capability: {}", capability);
            return Err(anyhow!("No providers found for capability: {}", capability));
        }
        
        info!("✅ Discovered {} providers for '{}'", providers.len(), capability);
        
        // Cache results
        self.cache.insert(
            capability.to_string(),
            providers.iter().map(|p| CachedProvider {
                provider: p.clone(),
                cached_at: Instant::now(),
            }).collect(),
        );
        
        Ok(providers)
    }
    
    /// Discover providers from environment variable
    ///
    /// Format: CAPABILITY_PROVIDERS='security=http://192.168.1.10:9000,storage=http://192.168.1.20:8000'
    async fn discover_from_environment(&self, capability: &str) -> Result<Vec<DiscoveredProvider>> {
        let env_var = std::env::var("CAPABILITY_PROVIDERS").ok();
        if env_var.is_none() {
            return Ok(vec![]);
        }
        
        let env_var = env_var.unwrap();
        let mut providers = Vec::new();
        
        for entry in env_var.split(',') {
            let parts: Vec<&str> = entry.split('=').collect();
            if parts.len() != 2 {
                continue;
            }
            
            let (cap, endpoint) = (parts[0].trim(), parts[1].trim());
            if cap == capability {
                providers.push(DiscoveredProvider {
                    provider_id: format!("env-{}", capability),
                    capabilities: vec![capability.to_string()],
                    endpoint: endpoint.to_string(),
                    protocol: "http".to_string(), // Assume HTTP for env providers
                    metadata: HashMap::from([
                        ("source".to_string(), "environment".to_string()),
                    ]),
                    discovered_at: Instant::now(),
                });
                
                debug!("Discovered provider from environment: {} -> {}", capability, endpoint);
            }
        }
        
        Ok(providers)
    }
    
    /// Discover providers from mDNS
    async fn discover_from_mdns(&self, capability: &str) -> Result<Vec<DiscoveredProvider>> {
        // TODO: Implement actual mDNS discovery
        // This would query for services advertising the capability
        debug!("mDNS discovery for '{}' not yet implemented", capability);
        Ok(vec![])
    }
    
    /// Call a provider's API
    ///
    /// Handles protocol translation (HTTP, tarpc, gRPC, etc.)
    ///
    /// # Example
    ///
    /// ```rust,no_run
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
    pub async fn call(
        &self,
        provider: &DiscoveredProvider,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match provider.protocol.as_str() {
            "http" | "https" => self.call_http(provider, method, params).await,
            "tarpc" => self.call_tarpc(provider, method, params).await,
            "grpc" => self.call_grpc(provider, method, params).await,
            protocol => Err(anyhow!("Unsupported protocol: {}", protocol)),
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
        
        let response = self.http_client
            .get(&url)
            .json(&params)
            .send()
            .await
            .context("Failed to call HTTP provider")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Provider returned error {}: {}", status, body));
        }
        
        response.json::<serde_json::Value>()
            .await
            .context("Failed to parse provider response")
    }
    
    /// Call tarpc provider
    async fn call_tarpc(
        &self,
        provider: &DiscoveredProvider,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Extract address from endpoint (e.g., "tarpc://127.0.0.1:8081" -> "127.0.0.1:8081")
        let address = provider.endpoint
            .trim_start_matches("tarpc://")
            .trim_start_matches("tcp://");

        debug!("Calling tarpc provider: {} on {}", method, address);

        // Parse address for tarpc connection
        let addr: std::net::SocketAddr = address
            .parse()
            .with_context(|| format!("Invalid tarpc address: {}", address))?;

        // Use tarpc client from universal adapter
        // Note: This uses the existing tarpc infrastructure from songbird-universal
        let endpoint = format!("tarpc://{}", addr);
        let client = songbird_universal::TarpcClient::new(&endpoint)?;
        
        // Call the method (tarpc client handles serialization)
        client.call_method(method, Some(params))
            .await
            .with_context(|| format!("tarpc call failed: {}", method))
    }
    
    /// Call gRPC provider
    ///
    /// NOTE: gRPC is intentionally not prioritized in Songbird's protocol hierarchy.
    /// Songbird uses: tarpc > JSON-RPC > HTTP (in order of preference).
    /// gRPC support may be added later if ecosystem needs arise.
    async fn call_grpc(
        &self,
        _provider: &DiscoveredProvider,
        _method: &str,
        _params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        Err(anyhow!(
            "gRPC protocol not supported. Songbird prioritizes: tarpc > JSON-RPC > HTTP.\n\
             If you need gRPC, please open an issue explaining your use case."
        ))
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
        let adapter = UniversalAdapter::new().await.unwrap();
        assert!(adapter.cache.is_empty());
    }
    
    #[tokio::test]
    async fn test_environment_discovery() {
        // Set test environment variable
        std::env::set_var("CAPABILITY_PROVIDERS", "test-cap=http://localhost:8000");
        
        let adapter = UniversalAdapter::new().await.unwrap();
        let providers = adapter.discover_from_environment("test-cap").await.unwrap();
        
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].endpoint, "http://localhost:8000");
        assert_eq!(providers[0].capabilities, vec!["test-cap"]);
        
        // Clean up
        std::env::remove_var("CAPABILITY_PROVIDERS");
    }
    
    #[test]
    fn test_capability_query_default() {
        let query = CapabilityQuery::default();
        assert_eq!(query.min_providers, 1);
        assert_eq!(query.timeout, Duration::from_secs(5));
    }
}

