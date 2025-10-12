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
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

use songbird_types::{SongbirdError, SongbirdResult};

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
        /// Variable name (e.g., "SERVICE_ENDPOINT")
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
    pub fn from_env(var_name: String) -> Self {
        Self {
            discovery_method: DiscoveryMethod::Environment {
                var_name: var_name.clone(),
                parser: EndpointParser::Url,
            },
            fallback_methods: vec![
                DiscoveryMethod::Environment {
                    var_name: format!("{}_HOST", var_name),
                    parser: EndpointParser::Hostname,
                },
                DiscoveryMethod::NetworkProbe {
                    host_patterns: vec!["localhost".to_string(), "127.0.0.1".to_string()],
                    port_range: (8000, 9000),
                    health_path: "/health".to_string(),
                },
            ],
            dev_fallback: Some(EndpointSpec {
                host: "localhost".to_string(),
                port: 8080,
                protocol: Some("http".to_string()),
                path: None,
            }),
            cache_discovery: true,
        }
    }

    /// Create for kubernetes service
    pub fn from_k8s_service(service_name: String, namespace: String, port: u16) -> Self {
        Self {
            discovery_method: DiscoveryMethod::KubernetesService {
                service_name: service_name.clone(),
                namespace: namespace.clone(),
                port: PortSpec::Number(port),
            },
            fallback_methods: vec![DiscoveryMethod::Environment {
                var_name: format!("{}_SERVICE_HOST", service_name.to_uppercase()),
                parser: EndpointParser::Hostname,
            }],
            dev_fallback: Some(EndpointSpec {
                host: format!("{}.{}.svc.cluster.local", service_name, namespace),
                port,
                protocol: Some("http".to_string()),
                path: None,
            }),
            cache_discovery: true,
        }
    }

    /// Create for consul service
    pub fn from_consul_service(service_name: String) -> Self {
        Self {
            discovery_method: DiscoveryMethod::ConsulService {
                service_name: service_name.clone(),
                consul_addr: None, // Will use default consul addr
            },
            fallback_methods: vec![
                DiscoveryMethod::Environment {
                    var_name: format!("{}_SERVICE_ADDR", service_name.to_uppercase()),
                    parser: EndpointParser::HostPort,
                },
                DiscoveryMethod::DnsServiceDiscovery {
                    service_name: format!("_{}.service.consul", service_name),
                },
            ],
            dev_fallback: None, // Consul services don't have dev fallbacks
            cache_discovery: true,
        }
    }

    /// Discover the endpoint using configured methods
    pub async fn discover(&self) -> SongbirdResult<EndpointSpec> {
        // Try primary method
        if let Ok(endpoint) = self.try_discovery_method(&self.discovery_method).await {
            return Ok(endpoint);
        }

        // Try fallback methods
        for method in &self.fallback_methods {
            if let Ok(endpoint) = self.try_discovery_method(method).await {
                return Ok(endpoint);
            }
        }

        // Use dev fallback if available and we're in development mode
        if is_development_mode() {
            if let Some(fallback) = &self.dev_fallback {
                return Ok(fallback.clone());
            }
        }

        Err(SongbirdError::Configuration {
            field: "endpoint".to_string(),
            message: "Could not discover endpoint using any method".to_string(),
            current_value: None,
            expected_format: None,
            suggestion: Some("Check environment variables or network connectivity".to_string()),
        })
    }

    /// Try a single discovery method
    async fn try_discovery_method(&self, method: &DiscoveryMethod) -> SongbirdResult<EndpointSpec> {
        match method {
            DiscoveryMethod::Environment { var_name, parser } => {
                let value = std::env::var(var_name).map_err(|_| {
                    SongbirdError::configuration_error(&format!(
                        "Environment variable {} not found",
                        var_name
                    ))
                })?;

                parse_endpoint(&value, parser)
            }

            DiscoveryMethod::Static { endpoint } => Ok(endpoint.clone()),

            DiscoveryMethod::NetworkProbe {
                host_patterns,
                port_range,
                health_path,
            } => {
                // Try probing each host pattern
                for host in host_patterns {
                    for port in port_range.0..=port_range.1 {
                        if let Ok(_) = probe_endpoint(host, port, health_path).await {
                            return Ok(EndpointSpec {
                                host: host.clone(),
                                port,
                                protocol: Some("http".to_string()),
                                path: None,
                            });
                        }
                    }
                }
                Err(SongbirdError::configuration_error("Network probe failed"))
            }

            DiscoveryMethod::KubernetesService {
                service_name,
                namespace,
                port,
            } => {
                // Check if we're in kubernetes
                if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
                    let port_num = match port {
                        PortSpec::Number(n) => *n,
                        PortSpec::Named(name) => resolve_named_port(name)?,
                        PortSpec::Environment(var) => {
                            std::env::var(var)
                                .ok()
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(8080)
                        }
                    };

                    Ok(EndpointSpec {
                        host: format!("{}.{}.svc.cluster.local", service_name, namespace),
                        port: port_num,
                        protocol: Some("http".to_string()),
                        path: None,
                    })
                } else {
                    Err(SongbirdError::configuration_error("Not in Kubernetes environment"))
                }
            }

            DiscoveryMethod::ConsulService {
                service_name,
                consul_addr,
            } => {
                // For now, return error - full consul integration would go here
                Err(SongbirdError::configuration_error(
                    "Consul discovery not yet implemented",
                ))
            }

            DiscoveryMethod::DnsServiceDiscovery { service_name } => {
                // For now, return error - full DNS-SD would go here
                Err(SongbirdError::configuration_error(
                    "DNS-SD not yet implemented",
                ))
            }
        }
    }
}

/// Parse endpoint from string based on parser type
fn parse_endpoint(value: &str, parser: &EndpointParser) -> SongbirdResult<EndpointSpec> {
    match parser {
        EndpointParser::Url => {
            // Parse full URL
            let url = url::Url::parse(value).map_err(|e| {
                SongbirdError::configuration_error(&format!("Invalid URL: {}", e))
            })?;

            Ok(EndpointSpec {
                host: url
                    .host_str()
                    .ok_or_else(|| SongbirdError::configuration_error("URL missing host"))?
                    .to_string(),
                port: url.port().unwrap_or(80),
                protocol: Some(url.scheme().to_string()),
                path: Some(url.path().to_string()),
            })
        }

        EndpointParser::HostPort => {
            // Parse host:port
            let parts: Vec<&str> = value.split(':').collect();
            if parts.len() != 2 {
                return Err(SongbirdError::configuration_error(
                    "Expected host:port format",
                ));
            }

            let port = parts[1].parse().map_err(|_| {
                SongbirdError::configuration_error("Invalid port number")
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

        EndpointParser::Pattern(pattern) => {
            // Custom pattern parsing would go here
            Err(SongbirdError::configuration_error(
                "Custom patterns not yet implemented",
            ))
        }
    }
}

/// Probe an endpoint to see if it's available
async fn probe_endpoint(host: &str, port: u16, health_path: &str) -> SongbirdResult<()> {
    // Quick TCP connection test
    let addr = format!("{}:{}", host, port);
    match tokio::time::timeout(
        std::time::Duration::from_millis(100),
        tokio::net::TcpStream::connect(&addr),
    )
    .await
    {
        Ok(Ok(_)) => Ok(()),
        _ => Err(SongbirdError::network_error("Probe failed")),
    }
}

/// Resolve named port to number
fn resolve_named_port(name: &str) -> SongbirdResult<u16> {
    match name {
        "http" => Ok(80),
        "https" => Ok(443),
        "grpc" => Ok(9090),
        _ => Err(SongbirdError::configuration_error(&format!(
            "Unknown port name: {}",
            name
        ))),
    }
}

/// Check if we're in development mode
fn is_development_mode() -> bool {
    std::env::var("SONGBIRD_ENV")
        .map(|v| v == "development" || v == "dev")
        .unwrap_or(false)
        || std::env::var("RUST_ENV")
            .map(|v| v == "development" || v == "dev")
            .unwrap_or(false)
}

impl EndpointSpec {
    /// Convert to full URL
    pub fn to_url(&self) -> String {
        let protocol = self.protocol.as_deref().unwrap_or("http");
        let path = self.path.as_deref().unwrap_or("");
        format!("{}://{}:{}{}", protocol, self.host, self.port, path)
    }

    /// Convert to socket address
    pub fn to_socket_addr(&self) -> SongbirdResult<SocketAddr> {
        // Try to parse host as IP
        if let Ok(ip) = IpAddr::from_str(&self.host) {
            return Ok(SocketAddr::new(ip, self.port));
        }

        // For hostnames, return error - DNS resolution would happen elsewhere
        Err(SongbirdError::configuration_error(
            "Cannot convert hostname to SocketAddr without DNS resolution",
        ))
    }
}

impl Default for DiscoverableEndpoint {
    fn default() -> Self {
        Self::from_env("SERVICE_ENDPOINT".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let spec = parse_endpoint("http://example.com:8080/api", &EndpointParser::Url).unwrap();
        assert_eq!(spec.host, "example.com");
        assert_eq!(spec.port, 8080);
        assert_eq!(spec.protocol, Some("http".to_string()));
    }

    #[test]
    fn test_parse_host_port() {
        let spec = parse_endpoint("localhost:3000", &EndpointParser::HostPort).unwrap();
        assert_eq!(spec.host, "localhost");
        assert_eq!(spec.port, 3000);
    }

    #[test]
    fn test_parse_hostname() {
        let spec = parse_endpoint("myservice", &EndpointParser::Hostname).unwrap();
        assert_eq!(spec.host, "myservice");
        assert_eq!(spec.port, 8080); // Default
    }

    #[test]
    fn test_endpoint_to_url() {
        let spec = EndpointSpec {
            host: "localhost".to_string(),
            port: 8080,
            protocol: Some("https".to_string()),
            path: Some("/api/v1".to_string()),
        };
        assert_eq!(spec.to_url(), "https://localhost:8080/api/v1");
    }
}

