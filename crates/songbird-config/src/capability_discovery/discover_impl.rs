// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_http_client::IpcHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};

use super::CapabilityDiscovery;
use super::types::ServiceEndpoint;

impl CapabilityDiscovery {
    /// Discover via environment variables
    #[allow(
        clippy::unused_async,
        reason = "no .await needed for environment variable reads; async for uniform discover_via_method"
    )]
    pub(super) async fn discover_via_environment(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        // Try {CAPABILITY}_ENDPOINT environment variable
        let env_var = format!("{}_ENDPOINT", capability.to_uppercase());

        if let Ok(endpoint_url) = self.read_env(&env_var) {
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
            backend: Some(String::from("environment")),
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
    /// _compute._tcp.local.  IN SRV 0 5 8001 compute-provider.local.
    /// ```
    pub(super) async fn discover_via_dnssd(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
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
                        backend: Some(String::from("dns-sd")),
                        retry_strategy: Some(String::from(
                            "Ensure service is advertising via DNS-SD",
                        )),
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
                    backend: Some(String::from("dns-sd")),
                    retry_strategy: Some(String::from(
                        "Check DNS configuration and service advertisement",
                    )),
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
    pub(super) async fn discover_via_mdns(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
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
    pub(super) async fn discover_via_registry(
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
                            backend: Some(String::from("registry")),
                            retry_strategy: Some(String::from(
                                "Check registry endpoint and format",
                            )),
                        }),
                    }
                } else {
                    Err(SongbirdError::Discovery {
                        message: format!("Registry returned error: {}", response.status()),
                        backend: Some(String::from("registry")),
                        retry_strategy: Some(String::from("Check registry endpoint availability")),
                    })
                }
            }
            Err(e) => Err(SongbirdError::Discovery {
                message: format!("Failed to query registry: {e}"),
                backend: Some(String::from("registry")),
                retry_strategy: Some(String::from("Check network connectivity to registry")),
            }),
        }
    }

    /// Discover via configuration file
    ///
    /// Reads service configurations from TOML, JSON, or YAML files.
    /// Supports standard configuration paths and formats.
    pub(crate) async fn discover_via_config_file(
        &self,
        capability: &str,
        config_path: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        debug!("Reading configuration file at {} for capability: {}", config_path, capability);

        // Read configuration file
        let config_content =
            tokio::fs::read_to_string(config_path).await.map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to read config file: {e}"),
                backend: Some(String::from("config_file")),
                retry_strategy: Some(String::from("Check file path and permissions")),
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
                backend: Some(String::from("config_file")),
                retry_strategy: Some(String::from("Use .toml, .json, or .yaml file")),
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
    pub(crate) fn parse_toml_config(
        content: &str,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        let config: toml::Value =
            toml::from_str(content).map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to parse TOML: {e}"),
                backend: Some(String::from("config_file")),
                retry_strategy: Some(String::from("Check TOML syntax")),
            })?;

        Ok(Self::extract_endpoints_from_config(&config, capability))
    }

    /// Parse JSON configuration
    pub(crate) fn parse_json_config(
        content: &str,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        let config: serde_json::Value =
            serde_json::from_str(content).map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to parse JSON: {e}"),
                backend: Some(String::from("config_file")),
                retry_strategy: Some(String::from("Check JSON syntax")),
            })?;

        Ok(Self::extract_endpoints_from_json(&config, capability))
    }

    /// Parse YAML configuration
    pub(crate) fn parse_yaml_config(
        content: &str,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        let config: serde_yaml::Value =
            serde_yaml::from_str(content).map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to parse YAML: {e}"),
                backend: Some(String::from("config_file")),
                retry_strategy: Some(String::from("Check YAML syntax")),
            })?;

        Ok(Self::extract_endpoints_from_yaml(&config, capability))
    }

    /// Extract endpoints from TOML config
    pub(crate) fn extract_endpoints_from_config(
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
    pub(crate) fn extract_endpoints_from_json(
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
    pub(crate) fn extract_endpoints_from_yaml(
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
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::CapabilityDiscovery;
    use crate::capability_discovery::DiscoveryMethod;

    #[test]
    fn parse_toml_config_extracts_matching_capability() {
        let toml = r#"
[services.compute-a]
url = "http://10.0.0.1:8000"
capabilities = ["compute", "batch"]
"#;
        let eps = CapabilityDiscovery::parse_toml_config(toml, "compute").expect("parse");
        assert_eq!(eps.len(), 1);
        assert_eq!(eps[0].url, "http://10.0.0.1:8000");
        assert!(eps[0].capabilities.contains(&String::from("compute")));
    }

    #[test]
    fn parse_json_config_extracts_matching_capability() {
        let json = r#"{"services":{"s1":{"url":"http://x:1","capabilities":["storage"]}}}"#;
        let eps = CapabilityDiscovery::parse_json_config(json, "storage").expect("parse");
        assert_eq!(eps.len(), 1);
        assert_eq!(eps[0].id, "s1");
    }

    #[test]
    fn parse_yaml_config_extracts_matching_capability() {
        let yaml = r"
services:
  y1:
    url: http://y:2
    capabilities: [ai]
";
        let eps = CapabilityDiscovery::parse_yaml_config(yaml, "ai").expect("parse");
        assert_eq!(eps.len(), 1);
        assert_eq!(eps[0].url, "http://y:2");
    }

    #[tokio::test]
    async fn find_providers_by_capability_hits_environment_only() {
        let disc =
            CapabilityDiscovery::with_methods_env_reader(vec![DiscoveryMethod::Environment], |k| {
                if k == "WIDGET_ENDPOINT" {
                    Ok(String::from("http://widget:1"))
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            });
        let eps = disc.find_providers_by_capability("widget").await.expect("found");
        assert_eq!(eps.len(), 1);
    }
}
