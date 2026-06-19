// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! DNS-SD (DNS Service Discovery) Backend
//!
//! Complete production implementation for discovering capabilities via DNS-SD.
//! Enables service discovery using standard DNS SRV and TXT records.

use super::{CapabilityProvider, CapabilityRequest, Protocol};
use hickory_resolver::TokioResolver;
use hickory_resolver::config::ResolverConfig;
use hickory_resolver::name_server::TokioConnectionProvider;
use songbird_types::defaults::timeouts::DEFAULT_DNSSD_TIMEOUT;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

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
            timeout: DEFAULT_DNSSD_TIMEOUT,
        }
    }

    /// Create from environment
    ///
    /// Reads `SONGBIRD_DNSSD_DOMAIN` for discovery domain
    ///
    /// # Errors
    /// Returns error if environment variable is not set
    pub fn from_env() -> SongbirdResult<Self> {
        let domain = songbird_process_env::var("SONGBIRD_DNSSD_DOMAIN")
            .unwrap_or_else(|_| String::from("songbird.local"));

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
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
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

        let resolver = hickory_resolver::Resolver::builder_with_config(
            self.resolver_config.clone(),
            TokioConnectionProvider::default(),
        )
        .build();

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
        resolver: &TokioResolver,
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
                    _ => None,
                };

            let service = Self::parse_service(srv_record, txt_lookup, request);
            services.push(service);
        }

        Ok(services)
    }

    /// Parse DNS records into service structure
    fn parse_service(
        srv: &hickory_resolver::proto::rr::rdata::SRV,
        txt_lookup: Option<hickory_resolver::lookup::TxtLookup>,
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
                    if let Ok(record_str) = std::str::from_utf8(data)
                        && let Some((key, value)) = record_str.split_once('=')
                    {
                        match key {
                            "features" => {
                                features = value.split(',').map(|s| s.trim().to_string()).collect();
                            }
                            "protocol" => {
                                protocol = match value {
                                    "https" => Protocol::Https,
                                    "tarpc" => Protocol::Tarpc,
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

        // Build endpoint
        let endpoint = format!(
            "{}://{}:{}",
            match protocol {
                Protocol::Https => "https",
                Protocol::Tarpc => "tarpc",
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
            return Err(SongbirdError::discovery(String::from(
                "No DNS-SD services support required features",
            )));
        }

        // Sort by priority (lower is better), then weight (higher is better)
        candidates.sort_by(|a, b| match a.priority.cmp(&b.priority) {
            std::cmp::Ordering::Equal => b.weight.cmp(&a.weight),
            other => other,
        });

        let selected = candidates.first().ok_or_else(|| {
            SongbirdError::discovery(String::from("No suitable DNS-SD service found"))
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
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use hickory_resolver::lookup::{Lookup, TxtLookup};
    use hickory_resolver::proto::op::Query;
    use hickory_resolver::proto::rr::rdata::{SRV, TXT};
    use hickory_resolver::proto::rr::{Name, RData, RecordType};
    use std::str::FromStr;

    #[test]
    fn test_dnssd_discovery_creation() {
        let discovery = DnsSDDiscovery::new("songbird.local");
        assert_eq!(discovery.domain, "songbird.local");
        assert_eq!(discovery.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_dnssd_from_env_default() {
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

        let expected = format!("_{}._tcp.{}", request.capability, discovery.domain);
        assert_eq!(expected, "_ai._tcp.songbird.local");
    }

    fn txt_lookup_fixture(strings: Vec<String>) -> TxtLookup {
        let name = Name::from_str("_fixture._tcp.local.").unwrap();
        let query = Query::query(name, RecordType::TXT);
        let txt = TXT::new(strings);
        Lookup::from_rdata(query, RData::TXT(txt)).into()
    }

    #[test]
    fn parse_service_builds_http_endpoint_without_txt() {
        let target = Name::from_str("compute-1.example.local.").unwrap();
        let srv = SRV::new(20, 100, 9100, target);
        let req = CapabilityRequest::new("compute");
        let svc = DnsSDDiscovery::parse_service(&srv, None, &req);
        assert_eq!(svc.endpoint, "http://compute-1.example.local.:9100");
        assert_eq!(svc.protocol, Protocol::Http);
        assert!(svc.features.is_empty());
        assert_eq!(svc.priority, 20);
        assert_eq!(svc.weight, 100);
    }

    #[test]
    fn parse_service_applies_txt_features_protocol_and_metadata() {
        let target = Name::from_str("registry-host.local.").unwrap();
        let srv = SRV::new(5, 50, 443, target);
        let txt = txt_lookup_fixture(vec![
            "features=kv,transactions".into(),
            "protocol=https".into(),
            "zone=us-east".into(),
        ]);
        let req = CapabilityRequest::new("storage");
        let svc = DnsSDDiscovery::parse_service(&srv, Some(txt), &req);
        assert_eq!(svc.endpoint, "https://registry-host.local.:443");
        assert_eq!(svc.protocol, Protocol::Https);
        assert_eq!(svc.features, vec!["kv", "transactions"]);
        assert_eq!(svc.metadata.get("zone"), Some(&String::from("us-east")));
        assert_eq!(svc.priority, 5);
    }

    #[test]
    fn parse_service_protocol_tarpc_ws_and_unknown_metadata_only() {
        let target = Name::from_str("tar-host.local.").unwrap();
        let srv_t = SRV::new(1, 0, 3030, target.clone());
        let txt_t = txt_lookup_fixture(vec!["protocol=tarpc".into()]);
        assert_eq!(
            DnsSDDiscovery::parse_service(&srv_t, Some(txt_t), &CapabilityRequest::new("compute"))
                .protocol,
            Protocol::Tarpc
        );

        let srv_w = SRV::new(1, 0, 8080, target.clone());
        let txt_w = txt_lookup_fixture(vec!["protocol=websocket".into()]);
        assert_eq!(
            DnsSDDiscovery::parse_service(&srv_w, Some(txt_w), &CapabilityRequest::new("compute"))
                .protocol,
            Protocol::WebSocket
        );

        let srv_u = SRV::new(2, 0, 80, target);
        let txt_u = txt_lookup_fixture(vec!["protocol=grpc".into(), "extra=v".into()]);
        let svc_u =
            DnsSDDiscovery::parse_service(&srv_u, Some(txt_u), &CapabilityRequest::new("compute"));
        assert_eq!(svc_u.protocol, Protocol::Http);
        assert_eq!(svc_u.metadata.get("extra"), Some(&String::from("v")));
    }

    #[test]
    fn select_best_service_orders_priority_then_weight() {
        let req = CapabilityRequest::new("ai").with_features(&["infer"]);
        let services = vec![
            DnsService {
                name: "low-weight.local.".into(),
                endpoint: "http://low-weight.local.:1".into(),
                protocol: Protocol::Http,
                features: vec!["infer".into()],
                metadata: HashMap::new(),
                priority: 10,
                weight: 10,
            },
            DnsService {
                name: "high-weight.local.".into(),
                endpoint: "http://high-weight.local.:2".into(),
                protocol: Protocol::Http,
                features: vec!["infer".into()],
                metadata: HashMap::new(),
                priority: 10,
                weight: 100,
            },
            DnsService {
                name: "worse-prio.local.".into(),
                endpoint: "http://worse-prio.local.:3".into(),
                protocol: Protocol::Http,
                features: vec!["infer".into()],
                metadata: HashMap::new(),
                priority: 99,
                weight: 1000,
            },
        ];

        let picked = DnsSDDiscovery::select_best_service(&services, &req).unwrap();
        assert_eq!(picked.endpoint, "http://high-weight.local.:2");
        assert_eq!(picked.name, "high-weight.local.");
    }

    #[test]
    fn select_best_service_filters_required_features() {
        let svc = DnsService {
            name: "limited.local.".into(),
            endpoint: "http://limited.local.:80".into(),
            protocol: Protocol::Http,
            features: vec!["alpha".into()],
            metadata: HashMap::new(),
            priority: 0,
            weight: 1,
        };
        assert!(
            DnsSDDiscovery::select_best_service(
                &[svc],
                &CapabilityRequest::new("compute").with_features(&["beta"]),
            )
            .is_err()
        );
    }
}
