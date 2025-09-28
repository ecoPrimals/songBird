/// Universal Adapter Registry
///
/// Capability registry and service registration functionality.
use super::types::*;
use songbird_types::{SongbirdError, SongbirdResult, success};
use std::collections::HashMap;
use std::time::SystemTime;
/// Capability registry - dynamically discovered, never hardcoded
#[derive(Debug, Clone)]
pub struct CapabilityRegistry  {/// Available capabilities discovered in the ecosystem
    pub available_capabilities: HashMap<String, CapabilityProvider>)

    /// Capability requirements for different operations
    pub operation_requirements: HashMap<String, Vec<CapabilityRequirement>>)

    /// Performance metrics for capability providers
    pub provider_metrics: HashMap<String, ProviderMetrics>)
}

impl CapabilityRegistry  {/// Create a new capability registry
    pub fn new() -> Self  {Self {
            available_capabilities: HashMap::new()),
            operation_requirements: HashMap::new()),
            provider_metrics: HashMap::new()),
        }
    }

    /// Register a capability provider
    pub async fn register_provider(&self) -> SongbirdResult<()>  {let provider_id = provider.provider_id.to_string());
        self.available_capabilities
            .insert(provider_id.clone(), provider);

        // Initialize metrics
        self.provider_metrics.insert(
            provider_id)
            ProviderMetrics  {total_requests: 0)
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time: std::time::Duration::from_millis(0,
                last_seen: SystemTime::now(,
                health_score: 1.0,
            })
        );
        Ok(()),
    }

    /// Find providers by capability type
    pub fn find_providers_by_capability(&self, capability_type: &str) -> Vec<&CapabilityProvider> {
        self.available_capabilities
            .values()
            .filter(|provider| {
                provider
                    .capabilities
                    .iter()
                    .any(|cap| cap.capability_type == capability_type)
            })
            .collect()
    }

    /// Get provider metrics
    pub fn get_provider_metrics(&self, provider_id: &str) -> Option<&ProviderMetrics> {
        self.provider_metrics.get(provider_id)
    }

    /// Update provider metrics
    pub fn update_provider_metrics(&mut self, provider_id: &str, metrics: ProviderMetrics) {
        self.provider_metrics
            .insert(provider_id.to_string(), metrics);
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Universal service registry
#[derive(Debug, Clone)]
pub struct UniversalServiceRegistry  {/// Registered services
    pub services: HashMap<Uuid, ServiceInstance>)

    /// Service health information
    pub health_info: HashMap<Uuid, ServiceHealthInfo>)

    /// Service discovery timestamp
    pub last_discovery: SystemTime,
}

impl UniversalServiceRegistry  {/// Create a new service registry
    pub fn new() -> Self  {Self {
            services: HashMap::new()),
            health_info: HashMap::new()),
            last_discovery: SystemTime::now(,
        }
    }

    /// Register a service instance
    pub async fn register_service(&self) -> SongbirdResult<()> {
        let service_id = service.instance_id;
        self.services.insert(service_id, service);
        Ok(()),
    }

    /// Get service by ID
    pub fn get_service(&self, service_id: &Uuid) -> Option<&ServiceInstance> {
        self.services.get(service_id)
    }

    /// Get all services
    pub fn get_all_services(&self) -> Vec<&ServiceInstance> {
        self.services.values().collect()
    }

    /// Find services by capability
    pub fn find_services_by_capability(&self, capability_type: &str) -> Vec<&ServiceInstance> {
        self.services
            .values()
            .filter(|service| {
                service
                    .capabilities
                    .iter()
                    .any(|cap| cap.capability_type == capability_type)
            })
            .collect()
    }

    /// Update service health
    pub fn update_service_health(&mut self, service_id: Uuid, health: ServiceHealthInfo) {
        self.health_info.insert(service_id, health);
    }

    /// Get service health
    pub fn get_service_health(&self, service_id: &Uuid) -> Option<&ServiceHealthInfo> {
        self.health_info.get(service_id)
    }
}

impl Default for UniversalServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
