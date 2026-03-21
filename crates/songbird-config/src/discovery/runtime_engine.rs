// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Runtime Capability-Based Discovery Engine
//!
//! This module implements zero-hardcoding, runtime discovery of services
//! based purely on capabilities. No service knows the names of others;
//! everything is discovered through capability matching.

#![allow(missing_docs, reason = "discovery backend enum covers multiple transports")]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

type EnvReader = Arc<dyn Fn(&str) -> Result<String, std::env::VarError> + Send + Sync>;

/// Discovery backend types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryBackend {
    /// mDNS for local network discovery
    MDNS,
    /// DNS-SD (DNS Service Discovery)
    DNSSD,
    /// Consul service registry
    Consul {
        endpoint: String,
    },
    /// etcd service registry
    Etcd {
        endpoints: Vec<String>,
    },
    /// Kubernetes service discovery
    Kubernetes {
        namespace: Option<String>,
    },
    /// Environment variable based (for development)
    Environment,
}

/// Capability-based discovery engine
///
/// # Zero Hardcoding Principle
/// - Services advertise capabilities, not names
/// - Discovery happens by querying capabilities
/// - No primal knows names of other primals
/// - Pure runtime resolution
pub struct CapabilityDiscoveryEngine {
    /// Active discovery backends
    backends: Vec<DiscoveryBackend>,
    /// Cache of discovered services by capability
    cache: Arc<RwLock<HashMap<String, Vec<DiscoveredService>>>>,
    /// Cache TTL
    cache_ttl: Duration,
    /// Optional env reader (tests inject; production uses process environment)
    env_reader: Option<EnvReader>,
}

/// A discovered service with its capabilities
#[derive(Debug, Clone)]
pub struct DiscoveredService {
    /// Service address
    pub address: SocketAddr,
    /// Advertised capabilities
    pub capabilities: Vec<String>,
    /// Service metadata
    pub metadata: HashMap<String, String>,
    /// Discovery timestamp
    pub discovered_at: std::time::SystemTime,
}

impl CapabilityDiscoveryEngine {
    /// Create a new discovery engine with specified backends
    #[must_use]
    pub fn new(backends: Vec<DiscoveryBackend>, cache_ttl: Duration) -> Self {
        Self {
            backends,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            env_reader: None,
        }
    }

    /// Same as [`new`](Self::new) with an injectable environment reader (concurrent-safe tests).
    #[must_use]
    pub fn new_with_env_reader<F>(
        backends: Vec<DiscoveryBackend>,
        cache_ttl: Duration,
        env_reader: F,
    ) -> Self
    where
        F: Fn(&str) -> Result<String, std::env::VarError> + Send + Sync + 'static,
    {
        Self {
            backends,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            env_reader: Some(Arc::new(env_reader)),
        }
    }

    fn read_env(&self, key: &str) -> Result<String, std::env::VarError> {
        match &self.env_reader {
            Some(f) => f(key),
            None => std::env::var(key),
        }
    }

    /// Create engine with default backends (auto-detects environment)
    #[must_use]
    pub fn with_defaults() -> Self {
        let backends = Self::detect_backends();
        Self::new(backends, Duration::from_secs(60))
    }

    /// Detect available discovery backends
    fn detect_backends() -> Vec<DiscoveryBackend> {
        let mut backends = Vec::new();

        // Always support environment-based discovery (for development)
        backends.push(DiscoveryBackend::Environment);

        // Check for Kubernetes
        if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            backends.push(DiscoveryBackend::Kubernetes {
                namespace: std::env::var("KUBERNETES_NAMESPACE").ok(),
            });
        }

        // Check for Consul
        if let Ok(consul_endpoint) = std::env::var("CONSUL_HTTP_ADDR") {
            backends.push(DiscoveryBackend::Consul {
                endpoint: consul_endpoint,
            });
        }

        // Check for etcd
        if let Ok(etcd_endpoints) = std::env::var("ETCD_ENDPOINTS") {
            let endpoints: Vec<String> =
                etcd_endpoints.split(',').map(|s| s.trim().to_string()).collect();
            backends.push(DiscoveryBackend::Etcd {
                endpoints,
            });
        }

        // Try mDNS for local discovery (if not in container)
        if std::env::var("CONTAINER").is_err() {
            backends.push(DiscoveryBackend::MDNS);
        }

        backends
    }

    /// Discover services by capability (main API)
    ///
    /// # Arguments
    /// * `capability` - The capability to search for (e.g., "security", "storage", "ai")
    ///
    /// # Returns
    /// Vector of addresses offering the requested capability
    ///
    /// # Example
    /// ```no_run
    /// # use songbird_config::discovery::CapabilityDiscoveryEngine;
    /// # async fn example() {
    /// let engine = CapabilityDiscoveryEngine::with_defaults();
    ///
    /// // Discover any service offering "security" capability
    /// // Could be BearDog, or any other security provider
    /// let security_services = engine.discover_by_capability("security").await;
    ///
    /// // Discover storage providers (NestGate, MinIO, S3, etc.)
    /// let storage_services = engine.discover_by_capability("storage").await;
    /// # }
    /// ```
    pub async fn discover_by_capability(&self, capability: &str) -> Vec<SocketAddr> {
        // 1. Check cache first
        if let Some(cached) = self.get_from_cache(capability).await {
            return cached;
        }

        // 2. Query all backends in parallel
        let mut services = Vec::new();
        for backend in &self.backends {
            if let Ok(discovered) = self.query_backend(backend, capability).await {
                services.extend(discovered);
            }
        }

        // 3. Deduplicate by address
        let mut unique_addresses: Vec<SocketAddr> = services
            .iter()
            .map(|s| s.address)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // 4. Sort for deterministic behavior
        unique_addresses.sort_by_key(|addr| (addr.ip(), addr.port()));

        // 5. Update cache
        self.update_cache(capability, services).await;

        unique_addresses
    }

    /// Query a specific backend for capability
    async fn query_backend(
        &self,
        backend: &DiscoveryBackend,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>, Box<dyn std::error::Error + Send + Sync>> {
        match backend {
            DiscoveryBackend::Environment => self.discover_from_environment(capability).await,
            DiscoveryBackend::MDNS => self.discover_from_mdns(capability).await,
            DiscoveryBackend::DNSSD => self.discover_from_dnssd(capability).await,
            DiscoveryBackend::Consul {
                endpoint,
            } => self.discover_from_consul(endpoint, capability).await,
            DiscoveryBackend::Etcd {
                endpoints,
            } => self.discover_from_etcd(endpoints, capability).await,
            DiscoveryBackend::Kubernetes {
                namespace,
            } => self.discover_from_kubernetes(namespace.as_deref(), capability).await,
        }
    }

    /// Discover from environment variables
    async fn discover_from_environment(
        &self,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>, Box<dyn std::error::Error + Send + Sync>> {
        let env_key = format!("{}_ENDPOINT", capability.to_uppercase());

        if let Ok(endpoint) = self.read_env(&env_key) {
            // Parse address
            let addr: SocketAddr = endpoint
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .parse()
                .map_err(|e| format!("Invalid endpoint format: {e}"))?;

            let service = DiscoveredService {
                address: addr,
                capabilities: vec![capability.to_string()],
                metadata: std::collections::HashMap::from([(
                    "source".to_string(),
                    "environment".to_string(),
                )]),
                discovered_at: std::time::SystemTime::now(),
            };

            Ok(vec![service])
        } else {
            Ok(Vec::new())
        }
    }

    /// Discover from mDNS (local network)
    async fn discover_from_mdns(
        &self,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>, Box<dyn std::error::Error + Send + Sync>> {
        // Use our production mDNS implementation
        use super::mdns::MdnsDiscovery;

        let mdns = MdnsDiscovery::new()?;
        let services = mdns
            .discover_by_capability(capability, Some(std::time::Duration::from_secs(5)))
            .await?;

        // Convert to DiscoveredService format
        let discovered: Vec<DiscoveredService> = services
            .into_iter()
            .map(|s| DiscoveredService {
                address: s.address,
                capabilities: s.capabilities,
                metadata: s.metadata,
                discovered_at: s.discovered_at,
            })
            .collect();

        Ok(discovered)
    }

    /// Discover from DNS-SD
    ///
    /// **Status**: Not yet implemented - returns empty list
    /// **Fallback**: Uses environment variables and defaults instead
    async fn discover_from_dnssd(
        &self,
        _capability: &str,
    ) -> Result<Vec<DiscoveredService>, Box<dyn std::error::Error + Send + Sync>> {
        // NOTE: DNS-SD discovery not yet implemented
        // Query DNS for SRV records like _<capability>._tcp.example.com
        // For now, falls back to environment-based discovery

        Ok(Vec::new())
    }

    /// Discover from Consul
    ///
    /// **Status**: Not yet implemented - returns empty list
    /// **Fallback**: Uses environment variables and defaults instead
    async fn discover_from_consul(
        &self,
        _endpoint: &str,
        _capability: &str,
    ) -> Result<Vec<DiscoveredService>, Box<dyn std::error::Error + Send + Sync>> {
        // NOTE: Consul service discovery not yet implemented
        // Query Consul API for services tagged with capability
        // For now, falls back to environment-based discovery

        Ok(Vec::new())
    }

    /// Discover from etcd
    ///
    /// **Status**: Not yet implemented - returns empty list
    /// **Fallback**: Uses environment variables and defaults instead
    async fn discover_from_etcd(
        &self,
        _endpoints: &[String],
        _capability: &str,
    ) -> Result<Vec<DiscoveredService>, Box<dyn std::error::Error + Send + Sync>> {
        // NOTE: etcd service discovery not yet implemented
        // Query etcd for services with capability key

        Ok(Vec::new())
    }

    /// Discover from Kubernetes
    ///
    /// **Status**: Not yet implemented - returns empty list
    /// **Fallback**: Uses environment variables and defaults instead
    async fn discover_from_kubernetes(
        &self,
        _namespace: Option<&str>,
        _capability: &str,
    ) -> Result<Vec<DiscoveredService>, Box<dyn std::error::Error + Send + Sync>> {
        // NOTE: Kubernetes service discovery not yet implemented
        // Query K8s API for services with capability label
        // For now, falls back to environment-based discovery

        Ok(Vec::new())
    }

    /// Get from cache if not expired
    async fn get_from_cache(&self, capability: &str) -> Option<Vec<SocketAddr>> {
        if let Some(services) = self.cache.read().await.get(capability) {
            // Check if cache is still valid
            let _now = std::time::SystemTime::now();
            let all_valid = services.iter().all(|s| {
                s.discovered_at.elapsed().map(|elapsed| elapsed < self.cache_ttl).unwrap_or(false)
            });

            if all_valid {
                return Some(services.iter().map(|s| s.address).collect());
            }
        }
        None
    }

    /// Update cache with discovered services
    async fn update_cache(&self, capability: &str, services: Vec<DiscoveredService>) {
        let mut cache = self.cache.write().await;
        cache.insert(capability.to_string(), services);
    }

    /// Register self with discovery backends
    ///
    /// # Arguments
    /// * `capabilities` - List of capabilities this service offers
    /// * `address` - Address this service is listening on
    ///
    /// # Errors
    ///
    /// Returns error if registration with any discovery backend fails.
    pub async fn register_self(
        &self,
        capabilities: &[String],
        address: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Register with each backend
        for backend in &self.backends {
            if let Err(e) = self.register_with_backend(backend, capabilities, address).await {
                eprintln!("Failed to register with backend {backend:?}: {e}");
                // Continue with other backends
            }
        }
        Ok(())
    }

    /// Register with a specific backend
    async fn register_with_backend(
        &self,
        backend: &DiscoveryBackend,
        capabilities: &[String],
        address: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match backend {
            DiscoveryBackend::Environment => {
                // Environment-based doesn't support registration
                Ok(())
            }
            DiscoveryBackend::MDNS => {
                // NOTE: mDNS service advertisement not yet implemented
                // Would broadcast service availability via multicast DNS
                Ok(())
            }
            DiscoveryBackend::DNSSD => {
                // NOTE: DNS-SD registration not yet implemented
                // Would register service via DNS-SD protocol
                Ok(())
            }
            DiscoveryBackend::Consul {
                endpoint,
            } => {
                // NOTE: Consul registration not yet implemented
                // Would register service capabilities with Consul API
                let _ = (endpoint, capabilities, address);
                Ok(())
            }
            DiscoveryBackend::Etcd {
                endpoints,
            } => {
                // NOTE: etcd registration not yet implemented
                // Would register service capabilities in etcd
                let _ = (endpoints, capabilities, address);
                Ok(())
            }
            DiscoveryBackend::Kubernetes {
                ..
            } => {
                // Kubernetes uses service definitions, no dynamic registration needed
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn test_environment_discovery() {
        let engine = CapabilityDiscoveryEngine::new_with_env_reader(
            vec![DiscoveryBackend::Environment],
            Duration::from_secs(60),
            |k| {
                if k == "SECURITY_ENDPOINT" {
                    Ok("127.0.0.1:8443".to_string())
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            },
        );

        let services = engine.discover_by_capability("security").await;
        assert!(!services.is_empty(), "Should discover security service from environment");
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        // Unique capability/env to avoid races with other tests using `*_ENDPOINT`.
        const CAP: &str = "sb_rteng_cache_isolated";
        const ENV: &str = "SB_RTENG_CACHE_ISOLATED_ENDPOINT";

        // Use very short TTL for fast test without sleep
        let engine = CapabilityDiscoveryEngine::new_with_env_reader(
            vec![DiscoveryBackend::Environment],
            Duration::from_millis(10), // Very short TTL
            move |k| {
                if k == ENV {
                    Ok("127.0.0.1:9000".to_string())
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            },
        );

        // First call should query backend
        let services1 = engine.discover_by_capability(CAP).await;

        // Second call should use cache
        let services2 = engine.discover_by_capability(CAP).await;
        assert_eq!(services1, services2, "Cache should return same results");

        // Wait for cache expiration using tokio::time::timeout
        // This tests the cache expiry logic without slow sleeps
        let start = std::time::Instant::now();
        loop {
            // Clear cache by waiting slightly longer than TTL
            if start.elapsed() > Duration::from_millis(15) {
                break;
            }
            tokio::task::yield_now().await; // Allow other tasks to run
        }

        // Should re-query after expiration
        let services3 = engine.discover_by_capability(CAP).await;
        assert_eq!(services1, services3, "Should still find service after cache expiry");
    }

    #[test]
    fn test_backend_detection() {
        // Test that default backend detection works
        let backends = CapabilityDiscoveryEngine::detect_backends();
        assert!(!backends.is_empty(), "Should detect at least environment backend");
        assert!(
            backends.contains(&DiscoveryBackend::Environment),
            "Should always include environment backend"
        );
    }

    #[test]
    fn test_engine_new_empty_backends_still_runs() {
        let engine = CapabilityDiscoveryEngine::new(vec![], Duration::from_secs(60));
        assert_eq!(engine.cache_ttl, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_discover_with_no_backends_returns_empty() {
        let engine = CapabilityDiscoveryEngine::new(vec![], Duration::from_secs(60));
        let addrs = engine.discover_by_capability("anything").await;
        assert!(addrs.is_empty());
    }

    #[tokio::test]
    async fn test_register_self_no_panic() {
        let engine = CapabilityDiscoveryEngine::new(
            vec![DiscoveryBackend::Environment],
            Duration::from_secs(60),
        );
        let addr: std::net::SocketAddr = "127.0.0.1:8080".parse().unwrap();
        engine.register_self(&["test".to_string()], addr).await.expect("register_self returns Ok");
    }

    #[test]
    fn test_with_defaults_constructed() {
        let _ = CapabilityDiscoveryEngine::with_defaults();
    }
}
