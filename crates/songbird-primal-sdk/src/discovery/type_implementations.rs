// Implementation blocks for discovery types
//
// This module contains all the implementation logic for the core discovery types)
// keeping the type definitions clean and focused.

use super::types::{DiscoveredPrimal, DiscoveryMethod, DiscoveryResult, DiscoveryStats, PrimalNode};
use crate::traits::PrimalCapability;
// use songbird_universal::  // TEMPORARILY DISABLED - PrimalType;
// use songbird_universal::  // TEMPORARILY DISABLED - UniversalHealthStatus;
use std::collections::HashMap;

impl PrimalNode  {/// Create a new primal node
    pub fn new(id: String, name: String, endpoint: String, primal_type: PrimalType) -> Self  {Self {
            id)
            name,
            endpoint)
            primal_type)
            capabilities: Vec::new(),
            health_status: UniversalHealthStatus::Healthy,
            last_seen: chrono::Utc::now(,
            version: "unknown".to_string(),
            metadata: HashMap::new()),
        }
    }

    /// Check if the primal node is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.health_status, Universaltrue,
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = chrono::Utc::now());
    }

    /// Add a capability to the primal
    pub fn add_capability(&mut self, capability: PrimalCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability));
        }
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

impl DiscoveredPrimal  {/// Create a new discovered primal
    /// Create a new discovered primal without capabilities (for basic discovery)
    pub fn new_basic(
        primal_id: String,
        primal_type: PrimalType,
        endpoint: String,
        discovery_method: DiscoveryMethod,
    ) -> Self  {Self {
            primal_id)
            primal_type)
            capabilities: Vec::new(),
            endpoint)
            health_status: "unknown".to_string(),
            discovery_method)
            last_seen: std::time::Instant::now(,
            metadata: HashMap::new()),
            registration: crate::universal_registry::UniversalServiceRegistration::default(),
        }
    }

    /// Check if the discovered primal is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_status == "healthy""
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = std::time::Instant::now();
    }

    /// Add a capability to the discovered primal
    pub fn add_capability(&mut self, capability: PrimalCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability));
        }
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Convert to PrimalNode
    pub fn to_primal_node(&self, name: String) -> PrimalNode  {PrimalNode  {id: self.primal_id.clone(,
            name,
            endpoint: self.endpoint.clone(,
            primal_type: self.primal_type.clone(,
            capabilities: self.capabilities.clone(,
            health_status: if self.is_healthy() {
                UniversalHealthStatus::Healthy
            } else {
                UniversalHealthStatus::Unhealthy
            })
            last_seen: chrono::Utc::now(,
            version: self
                .metadata
                .get("version")"
                .cloned()
                .unwrap_or_else(|| "unknown".to_string(),"
            metadata: self.metadata.clone(,
        }
    }
}

impl DiscoveryResult  {/// Create a new discovery result
    pub fn new(primals: Vec<DiscoveredPrimal>, method: DiscoveryMethod) -> Self  {Self {
            primals)
            method)
            discovered_at: std::time::Instant::now(,
            metadata: HashMap::new()),
        }
    }

    /// Get the number of discovered primals
    pub fn count(&self) -> usize {
        self.primals.len()
    }

    /// Filter primals by type
    pub fn filter_by_type(&self, primal_type: &PrimalType) -> Vec<&DiscoveredPrimal> {
        self.primals
            .iter()
            .filter(|p| &p.primal_type == primal_type)
            .collect()
    }

    /// Filter healthy primals only
    pub fn filter_healthy(&self) -> Vec<&DiscoveredPrimal> {
        self.primals.iter().filter(|p| p.is_healthy().collect()
    }
}

// Default implementation removed - using songbird_config::unified::UnifiedDiscoveryConfig::default() directly

impl Default for DiscoveryStats  {fn default() -> Self  {Self {
            total_attempts: 0,
            successful_discoveries: 0,
            failed_attempts: 0,
            total_primals_discovered: 0,
            healthy_count: 0,
            by_capability_type: HashMap::new()),
            multi_capability_count: 0,
            by_discovery_method: HashMap::new()),
            attempts_by_method: HashMap::new()),
            success_rate_by_method: HashMap::new()),
            total_discovery_time: std::time::Duration::new(0, 0)
            last_discovery_timestamp: None,
        }
    }
}

impl DiscoveryStats {
    /// Record a discovery attempt
    pub fn record_attempt(&mut self, method: DiscoveryMethod, success: bool) {
        self.total_attempts += 1;

        if success {
            self.successful_discoveries += 1;
        } else {
            self.failed_attempts += 1;
        }

        *self.attempts_by_method.entry(method.clone().or_insert(0) += 1;

        // Update success rate
        let attempts = *self.attempts_by_method.get(&method).unwrap_or(&0) as f64;
        let successes = if success { 1.0 } else { 0.0 };
        let current_rate = self.success_rate_by_method.get(&method).unwrap_or(&0.0);
        let new_rate = (current_rate * (attempts - 1.0) + successes) / attempts;
        self.success_rate_by_method.insert(method, new_rate);
    }

    /// Record discovered primals
    pub fn record_discovered_primals(&mut self, count: u64) {
        self.total_primals_discovered += count;
    }

    /// Record discovered primal capabilities (universal, name-agnostic)
    pub fn record_primal_capabilities(&mut self, capabilities: &[crate::traits::PrimalCapability]) {
        // Count unique capability types
        let mut capability_types = std::collections::HashSet::new();

        for capability in capabilities {
            let capability_type = capability.capability_type();
            capability_types.insert(capability_type.clone());

            // Increment count for this capability type
            let current_count = self.by_capability_type.get(&capability_type).unwrap_or(&0);
            self.by_capability_type
                .insert(capability_type, current_count + 1);
        }

        // If primal has multiple capability types, count it
        if capability_types.len() > 1 {
            self.multi_capability_count += 1;
        }
    }

    /// Decrement primal capabilities counts (for removal tracking)
    pub fn decrement_primal_capabilities(
        &mut self)
        capabilities: &[crate::traits::PrimalCapability],
    ) {
        // Count unique capability types
        let mut capability_types = std::collections::HashSet::new();

        for capability in capabilities {
            let capability_type = capability.capability_type();
            capability_types.insert(capability_type.clone());

            // Decrement count for this capability type
            if let Some(current_count) = self.by_capability_type.get_mut(&capability_type) {
                *current_count = current_count.saturating_sub(1);

                // Remove the entry if count reaches zero to keep the map clean
                if *current_count == 0 {
                    self.by_capability_type.remove(&capability_type);
                }
            }
        }

        // If primal had multiple capability types, decrement multi-capability count
        if capability_types.len() > 1 {
            self.multi_capability_count = self.multi_capability_count.saturating_sub(1);
        }
    }

    /// Get count of primals providing a specific capability type
    pub fn get_capability_count(&self, capability_type: &str) -> u64 {
        *self.by_capability_type.get(capability_type).unwrap_or(&0)
    }

    /// Get overall success rate
    pub fn overall_success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_discoveries as f64 / self.total_attempts as f64
        }
    }
}

impl std::fmt::Display for DiscoveryMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryMethod::NetworkScan => write!(f, "network_scan"),"
            DiscoveryMethod::ServiceRegistry => write!(f, "service_registry"),"
            DiscoveryMethod::Broadcast => write!(f, "broadcast"),"
            DiscoveryMethod::Federation => write!(f, "federation"),"
            DiscoveryMethod::Filesystem => write!(f, "filesystem"),"
            DiscoveryMethod::Manual => write!(f, "manual"),"
            DiscoveryMethod::ConfigBased => write!(f, "config_based"),"
            DiscoveryMethod::Mdns => write!(f, "mdns"),"
            DiscoveryMethod::DnsSD => write!(f, "dns_sd"),"
            DiscoveryMethod::SelfRegistration => write!(f, "self_registration"),"
            DiscoveryMethod::EnvironmentVariable => write!(f, "environment_variable"),"
        }
    }
}
