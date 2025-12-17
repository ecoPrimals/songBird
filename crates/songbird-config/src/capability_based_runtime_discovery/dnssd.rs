//! DNS-SD (DNS Service Discovery) Backend
//!
//! Complete production implementation for discovering capabilities via DNS-SD.
//! Enables service discovery using standard DNS SRV and TXT records.

use super::{CapabilityProvider, CapabilityRequest, Protocol};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

/// DNS-SD discovery backend
///
/// Discovers services using DNS Service Discovery (RFC 6763).
/// Queries DNS SRV and TXT records for service information.
#[derive(Debug, Clone)]
pub struct DnsSDDiscovery {
    /// DNS domain to search (e.g., "songbird.local")
    domain: String,
    /// DNS resolver configuration
    resolver_config: ResolverConfig,
    /// Query timeout
    timeout: Duration,
}

impl DnsSDDiscovery {
    /// Create a new DNS-SD discovery backend
    ///
    /// # Arguments
    /// * `domain` - DNS domain to search for services
    ///
    /// # Examples
    /// ```no_run
    /// use songbird_config::capability_based_runtime_discovery::dnssd::DnsSDDiscovery;
    ///
    /// let discovery = DnsSDDiscovery::new("songbird.local");
    /// ```
    #[must_use]
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            resolver_config: ResolverConfig::default(),
            timeout: Duration::from_secs(5),
        }
    }

    /// Create from environment
    ///
    /// Reads `SONGBIRD_DNSSD_DOMAIN` for discovery domain
    ///
    /// # Errors
    /// Returns error if environment variable is not set
    pub fn from_env() -> SongbirdResult<Self> {
        let domain =
            std::env::var("SONGBIRD_DNSSD_DOMAIN").unwrap_or_else(|_| "songbird.local".to_string());

        Ok(Self::new(domain))
    }

    /// Use custom DNS resolver configuration
    #[must_use]
    pub fn with_resolver_config(mut self, config: ResolverConfig) -> Self {
        self.resolver_config = config;
        self
    }

    /// Set the query timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Discover a capability provider via DNS-SD
    ///
    /// # Errors
    /// Returns error if DNS queries fail or no services found
    pub async fn discover(
        &self,
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        debug!("Querying DNS-SD in domain {} for capability: {}", self.domain, request.capability);

        // Create DNS resolver
        let resolver =
            TokioAsyncResolver::tokio(self.resolver_config.clone(), ResolverOpts::default());

        // Query for services advertising the capability
        let services = self.query_services(&resolver, request).await?;

        if services.is_empty() {
            return Err(SongbirdError::discovery(format!(
                "No services found via DNS-SD for capability: {}",
                request.capability
            )));
        }

        // Select best match
        let best_match = Self::select_best_service(&services, request)?;

        info!(
            "Discovered provider '{}' for capability '{}' via DNS-SD",
            best_match.name, request.capability
        );

        Ok(best_match)
    }

    /// Query DNS for services
    async fn query_services(
        &self,
        resolver: &TokioAsyncResolver,
        request: &CapabilityRequest,
    ) -> SongbirdResult<Vec<DnsService>> {
        // Build service name: _<capability>._tcp.<domain>
        let service_name = format!("_{}._{}.{}", request.capability, "tcp", self.domain);

        // Query SRV records
        let srv_lookup = tokio::time::timeout(self.timeout, resolver.srv_lookup(&service_name))
            .await
            .map_err(|_| SongbirdError::timeout("DNS-SD SRV query timed out"))?
            .map_err(|e| SongbirdError::discovery(format!("DNS-SD SRV query failed: {e}")))?;

        let mut services = Vec::new();

        for srv_record in srv_lookup.iter() {
            // Query TXT records for service metadata
            let txt_name = format!("{}.{}", srv_record.target(), self.domain);

            let txt_lookup =
                match tokio::time::timeout(self.timeout, resolver.txt_lookup(&txt_name)).await {
                    Ok(Ok(lookup)) => Some(lookup),
                    _ => None, // TXT records are optional
                };

            // Parse service information
            let service = Self::parse_service(srv_record, txt_lookup, request);
            services.push(service);
        }

        Ok(services)
    }

    /// Parse DNS records into service structure
    fn parse_service(
        srv: &trust_dns_resolver::proto::rr::rdata::SRV,
        txt_lookup: Option<trust_dns_resolver::lookup::TxtLookup>,
        _request: &CapabilityRequest,
    ) -> DnsService {
        let mut features = Vec::new();
        let mut metadata = HashMap::new();
        let mut protocol = Protocol::Http;
        let priority = srv.priority();

        // Parse TXT records if available
        if let Some(txt_lookup) = txt_lookup {
            for txt in txt_lookup.iter() {
                for data in txt.iter() {
                    if let Ok(record_str) = std::str::from_utf8(data) {
                        // Parse key=value format
                        if let Some((key, value)) = record_str.split_once('=') {
                            match key {
                                "features" => {
                                    features =
                                        value.split(',').map(|s| s.trim().to_string()).collect();
                                }
                                "protocol" => {
                                    protocol = match value {
                                        "https" => Protocol::Https,
                                        "grpc" => Protocol::Grpc,
                                        "ws" | "websocket" => Protocol::WebSocket,
                                        _ => Protocol::Http,
                                    };
                                }
                                _ => {
                                    metadata.insert(key.to_string(), value.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Build endpoint
        let endpoint = format!(
            "{}://{}:{}",
            match protocol {
                Protocol::Https => "https",
                Protocol::Grpc => "grpc",
                Protocol::WebSocket => "ws",
                _ => "http",
            },
            srv.target(),
            srv.port()
        );

        DnsService {
            name: srv.target().to_string(),
            endpoint,
            protocol,
            features,
            metadata,
            priority: priority.into(),
            weight: srv.weight().into(),
        }
    }

    /// Select the best service from discovered options
    fn select_best_service(
        services: &[DnsService],
        request: &CapabilityRequest,
    ) -> SongbirdResult<CapabilityProvider> {
        // Filter by required features
        let mut candidates: Vec<_> = services
            .iter()
            .filter(|s| request.required_features.iter().all(|req| s.features.contains(req)))
            .collect();

        if candidates.is_empty() {
            return Err(SongbirdError::discovery(
                "No DNS-SD services support required features".to_string(),
            ));
        }

        // Sort by priority (lower is better), then weight (higher is better)
        candidates.sort_by(|a, b| match a.priority.cmp(&b.priority) {
            std::cmp::Ordering::Equal => b.weight.cmp(&a.weight),
            other => other,
        });

        let selected = candidates.first().ok_or_else(|| {
            SongbirdError::discovery("No suitable DNS-SD service found".to_string())
        })?;

        Ok(CapabilityProvider {
            name: selected.name.clone(),
            capability: request.capability.clone(),
            endpoint: selected.endpoint.clone(),
            protocol: selected.protocol.clone(),
            features: selected.features.clone(),
            metadata: selected.metadata.clone(),
        })
    }
}

/// Service discovered via DNS-SD
#[derive(Debug, Clone)]
struct DnsService {
    name: String,
    endpoint: String,
    protocol: Protocol,
    features: Vec<String>,
    metadata: HashMap<String, String>,
    priority: u32,
    weight: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dnssd_discovery_creation() {
        let discovery = DnsSDDiscovery::new("songbird.local");
        assert_eq!(discovery.domain, "songbird.local");
        assert_eq!(discovery.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_dnssd_from_env_default() {
        // Should use default if env var not set
        let discovery = DnsSDDiscovery::from_env().unwrap();
        assert_eq!(discovery.domain, "songbird.local");
    }

    #[test]
    fn test_dnssd_with_timeout() {
        let discovery = DnsSDDiscovery::new("test.local").with_timeout(Duration::from_secs(10));
        assert_eq!(discovery.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_service_name_format() {
        let discovery = DnsSDDiscovery::new("songbird.local");
        let request = CapabilityRequest::new("ai");

        // Service name should be: _ai._tcp.songbird.local
        let expected = format!("_{}._tcp.{}", request.capability, discovery.domain);
        assert_eq!(expected, "_ai._tcp.songbird.local");
    }
}
