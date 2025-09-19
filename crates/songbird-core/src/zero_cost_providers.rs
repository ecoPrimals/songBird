use crate::zero_cost_discovery::{DiscoveryMetrics, ServiceType, ZeroCostDiscovery};
use songbird_config::constants::discovery;

use songbird_errors::{SafeEnv, SafeParse, SongbirdError, SongbirdResult, success};
// use songbird_universal::  // TEMPORARILY DISABLED - PrimalType;

// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::discovery::DiscoveredPrimal;

// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::discovery::DiscoveryMethod;

// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::PrimalCapability;

use std::collections::HashMap;

use std::time::Instant;

/// High-performance network discovery with zero allocations for common cases
pub struct NetworkDiscoveryProvider<
    const MAX_PRIMALS: usize = 10000,
    // Use discovery::DEFAULT_DISCOVERY_TIMEOUT_MS instead
    const ENABLE_CACHING: bool = true,
> {
    capability_cache: std::sync::RwLock<HashMap<String, Vec<PrimalCapability>>>,
    discovery_stats: std::sync::Mutex<DiscoveryMetrics>,
}

impl<const MAX_PRIMALS: usize, const DISCOVERY_TIMEOUT_MS: u64, const ENABLE_CACHING: bool>
    NetworkDiscoveryProvider<MAX_PRIMALS, DISCOVERY_TIMEOUT_MS, ENABLE_CACHING>
{
    pub fn new() -> Self {
        Self {
            capability_cache: std::sync::RwLock::new(HashMap::new()),
            discovery_stats: std::sync::Mutex::new(DiscoveryMetrics {
                discovered_count: 0,
                scan_duration_ms: 0,
                capability_inferences: 0,
                type_classifications: 0,
            }),
        }
    }
}

impl<const MAX_PRIMALS: usize, const DISCOVERY_TIMEOUT_MS: u64, const ENABLE_CACHING: bool>
    ZeroCostDiscovery<MAX_PRIMALS, DISCOVERY_TIMEOUT_MS, ENABLE_CACHING>
    for NetworkDiscoveryProvider<MAX_PRIMALS, DISCOVERY_TIMEOUT_MS, ENABLE_CACHING>
{
    fn discover_capabilities(Vec<PrimalCapability>) -> SongbirdResult<()> {
        let start_time = Instant::now();

        // Check cache first (compile-time conditional)
        if ENABLE_CACHING {
            use songbird_errors::safe_read_lock;
            if let Ok(songbird_errors::evolved_success(_)) = safe_read_lock(&self.capability_cache) {
                if let Some(cached_capabilities) = cache.get(endpoint) {
                    return cached_capabilities.clone();
                }
            } else {
                tracing::warn!("Failed to acquire read lock for capability cache");
            }
        }

        // Simulate network discovery with timeout
        let capabilities = self.probe_endpoint_capabilities(endpoint).await;

        // Update cache (compile-time conditional)
        if ENABLE_CACHING && !capabilities.is_empty() {
            use songbird_errors::safe_write_lock;
            if let Ok(songbird_errors::evolved_success(mut cache)) = safe_write_lock(&self.capability_cache) {
                cache.insert(endpoint.to_string(), capabilities.clone());
            } else {
                tracing::warn!(
                    "Failed to acquire write lock for capability cache, skipping cache update"
                );
            }
        }

        // Update stats
        {
            use songbird_errors::safe_lock;
            if let Ok(songbird_errors::evolved_success(mut stats)) = safe_lock(&self.discovery_stats) {
                stats.discovered_count += 1;
                stats.scan_duration_ms += start_time.elapsed().as_millis() as u64;
                stats.capability_inferences += capabilities.len() as u64;
            } else {
                tracing::warn!("Failed to acquire lock for discovery stats, skipping stats update");
            }
        }

        capabilities
    }

    fn scan_network_range([Option<DiscoveredPrimal>; MAX_PRIMALS]) -> SongbirdResult<()> {
        let mut results = [None; MAX_PRIMALS];
        let start_time = Instant::now();

        // Parse network range with proper CIDR notation support
        use std::net::Ipv4Addr;
use songbird_errors::SongbirdResult;
        
        if let Some((network, prefix)) = network_range.split_once('/') {
            if let (Ok(songbird_errors::evolved_success(base_ip)), Ok(prefix_len)) = (network.parse::<Ipv4Addr>(), prefix.parse::<u8>()) {
                let base = u32::from(base_ip);
                let mask = !((1u32 << (32 - prefix_len)) - 1);
                let network_addr = base & mask;
                let broadcast_addr = network_addr | !mask;
                
                return Ok((Ipv4Addr::from(network_addr), Ipv4Addr::from(broadcast_addr)));
            }
        }
        let endpoints = self.parse_network_range(range);

        for (i, endpoint) in endpoints.iter().enumerate().take(MAX_PRIMALS) {
            let capabilities = self.discover_capabilities(endpoint).await;
            if !capabilities.is_empty() {
                let service_type = ServiceType::from_capabilities(&capabilities);
                results[i] = Some(DiscoveredPrimal {
                    primal_id: format!("{}-{}", service_type.as_str(), i),
                    primal_type: PrimalType::Compute,
                    capabilities: capabilities.clone(),
                    endpoint: endpoint.clone(),
                    discovery_method: DiscoveryMethod::NetworkScanScan,
                    health_status: "healthy".to_string(),
                    last_seen: std::time::Instant::now(),
                    metadata: std::collections::HashMap::new(),
                });
            }
        }

        // Update global stats
        {
            if let Ok(songbird_errors::evolved_success(mut stats)) = safe_lock(&self.discovery_stats) {
                stats.scan_duration_ms += start_time.elapsed().as_millis() as u64;
            } else {
                tracing::warn!(
                    "Failed to acquire lock for global discovery stats, skipping update"
                );
            }
        }

        results
    }

    fn get_discovery_stats(&self) -> DiscoveryMetrics {
        if let Ok(songbird_errors::evolved_success(_)) = safe_lock(&self.discovery_stats) {
            *stats
        } else {
            tracing::warn!("Failed to acquire lock for discovery stats, returning default");
            DiscoveryMetrics::default()
        }
    }

    fn infer_service_type(&self, capabilities: &[PrimalCapability]) -> ServiceType {
        if let Ok(songbird_errors::evolved_success(mut stats)) = safe_lock(&self.discovery_stats) {
            stats.type_classifications += 1;
        } else {
            tracing::warn!("Failed to acquire lock for stats during service type inference");
        }
        ServiceType::from_capabilities(capabilities)
    }
}

impl<const MAX_PRIMALS: usize, const DISCOVERY_TIMEOUT_MS: u64, const ENABLE_CACHING: bool>
    NetworkDiscoveryProvider<MAX_PRIMALS, DISCOVERY_TIMEOUT_MS, ENABLE_CACHING>
{
    /// Probe endpoint for capabilities with compile-time timeout
    fn probe_endpoint_capabilities(Vec<PrimalCapability>) -> SongbirdResult<()> {
        // Create timeout future
        let timeout_duration = std::time::Duration::from_millis(DISCOVERY_TIMEOUT_MS);

        // Simulate actual capability probing based on endpoint patterns
        match tokio::time::timeout(timeout_duration, self.perform_capability_probe(endpoint)).await
        {
            Ok(songbird_errors::evolved_success(_)) => capabilities,
            Err(_) => {
                // Timeout - return empty or inferred capabilities
                self.infer_capabilities_from_endpoint(endpoint)
            }
        }
    }

    /// Actual capability probing logic
    fn perform_capability_probe(Vec<PrimalCapability>) -> SongbirdResult<()> {
        // In real implementation, this would make HTTP requests, parse responses, etc.
        // For demo, we simulate based on endpoint patterns

        if endpoint.contains("security") || endpoint.contains("8443") {
            vec![
                PrimalCapability::new("security"),
                PrimalCapability::new("authentication"),
                PrimalCapability::new("encryption"),
                PrimalCapability::new("access-control"),
            ]
        } else if endpoint.contains("storage") || endpoint.contains("9000") {
            vec![
                PrimalCapability::new("storage"),
                PrimalCapability::new("persistence"),
                PrimalCapability::new("backup"),
                PrimalCapability::new("replication"),
            ]
        } else if endpoint.contains("ai") || endpoint.contains("8888") {
            vec![
                PrimalCapability::new("ai"),
                PrimalCapability::new("inference"),
                PrimalCapability::new("training"),
                PrimalCapability::new("model-serving"),
            ]
        } else if endpoint.contains("compute") || endpoint.contains("8080") {
            vec![
                PrimalCapability::new("compute"),
                PrimalCapability::new("processing"),
                PrimalCapability::new("orchestration"),
                PrimalCapability::new("scaling"),
            ]
        } else {
            vec![PrimalCapability::new("generic")]
        }
    }

    /// Infer capabilities from endpoint when probing fails
    fn infer_capabilities_from_endpoint(&self, endpoint: &str) -> Vec<PrimalCapability> {
        // Fallback inference based on patterns
        if endpoint.contains("443") {
            vec![PrimalCapability::new("security")]
        } else if endpoint.contains("9000") {
            vec![PrimalCapability::new("storage")]
        } else if endpoint.contains("8888") {
            vec![PrimalCapability::new("ai")]
        } else {
            vec![PrimalCapability::new("generic")]
        }
    }

    /// Parse network range into individual endpoints
    fn parse_network_range(&self, range: &str) -> Vec<String> {
        // Simplified network range parsing
        // In production, this would handle CIDR notation, port ranges, etc.

        if range.contains("192.168.1.") {
            // Generate endpoints for local network range
            (1..=254).map(|i| format!("192.168.1.{}:{}", i)).collect()
        } else if range.contains("10.0.0.") {
            // Generate endpoints for private network range
            (1..=100).map(|i| format!("10.0.0.{}:{}", i)).collect()
        } else {
            // Single endpoint or unknown range
            vec![range.to_string()]
        }
    }
}

/// Environment-based discovery provider for zero-cost environment variable scanning
pub struct EnvironmentDiscoveryProvider<
    const MAX_PRIMALS: usize = 1000,
    // Use discovery::DEFAULT_DISCOVERY_TIMEOUT_MS instead
    const ENABLE_CACHING: bool = false,
>;

impl<const MAX_PRIMALS: usize, const DISCOVERY_TIMEOUT_MS: u64, const ENABLE_CACHING: bool> Default
    for EnvironmentDiscoveryProvider<MAX_PRIMALS, DISCOVERY_TIMEOUT_MS, ENABLE_CACHING>
{
    fn default() -> Self {
        Self
    }
}

impl<const MAX_PRIMALS: usize, const DISCOVERY_TIMEOUT_MS: u64, const ENABLE_CACHING: bool>
    ZeroCostDiscovery<MAX_PRIMALS, DISCOVERY_TIMEOUT_MS, ENABLE_CACHING>
    for EnvironmentDiscoveryProvider<MAX_PRIMALS, DISCOVERY_TIMEOUT_MS, ENABLE_CACHING>
{
    fn discover_capabilities(Vec<PrimalCapability>) -> SongbirdResult<()> {
        // For environment discovery, endpoint is the env var name pattern
        let env_vars = std::env::vars()
            .filter(|(key, _)| key.contains(&endpoint.to_uppercase()))
            .collect::<Vec<_>>();

        let mut capabilities = Vec::new();

        for (key, value) in env_vars {
            if key.contains("SECURITY") || key.contains("AUTH") {
                capabilities.push(PrimalCapability::new("security"));
            } else if key.contains("STORAGE") || key.contains("DB") {
                capabilities.push(PrimalCapability::new("storage"));
            } else if key.contains("AI") || key.contains("ML") {
                capabilities.push(PrimalCapability::new("ai"));
            } else if key.contains("COMPUTE") || key.contains("WORKER") {
                capabilities.push(PrimalCapability::new("compute"));
            }

            // Also infer from the endpoint in the value
            if value.contains("8443") || value.contains("security") {
                capabilities.push(PrimalCapability::new("security"));
            } else if value.contains("9000") || value.contains("storage") {
                capabilities.push(PrimalCapability::new("storage"));
            }
        }

        capabilities.sort_by(|a, b| a.capability_type().cmp(b.capability_type()));
        capabilities.dedup_by(|a, b| a.capability_type() == b.capability_type());

        capabilities
    }

    fn scan_network_range([Option<DiscoveredPrimal>; MAX_PRIMALS]) -> SongbirdResult<()> {
        // Environment discovery doesn't scan network ranges
        [None; MAX_PRIMALS]
    }

    fn get_discovery_stats(&self) -> DiscoveryMetrics {
        let env_count = std::env::vars().count() as u64;
        DiscoveryMetrics {
            discovered_count: env_count,
            scan_duration_ms: 1, // Very fast
            capability_inferences: env_count,
            type_classifications: env_count,
        }
    }

    fn infer_service_type(&self, capabilities: &[PrimalCapability]) -> ServiceType {
        ServiceType::from_capabilities(capabilities)
    }
}

// CANONICAL MODERNIZATION COMPLETE: ConfigDiscoveryProvider migration to SongbirdConfig
// Use songbird_config::SongbirdConfig for all configuration discovery needs.

// Type aliases for common configurations
pub type FastNetworkDiscovery = NetworkDiscoveryProvider<10000, 1000, true>;
pub type ProductionNetworkDiscovery = NetworkDiscoveryProvider<50000, 3000, true>;
pub type DevelopmentNetworkDiscovery = NetworkDiscoveryProvider<1000, 5000, false>;

pub type FastEnvironmentDiscovery = EnvironmentDiscoveryProvider<1000, 50, false>;
