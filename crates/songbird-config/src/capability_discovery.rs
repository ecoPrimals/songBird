// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-Based Service Discovery
//!
//! Modern replacement for hardcoded primal endpoints. Each primal discovers others
//! through runtime capability-based discovery, respecting sovereignty principles.
//!
//! ## Sovereignty Principles
//!
//! 1. **Self-Knowledge Only**: Each primal knows only about itself
//! 2. **Runtime Discovery**: All inter-primal communication discovered at runtime
//! 3. **No Hardcoding**: Zero compile-time dependencies on other primals
//! 4. **Capability-Based**: Route by what you need, not who provides it

use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Service endpoint discovered through capability-based discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Service identifier (not necessarily a primal name)
    pub id: String,

    /// Endpoint URL
    pub url: String,

    /// Capabilities this service offers
    pub capabilities: Vec<String>,

    /// Health score (0.0-1.0)
    pub health_score: f64,

    /// Last seen timestamp
    pub last_seen: std::time::SystemTime,
}

/// Discovery method for finding services
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Environment variables (`COMPUTE_ENDPOINT`, `STORAGE_ENDPOINT`, etc.)
    Environment,

    /// DNS Service Discovery (_compute._tcp, etc.)
    DnsSD,

    /// Multicast DNS (zero-conf)
    MDNS,

    /// Central registry (Songbird's capability registry)
    Registry {
        endpoint: String,
    },

    /// Direct configuration file
    ConfigFile {
        path: String,
    },
}

/// Capability-based service discovery engine
pub struct CapabilityDiscovery {
    /// Discovered services cache
    services: Arc<RwLock<HashMap<String, Vec<ServiceEndpoint>>>>,

    /// Enabled discovery methods
    methods: Vec<DiscoveryMethod>,

    /// Cache TTL for service endpoint validity
    /// Used in future cache invalidation logic
    #[expect(dead_code, reason = "reserved for future cache invalidation")]
    cache_ttl: Duration,
}

impl CapabilityDiscovery {
    /// Create new discovery engine with default methods
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            methods: vec![
                DiscoveryMethod::Environment,
                DiscoveryMethod::DnsSD,
                DiscoveryMethod::MDNS,
            ],
            cache_ttl: Duration::from_secs(60),
        }
    }

    /// Create with specific discovery methods
    #[must_use]
    pub fn with_methods(methods: Vec<DiscoveryMethod>) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            methods,
            cache_ttl: Duration::from_secs(60),
        }
    }

    /// Discover services providing a specific capability
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use songbird_config::capability_discovery::CapabilityDiscovery;
    ///
    /// #[tokio::main]
    /// async fn main() -> songbird_types::SongbirdResult<()> {
    ///     let discovery = CapabilityDiscovery::new();
    ///     
    ///     // Discover ANY provider offering "compute" capability
    ///     // Could be ToadStool, or ANY other compute provider
    ///     let compute_providers = discovery
    ///         .find_providers_by_capability("compute")
    ///         .await?;
    ///     
    ///     for provider in compute_providers {
    ///         println!("Found compute provider: {} at {}", provider.id, provider.url);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Discovery methods fail to find providers
    /// - Network errors occur during discovery
    /// - Cache operations fail
    pub async fn find_providers_by_capability(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        // Check cache first
        {
            let services = self.services.read().await;
            if let Some(cached) = services.get(capability)
                && !cached.is_empty()
            {
                return Ok(cached.clone());
            }
        }

        // Discover through enabled methods
        let mut discovered = Vec::new();

        for method in &self.methods {
            match self.discover_via_method(capability, method).await {
                Ok(mut endpoints) => {
                    debug!(
                        "Discovered {} providers for '{}' via {:?}",
                        endpoints.len(),
                        capability,
                        method
                    );
                    discovered.append(&mut endpoints);
                }
                Err(e) => {
                    debug!("Discovery failed for '{}' via {:?}: {}", capability, method, e);
                }
            }
        }

        if discovered.is_empty() {
            return Err(SongbirdError::Discovery {
                message: format!(
                    "No providers found for capability '{}'. Enable discovery or set {}_ENDPOINT environment variable.",
                    capability,
                    capability.to_uppercase()
                ),
                backend: Some("all_methods".to_string()),
                retry_strategy: Some(
                    "Set environment variable or enable discovery methods".to_string(),
                ),
            });
        }

        // Cache results
        {
            let mut services = self.services.write().await;
            services.insert(capability.to_string(), discovered.clone());
        }

        info!("✅ Discovered {} providers for capability '{}'", discovered.len(), capability);

        Ok(discovered)
    }

    /// Discover via specific method
    async fn discover_via_method(
        &self,
        capability: &str,
        method: &DiscoveryMethod,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        match method {
            DiscoveryMethod::Environment => self.discover_via_environment(capability).await,
            DiscoveryMethod::DnsSD => self.discover_via_dnssd(capability).await,
            DiscoveryMethod::MDNS => self.discover_via_mdns(capability).await,
            DiscoveryMethod::Registry {
                endpoint,
            } => self.discover_via_registry(capability, endpoint).await,
            DiscoveryMethod::ConfigFile {
                path,
            } => self.discover_via_config_file(capability, path).await,
        }
    }

    /// Discover via environment variables
    #[expect(
        clippy::unused_async,
        reason = "no .await needed for environment variable reads; async for uniform discover_via_method"
    )]
    async fn discover_via_environment(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        // Try {CAPABILITY}_ENDPOINT environment variable
        let env_var = format!("{}_ENDPOINT", capability.to_uppercase());

        if let Ok(endpoint_url) = std::env::var(&env_var) {
            debug!("Found {} in environment: {}", env_var, endpoint_url);

            return Ok(vec![ServiceEndpoint {
                id: format!("{capability}-provider-env"),
                url: endpoint_url,
                capabilities: vec![capability.to_string()],
                health_score: 1.0,
                last_seen: std::time::SystemTime::now(),
            }]);
        }

        Err(SongbirdError::Discovery {
            message: format!("Environment variable {env_var} not set"),
            backend: Some("environment".to_string()),
            retry_strategy: Some(format!("Set {env_var} environment variable")),
        })
    }

    /// Discover via DNS-SD (DNS Service Discovery)
    ///
    /// Uses DNS SRV records to discover services advertising capabilities.
    /// Service format: _{capability}._tcp.local
    ///
    /// ## Example DNS-SD Record
    /// ```text
    /// _compute._tcp.local.  IN SRV 0 5 8001 toadstool.local.
    /// ```
    async fn discover_via_dnssd(&self, capability: &str) -> SongbirdResult<Vec<ServiceEndpoint>> {
        use hickory_resolver::TokioAsyncResolver;
        use hickory_resolver::config::{ResolverConfig, ResolverOpts};

        // DNS-SD service type: _{capability}._tcp.local
        let service_name = format!("_{capability}._tcp.local");

        debug!("Attempting DNS-SD discovery for: {}", service_name);

        // Create resolver with system configuration
        // Note: tokio() returns the resolver directly, not a Result
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        // Query SRV records for service discovery
        match resolver.srv_lookup(&service_name).await {
            Ok(srv_records) => {
                let mut endpoints = Vec::new();

                for srv in srv_records.iter() {
                    let target = srv.target().to_utf8();
                    let port = srv.port();

                    // Construct endpoint URL
                    let url = format!("http://{target}:{port}");

                    debug!("Discovered via DNS-SD: {}", url);

                    endpoints.push(ServiceEndpoint {
                        id: format!("{capability}-dnssd-{target}"),
                        url,
                        capabilities: vec![capability.to_string()],
                        health_score: 1.0,
                        last_seen: std::time::SystemTime::now(),
                    });
                }

                if endpoints.is_empty() {
                    Err(SongbirdError::Discovery {
                        message: format!("No DNS-SD records found for {service_name}"),
                        backend: Some("dns-sd".to_string()),
                        retry_strategy: Some(
                            "Ensure service is advertising via DNS-SD".to_string(),
                        ),
                    })
                } else {
                    info!(
                        "Discovered {} services via DNS-SD for '{}'",
                        endpoints.len(),
                        capability
                    );
                    Ok(endpoints)
                }
            }
            Err(e) => {
                debug!("DNS-SD lookup failed for {}: {}", service_name, e);
                Err(SongbirdError::Discovery {
                    message: format!("DNS-SD lookup failed: {e}"),
                    backend: Some("dns-sd".to_string()),
                    retry_strategy: Some(
                        "Check DNS configuration and service advertisement".to_string(),
                    ),
                })
            }
        }
    }

    /// Discover via mDNS (Multicast DNS / Zero-conf)
    ///
    /// Uses multicast DNS for zero-configuration service discovery on local networks.
    /// This is ideal for development and small deployments where services can discover
    /// each other without centralized DNS infrastructure.
    ///
    /// ## Implementation Note
    ///
    /// mDNS discovery requires a platform-specific implementation:
    /// - **Linux**: Avahi D-Bus integration
    /// - **macOS/iOS**: Bonjour framework
    /// - **Windows**: DNS-SD API
    ///
    /// For cross-platform compatibility, consider using:
    /// - `mdns` crate (pure Rust, limited platform support)
    /// - `zeroconf` crate (native bindings, better compatibility)
    ///
    /// ## Future Enhancement
    ///
    /// ```rust,ignore
    /// use mdns::{Record, RecordKind};
    /// use std::time::Duration;
    ///
    /// async fn discover_mdns(capability: &str) -> Result<Vec<ServiceEndpoint>> {
    ///     let service_name = format!("_{}._tcp.local", capability);
    ///     
    ///     // Browse for services on local network
    ///     let responses = mdns::discover::all(service_name, Duration::from_secs(5))?
    ///         .listen();
    ///     
    ///     // Collect discovered services
    ///     for response in responses {
    ///         if let Some(ip) = response.ip_addr() {
    ///             endpoints.push(ServiceEndpoint {
    ///                 url: format!("http://{}:{}", ip, port),
    ///                 // ...
    ///             });
    ///         }
    ///     }
    /// }
    /// ```
    async fn discover_via_mdns(&self, capability: &str) -> SongbirdResult<Vec<ServiceEndpoint>> {
        use crate::discovery::MdnsDiscovery;

        debug!("🌐 Starting mDNS discovery for capability: {}", capability);

        // Create mDNS discovery client
        let mdns = match MdnsDiscovery::new() {
            Ok(mdns) => mdns,
            Err(e) => {
                warn!("Failed to initialize mDNS discovery: {} - falling back to other methods", e);
                return Ok(vec![]);
            }
        };

        // Discover services with this capability (3 second timeout)
        let timeout = Duration::from_secs(3);
        match mdns.discover_by_capability(capability, Some(timeout)).await {
            Ok(services) => {
                info!(
                    "✅ mDNS discovered {} service(s) for capability '{}'",
                    services.len(),
                    capability
                );

                // Convert mDNS MdnsServiceInfo to our ServiceEndpoint
                let endpoints: Vec<ServiceEndpoint> = services
                    .into_iter()
                    .map(|svc| ServiceEndpoint {
                        id: format!("mdns-{}", svc.address), // Use address as ID since service_name not in struct
                        url: format!("http://{}", svc.address), // Convert SocketAddr to URL
                        capabilities: svc.capabilities,
                        health_score: 1.0, // Assume healthy if discovered
                        last_seen: svc.discovered_at,
                    })
                    .collect();

                Ok(endpoints)
            }
            Err(e) => {
                debug!(
                    "mDNS discovery returned no results for '{}': {} - trying other methods",
                    capability, e
                );
                Ok(vec![])
            }
        }
    }

    /// Discover via central registry (Songbird's capability registry)
    async fn discover_via_registry(
        &self,
        capability: &str,
        registry_endpoint: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        debug!("Querying registry at {} for capability: {}", registry_endpoint, capability);

        // Query Songbird's capability registry
        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;
        let url = format!("{registry_endpoint}/api/capabilities/{capability}");

        match client.get(&url).await {
            Ok(response) => {
                if response.is_success() {
                    match response.json::<Vec<ServiceEndpoint>>().await {
                        Ok(endpoints) => {
                            info!(
                                "Registry returned {} providers for '{}'",
                                endpoints.len(),
                                capability
                            );
                            Ok(endpoints)
                        }
                        Err(e) => Err(SongbirdError::Discovery {
                            message: format!("Failed to parse registry response: {e}"),
                            backend: Some("registry".to_string()),
                            retry_strategy: Some("Check registry endpoint and format".to_string()),
                        }),
                    }
                } else {
                    Err(SongbirdError::Discovery {
                        message: format!("Registry returned error: {}", response.status()),
                        backend: Some("registry".to_string()),
                        retry_strategy: Some("Check registry endpoint availability".to_string()),
                    })
                }
            }
            Err(e) => Err(SongbirdError::Discovery {
                message: format!("Failed to query registry: {e}"),
                backend: Some("registry".to_string()),
                retry_strategy: Some("Check network connectivity to registry".to_string()),
            }),
        }
    }

    /// Discover via configuration file
    ///
    /// Reads service configurations from TOML, JSON, or YAML files.
    /// Supports standard configuration paths and formats.
    async fn discover_via_config_file(
        &self,
        capability: &str,
        config_path: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        debug!("Reading configuration file at {} for capability: {}", config_path, capability);

        // Read configuration file
        let config_content =
            tokio::fs::read_to_string(config_path).await.map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to read config file: {e}"),
                backend: Some("config_file".to_string()),
                retry_strategy: Some("Check file path and permissions".to_string()),
            })?;

        // Parse based on file extension using Path for case-insensitive comparison
        let path = Path::new(config_path);
        let endpoints = if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("toml")) {
            Self::parse_toml_config(&config_content, capability)?
        } else if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("json")) {
            Self::parse_json_config(&config_content, capability)?
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
        {
            Self::parse_yaml_config(&config_content, capability)?
        } else {
            return Err(SongbirdError::Discovery {
                message: format!("Unsupported config file format: {config_path}"),
                backend: Some("config_file".to_string()),
                retry_strategy: Some("Use .toml, .json, or .yaml file".to_string()),
            });
        };

        if endpoints.is_empty() {
            warn!("No endpoints found for capability '{}' in config file", capability);
        } else {
            info!(
                "Found {} endpoints for capability '{}' in config file",
                endpoints.len(),
                capability
            );
        }

        Ok(endpoints)
    }

    /// Parse TOML configuration
    fn parse_toml_config(content: &str, capability: &str) -> SongbirdResult<Vec<ServiceEndpoint>> {
        let config: toml::Value =
            toml::from_str(content).map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to parse TOML: {e}"),
                backend: Some("config_file".to_string()),
                retry_strategy: Some("Check TOML syntax".to_string()),
            })?;

        Ok(Self::extract_endpoints_from_config(&config, capability))
    }

    /// Parse JSON configuration
    fn parse_json_config(content: &str, capability: &str) -> SongbirdResult<Vec<ServiceEndpoint>> {
        let config: serde_json::Value =
            serde_json::from_str(content).map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to parse JSON: {e}"),
                backend: Some("config_file".to_string()),
                retry_strategy: Some("Check JSON syntax".to_string()),
            })?;

        Ok(Self::extract_endpoints_from_json(&config, capability))
    }

    /// Parse YAML configuration
    fn parse_yaml_config(content: &str, capability: &str) -> SongbirdResult<Vec<ServiceEndpoint>> {
        let config: serde_yaml::Value =
            serde_yaml::from_str(content).map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to parse YAML: {e}"),
                backend: Some("config_file".to_string()),
                retry_strategy: Some("Check YAML syntax".to_string()),
            })?;

        Ok(Self::extract_endpoints_from_yaml(&config, capability))
    }

    /// Extract endpoints from TOML config
    fn extract_endpoints_from_config(
        config: &toml::Value,
        capability: &str,
    ) -> Vec<ServiceEndpoint> {
        let mut endpoints = Vec::new();

        if let Some(services) = config.get("services").and_then(|s| s.as_table()) {
            for (service_id, service_config) in services {
                if let Some(caps) = service_config.get("capabilities").and_then(|c| c.as_array()) {
                    let has_capability = caps.iter().any(|c| c.as_str() == Some(capability));

                    if has_capability
                        && let Some(url) = service_config.get("url").and_then(|u| u.as_str())
                    {
                        endpoints.push(ServiceEndpoint {
                            id: service_id.clone(),
                            url: url.to_string(),
                            capabilities: caps
                                .iter()
                                .filter_map(|c| c.as_str().map(String::from))
                                .collect(),
                            health_score: service_config
                                .get("health_score")
                                .and_then(toml::Value::as_float)
                                .unwrap_or(1.0),
                            last_seen: std::time::SystemTime::now(),
                        });
                    }
                }
            }
        }

        endpoints
    }

    /// Extract endpoints from JSON config
    fn extract_endpoints_from_json(
        config: &serde_json::Value,
        capability: &str,
    ) -> Vec<ServiceEndpoint> {
        let mut endpoints = Vec::new();

        if let Some(services) = config.get("services").and_then(|s| s.as_object()) {
            for (service_id, service_config) in services {
                if let Some(caps) = service_config.get("capabilities").and_then(|c| c.as_array()) {
                    let has_capability = caps.iter().any(|c| c.as_str() == Some(capability));

                    if has_capability
                        && let Some(url) = service_config.get("url").and_then(|u| u.as_str())
                    {
                        endpoints.push(ServiceEndpoint {
                            id: service_id.clone(),
                            url: url.to_string(),
                            capabilities: caps
                                .iter()
                                .filter_map(|c| c.as_str().map(String::from))
                                .collect(),
                            health_score: service_config
                                .get("health_score")
                                .and_then(serde_json::Value::as_f64)
                                .unwrap_or(1.0),
                            last_seen: std::time::SystemTime::now(),
                        });
                    }
                }
            }
        }

        endpoints
    }

    /// Extract endpoints from YAML config
    fn extract_endpoints_from_yaml(
        config: &serde_yaml::Value,
        capability: &str,
    ) -> Vec<ServiceEndpoint> {
        let mut endpoints = Vec::new();

        if let Some(services) = config.get("services").and_then(|s| s.as_mapping()) {
            for (service_id, service_config) in services {
                if let Some(caps) = service_config.get("capabilities").and_then(|c| c.as_sequence())
                {
                    let has_capability = caps.iter().any(|c| c.as_str() == Some(capability));

                    if has_capability
                        && let Some(url) = service_config.get("url").and_then(|u| u.as_str())
                    {
                        let service_id_str = service_id.as_str().unwrap_or("unknown").to_string();

                        endpoints.push(ServiceEndpoint {
                            id: service_id_str,
                            url: url.to_string(),
                            capabilities: caps
                                .iter()
                                .filter_map(|c| c.as_str().map(String::from))
                                .collect(),
                            health_score: service_config
                                .get("health_score")
                                .and_then(serde_yaml::Value::as_f64)
                                .unwrap_or(1.0),
                            last_seen: std::time::SystemTime::now(),
                        });
                    }
                }
            }
        }

        endpoints
    }

    /// Clear cache for a specific capability
    pub async fn clear_cache(&self, capability: &str) {
        let mut services = self.services.write().await;
        services.remove(capability);
    }

    /// Clear all cached discoveries
    pub async fn clear_all_caches(&self) {
        let mut services = self.services.write().await;
        services.clear();
    }
}

impl Default for CapabilityDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common capabilities
impl CapabilityDiscovery {
    /// Discover compute providers (replaces hardcoded `ToadStool` endpoint)
    ///
    /// # Errors
    ///
    /// Returns an error if no compute providers are found via any discovery method.
    pub async fn discover_compute(&self) -> SongbirdResult<Vec<ServiceEndpoint>> {
        self.find_providers_by_capability("compute").await
    }

    /// Discover storage providers (replaces hardcoded `NestGate` endpoint)
    ///
    /// # Errors
    ///
    /// Returns an error if no storage providers are found via any discovery method.
    pub async fn discover_storage(&self) -> SongbirdResult<Vec<ServiceEndpoint>> {
        self.find_providers_by_capability("storage").await
    }

    /// Discover security providers (replaces hardcoded `BearDog` endpoint)
    ///
    /// # Errors
    ///
    /// Returns an error if no security providers are found via any discovery method.
    pub async fn discover_security(&self) -> SongbirdResult<Vec<ServiceEndpoint>> {
        self.find_providers_by_capability("security").await
    }

    /// Discover AI providers (replaces hardcoded `Squirrel` endpoint)
    ///
    /// # Errors
    ///
    /// Returns an error if no AI providers are found via any discovery method.
    pub async fn discover_ai(&self) -> SongbirdResult<Vec<ServiceEndpoint>> {
        self.find_providers_by_capability("ai").await
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_discovery() {
        songbird_process_env::set_var("COMPUTE_ENDPOINT", "http://10.0.0.100:8001");

        let discovery = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);

        let providers = discovery.discover_compute().await.expect("compute from env");
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].url, "http://10.0.0.100:8001");
        assert!(providers[0].capabilities.contains(&"compute".to_string()));

        songbird_process_env::remove_var("COMPUTE_ENDPOINT");
    }

    #[tokio::test]
    async fn test_no_providers_found() {
        songbird_process_env::remove_var("NONEXISTENT_ENDPOINT");

        let discovery = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);

        let result = discovery.find_providers_by_capability("nonexistent").await;
        assert!(result.is_err());

        if let Err(SongbirdError::Discovery {
            message,
            ..
        }) = result
        {
            assert!(message.contains("No providers found"));
            assert!(message.contains("NONEXISTENT_ENDPOINT"));
        } else {
            panic!("Expected Discovery error");
        }
    }

    #[tokio::test]
    async fn test_cache_behavior() {
        songbird_process_env::set_var("TEST_CAPABILITY_ENDPOINT", "http://test:1234");

        let discovery = CapabilityDiscovery::with_methods(vec![DiscoveryMethod::Environment]);

        // First discovery
        let providers1 = discovery
            .find_providers_by_capability("test_capability")
            .await
            .expect("first discovery");

        // Second discovery should use cache
        let providers2 = discovery
            .find_providers_by_capability("test_capability")
            .await
            .expect("cached discovery");

        assert_eq!(providers1.len(), providers2.len());

        // Clear cache
        discovery.clear_cache("test_capability").await;

        songbird_process_env::remove_var("TEST_CAPABILITY_ENDPOINT");
    }

    #[test]
    fn test_parse_toml_config_rejects_invalid_syntax() {
        let err = CapabilityDiscovery::parse_toml_config("{{{not_toml", "compute")
            .expect_err("invalid TOML");
        match err {
            SongbirdError::Discovery {
                message,
                backend: Some(b),
                ..
            } => {
                assert_eq!(b, "config_file");
                assert!(message.contains("TOML"), "message: {message}");
            }
            other => panic!("expected Discovery error, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_json_config_rejects_invalid_syntax() {
        let err = CapabilityDiscovery::parse_json_config("{", "compute").expect_err("invalid JSON");
        assert!(
            matches!(err, SongbirdError::Discovery { ref backend, .. } if backend.as_deref() == Some("config_file")),
            "{err:?}"
        );
    }

    #[test]
    fn test_parse_yaml_config_rejects_invalid_syntax() {
        let err =
            CapabilityDiscovery::parse_yaml_config(":\n  -", "compute").expect_err("invalid YAML");
        assert!(
            matches!(err, SongbirdError::Discovery { ref backend, .. } if backend.as_deref() == Some("config_file")),
            "{err:?}"
        );
    }

    #[test]
    fn test_extract_endpoints_from_toml_respects_capability_and_health() {
        let toml = r#"
[services.alpha]
url = "http://alpha:1"
capabilities = ["compute"]
health_score = 0.42

[services.beta]
url = "http://beta:2"
capabilities = ["storage"]
"#;
        let v: toml::Value = toml::from_str(toml).expect("fixture TOML");
        let endpoints = CapabilityDiscovery::extract_endpoints_from_config(&v, "compute");
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].id, "alpha");
        assert_eq!(endpoints[0].url, "http://alpha:1");
        assert!((endpoints[0].health_score - 0.42).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extract_endpoints_from_json_empty_when_capability_missing() {
        let json = r#"{"services":{"only":{"url":"http://x:1","capabilities":["ai"]}}}"#;
        let v: serde_json::Value = serde_json::from_str(json).expect("fixture JSON");
        let endpoints = CapabilityDiscovery::extract_endpoints_from_json(&v, "compute");
        assert!(endpoints.is_empty());
    }

    #[test]
    fn test_extract_endpoints_from_yaml_matches_capability() {
        let yaml = r"
services:
  svc1:
    url: http://y:3
    capabilities:
      - compute
";
        let v: serde_yaml::Value = serde_yaml::from_str(yaml).expect("fixture YAML");
        let endpoints = CapabilityDiscovery::extract_endpoints_from_yaml(&v, "compute");
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].url, "http://y:3");
    }

    #[tokio::test]
    async fn test_discover_via_config_file_rejects_unsupported_extension() {
        let path =
            std::env::temp_dir().join(format!("songbird_cfg_unsup_{}.bin", uuid::Uuid::new_v4()));
        tokio::fs::write(&path, b"{}").await.expect("write fixture");

        let discovery = CapabilityDiscovery::new();
        let path_str = path.to_str().expect("utf8 path");
        let err = discovery
            .discover_via_config_file("compute", path_str)
            .await
            .expect_err("unsupported extension");

        let _ = tokio::fs::remove_file(&path).await;

        match err {
            SongbirdError::Discovery {
                message,
                backend: Some(b),
                ..
            } => {
                assert_eq!(b, "config_file");
                assert!(message.contains("Unsupported"), "message: {message}");
            }
            other => panic!("expected Discovery error, got {other:?}"),
        }
    }
}
