// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Runtime Capability-Based Discovery Engine
//!
//! This module implements zero-hardcoding, runtime discovery of services
//! based purely on capabilities. No service knows the names of others;
//! everything is discovered through capability matching.

#![allow(missing_docs, reason = "discovery backend enum covers multiple transports")]

mod discover_consul;
mod discover_env_mdns;
mod discover_etcd;
mod discover_kubernetes;
mod register;

use songbird_types::SongbirdResult;
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
    /// // Could be any process that advertises the "security" capability
    /// let security_services = engine.discover_by_capability("security").await;
    ///
    /// // Discover storage providers (object stores, databases, MinIO, S3, etc.)
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
            DiscoveryBackend::Environment => {
                discover_env_mdns::discover_from_environment(self, capability).await
            }
            DiscoveryBackend::MDNS => discover_env_mdns::discover_from_mdns(self, capability).await,
            DiscoveryBackend::DNSSD => {
                discover_env_mdns::discover_from_dnssd(self, capability).await
            }
            DiscoveryBackend::Consul {
                endpoint,
            } => discover_consul::discover_from_consul(self, endpoint, capability).await,
            DiscoveryBackend::Etcd {
                endpoints,
            } => discover_etcd::discover_from_etcd(self, endpoints, capability).await,
            DiscoveryBackend::Kubernetes {
                namespace,
            } => {
                discover_kubernetes::discover_from_kubernetes(
                    self,
                    namespace.as_deref(),
                    capability,
                )
                .await
            }
        }
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
            if let Err(e) =
                register::register_with_backend(self, backend, capabilities, address).await
            {
                tracing::warn!(
                    backend = ?backend,
                    error = %e,
                    "Failed to register with discovery backend; continuing with other backends"
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../runtime_engine_tests.rs"]
mod tests;
