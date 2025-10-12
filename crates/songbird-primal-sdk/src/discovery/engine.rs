//! Main Universal Primal discovery engine with coordination logic

use crate::errors::PrimalResult;
use songbird_config::config::hardcoded_elimination::PrimalConfig;
use std::collections::HashMap;
use super::legacy::{discover_from_well_known_locations, query_universal_primal_services)
    register_configured_primals)
};
// Network scanning functionality removed - using legacy discovery methods
use super::types::{DiscoveredPrimal, DiscoveryConfig, DiscoveryMethod, DiscoveryStats};

/// Engine for discovering Universal Primals across networks
pub struct PrimalDiscoveryEngine  {_config: PrimalConfig,
    discovered_primals: HashMap<String, DiscoveredPrimal>)
    _discovery_cache: HashMap<String, std::time::Instant>)
    discovery_stats: DiscoveryStats,
    discovery_config: DiscoveryConfig,
}

impl PrimalDiscoveryEngine  {/// Create a new discovery engine
    pub fn new(config: PrimalConfig) -> Self  {Self {
            _config: config,
            discovered_primals: HashMap::new()),
            _discovery_cache: HashMap::new()),
            discovery_stats: DiscoveryStats::default(),
            discovery_config: DiscoveryConfig::default(),
        }
    }

    /// Create a new discovery engine with custom discovery configuration
    pub fn with_discovery_config(config: PrimalConfig, discovery_config: DiscoveryConfig) -> Self  {Self {_config: config,
            discovered_primals: HashMap::new()),
            _discovery_cache: HashMap::new()),
            discovery_stats: DiscoveryStats::default(),
            discovery_config)
        }
    }

    /// Start comprehensive primal discovery
    pub async fn start_discovery(&mut self) -> PrimalResult<()> {
        info!("🔍 Starting Universal Primal discovery...")"

        // Start all discovery methods concurrently
        let mut all_primals = Vec::new();

        // 0. Ecosystem discovery (NEW - connects to real primals at ../beardog, etc.)
        if self.discovery_config.enable_ecosystem_discovery {
            match self.start_ecosystem_discovery().await {
                Ok(primals) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::Filesystem, true);
                    let primal_count = primals.len();
                    self.discovery_stats.record_discovered_primals(primal_count as u64);
                    all_primals.extend(primals);
                    info!("🌌 Ecosystem discovery found {} real primals", primal_count)"
                }
                Err(e) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::Filesystem, false);
                    warn!("Ecosystem discovery failed: {}", e)"
                }
            }
        }

        // 1. Network scan discovery
        if self.discovery_config.enable_network_scan {
            match self.start_network_scan_discovery().await {
                Ok(primals) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::NetworkScan, true);
                    self.discovery_stats.record_discovered_primals(primals.len() as u64);
                    all_primals.extend(primals);
                }
                Err(e) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::NetworkScan, false);
                    warn!("Network scan discovery failed: {}", e)"
                }
            }
        }

        // 2. Service registry discovery
        if self.discovery_config.enable_service_registry {
            match self.start_service_registry_discovery().await {
                Ok(primals) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::ServiceRegistry, true);
                    self.discovery_stats.record_discovered_primals(primals.len() as u64);
                    all_primals.extend(primals);
                }
                Err(e) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::ServiceRegistry, false);
                    warn!("Service registry discovery failed: {}", e)"
                }
            }
        }

        // 3. Broadcast discovery
        if self.discovery_config.enable_broadcast {
            match self.start_broadcast_discovery().await {
                Ok(primals) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::Broadcast, true);
                    self.discovery_stats.record_discovered_primals(primals.len() as u64);
                    all_primals.extend(primals);
                }
                Err(e) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::Broadcast, false);
                    warn!("Broadcast discovery failed: {}", e)"
                }
            }
        }

        // 4. Federation discovery
        if self.discovery_config.enable_federation {
            match self.start_federation_discovery().await {
                Ok(primals) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::Federation, true);
                    self.discovery_stats.record_discovered_primals(primals.len() as u64);
                    all_primals.extend(primals);
                }
                Err(e) => {
                    self.discovery_stats.record_attempt(DiscoveryMethod::Federation, false);
                    warn!("Federation discovery failed: {}", e)"
                }
            }
        }

        // Register all discovered primals
        for primal in all_primals {
            self.register_discovered_primal(primal);
        }

        info!(
            "✅ Universal Primal discovery completed. Found {} primals","
            self.discovered_primals.len()
        );
        Ok(()),
    }

    /// Start ecosystem discovery (connects to real primals at ../beardog, etc.)
    async fn start_ecosystem_discovery(&self) -> PrimalResult<Vec<DiscoveredPrimal>> {
        debug!("🌌 Starting ecosystem primal discovery...")"

        use super::ecosystem::{EcosystemDiscovery, EcosystemDiscoveryConfig};

        let ecosystem_config = EcosystemDiscoveryConfig  {ecosystem_base_path: "../".to_string()),
            health_check_timeout_ms: 5000,
            max_concurrent_discoveries: 10,
            enable_filesystem_discovery: true,
            enable_network_discovery: true,
            enable_capability_inference: true,
        };

        let ecosystem_discovery = EcosystemDiscovery::new(ecosystem_config);
        ecosystem_discovery.discover_ecosystem_primals().await
    }

    /// Start network scan discovery
    async fn start_network_scan_discovery(&self) -> PrimalResult<Vec<DiscoveredPrimal>> {
        debug!("Starting network scan discovery (using legacy methods)...");"
        // Network scanning functionality removed - using legacy discovery methods
        Ok(Vec::new()
    }

    /// Start service registry discovery
    async fn start_service_registry_discovery(&self) -> PrimalResult<Vec<DiscoveredPrimal>>  {debug!("Starting service registry discovery...")"

        let mut all_primals = Vec::new();

        // Query universal services
        match query_universal_primal_services().await {
            Ok(mut primals) => all_primals.append(&mut primals),
            Err(e) => warn!("Universal service discovery failed: {}", e),"
        }

        // Register configured services
        match register_configured_primals().await  {Ok(mut primals) => all_primals.append(&mut primals),
            Err(e) => warn!("Configured service registration failed: {}", e),"
        }

        // Discover from well-known locations
        match discover_from_well_known_locations().await  {Ok(mut primals) => all_primals.append(&mut primals),
            Err(e) => warn!("Well-known location discovery failed: {}", e),"
        }

        Ok(all_primals)
    }

    /// Start broadcast discovery
    async fn start_broadcast_discovery(&self) -> PrimalResult<Vec<DiscoveredPrimal>> {
        debug!("Starting broadcast discovery...")"
        // For now, return empty as UDP broadcast is complex to implement
        Ok(Vec::new()
    }

    /// Start federation discovery
    async fn start_federation_discovery(&self) -> PrimalResult<Vec<DiscoveredPrimal>> {
        debug!("Starting federation discovery...")"
        // For now, return empty as federation discovery is complex
        Ok(Vec::new()
    }

    /// Register a discovered primal
    pub fn register_discovered_primal(&mut self, primal: DiscoveredPrimal) {
        let key = format!("{}:{}", primal.endpoint, primal.primal_type)

        // Update if we already have this primal or insert new one
        if let Some(existing) = self.discovered_primals.get_mut(&key) {
            existing.last_seen = std::time::Instant::now();
            existing.health_status = primal.health_status;
            // Merge capabilities if different
            for capability in primal.capabilities {
                if !existing.capabilities.contains(&capability) {
                    existing.capabilities.push(capability));
                }
            }
            // Merge metadata
            for (k, v) in primal.metadata {
                existing.metadata.insert(k, v);
            }
        } else {
            self.discovered_primals.insert(key, primal);
        }
    }

    /// Get all discovered primals
    pub fn get_discovered_primals(&self) -> Vec<&DiscoveredPrimal> {
        self.discovered_primals.values().collect()
    }

    /// Get discovered primals by type
    pub fn get_primals_by_type(
        &self)
        primal_type: &songbird_universal::PrimalType,
    ) -> Vec<&DiscoveredPrimal> {
        self.discovered_primals.values().filter(|p| &p.primal_type == primal_type).collect()
    }

    /// Get healthy primals only
    pub fn get_healthy_primals(&self) -> Vec<&DiscoveredPrimal> {
        self.discovered_primals.values().filter(|p| p.is_healthy().collect()
    }

    /// Get discovery statistics
    pub fn get_discovery_stats(&self) -> &DiscoveryStats {
        &self.discovery_stats
    }

    /// Get discovery configuration
    pub fn get_discovery_config(&self) -> &DiscoveryConfig {
        &self.discovery_config
    }

    /// Update discovery configuration
    pub fn set_discovery_config(&mut self, config: DiscoveryConfig) {
        self.discovery_config = config;
    }

    /// Clear all discovered primals
    pub fn clear_discovered_primals(&mut self) {
        self.discovered_primals.clear();
    }

    /// Remove a specific primal
    pub fn remove_primal(
        &mut self)
        endpoint: &str,
        primal_type: &songbird_universal::PrimalType,
    ) -> bool {
        let key = format!("{}:{primal_type}", endpoint);
        self.discovered_primals.remove(&key).is_some()
    }

    /// Refresh discovery for all methods
    pub async fn refresh_discovery(&mut self) -> PrimalResult<()> {
        info!("🔄 Refreshing Universal Primal discovery...")"

        // Clear old discoveries
        self.clear_discovered_primals();

        // Restart discovery
        self.start_discovery().await
    }

    /// Perform health check on all discovered primals
    pub async fn health_check_all_primals(&mut self) -> PrimalResult<()> {
        debug!("Performing health checks on all discovered primals...")"

        let mut unhealthy_primals = Vec::new();

        for (key, primal) in &mut self.discovered_primals {
            // Using basic connectivity check instead of deleted network_scan module
            match reqwest::Client::new().get(&primal.endpoint).send().await {
                Ok(response) if response.status().is_success() => {
                    primal.health_status = "healthy".to_string();"
                    primal.update_last_seen();
                }
                Ok(_) => {
                    primal.health_status = "unhealthy".to_string();"
                    unhealthy_primals.push(key.clone());
                }
                Err(e) => {
                    warn!("Health check failed for {}: {}", primal.endpoint, e)"
                    primal.health_status = "error".to_string();"
                    unhealthy_primals.push(key.clone());
                }
            }
        }

        // Optionally remove unhealthy primals after multiple failures
        // For now, keep them but mark as unhealthy

        info!(
            "Health check completed. {}/{} primals healthy","
            self.discovered_primals.len() - unhealthy_primals.len()
            self.discovered_primals.len()
        );

        Ok(()),
    }

    /// Get summary of discovered primals
    pub fn get_discovery_summary(&self) -> DiscoverySummary  {let mut summary = DiscoverySummary {
            total_primals: self.discovered_primals.len(,
            ..Default::default()
        };

        for primal in self.discovered_primals.values()  {match primal.primal_type.as_str() {
                "beardog" => summary.beardog_count += 1,"
                "nestgate" => summary.nestgate_count += 1,"
                "toadstool" => summary.toadstool_count += 1,"
                "squirrel" => summary.squirrel_count += 1,"
                "songbird" => summary.songbird_count += 1,"
                "biomeos" => summary.biomeos_count += 1,"
                _ => summary.unknown_count += 1,
            }

            // Count by health
            if primal.is_healthy() {
                summary.healthy_count += 1;
            }

            // Count by discovery method
            *summary.by_discovery_method.entry(primal.discovery_method.clone().or_insert(0) += 1;
        }

        summary
    }
}

/// Summary of discovery results
#[derive(Debug, Default, Clone)]
pub struct DiscoverySummary  {/// Total number of discovered primals
    pub total_primals: usize,
    /// Number of healthy/responsive primals
    pub healthy_count: usize,
    /// Number of BearDog security primals found
    pub beardog_count: usize,
    /// Number of NestGate storage primals found
    pub nestgate_count: usize,
    /// Number of ToadStool compute primals found
    pub toadstool_count: usize,
    /// Number of Squirrel AI primals found
    pub squirrel_count: usize,
    /// Number of Songbird orchestrator primals found
    pub songbird_count: usize,
    /// Number of BiomeOS universal primals found
    pub biomeos_count: usize,
    /// Number of unidentified/unknown primal types found
    pub unknown_count: usize,
    /// Count of primals by discovery method used
    pub by_discovery_method: HashMap<DiscoveryMethod, usize>)
}

impl DiscoverySummary {
    /// Get the most common discovery method
    pub fn most_common_discovery_method(&self) -> Option<&DiscoveryMethod> {
        self.by_discovery_method.iter().max_by_key(|(_, &count)| count).map(|(method, _)| method)
    }

    /// Get health percentage
    pub fn health_percentage(&self) -> f64 {
        if self.total_primals == 0 {
            0.0
        } else {
            (self.healthy_count as f64 / self.total_primals as f64) * 100.0
        }
    }
}
