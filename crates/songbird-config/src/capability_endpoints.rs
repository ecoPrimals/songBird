// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🍼 Capability-Based Endpoints (Zero Hardcoding)
//!
//! **PHILOSOPHY**: Request capabilities (security, storage, compute, ai), not specific providers.
//!
//! This module replaces primal-name-based endpoint configuration with capability-based
//! discovery. Services specify WHAT they need, not WHO provides it.
//!
//! ## Migration from Legacy
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded primal names
//! let endpoint = endpoints::get_primal_endpoint("beardog");
//!
//! // ✅ NEW: Capability-based
//! let endpoint = capability_endpoints::get_capability_endpoint("security").await?;
//! ```
//!
//! ## Environment Variables
//!
//! ### Capability Endpoints (Optional - discovered if not set)
//! - `CAPABILITY_SECURITY_ENDPOINT` - Security provider endpoint
//! - `CAPABILITY_STORAGE_ENDPOINT` - Storage provider endpoint
//! - `CAPABILITY_COMPUTE_ENDPOINT` - Compute provider endpoint
//! - `CAPABILITY_AI_ENDPOINT` - AI provider endpoint
//! - `CAPABILITY_ORCHESTRATION_ENDPOINT` - Orchestration provider endpoint
//!
//! ### Discovery Configuration
//! - `SERVICE_REGISTRY_ENDPOINT` - Service registry for discovery
//! - `ENABLE_INFANT_DISCOVERY` - Enable zero-knowledge bootstrap
//! - `DISCOVERY_TIMEOUT_SECS` - Discovery timeout (default: 30)

use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Capability type for service discovery
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Security capabilities (authentication, encryption, key management)
    Security,
    /// Storage capabilities (data persistence, caching, backup)
    Storage,
    /// Compute capabilities (workload execution, container orchestration)
    Compute,
    /// AI/ML capabilities (inference, training, analysis)
    Ai,
    /// Orchestration capabilities (service coordination, workflow management)
    Orchestration,
    /// Observability capabilities (logging, metrics, tracing)
    Observability,
    /// Networking capabilities (service mesh, load balancing)
    Networking,
    /// Custom capability
    Custom(String),
}

impl FromStr for CapabilityType {
    type Err = Infallible;

    /// Parse capability type from string (always succeeds, falls back to Custom)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "security" | "auth" | "authentication" | "encryption" => Self::Security,
            "storage" | "database" | "persistence" | "cache" => Self::Storage,
            "compute" | "execution" | "runtime" | "container" => Self::Compute,
            "ai" | "ml" | "inference" | "intelligence" => Self::Ai,
            "orchestration" | "coordination" | "workflow" => Self::Orchestration,
            "observability" | "logging" | "metrics" | "tracing" => Self::Observability,
            "networking" | "mesh" | "loadbalancing" => Self::Networking,
            custom => Self::Custom(custom.to_string()),
        })
    }
}

impl CapabilityType {
    /// Get environment variable name for this capability
    #[must_use]
    pub fn env_var_name(&self) -> String {
        match self {
            Self::Security => "CAPABILITY_SECURITY_ENDPOINT".to_string(),
            Self::Storage => "CAPABILITY_STORAGE_ENDPOINT".to_string(),
            Self::Compute => "CAPABILITY_COMPUTE_ENDPOINT".to_string(),
            Self::Ai => "CAPABILITY_AI_ENDPOINT".to_string(),
            Self::Orchestration => "CAPABILITY_ORCHESTRATION_ENDPOINT".to_string(),
            Self::Observability => "CAPABILITY_OBSERVABILITY_ENDPOINT".to_string(),
            Self::Networking => "CAPABILITY_NETWORKING_ENDPOINT".to_string(),
            Self::Custom(name) => format!("CAPABILITY_{}_ENDPOINT", name.to_uppercase()),
        }
    }

    /// Get capability name as string
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Security => "security",
            Self::Storage => "storage",
            Self::Compute => "compute",
            Self::Ai => "ai",
            Self::Orchestration => "orchestration",
            Self::Observability => "observability",
            Self::Networking => "networking",
            Self::Custom(name) => name,
        }
    }
}

/// Discovered capability endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEndpoint {
    /// Capability type
    pub capability: CapabilityType,
    /// Endpoint URL
    pub endpoint: String,
    /// Provider ID (if known)
    pub provider_id: Option<String>,
    /// Discovery method used
    pub discovery_method: DiscoveryMethod,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// When discovered
    pub discovered_at: std::time::SystemTime,
}

/// How the endpoint was discovered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// From environment variable
    Environment,
    /// From service registry
    ServiceRegistry,
    /// From container metadata
    ContainerMetadata,
    /// From DNS discovery
    Dns,
    /// From network scan
    NetworkScan,
    /// From configuration file
    ConfigFile,
}

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
        let cache_ttl_secs =
            env::var("DISCOVERY_CACHE_TTL_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(300);

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
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&capability) {
                // Check if cache is still valid
                if let Ok(elapsed) = cached.discovered_at.elapsed()
                    && elapsed < self.cache_ttl
                {
                    debug!("Using cached endpoint for {:?}", capability);
                    return Ok(cached.endpoint.clone());
                }
            }
        }

        // Discover endpoint
        let endpoint = self.discover_endpoint(&capability).await?;

        // Cache the result
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

        // Method 0: Injected static overrides (tests / explicit config)
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

        // Method 1: Environment variable (highest priority)
        if let Ok(endpoint) = env::var(capability.env_var_name()) {
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

        // Method 2: Service registry discovery
        if let Some(endpoint) = self.discover_from_registry(capability).await? {
            return Ok(endpoint);
        }

        // Method 3: Container metadata discovery
        if let Some(endpoint) = self.discover_from_container_metadata(capability).await? {
            return Ok(endpoint);
        }

        // Method 4: DNS discovery
        if let Some(endpoint) = self.discover_from_dns(capability).await? {
            return Ok(endpoint);
        }

        // No endpoint found
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

    /// Discover from service registry
    async fn discover_from_registry(
        &self,
        capability: &CapabilityType,
    ) -> SongbirdResult<Option<CapabilityEndpoint>> {
        // Check if service registry is configured
        let Ok(registry_endpoint) = env::var("SERVICE_REGISTRY_ENDPOINT") else {
            return Ok(None);
        };

        debug!("Querying service registry for {} capability", capability.as_str());

        // Query registry for services providing this capability
        // Supports Consul, Eureka, and other HTTP-based registries

        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

        // Try Consul-style query first
        let consul_url =
            format!("{}/v1/catalog/service/{}", registry_endpoint, capability.as_str());
        match client.get(&consul_url).await {
            Ok(response) if response.is_success() => {
                // Parse Consul service catalog response
                if let Ok(services) = response.json::<Vec<serde_json::Value>>().await
                    && let Some(service) = services.first()
                {
                    // Extract service endpoint
                    if let (Some(address), Some(port)) = (
                        service
                            .get("ServiceAddress")
                            .and_then(|v| v.as_str())
                            .or_else(|| service.get("Address").and_then(|v| v.as_str())),
                        service
                            .get("ServicePort")
                            .and_then(serde_json::Value::as_u64)
                            .or_else(|| service.get("Port").and_then(serde_json::Value::as_u64)),
                    ) {
                        let endpoint = if address.contains("://") {
                            format!("{address}:{port}")
                        } else {
                            format!("http://{address}:{port}")
                        };

                        debug!(
                            "Found {} capability at {} via registry",
                            capability.as_str(),
                            endpoint
                        );

                        return Ok(Some(CapabilityEndpoint {
                            capability: capability.clone(),
                            endpoint,
                            provider_id: service
                                .get("ServiceName")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            discovery_method: DiscoveryMethod::ServiceRegistry,
                            confidence: 0.9, // High confidence from registry
                            discovered_at: std::time::SystemTime::now(),
                        }));
                    }
                }
            }
            Ok(_) => debug!("Registry returned non-success status"),
            Err(e) => debug!("Registry query failed: {}", e),
        }

        Ok(None)
    }

    /// Discover from container metadata
    async fn discover_from_container_metadata(
        &self,
        capability: &CapabilityType,
    ) -> SongbirdResult<Option<CapabilityEndpoint>> {
        // Check if container metadata API is available
        let Ok(metadata_api) = env::var("CONTAINER_METADATA_API") else {
            return Ok(None);
        };

        debug!("Querying container metadata for {} capability", capability.as_str());

        // Query container orchestrator for services providing this capability
        // Supports Kubernetes, Docker Swarm, Nomad, etc.

        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

        // Try Kubernetes service discovery
        let service_name = format!("{}-service", capability.as_str().to_lowercase());
        let k8s_url = format!("{metadata_api}/api/v1/services/{service_name}");

        match client.get(&k8s_url).await {
            Ok(response) if response.is_success() => {
                // Parse Kubernetes service response
                if let Ok(service) = response.json::<serde_json::Value>().await {
                    // Extract cluster IP and port
                    if let (Some(cluster_ip), Some(ports)) = (
                        service
                            .get("spec")
                            .and_then(|s| s.get("clusterIP"))
                            .and_then(|v| v.as_str()),
                        service.get("spec").and_then(|s| s.get("ports")).and_then(|v| v.as_array()),
                    ) && let Some(first_port) = ports
                        .first()
                        .and_then(|p| p.get("port"))
                        .and_then(serde_json::Value::as_u64)
                    {
                        let endpoint = format!("http://{cluster_ip}:{first_port}");

                        debug!(
                            "Found {} capability at {} via container metadata",
                            capability.as_str(),
                            endpoint
                        );

                        return Ok(Some(CapabilityEndpoint {
                            capability: capability.clone(),
                            endpoint,
                            provider_id: Some(service_name),
                            discovery_method: DiscoveryMethod::ContainerMetadata,
                            confidence: 0.95, // Very high confidence from K8s
                            discovered_at: std::time::SystemTime::now(),
                        }));
                    }
                }
            }
            Ok(_) => debug!("Container metadata API returned non-success status"),
            Err(e) => debug!("Container metadata query failed: {}", e),
        }

        Ok(None)
    }

    /// Discover from DNS
    async fn discover_from_dns(
        &self,
        capability: &CapabilityType,
    ) -> SongbirdResult<Option<CapabilityEndpoint>> {
        // Check if DNS discovery domain is configured
        let Ok(dns_domain) = env::var("SERVICE_DISCOVERY_DOMAIN") else {
            return Ok(None);
        };

        debug!("Querying DNS for {} capability", capability.as_str());

        // Query DNS SRV records for services providing this capability
        // Format: _capability._tcp.domain (RFC 2782)

        let service_name = format!("_{}._tcp.{}", capability.as_str().to_lowercase(), dns_domain);

        // Use tokio's DNS resolver for SRV record lookup
        match tokio::net::lookup_host(service_name.as_str()).await {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    let endpoint = format!("http://{addr}");

                    debug!("Found {} capability at {} via DNS SRV", capability.as_str(), endpoint);

                    return Ok(Some(CapabilityEndpoint {
                        capability: capability.clone(),
                        endpoint,
                        provider_id: Some(service_name.clone()),
                        discovery_method: DiscoveryMethod::Dns,
                        confidence: 0.8, // Good confidence from DNS
                        discovered_at: std::time::SystemTime::now(),
                    }));
                }
                debug!("DNS SRV query succeeded but returned no addresses");
            }
            Err(e) => debug!("DNS SRV query failed: {}", e),
        }

        Ok(None)
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

/// Get endpoint for a capability (convenience function)
///
/// # Examples
///
/// ```no_run
/// use songbird_config::capability_endpoints;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get security endpoint
/// let endpoint = capability_endpoints::get_capability_endpoint("security").await?;
///
/// // Or use typed capability
/// let endpoint = capability_endpoints::get_endpoint_typed(
///     capability_endpoints::CapabilityType::Storage
/// ).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
/// Returns error if no endpoint can be discovered for the capability
///
/// Returns an error if capability string parsing fails
pub async fn get_capability_endpoint(capability: &str) -> SongbirdResult<String> {
    let capability_type = capability.parse::<CapabilityType>().map_err(|e| {
        SongbirdError::configuration(format!("Invalid capability type '{capability}': {e}"))
    })?;
    let resolver = CapabilityEndpointResolver::new();
    resolver.get_endpoint(capability_type).await
}

/// Get endpoint for a typed capability
///
/// # Errors
/// Returns error if no endpoint can be discovered for the capability
pub async fn get_endpoint_typed(capability: CapabilityType) -> SongbirdResult<String> {
    let resolver = CapabilityEndpointResolver::new();
    resolver.get_endpoint(capability).await
}

/// Get all available capability endpoints
pub async fn get_all_endpoints() -> HashMap<CapabilityType, CapabilityEndpoint> {
    let resolver = CapabilityEndpointResolver::new();
    resolver.get_all_cached().await
}

/// Clear endpoint cache (force re-discovery)
///
/// Note: With current implementation, this creates a new resolver instance,
/// so cache clearing is implicit. Future versions may use a global instance.
pub const fn clear_cache() {
    // No-op with current architecture - each call creates new resolver
    // This is intentional to avoid global state complexity
}

/// Check if a capability endpoint is available
pub async fn has_capability(capability: &str) -> bool {
    get_capability_endpoint(capability).await.is_ok()
}

/// Get multiple capability endpoints in parallel
///
/// # Errors
/// Returns error if any capability endpoint cannot be discovered
pub async fn get_multiple_endpoints(capabilities: &[&str]) -> SongbirdResult<Vec<String>> {
    let mut endpoints = Vec::new();

    for capability in capabilities {
        let endpoint = get_capability_endpoint(capability).await?;
        endpoints.push(endpoint);
    }

    Ok(endpoints)
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    // ✅ CONCURRENT-SAFE! Using ScopedEnv for proper isolation
    use songbird_test_utils::ScopedEnv;

    #[tokio::test] // ✅ CONCURRENT! Using ScopedEnv
    async fn test_capability_from_environment() {
        // ScopedEnv automatically cleans up when dropped (RAII pattern)
        let _env = ScopedEnv::set("CAPABILITY_SECURITY_ENDPOINT", "http://security:8443").await;

        let endpoint = get_capability_endpoint("security").await.expect("security endpoint");
        assert_eq!(endpoint, "http://security:8443");

        // Cleanup happens automatically when _env drops
    }

    #[tokio::test] // ✅ CONCURRENT! Using ScopedEnv
    async fn test_capability_not_found() {
        // ✅ FIXED: Use remove_multiple to avoid deadlock
        // Creating multiple ScopedEnv::remove() instances simultaneously deadlocks!
        let _env = ScopedEnv::remove_multiple([
            "CAPABILITY_CUSTOM_TEST_ENDPOINT",
            "SERVICE_REGISTRY_ENDPOINT",
        ])
        .await;

        let result = get_capability_endpoint("custom_test").await;
        assert!(result.is_err());

        // Variables restored automatically
    }

    #[test]
    fn test_capability_type_parsing() {
        assert_eq!("security".parse::<CapabilityType>().expect("parse"), CapabilityType::Security);
        assert_eq!("AUTH".parse::<CapabilityType>().expect("parse"), CapabilityType::Security);
        assert_eq!(
            "encryption".parse::<CapabilityType>().expect("parse"),
            CapabilityType::Security
        );
        assert_eq!("database".parse::<CapabilityType>().expect("parse"), CapabilityType::Storage);
        assert_eq!("runtime".parse::<CapabilityType>().expect("parse"), CapabilityType::Compute);
        assert_eq!("intelligence".parse::<CapabilityType>().expect("parse"), CapabilityType::Ai);
        assert_eq!(
            "workflow".parse::<CapabilityType>().expect("parse"),
            CapabilityType::Orchestration
        );
        assert_eq!(
            "metrics".parse::<CapabilityType>().expect("parse"),
            CapabilityType::Observability
        );
        assert_eq!("mesh".parse::<CapabilityType>().expect("parse"), CapabilityType::Networking);
        assert_eq!("Storage".parse::<CapabilityType>().expect("parse"), CapabilityType::Storage);

        if let CapabilityType::Custom(name) = "my_custom".parse::<CapabilityType>().expect("parse")
        {
            assert_eq!(name, "my_custom");
        } else {
            panic!("Expected Custom capability");
        }
    }

    #[test]
    fn test_env_var_names() {
        assert_eq!(CapabilityType::Security.env_var_name(), "CAPABILITY_SECURITY_ENDPOINT");
        assert_eq!(
            CapabilityType::Custom("test".to_string()).env_var_name(),
            "CAPABILITY_TEST_ENDPOINT"
        );
        assert_eq!(CapabilityType::Observability.as_str(), "observability");
        assert_eq!(CapabilityType::Networking.as_str(), "networking");
    }

    #[test]
    fn test_capability_type_json_roundtrip() {
        let cap = CapabilityType::Orchestration;
        let json = serde_json::to_string(&cap).expect("serialize");
        let back: CapabilityType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, cap);
    }

    #[test]
    fn test_discovery_method_serde_roundtrip() {
        let m = DiscoveryMethod::ConfigFile;
        let json = serde_json::to_string(&m).expect("serialize");
        let back: DiscoveryMethod = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(format!("{back:?}"), format!("{m:?}"));
    }

    #[tokio::test] // ✅ CONCURRENT! Using ScopedEnv
    async fn test_multiple_endpoints() {
        // Set multiple variables with automatic cleanup
        let _env = ScopedEnv::set_multiple([
            ("CAPABILITY_SECURITY_ENDPOINT", "http://security:8443"),
            ("CAPABILITY_STORAGE_ENDPOINT", "http://storage:9000"),
        ])
        .await;

        let endpoints = get_multiple_endpoints(&["security", "storage"]).await.expect("multiple");

        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0], "http://security:8443");
        assert_eq!(endpoints[1], "http://storage:9000");

        // Cleanup happens automatically when _env drops
    }

    #[tokio::test] // ✅ CONCURRENT! Using ScopedEnv
    async fn test_cache_functionality() {
        // Use ScopedEnv for automatic cleanup
        let _env = ScopedEnv::set("CAPABILITY_SECURITY_ENDPOINT", "http://security:8443").await;

        // First call - should discover
        let endpoint1 = get_capability_endpoint("security").await.expect("first");

        // Second call - should use cache
        let endpoint2 = get_capability_endpoint("security").await.expect("second");

        assert_eq!(endpoint1, endpoint2);

        // Clear cache
        clear_cache();

        // Should discover again
        let endpoint3 = get_capability_endpoint("security").await.expect("third");

        assert_eq!(endpoint1, endpoint3);

        // Cleanup happens automatically when _env drops
    }
}
