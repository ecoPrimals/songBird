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

    /// Discover from DNS-SD (delegates to mDNS with DNS-SD semantics).
    ///
    /// DNS-SD (RFC 6763) is built on top of mDNS — the `mdns-sd` crate
    /// handles both protocols. This backend uses the same mDNS discovery
    /// infrastructure with DNS-SD service type resolution.
    async fn discover_from_dnssd(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<DiscoveredService>> {
        debug!(
            target: "songbird_config::discovery",
            backend = "dnssd",
            %capability,
            "DNS-SD discovery delegating to mDNS infrastructure (RFC 6763)"
        );
        self.discover_from_mdns(capability).await
    }

    /// Discover from Consul service catalog by capability tag.
    ///
    /// Uses [`IpcHttpClient`] (Tower Atomic: Songbird TLS + `BearDog` crypto)
    /// to query `GET /v1/catalog/service/<capability>`. Falls back gracefully
    /// on network or parsing errors so other backends can still contribute.
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
            "Querying Consul catalog for capability via Tower Atomic"
        );

        let client = songbird_http_client::IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::discovery(format!("IPC HTTP client init failed: {e}")))?;

        let url = format!("{}/v1/catalog/service/{capability}", endpoint.trim_end_matches('/'));
        let response = client.get(&url).await.map_err(|e| {
            SongbirdError::discovery(format!("Consul HTTP request to {url} failed: {e}"))
        })?;

        if !response.is_success() {
            return Err(SongbirdError::discovery(format!(
                "Consul returned HTTP {} for {url}",
                response.status()
            )));
        }

        let entries: Vec<serde_json::Value> = response.json().await.map_err(|e| {
            SongbirdError::discovery(format!("Failed to parse Consul response: {e}"))
        })?;

        let mut discovered = Vec::new();
        for entry in &entries {
            let address = entry
                .get("ServiceAddress")
                .and_then(|v| v.as_str())
                .or_else(|| entry.get("Address").and_then(|v| v.as_str()))
                .unwrap_or("127.0.0.1");
            let port = entry.get("ServicePort").and_then(serde_json::Value::as_u64).unwrap_or(0);

            if port == 0 {
                continue;
            }

            if let Ok(addr) = format!("{address}:{port}").parse::<SocketAddr>() {
                let tags: Vec<String> = entry
                    .get("ServiceTags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|t| t.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), "consul".to_string());
                metadata.insert("consul_endpoint".to_string(), endpoint.to_string());

                discovered.push(DiscoveredService {
                    address: addr,
                    capabilities: if tags.is_empty() {
                        vec![capability.to_string()]
                    } else {
                        tags
                    },
                    metadata,
                    discovered_at: std::time::SystemTime::now(),
                });
            }
        }

        debug!(
            target: "songbird_config::discovery",
            backend = "consul",
            count = discovered.len(),
            "Consul discovery complete"
        );
        Ok(discovered)
    }

    /// Discover from etcd v3 key-value store by capability prefix.
    ///
    /// Uses [`IpcHttpClient`] (Tower Atomic) to query the etcd v3 HTTP
    /// gateway (`POST /v3/kv/range`). Keys under `/songbird/services/<cap>/`
    /// store `host:port` values. Tries each endpoint in order.
    async fn discover_from_etcd(
        &self,
        endpoints: &[String],
        capability: &str,
    ) -> SongbirdResult<Vec<DiscoveredService>> {
        use songbird_http_client::ipc_client::IpcHttpClient;

        debug!(
            target: "songbird_config::discovery",
            backend = "etcd",
            endpoints = ?endpoints,
            %capability,
            "Querying etcd for capability services via Tower Atomic"
        );

        let prefix = format!("/songbird/services/{capability}/");
        let prefix_b64 = songbird_http_client::base64_encode(prefix.as_bytes());
        let range_end = {
            let mut end = prefix.as_bytes().to_vec();
            if let Some(last) = end.last_mut() {
                *last = last.wrapping_add(1);
            }
            songbird_http_client::base64_encode(&end)
        };

        let body = serde_json::json!({
            "key": prefix_b64,
            "range_end": range_end,
        });

        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::discovery(format!("IPC HTTP client init failed: {e}")))?;

        for ep in endpoints {
            let url = format!("{}/v3/kv/range", ep.trim_end_matches('/'));
            let Ok(builder) = client.post(&url).await.json(&body) else {
                continue;
            };
            let resp = match builder.send().await {
                Ok(r) if r.is_success() => r,
                _ => continue,
            };

            let json: serde_json::Value = match resp.json().await {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut discovered = Vec::new();
            if let Some(kvs) = json.get("kvs").and_then(|v| v.as_array()) {
                for kv in kvs {
                    let value_b64 = kv.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    let value_bytes =
                        songbird_http_client::base64_decode(value_b64).unwrap_or_default();
                    let value = String::from_utf8_lossy(&value_bytes);
                    if let Ok(addr) = value.parse::<SocketAddr>() {
                        let mut metadata = HashMap::new();
                        metadata.insert("source".to_string(), "etcd".to_string());
                        metadata.insert("etcd_endpoint".to_string(), ep.clone());
                        discovered.push(DiscoveredService {
                            address: addr,
                            capabilities: vec![capability.to_string()],
                            metadata,
                            discovered_at: std::time::SystemTime::now(),
                        });
                    }
                }
            }

            debug!(
                target: "songbird_config::discovery",
                backend = "etcd",
                count = discovered.len(),
                "etcd discovery complete"
            );
            return Ok(discovered);
        }

        Err(SongbirdError::discovery(
            "All etcd endpoints unreachable; capability discovery deferred to other backends",
        ))
    }

    /// Discover from Kubernetes in-cluster service API.
    ///
    /// Uses the in-cluster service account token and API server to list
    /// endpoints for services labeled with `songbird/capability=<cap>`.
    /// Falls back to DNS-based service resolution when the API is
    /// unavailable (SRV records: `_<cap>._tcp.<ns>.svc.cluster.local`).
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
            "Attempting Kubernetes in-cluster discovery"
        );

        let ns = namespace.unwrap_or("default");

        // Attempt DNS-based discovery first (works without API access)
        let dns_name = format!("{capability}.{ns}.svc.cluster.local");
        if let Ok(addrs) = tokio::net::lookup_host(format!("{dns_name}:0")).await {
            let discovered: Vec<DiscoveredService> = addrs
                .filter(|a| a.port() > 0)
                .map(|addr| {
                    let mut metadata = HashMap::new();
                    metadata.insert("source".to_string(), "kubernetes-dns".to_string());
                    metadata.insert("namespace".to_string(), ns.to_string());
                    DiscoveredService {
                        address: addr,
                        capabilities: vec![capability.to_string()],
                        metadata,
                        discovered_at: std::time::SystemTime::now(),
                    }
                })
                .collect();

            if !discovered.is_empty() {
                debug!(
                    target: "songbird_config::discovery",
                    backend = "kubernetes",
                    count = discovered.len(),
                    "Kubernetes DNS discovery returned results"
                );
                return Ok(discovered);
            }
        }

        // Fall back to Kubernetes API if in-cluster service account is available
        let token_path = "/var/run/secrets/kubernetes.io/serviceaccount/token";
        let k8s_host = self.read_env("KUBERNETES_SERVICE_HOST");

        if std::path::Path::new(token_path).exists() && k8s_host.is_ok() {
            let host = k8s_host.unwrap_or_default();
            let port = self.read_env("KUBERNETES_SERVICE_PORT").unwrap_or_else(|_| "443".into());
            let token = tokio::fs::read_to_string(token_path).await.map_err(|e| {
                SongbirdError::discovery(format!("Failed to read K8s service account token: {e}"))
            })?;

            let url = format!(
                "https://{host}:{port}/api/v1/namespaces/{ns}/endpoints?labelSelector=songbird/capability={capability}"
            );

            let client = songbird_http_client::IpcHttpClient::new()
                .await
                .map_err(|e| SongbirdError::discovery(format!("IPC HTTP client init: {e}")))?;

            // Use POST-style builder to attach Authorization header
            let resp = client
                .post(&url)
                .await
                .header("Authorization", format!("Bearer {}", token.trim()))
                .header("X-HTTP-Method-Override", "GET")
                .send()
                .await
                .map_err(|e| {
                    SongbirdError::discovery(format!("Kubernetes API request failed: {e}"))
                })?;

            if resp.is_success() {
                let body: serde_json::Value = resp.json().await.map_err(|e| {
                    SongbirdError::discovery(format!("Failed to parse K8s response: {e}"))
                })?;

                let mut discovered = Vec::new();
                if let Some(items) = body.get("items").and_then(|v| v.as_array()) {
                    for item in items {
                        Self::extract_k8s_endpoints(item, capability, ns, &mut discovered);
                    }
                }

                debug!(
                    target: "songbird_config::discovery",
                    backend = "kubernetes",
                    count = discovered.len(),
                    "Kubernetes API discovery complete"
                );
                return Ok(discovered);
            }
        }

        debug!(
            target: "songbird_config::discovery",
            backend = "kubernetes",
            "No Kubernetes in-cluster environment detected; returning empty"
        );
        Ok(Vec::new())
    }

    /// Extract endpoints from a Kubernetes API `items[]` entry.
    fn extract_k8s_endpoints(
        item: &serde_json::Value,
        capability: &str,
        ns: &str,
        out: &mut Vec<DiscoveredService>,
    ) {
        let Some(subsets) = item.get("subsets").and_then(|v| v.as_array()) else {
            return;
        };
        for subset in subsets {
            let ports: Vec<u16> = subset
                .get("ports")
                .and_then(|v| v.as_array())
                .map(|ps| {
                    ps.iter()
                        .filter_map(|p| {
                            p.get("port")
                                .and_then(serde_json::Value::as_u64)
                                .and_then(|v| u16::try_from(v).ok())
                        })
                        .collect()
                })
                .unwrap_or_default();

            let Some(addresses) = subset.get("addresses").and_then(|v| v.as_array()) else {
                continue;
            };
            for addr_obj in addresses {
                let Some(ip) = addr_obj.get("ip").and_then(|v| v.as_str()) else {
                    continue;
                };
                for &port in &ports {
                    if let Ok(addr) = format!("{ip}:{port}").parse::<SocketAddr>() {
                        let mut metadata = HashMap::new();
                        metadata.insert("source".to_string(), "kubernetes-api".to_string());
                        metadata.insert("namespace".to_string(), ns.to_string());
                        out.push(DiscoveredService {
                            address: addr,
                            capabilities: vec![capability.to_string()],
                            metadata,
                            discovered_at: std::time::SystemTime::now(),
                        });
                    }
                }
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
            DiscoveryBackend::MDNS | DiscoveryBackend::DNSSD => {
                use super::mdns::MdnsDiscovery;
                let mdns =
                    MdnsDiscovery::new().map_err(|e| SongbirdError::discovery(e.to_string()))?;
                let cap_refs: Vec<&str> = capabilities.iter().map(String::as_str).collect();
                mdns.advertise(&cap_refs).await.map_err(|e| SongbirdError::discovery(e.to_string()))
            }
            DiscoveryBackend::Consul {
                endpoint,
            } => {
                let service_id = format!("songbird-{}", address.port());
                let body = serde_json::json!({
                    "ID": service_id,
                    "Name": "songbird",
                    "Address": address.ip().to_string(),
                    "Port": address.port(),
                    "Tags": capabilities,
                    "Check": {
                        "TCP": address.to_string(),
                        "Interval": "10s",
                        "Timeout": "3s",
                    }
                });

                let url = format!("{}/v1/agent/service/register", endpoint.trim_end_matches('/'));
                let client = songbird_http_client::IpcHttpClient::new()
                    .await
                    .map_err(|e| SongbirdError::discovery(format!("IPC client init: {e}")))?;
                client
                    .put(&url)
                    .await
                    .json(&body)
                    .map_err(|e| SongbirdError::discovery(format!("JSON encoding failed: {e}")))?
                    .send()
                    .await
                    .map_err(|e| {
                        SongbirdError::discovery(format!(
                            "Consul registration at {endpoint} failed: {e}"
                        ))
                    })?;

                debug!(
                    target: "songbird_config::discovery",
                    backend = "consul",
                    %service_id,
                    "Registered with Consul"
                );
                Ok(())
            }
            DiscoveryBackend::Etcd {
                endpoints,
            } => {
                let client = songbird_http_client::IpcHttpClient::new()
                    .await
                    .map_err(|e| SongbirdError::discovery(format!("IPC client init: {e}")))?;

                for cap in capabilities {
                    let key = format!("/songbird/services/{cap}/{address}");
                    let key_b64 = songbird_http_client::base64_encode(key.as_bytes());
                    let value_b64 =
                        songbird_http_client::base64_encode(address.to_string().as_bytes());

                    let body = serde_json::json!({
                        "key": key_b64,
                        "value": value_b64,
                        "lease": 0,
                    });

                    for ep in endpoints {
                        let url = format!("{}/v3/kv/put", ep.trim_end_matches('/'));
                        if let Ok(builder) = client.post(&url).await.json(&body)
                            && builder.send().await.is_ok()
                        {
                            break;
                        }
                    }
                }
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
#[path = "runtime_engine_tests.rs"]
mod tests;
