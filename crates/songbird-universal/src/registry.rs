//! Universal registry types and patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{HealthStatus, PrimalType};

/// Universal registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatistics {
    pub total_services: u64,
    pub services_by_primal: HashMap<PrimalType, u64>,
    pub services_by_health: HashMap<HealthStatus, u64>,
    pub capability_distribution: HashMap<String, usize>,
}

/// Universal registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRegistryConfig {
    pub store_type: StoreType,
    pub health_check_interval: std::time::Duration,
    pub service_expiry_duration: std::time::Duration,
    pub capability_validation_enabled: bool,
    pub event_publishing_enabled: bool,
    pub discovery_backends: Vec<DiscoveryBackendConfig>,
    pub replication_factor: usize,
    pub consistency_level: ConsistencyLevel,
}

impl Default for UniversalRegistryConfig {
    fn default() -> Self {
        Self {
            store_type: StoreType::InMemory,
            health_check_interval: std::time::Duration::from_secs(30),
            service_expiry_duration: std::time::Duration::from_secs(300),
            capability_validation_enabled: true,
            event_publishing_enabled: true,
            discovery_backends: Vec::new(),
            replication_factor: 3,
            consistency_level: ConsistencyLevel::Strong,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoreType {
    InMemory,
    Distributed {
        backend: String,
        endpoints: Vec<String>,
    },
    Database {
        connection_string: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    Eventual,
    Strong,
    Quorum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryBackendConfig {
    pub backend_type: String,
    pub enabled: bool,
    pub configuration: HashMap<String, String>,
}

/// Universal capability index
pub struct CapabilityIndex {
    service_capabilities: HashMap<String, Vec<crate::ServiceCapability>>,
    capability_services: HashMap<String, Vec<String>>,
}

impl Default for CapabilityIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityIndex {
    pub fn new() -> Self {
        Self {
            service_capabilities: HashMap::new(),
            capability_services: HashMap::new(),
        }
    }

    pub fn add_service_capability(
        &mut self,
        service_id: &str,
        capability: crate::ServiceCapability,
    ) {
        // Add to service -> capabilities mapping
        self.service_capabilities
            .entry(service_id.to_string())
            .or_default()
            .push(capability.clone());

        // Add to capability -> services mapping
        let capability_key = capability.name();
        self.capability_services
            .entry(capability_key)
            .or_default()
            .push(service_id.to_string());
    }

    pub fn remove_service_capabilities(&mut self, service_id: &str) {
        // Remove from service -> capabilities mapping
        if let Some(capabilities) = self.service_capabilities.remove(service_id) {
            // Remove from capability -> services mapping
            for capability in capabilities {
                let capability_key = capability.name();
                if let Some(services) = self.capability_services.get_mut(&capability_key) {
                    services.retain(|s| s != service_id);
                    if services.is_empty() {
                        self.capability_services.remove(&capability_key);
                    }
                }
            }
        }
    }

    pub fn find_services_with_capability(
        &self,
        requirement: &crate::CapabilityRequirement,
    ) -> std::collections::HashSet<String> {
        // This is a simplified implementation
        // In practice, you'd need more sophisticated matching
        let mut matching_services = std::collections::HashSet::new();

        for (service_id, capabilities) in &self.service_capabilities {
            for capability in capabilities {
                if requirement.is_satisfied_by(capability) {
                    matching_services.insert(service_id.clone());
                    break;
                }
            }
        }

        matching_services
    }

    pub fn get_capability_distribution(&self) -> HashMap<String, usize> {
        self.capability_services
            .iter()
            .map(|(capability, services)| (capability.clone(), services.len()))
            .collect()
    }
}
