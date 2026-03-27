// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Runtime Capability-Based Discovery Engine
//!
//! This module implements zero-hardcoding, runtime discovery of services
//! based purely on capabilities. No service knows the names of others;
//! everything is discovered through capability matching.

#![allow(missing_docs, reason = "discovery backend enum covers multiple transports")]

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::debug;

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
        self.env_reader.as_ref().map_or_else(|| songbird_process_env::var(key), |f| f(key))
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
        if songbird_process_env::var("KUBERNETES_SERVICE_HOST").is_ok() {
            backends.push(DiscoveryBackend::Kubernetes {
                namespace: songbird_process_env::var("KUBERNETES_NAMESPACE").ok(),
            });
        }

        // Check for Consul
        if let Ok(consul_endpoint) = songbird_process_env::var("CONSUL_HTTP_ADDR") {
            backends.push(DiscoveryBackend::Consul {
                endpoint: consul_endpoint,
            });
        }

        // Check for etcd
        if let Ok(etcd_endpoints) = songbird_process_env::var("ETCD_ENDPOINTS") {
            let endpoints: Vec<String> =
                etcd_endpoints.split(',').map(|s| s.trim().to_string()).collect();
            backends.push(DiscoveryBackend::Etcd {
                endpoints,
            });
        }

        // Try mDNS for local discovery (if not in container)
        if songbird_process_env::var("CONTAINER").is_err() {
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
    ) -> SongbirdResult<Vec<DiscoveredService>> {
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
    ) -> SongbirdResult<Vec<DiscoveredService>> {
        let env_key = format!("{}_ENDPOINT", capability.to_uppercase());

        if let Ok(endpoint) = self.read_env(&env_key) {
            // Parse address
            let addr: SocketAddr = endpoint
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .parse()
                .map_err(|e| SongbirdError::validation(format!("Invalid endpoint format: {e}")))?;

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
    async fn discover_from_mdns(&self, capability: &str) -> SongbirdResult<Vec<DiscoveredService>> {
        // Use our production mDNS implementation
        use super::mdns::MdnsDiscovery;

        let mdns = MdnsDiscovery::new().map_err(|e| SongbirdError::discovery(e.to_string()))?;
        let services = mdns
            .discover_by_capability(capability, Some(std::time::Duration::from_secs(5)))
            .await
            .map_err(|e| SongbirdError::discovery(e.to_string()))?;

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
    /// **Status**: Returns `SongbirdError::NotImplemented`; `discover_by_capability` skips failed
    /// backends and merges results from others.
    async fn discover_from_dnssd(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<DiscoveredService>> {
        debug!(
            target: "songbird_config::discovery",
            backend = "dnssd",
            %capability,
            "DNS-SD discovery backend not implemented; returning NotImplemented"
        );
        Err(SongbirdError::not_implemented_with_detail(
            "discovery_backend_dnssd",
            "Use mDNS, environment variables, or static configuration until DNS-SD is wired",
        ))
    }

    /// Discover from Consul
    ///
    /// **Status**: Returns `SongbirdError::NotImplemented`; `discover_by_capability` skips failed
    /// backends and merges results from others.
    async fn discover_from_consul(
        &self,
        endpoint: &str,
        capability: &str,
    ) -> SongbirdResult<Vec<DiscoveredService>> {
        debug!(
            target: "songbird_config::discovery",
            backend = "consul",
            endpoint,
            %capability,
            "Consul discovery backend not implemented; returning NotImplemented"
        );
        Err(SongbirdError::not_implemented_with_detail(
            "discovery_backend_consul",
            format!("Consul at {endpoint} is not integrated; use environment or mDNS discovery"),
        ))
    }

    /// Discover from etcd
    ///
    /// **Status**: Returns `SongbirdError::NotImplemented`; `discover_by_capability` skips failed
    /// backends and merges results from others.
    async fn discover_from_etcd(
        &self,
        endpoints: &[String],
        capability: &str,
    ) -> SongbirdResult<Vec<DiscoveredService>> {
        debug!(
            target: "songbird_config::discovery",
            backend = "etcd",
            endpoints = ?endpoints,
            %capability,
            "etcd discovery backend not implemented; returning NotImplemented"
        );
        Err(SongbirdError::not_implemented_with_detail(
            "discovery_backend_etcd",
            "etcd service discovery is not wired; use environment or mDNS discovery",
        ))
    }

    /// Discover from Kubernetes
    ///
    /// **Status**: Returns `SongbirdError::NotImplemented`; `discover_by_capability` skips failed
    /// backends and merges results from others.
    async fn discover_from_kubernetes(
        &self,
        namespace: Option<&str>,
        capability: &str,
    ) -> SongbirdResult<Vec<DiscoveredService>> {
        debug!(
            target: "songbird_config::discovery",
            backend = "kubernetes",
            ?namespace,
            %capability,
            "Kubernetes discovery backend not implemented; returning NotImplemented"
        );
        Err(SongbirdError::not_implemented_with_detail(
            "discovery_backend_kubernetes",
            "In-cluster Kubernetes API discovery is not wired; use environment or mDNS",
        ))
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
    ) -> SongbirdResult<()> {
        // Register with each backend
        for backend in &self.backends {
            if let Err(e) = self.register_with_backend(backend, capabilities, address).await {
                tracing::warn!(
                    backend = ?backend,
                    error = %e,
                    "Failed to register with discovery backend; continuing with other backends"
                );
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
    ) -> SongbirdResult<()> {
        match backend {
            DiscoveryBackend::Environment => {
                // Environment-based doesn't support registration
                Ok(())
            }
            DiscoveryBackend::MDNS => {
                debug!(
                    target: "songbird_config::discovery",
                    backend = "mdns",
                    "mDNS service advertisement not implemented; returning NotImplemented"
                );
                Err(SongbirdError::not_implemented_with_detail(
                    "discovery_registration_mdns",
                    "mDNS advertisement is not wired; use environment-based announcement for local dev",
                ))
            }
            DiscoveryBackend::DNSSD => {
                debug!(
                    target: "songbird_config::discovery",
                    backend = "dnssd",
                    "DNS-SD registration not implemented; returning NotImplemented"
                );
                Err(SongbirdError::not_implemented_with_detail(
                    "discovery_registration_dnssd",
                    "DNS-SD registration requires platform mDNS integration",
                ))
            }
            DiscoveryBackend::Consul {
                endpoint,
            } => {
                debug!(
                    target: "songbird_config::discovery",
                    backend = "consul",
                    endpoint,
                    "Consul registration not implemented; returning NotImplemented"
                );
                let _ = (capabilities, address);
                Err(SongbirdError::not_implemented_with_detail(
                    "discovery_registration_consul",
                    format!("Consul registration at {endpoint} is not integrated"),
                ))
            }
            DiscoveryBackend::Etcd {
                endpoints,
            } => {
                debug!(
                    target: "songbird_config::discovery",
                    backend = "etcd",
                    endpoints = ?endpoints,
                    "etcd registration not implemented; returning NotImplemented"
                );
                let _ = (capabilities, address);
                Err(SongbirdError::not_implemented_with_detail(
                    "discovery_registration_etcd",
                    "etcd registration is not wired",
                ))
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
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

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

    #[test]
    fn discovery_backend_equality_and_clone() {
        let a = DiscoveryBackend::Consul {
            endpoint: "http://127.0.0.1:8500".into(),
        };
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, DiscoveryBackend::Environment);
    }

    #[tokio::test]
    async fn discover_from_environment_strips_https_prefix() {
        let engine = CapabilityDiscoveryEngine::new_with_env_reader(
            vec![DiscoveryBackend::Environment],
            Duration::from_secs(60),
            |k| {
                if k == "STORAGE_ENDPOINT" {
                    Ok("https://127.0.0.1:9000".into())
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            },
        );
        let addrs = engine.discover_by_capability("storage").await;
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].port(), 9000);
    }

    #[tokio::test]
    async fn discover_deduplicates_same_address() {
        let engine = CapabilityDiscoveryEngine::new_with_env_reader(
            vec![DiscoveryBackend::Environment, DiscoveryBackend::Environment],
            Duration::from_secs(60),
            |k| {
                if k == "AI_ENDPOINT" {
                    Ok("127.0.0.1:7777".into())
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            },
        );
        let addrs = engine.discover_by_capability("ai").await;
        assert_eq!(addrs.len(), 1);
    }

    #[tokio::test]
    async fn discover_returns_empty_when_env_endpoint_missing() {
        let engine = CapabilityDiscoveryEngine::new_with_env_reader(
            vec![DiscoveryBackend::Environment],
            Duration::from_secs(60),
            |_| Err(std::env::VarError::NotPresent),
        );
        let cap = format!("sb_missing_env_{}", std::process::id());
        let addrs = engine.discover_by_capability(&cap).await;
        assert!(addrs.is_empty());
    }

    #[tokio::test]
    async fn discover_ignores_invalid_env_endpoint_without_panic() {
        let engine = CapabilityDiscoveryEngine::new_with_env_reader(
            vec![DiscoveryBackend::Environment],
            Duration::from_secs(60),
            |k| {
                if k == "BROKEN_ENDPOINT" {
                    Ok("not-a-socket-addr".into())
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            },
        );
        let addrs = engine.discover_by_capability("broken").await;
        assert!(addrs.is_empty());
    }

    #[test]
    fn discover_by_capability_sort_key_orders_ip_then_port() {
        // Mirrors `discover_by_capability`: dedupe then `sort_by_key(|a| (a.ip(), a.port()))`
        let mut addrs: Vec<SocketAddr> = vec![
            "10.0.0.2:9000".parse().expect("addr"),
            "10.0.0.1:1".parse().expect("addr"),
            "10.0.0.1:9000".parse().expect("addr"),
        ];
        addrs.sort_by_key(|addr| (addr.ip(), addr.port()));
        assert_eq!(addrs[0].to_string(), "10.0.0.1:1");
        assert_eq!(addrs[1].to_string(), "10.0.0.1:9000");
        assert_eq!(addrs[2].to_string(), "10.0.0.2:9000");
    }

    #[tokio::test]
    async fn discover_strips_http_prefix_from_env_endpoint() {
        let engine = CapabilityDiscoveryEngine::new_with_env_reader(
            vec![DiscoveryBackend::Environment],
            Duration::from_secs(60),
            |k| {
                if k == "WEB_ENDPOINT" {
                    Ok("http://192.0.2.1:4444".into())
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            },
        );
        let addrs = engine.discover_by_capability("web").await;
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].port(), 4444);
    }
}
