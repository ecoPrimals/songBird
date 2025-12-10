//! DNS-based Service Discovery (DNS-SD)
//!
//! Implements RFC 6763 DNS-Based Service Discovery for internet-wide service discovery.
//! Uses standard DNS PTR, SRV, and TXT records for service enumeration and metadata.
//!
//! ## Features
//! - Service discovery via DNS queries
//! - Support for multiple search domains
//! - Service metadata via TXT records
//! - Result caching with configurable TTL
//! - Integration with existing discovery system
//!
//! ## Usage
//! ```rust,no_run
//! use songbird_discovery::DnsDiscovery;
//!
//! # async fn example() -> songbird_types::errors::SongbirdResult<()> {
//! let discovery = DnsDiscovery::new(
//!     "_songbird._tcp".to_string(),
//!     vec!["example.com".to_string()],
//! ).await?;
//!
//! let services = discovery.discover_services().await?;
//! # Ok(())
//! # }
//! ```

use hickory_resolver::proto::rr::{RData, RecordType};
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Name, TokioAsyncResolver,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::traits::service::ServiceInfo;
use chrono::Utc;
use songbird_types::errors::{SongbirdError, SongbirdResult};

/// DNS-based service discovery implementation
#[derive(Clone)]
pub struct DnsDiscovery {
    /// DNS resolver
    resolver: Arc<TokioAsyncResolver>,

    /// Service type to query (e.g., "_songbird._tcp")
    service_type: String,

    /// Search domains for service discovery
    search_domains: Vec<String>,

    /// Cached discovered services
    cache: Arc<RwLock<HashMap<String, CachedService>>>,

    /// Cache time-to-live
    cache_ttl: Duration,
}

/// Cached service information
#[derive(Clone, Debug)]
struct CachedService {
    service: ServiceInfo,
    discovered_at: Instant,
}

impl DnsDiscovery {
    /// Create a new DNS-based service discovery instance
    ///
    /// # Arguments
    /// * `service_type` - Service type to discover (e.g., "_songbird._tcp")
    /// * `search_domains` - List of domains to search in
    ///
    /// # Errors
    /// Returns error if DNS resolver cannot be initialized
    #[allow(clippy::unused_async)] // Kept async for API consistency
    pub async fn new(service_type: String, search_domains: Vec<String>) -> SongbirdResult<Self> {
        info!("Initializing DNS discovery for service type: {}", service_type);

        // Create resolver with system configuration
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        Ok(Self {
            resolver: Arc::new(resolver),
            service_type,
            search_domains,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: songbird_config::defaults::timeouts::cache_expiry(),
        })
    }

    /// Create DNS discovery with custom resolver configuration
    ///
    /// # Errors
    /// Returns error if DNS resolver cannot be initialized
    #[allow(clippy::unused_async)] // Kept async for API consistency
    pub async fn with_config(
        service_type: String,
        search_domains: Vec<String>,
        resolver_config: ResolverConfig,
        resolver_opts: ResolverOpts,
    ) -> SongbirdResult<Self> {
        info!("Initializing DNS discovery with custom config for: {}", service_type);

        let resolver = TokioAsyncResolver::tokio(resolver_config, resolver_opts);

        Ok(Self {
            resolver: Arc::new(resolver),
            service_type,
            search_domains,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: songbird_config::defaults::timeouts::cache_expiry(),
        })
    }

    /// Set cache TTL
    #[must_use]
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Discover services via DNS-SD
    ///
    /// Queries DNS for PTR records listing services, then queries SRV/TXT records
    /// for each service to get endpoint and metadata information.
    ///
    /// # Errors
    /// Returns error if DNS queries fail
    pub async fn discover_services(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!("Starting DNS service discovery");

        let mut services = Vec::new();

        for domain in &self.search_domains {
            // Construct PTR query: _service._proto.domain
            let ptr_query = format!("{}.{}", self.service_type, domain);
            debug!("Querying PTR records for: {}", ptr_query);

            // Query PTR records to enumerate services
            match self.query_ptr_records(&ptr_query).await {
                Ok(service_names) => {
                    debug!("Found {} services in domain {}", service_names.len(), domain);

                    // For each service name, query SRV and TXT records
                    for service_name in service_names {
                        match self.query_service_details(&service_name, domain).await {
                            Ok(service) => services.push(service),
                            Err(e) => {
                                warn!("Failed to query details for {}: {}", service_name, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("No services found in domain {}: {}", domain, e);
                }
            }
        }

        // Update cache
        self.update_cache(&services).await;

        info!("DNS discovery found {} services", services.len());
        Ok(services)
    }

    /// Query PTR records to enumerate service instances
    async fn query_ptr_records(&self, query: &str) -> SongbirdResult<Vec<String>> {
        let name = Name::from_utf8(query)
            .map_err(|e| SongbirdError::discovery(format!("Invalid DNS name: {e}")))?;

        let response = self
            .resolver
            .lookup(name, RecordType::PTR)
            .await
            .map_err(|e| SongbirdError::discovery(format!("PTR lookup failed: {e}")))?;

        let mut service_names = Vec::new();
        for record in response.record_iter() {
            if let Some(RData::PTR(ptr)) = record.data() {
                service_names.push(ptr.to_string());
            }
        }

        Ok(service_names)
    }

    /// Query SRV and TXT records for service details
    async fn query_service_details(
        &self,
        service_name: &str,
        domain: &str,
    ) -> SongbirdResult<ServiceInfo> {
        // Query SRV record for host and port
        let srv_name = Name::from_utf8(service_name)
            .map_err(|e| SongbirdError::discovery(format!("Invalid service name: {e}")))?;

        let srv_response = self
            .resolver
            .lookup(srv_name.clone(), RecordType::SRV)
            .await
            .map_err(|e| SongbirdError::discovery(format!("SRV lookup failed: {e}")))?;

        // Get best SRV record based on priority (lower is better) and weight (higher is better)
        // RFC 2782: Select records with lowest priority first, then use weight for load balancing
        let srv_record = srv_response
            .record_iter()
            .filter_map(|r| {
                if let Some(RData::SRV(srv)) = r.data() {
                    Some(srv)
                } else {
                    None
                }
            })
            .min_by_key(|srv| (srv.priority(), u16::MAX - srv.weight()))
            .ok_or_else(|| SongbirdError::discovery("No SRV record found".to_string()))?;

        let target_host = srv_record.target().to_string();
        let port = srv_record.port();

        // Query A/AAAA records for IP address
        let target_name = Name::from_utf8(&target_host)
            .map_err(|e| SongbirdError::discovery(format!("Invalid target host: {e}")))?;

        // Try A record first (IPv4)
        let ip_addr = match self.resolver.lookup(target_name.clone(), RecordType::A).await {
            Ok(response) => response.record_iter().find_map(|r| {
                if let Some(RData::A(a)) = r.data() {
                    Some(IpAddr::V4(a.0))
                } else {
                    None
                }
            }),
            Err(_) => {
                // Try AAAA record (IPv6)
                self.resolver.lookup(target_name, RecordType::AAAA).await.ok().and_then(
                    |response| {
                        response.record_iter().find_map(|r| {
                            if let Some(RData::AAAA(aaaa)) = r.data() {
                                Some(IpAddr::V6(aaaa.0))
                            } else {
                                None
                            }
                        })
                    },
                )
            }
        };

        // Query TXT records for metadata
        let mut metadata = HashMap::new();
        let mut txt_records = Vec::new();

        if let Ok(txt_response) = self.resolver.lookup(srv_name, RecordType::TXT).await {
            for record in txt_response.record_iter() {
                if let Some(RData::TXT(txt)) = record.data() {
                    // Parse TXT record key=value pairs
                    for data in txt.iter() {
                        let text = String::from_utf8_lossy(data);
                        txt_records.push(text.to_string());

                        // Parse key=value format
                        if let Some((key, value)) = text.split_once('=') {
                            metadata.insert(
                                key.to_string(),
                                serde_json::Value::String(value.to_string()),
                            );
                        }
                    }
                }
            }
        }

        // Extract service instance name from full service name
        let instance_name = service_name.split('.').next().unwrap_or(service_name).to_string();

        // Build ServiceInfo
        let service_info = ServiceInfo {
            service_id: format!("dns:{domain}:{service_name}:{port}"),
            name: instance_name.clone(),
            version: metadata
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            service_type: self.service_type.clone(),
            description: metadata.get("description").and_then(|v| v.as_str()).map(str::to_string),
            endpoints: vec![], // Could be populated from TXT records
            health_check_endpoint: metadata
                .get("health")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            metadata,
            tags: txt_records,
            dependencies: vec![],
            status: crate::traits::service::ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: instance_name,
            host: ip_addr.map(|ip| ip.to_string()).unwrap_or(target_host),
            port,
        };

        Ok(service_info)
    }

    /// Get cached services (returns non-expired entries)
    pub async fn get_cached_services(&self) -> Vec<ServiceInfo> {
        let cache = self.cache.read().await;
        let now = Instant::now();

        cache
            .values()
            .filter(|cached| now.duration_since(cached.discovered_at) < self.cache_ttl)
            .map(|cached| cached.service.clone())
            .collect()
    }

    /// Update cache with discovered services
    async fn update_cache(&self, services: &[ServiceInfo]) {
        let mut cache = self.cache.write().await;
        let now = Instant::now();

        // Clear expired entries
        cache.retain(|_, cached| now.duration_since(cached.discovered_at) < self.cache_ttl);

        // Add/update services
        for service in services {
            cache.insert(
                service.service_id.clone(),
                CachedService {
                    service: service.clone(),
                    discovered_at: now,
                },
            );
        }
    }

    /// Clear the service cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_discovery_creation() {
        let discovery =
            DnsDiscovery::new("_songbird._tcp".to_string(), vec!["local".to_string()]).await;

        assert!(discovery.is_ok());
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let discovery =
            DnsDiscovery::new("_test._tcp".to_string(), vec!["example.com".to_string()])
                .await
                .unwrap();

        // Initially empty
        let cached = discovery.get_cached_services().await;
        assert!(cached.is_empty());

        // Clear should work on empty cache
        discovery.clear_cache().await;
    }

    #[tokio::test]
    async fn test_custom_cache_ttl() {
        let discovery =
            DnsDiscovery::new("_test._tcp".to_string(), vec!["example.com".to_string()])
                .await
                .unwrap()
                .with_cache_ttl(Duration::from_secs(60));

        assert_eq!(discovery.cache_ttl, Duration::from_secs(60));
    }
}
