// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Discoverable Endpoint System
//!
//! Eliminates hardcoded ports, IPs, and URLs by providing environment-based
//! discovery with intelligent fallbacks.
//!
//! # Philosophy
//!
//! Code should NEVER hardcode network endpoints. Instead, endpoints should be:
//! 1. Discovered from environment variables
//! 2. Auto-discovered via network probing
//! 3. Fall back to safe defaults only for development

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

/// Discoverable endpoint configuration
///
/// This type represents an endpoint that can be discovered through multiple methods.
/// It NEVER uses hardcoded values in production - only for development fallbacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverableEndpoint {
    /// Primary discovery method
    pub discovery_method: DiscoveryMethod,
    /// Secondary fallback methods
    pub fallback_methods: Vec<DiscoveryMethod>,
    /// Development-only fallback (used only when no other method succeeds)
    pub dev_fallback: Option<EndpointSpec>,
    /// Whether to cache discovered endpoint
    pub cache_discovery: bool,
}

/// Discovery methods for finding endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Environment variable
    Environment {
        /// Variable name (e.g., "`SERVICE_ENDPOINT`")
        var_name: String,
        /// Optional parser for the value
        parser: EndpointParser,
    },
    /// DNS service discovery
    DnsServiceDiscovery {
        /// Service name (e.g., "_http._tcp.local")
        service_name: String,
    },
    /// Network probe
    NetworkProbe {
        /// Host patterns to try
        host_patterns: Vec<String>,
        /// Port ranges to try
        port_range: (u16, u16),
        /// Health check path
        health_path: String,
    },
    /// Kubernetes service discovery
    KubernetesService {
        /// Service name
        service_name: String,
        /// Namespace
        namespace: String,
        /// Port name or number
        port: PortSpec,
    },
    /// Consul service discovery
    ConsulService {
        /// Service name in consul
        service_name: String,
        /// Consul agent address (can itself be discovered)
        consul_addr: Option<String>,
    },
    /// Static configuration (discouraged, but supported)
    Static {
        /// Static endpoint specification
        endpoint: EndpointSpec,
    },
}

/// Endpoint specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSpec {
    /// Host (IP or hostname)
    pub host: String,
    /// Port
    pub port: u16,
    /// Protocol (http, https, grpc, tcp, etc.)
    pub protocol: Option<String>,
    /// Path prefix
    pub path: Option<String>,
}

/// Port specification (flexible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortSpec {
    /// Named port (e.g., "http", "https")
    Named(String),
    /// Numeric port
    Number(u16),
    /// Environment variable containing port
    Environment(String),
}

/// Endpoint parser for environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointParser {
    /// Parse as full URL (http://host:port/path)
    Url,
    /// Parse as host:port
    HostPort,
    /// Parse as just hostname (use default port)
    Hostname,
    /// Custom parser pattern
    Pattern(String),
}

impl DiscoverableEndpoint {
    /// Create a new discoverable endpoint with environment variable discovery
    #[must_use]
    pub fn from_env(var_name: &str) -> Self {
        Self {
            discovery_method: DiscoveryMethod::Environment {
                var_name: var_name.to_string(),
                parser: EndpointParser::Url,
            },
            fallback_methods: vec![
                DiscoveryMethod::Environment {
                    var_name: format!("{var_name}_HOST"),
                    parser: EndpointParser::Hostname,
                },
                DiscoveryMethod::NetworkProbe {
                    host_patterns: vec![
                        songbird_types::constants::LOCALHOST_HOSTNAME.to_string(),
                        songbird_types::constants::LOCALHOST.to_string(),
                    ],
                    port_range: (8000, 9000),
                    health_path: "/health".to_string(),
                },
            ],
            dev_fallback: Some(EndpointSpec {
                host: songbird_types::constants::LOCALHOST_HOSTNAME.to_string(),
                port: songbird_types::defaults::ports::DEFAULT_HTTP_PORT,
                protocol: Some("http".to_string()),
                path: None,
            }),
            cache_discovery: true,
        }
    }

    /// Create for kubernetes service
    #[must_use]
    pub fn from_k8s_service(service_name: &str, namespace: &str, port: u16) -> Self {
        Self {
            discovery_method: DiscoveryMethod::KubernetesService {
                service_name: service_name.to_string(),
                namespace: namespace.to_string(),
                port: PortSpec::Number(port),
            },
            fallback_methods: vec![DiscoveryMethod::Environment {
                var_name: format!("{}_SERVICE_HOST", service_name.to_uppercase()),
                parser: EndpointParser::Hostname,
            }],
            dev_fallback: Some(EndpointSpec {
                host: format!("{service_name}.{namespace}.svc.cluster.local"),
                port,
                protocol: Some("http".to_string()),
                path: None,
            }),
            cache_discovery: true,
        }
    }

    /// Create for consul service
    #[must_use]
    pub fn from_consul_service(service_name: &str) -> Self {
        Self {
            discovery_method: DiscoveryMethod::ConsulService {
                service_name: service_name.to_string(),
                consul_addr: None, // Will use default consul addr
            },
            fallback_methods: vec![
                DiscoveryMethod::Environment {
                    var_name: format!("{}_SERVICE_ADDR", service_name.to_uppercase()),
                    parser: EndpointParser::HostPort,
                },
                DiscoveryMethod::DnsServiceDiscovery {
                    service_name: format!("_{service_name}.service.consul"),
                },
            ],
            dev_fallback: None, // Consul services don't have dev fallbacks
            cache_discovery: true,
        }
    }

    /// Discover the endpoint using configured methods
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails for all configured methods (primary and fallback)
    pub async fn discover(&self) -> SongbirdResult<EndpointSpec> {
        self.discover_with(|k| songbird_process_env::var(k)).await
    }

    /// Same as [`discover`](Self::discover) with an injectable env reader.
    pub async fn discover_with(
        &self,
        env: impl Fn(&str) -> Result<String, std::env::VarError> + Send + Sync,
    ) -> SongbirdResult<EndpointSpec> {
        // Try primary method
        if let Ok(endpoint) = self.try_discovery_method_with(&self.discovery_method, &env).await {
            return Ok(endpoint);
        }

        // Try fallback methods
        for method in &self.fallback_methods {
            if let Ok(endpoint) = self.try_discovery_method_with(method, &env).await {
                return Ok(endpoint);
            }
        }

        // Use dev fallback if available and we're in development mode
        if is_development_mode_with(&env)
            && let Some(fallback) = &self.dev_fallback
        {
            return Ok(fallback.clone());
        }

        Err(SongbirdError::Configuration {
            message: "Could not discover endpoint using any method".to_string(),
            field: Some("endpoint".to_string()),
            suggestion: Some("Check environment variables or network connectivity".to_string()),
        })
    }

    async fn try_discovery_method_with(
        &self,
        method: &DiscoveryMethod,
        env: &(impl Fn(&str) -> Result<String, std::env::VarError> + Send + Sync),
    ) -> SongbirdResult<EndpointSpec> {
        match method {
            DiscoveryMethod::Environment {
                var_name,
                parser,
            } => {
                let value = env(var_name).map_err(|_| SongbirdError::Configuration {
                    message: format!("Environment variable {var_name} not found"),
                    field: Some(var_name.clone()),
                    suggestion: Some(format!("Set {var_name} environment variable")),
                })?;

                parse_endpoint(&value, parser)
            }

            DiscoveryMethod::Static {
                endpoint,
            } => Ok(endpoint.clone()),

            DiscoveryMethod::NetworkProbe {
                host_patterns,
                port_range,
                health_path,
            } => {
                // Try probing each host pattern
                for host in host_patterns {
                    for port in port_range.0..=port_range.1 {
                        if matches!(probe_endpoint(host, port, health_path).await, Ok(())) {
                            return Ok(EndpointSpec {
                                host: host.clone(),
                                port,
                                protocol: Some("http".to_string()),
                                path: None,
                            });
                        }
                    }
                }
                Err(SongbirdError::Configuration {
                    message: "Network probe failed".to_string(),
                    field: None,
                    suggestion: Some("Check network connectivity".to_string()),
                })
            }

            DiscoveryMethod::KubernetesService {
                service_name,
                namespace,
                port,
            } => {
                // Check if we're in kubernetes
                if env("KUBERNETES_SERVICE_HOST").is_ok() {
                    let port_num = match port {
                        PortSpec::Number(n) => *n,
                        PortSpec::Named(name) => resolve_named_port(name)?,
                        PortSpec::Environment(var) => env(var)
                            .ok()
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(songbird_types::defaults::ports::DEFAULT_HTTP_PORT),
                    };

                    Ok(EndpointSpec {
                        host: format!("{service_name}.{namespace}.svc.cluster.local"),
                        port: port_num,
                        protocol: Some("http".to_string()),
                        path: None,
                    })
                } else {
                    Err(SongbirdError::Configuration {
                        message: "Not in Kubernetes environment".to_string(),
                        field: None,
                        suggestion: Some("Run inside a Kubernetes pod".to_string()),
                    })
                }
            }

            DiscoveryMethod::ConsulService {
                service_name,
                consul_addr,
            } => resolve_consul_service(service_name, consul_addr.as_deref()),

            DiscoveryMethod::DnsServiceDiscovery {
                service_name: _,
            } => {
                // For now, return error - full DNS-SD would go here
                Err(SongbirdError::not_implemented_with_detail(
                    "dns_sd_discovery",
                    "Use environment variables or static configuration instead",
                ))
            }
        }
    }
}

/// Resolve a service via Consul's catalog API using a blocking TCP request.
fn resolve_consul_service(
    service_name: &str,
    consul_addr: Option<&str>,
) -> SongbirdResult<EndpointSpec> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let consul_base = consul_addr.unwrap_or("http://127.0.0.1:8500");
    let url = format!("{consul_base}/v1/catalog/service/{service_name}?passing=true");

    let parsed = url::Url::parse(&url).map_err(|e| SongbirdError::Configuration {
        message: format!("Invalid consul URL: {e}"),
        field: Some("consul_addr".to_string()),
        suggestion: None,
    })?;

    let host = parsed.host_str().unwrap_or("127.0.0.1");
    let port = parsed.port().unwrap_or(8500);
    let path = parsed.path();

    let mut stream = TcpStream::connect(format!("{host}:{port}")).map_err(|e| {
        SongbirdError::Configuration {
            message: format!("Cannot connect to Consul at {host}:{port}: {e}"),
            field: Some("consul_addr".to_string()),
            suggestion: Some("Ensure Consul agent is running".to_string()),
        }
    })?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .ok();

    let req = format!("GET {path}?passing=true HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).ok();

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).ok();
    let response = String::from_utf8_lossy(&buf);

    let body_start = response.find("\r\n\r\n").map_or(0, |i| i + 4);
    let body = &response[body_start..];

    let entries: Vec<serde_json::Value> = serde_json::from_str(body).unwrap_or_default();

    if let Some(entry) = entries.first() {
        let svc_host = entry["ServiceAddress"]
            .as_str()
            .or_else(|| entry["Address"].as_str())
            .unwrap_or("127.0.0.1");
        let svc_port = entry["ServicePort"].as_u64().unwrap_or(8080);

        Ok(EndpointSpec {
            host: svc_host.to_string(),
            port: u16::try_from(svc_port)
                .unwrap_or(songbird_types::defaults::ports::DEFAULT_HTTP_PORT),
            protocol: Some("http".to_string()),
            path: None,
        })
    } else {
        Err(SongbirdError::Configuration {
            message: format!("No healthy instances of '{service_name}' found in Consul"),
            field: Some("service_name".to_string()),
            suggestion: Some("Register the service or check Consul health checks".to_string()),
        })
    }
}

/// Parse endpoint from string based on parser type
fn parse_endpoint(value: &str, parser: &EndpointParser) -> SongbirdResult<EndpointSpec> {
    match parser {
        EndpointParser::Url => {
            // Parse full URL
            let url = url::Url::parse(value).map_err(|e| SongbirdError::Configuration {
                message: format!("Invalid URL: {e}"),
                field: Some("url".to_string()),
                suggestion: Some("Provide a valid HTTP/HTTPS URL".to_string()),
            })?;

            Ok(EndpointSpec {
                host: url
                    .host_str()
                    .ok_or_else(|| SongbirdError::Configuration {
                        message: "URL missing host".to_string(),
                        field: Some("url".to_string()),
                        suggestion: Some("Provide a URL with a hostname".to_string()),
                    })?
                    .to_string(),
                port: url.port_or_known_default().unwrap_or(80),
                protocol: Some(url.scheme().to_string()),
                path: Some(url.path().to_string()),
            })
        }

        EndpointParser::HostPort => {
            // Parse host:port
            let parts: Vec<&str> = value.split(':').collect();
            if parts.len() != 2 {
                return Err(SongbirdError::Configuration {
                    message: "Expected host:port format".to_string(),
                    field: Some("endpoint".to_string()),
                    suggestion: Some(
                        "Use format: hostname:port (e.g., localhost:8080)".to_string(),
                    ),
                });
            }

            let port = parts[1].parse().map_err(|_| SongbirdError::Configuration {
                message: "Invalid port number".to_string(),
                field: Some("port".to_string()),
                suggestion: Some("Port must be between 0 and 65535".to_string()),
            })?;

            Ok(EndpointSpec {
                host: parts[0].to_string(),
                port,
                protocol: Some("http".to_string()),
                path: None,
            })
        }

        EndpointParser::Hostname => {
            // Just hostname, use default port
            Ok(EndpointSpec {
                host: value.to_string(),
                port: 8080,
                protocol: Some("http".to_string()),
                path: None,
            })
        }

        EndpointParser::Pattern(_pattern) => {
            // Custom pattern parsing would go here
            Err(SongbirdError::not_implemented_with_detail(
                "endpoint_parser_pattern",
                "Use Url or HostPort parser instead",
            ))
        }
    }
}

/// Probe an endpoint to see if it's available
async fn probe_endpoint(host: &str, port: u16, _health_path: &str) -> SongbirdResult<()> {
    // Quick TCP connection test
    let addr = format!("{host}:{port}");
    match tokio::time::timeout(
        std::time::Duration::from_millis(100),
        tokio::net::TcpStream::connect(&addr),
    )
    .await
    {
        Ok(Ok(_)) => Ok(()),
        _ => Err(SongbirdError::Network {
            message: "Probe failed".to_string(),
            interface: None,
            suggestion: Some("Check network connectivity and firewall settings".to_string()),
        }),
    }
}

/// Resolve named port to number
fn resolve_named_port(name: &str) -> SongbirdResult<u16> {
    match name {
        "http" => Ok(80),
        "https" => Ok(443),
        "grpc" => Ok(9090),
        _ => Err(SongbirdError::Configuration {
            message: format!("Unknown port name: {name}"),
            field: Some("port".to_string()),
            suggestion: Some("Use 'http' (80), 'https' (443), or 'grpc' (9090)".to_string()),
        }),
    }
}

fn is_development_mode_with(
    env: &(impl Fn(&str) -> Result<String, std::env::VarError> + Send + Sync),
) -> bool {
    let sb = env("SONGBIRD_ENV").unwrap_or_default();
    let rust = env("RUST_ENV").unwrap_or_default();
    sb.as_str() == "development"
        || sb.as_str() == "dev"
        || rust.as_str() == "development"
        || rust.as_str() == "dev"
}

impl EndpointSpec {
    /// Convert to full URL
    #[must_use]
    pub fn to_url(&self) -> String {
        let protocol = self.protocol.as_deref().unwrap_or("http");
        let path = self.path.as_deref().unwrap_or("");
        format!("{}://{}:{}{}", protocol, self.host, self.port, path)
    }

    /// Convert to socket address
    ///
    /// # Errors
    ///
    /// Returns an error if the host is a hostname rather than an IP address (DNS resolution required)
    pub fn to_socket_addr(&self) -> SongbirdResult<SocketAddr> {
        // Try to parse host as IP
        if let Ok(ip) = IpAddr::from_str(&self.host) {
            return Ok(SocketAddr::new(ip, self.port));
        }

        // For hostnames, return error - DNS resolution would happen elsewhere
        Err(SongbirdError::Configuration {
            message: "Cannot convert hostname to SocketAddr without DNS resolution".to_string(),
            field: Some("host".to_string()),
            suggestion: Some("Use an IP address or resolve DNS separately".to_string()),
        })
    }
}

impl Default for DiscoverableEndpoint {
    fn default() -> Self {
        Self::from_env("SERVICE_ENDPOINT")
    }
}

#[cfg(test)]
#[path = "discoverable_endpoint_tests.rs"]
mod tests;
