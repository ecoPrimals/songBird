use serde::{Deserialize, Serialize};
use serde_json;
use songbird_config::UniversalHealthStatus;
use songbird_errors::{SongbirdError, service_error, SongbirdResult};
// Universal Capability Adapter System
//
// This module provides the capability adapter system that can work with
// any service provider based on capabilities, not hardcoded names.

use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Universal capability-based adapters
/// These adapters discover and route to services based on capabilities, not hardcoded primal names
pub mod ai;
pub mod compute;
pub mod context;
pub mod primal_integration;
pub mod routing;
pub mod security;
pub mod storage;
pub mod types;

// Universal adapter interfaces - capability-based, not primal-specific
pub use ai::AICapabilityAdapter;
pub use compute::UniversalComputeProvider;
pub use context::AdapterContext;
pub use security::SecurityCapabilityAdapter;
pub use storage::StorageAdapter;

/// Universal Capability Adapter - routes to any service providing the required capabilities
use crate::types::UniversalServiceRegistry;
use std::sync::Arc;

pub struct UniversalCapabilityAdapter {
    registry: Arc<UniversalServiceRegistry>,
}

impl UniversalCapabilityAdapter {
    pub fn new(registry: Arc<UniversalServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Route a request to the best available service for the given capability
    pub async fn route_capability_request(&self) -> SongbirdResult<serde_json::Value> {
        // Find the best service for this capability
        if let Some(service) = self
            .registry
            .get_best_service_for_capability(capability_type, capability_name)
        {
            // Route the request to the selected service
            self.send_request_to_service(&service, request_data).await
        } else {
            Err(songbird_errors::service_error!(format!(
                "No healthy service found for capability {}:{}",
                capability_type, capability_name
            )))
        }
    }

    async fn send_request_to_service(&self) -> SongbirdResult<serde_json::Value> {
        // Implementation would send HTTP/gRPC/etc request to service
        // For now, return success to demonstrate the pattern

        let response_data = serde_json::json!({
            "success": true,
            "service_id": service.service_id,
            "service_name": service.name,
            "data": request_data
        });

        Ok(songbird_errors::evolved_success(success(response_data)))
    }
}

/// Universal capability types - no provider assumptions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    Security {
        encryption: bool,
        authentication: bool,
        authorization: bool,
    },
    Storage {
        persistence: bool,
        caching: bool,
        distributed: bool,
    },
    Compute {
        cpu_intensive: bool,
        memory_intensive: bool,
        gpu_capable: bool,
    },
    AI {
        machine_learning: bool,
        natural_language: bool,
        computer_vision: bool,
    },
    Gaming {
        protocol_translation: bool,
        session_management: bool,
        latency_optimization: bool,
    },
    Custom(String),
}

/// Any service can register with capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProvider {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<CapabilityType>,
    pub health_status: UniversalHealthStatus,
    pub performance_metrics: PerformanceMetrics,
    pub priority: u8,
}

/// Health status - canonical re-export
/// Using ecosystem standard UniversalHealthStatus for consistency
pub use crate::UniversalHealthStatus as HealthStatus;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
    pub current_load: f64,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

/// Pure capability-based routing
#[derive(Debug)]
pub struct CapabilityRouter {
    providers: HashMap<Uuid, CapabilityProvider>,
    capability_index: HashMap<CapabilityType, Vec<Uuid>>,
}

impl Default for CapabilityRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityRouter {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            capability_index: HashMap::new(),
        }
    }

    /// Register any provider with any capabilities
    pub async fn register_provider(&self) -> SongbirdResult<()> {
        debug!(
            "Registering provider '{}' with {} capabilities",
            provider.name,
            provider.capabilities.len()
        );

        // Update capability index
        for capability in &provider.capabilities {
            self.capability_index
                .entry(capability.clone())
                .or_default()
                .push(provider.id);
        }

        self.providers.insert(provider.id, provider);
        Ok(songbird_errors::success(()))
    }

    /// Find all providers that support a specific capability
    pub async fn find_providers_with_capability(&self) -> SongbirdResult<()> {let providers = &self.providers;
        let filtered: Vec<&CapabilityProvider> = providers
            .values()
            .filter(|provider| provider.capabilities.contains(capability))
            .collect();

        Ok(songbird_errors::evolved_success(filtered))
    }

    /// Route a capability request to available providers
    pub async fn route_capability(&self) -> SongbirdResult<CapabilityResponse> {
        // Find providers that support this capability
        let providers = self
            .find_providers_with_capability(&request.capability)
            .await
            .map_err(|e| service_error!(format!("Failed to find providers: {e}")))?;

        if providers.is_empty() {
            return Err(service_error!(format!(
                "No providers available for capability {:?}",
                request.capability
            )));
        }

        info!("✅ Found {} providers for capability", providers.len());
        Ok(success(CapabilityResponse {
            request_id: uuid::Uuid::new_v4(), // Generate a new ID since CapabilityRequest doesn't have one
            provider_id: uuid::Uuid::new_v4(), // Use a generic provider ID for routing
            success: true,
            data: Some(serde_json::json!({
                "providers_found": providers.len(),
                "capability": request.capability
            })),
            error: None,
            processing_time_ms: 0, // Could be calculated if needed
        }))
    }

    /// Route to the best provider for a capability
    pub async fn route_to_best_provider(&self) -> SongbirdResult<CapabilityProvider> {
        let providers = self
            .find_providers_with_capability(capability)
            .await
            .map_err(|e| service_error!(format!("Failed to find providers: {e}")))?;

        if providers.is_empty() {
            return Err(service_error!(format!(
                "No providers available for capability {:?}",
                capability
            )));
        }

        // Return the first available provider for now
        // In a real implementation, this would select based on requirements
        let best_provider = providers.into_iter().next().ok_or_else(|| {
            service_error!(format!(
                "No providers available for capability {:?}",
                capability
            ))
        })?;

        Ok(success(best_provider.clone()))
    }

    #[allow(dead_code)]
    fn meets_requirements(
        &self,
        provider: &CapabilityProvider,
        requirements: &ServiceRequirements,
    ) -> bool {
        // Check if provider meets minimum requirements
        provider.performance_metrics.success_rate >= requirements.min_success_rate
            && provider.performance_metrics.avg_response_time_ms
                <= requirements.max_response_time_ms
            && provider.performance_metrics.current_load <= requirements.max_load
    }

    #[allow(dead_code)]
    fn calculate_provider_score(&self, provider: &CapabilityProvider) -> f64 {
        // Lower score is better
        let health_penalty = match provider.health_status {
            UniversalHealthStatus::Healthy => 0.0,
            UniversalHealthStatus::Degraded => 25.0,
            UniversalHealthStatus::Unhealthy => 1000.0,
            UniversalHealthStatus::Unknown => 10000.0,
            UniversalHealthStatus::Maintenance => 500.0,
            UniversalHealthStatus::Starting => 100.0,
            UniversalHealthStatus::Stopping => 2000.0,
            UniversalHealthStatus::Failed => 5000.0,
        };

        let performance_score = provider.performance_metrics.avg_response_time_ms
            + (100.0 - provider.performance_metrics.success_rate) * 10.0
            + provider.performance_metrics.current_load * 5.0;

        health_penalty + performance_score
    }

    pub async fn get_system_info(&self) -> SongbirdResult<()> {
        info!("📊 Universal capability system operational");
        Ok(songbird_errors::success(()))
    }
}

/// Service requirements for capability matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequirements {
    pub min_success_rate: f64,
    pub max_response_time_ms: f64,
    pub max_load: f64,
    pub required_features: Vec<String>,
}

impl Default for ServiceRequirements {
    fn default() -> Self {
        Self {
            min_success_rate: 90.0,       // 90% minimum success rate
            max_response_time_ms: 1000.0, // 1 second max response time
            max_load: 80.0,               // 80% max load
            required_features: Vec::new(),
        }
    }
}

/// Request to execute a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub capability: CapabilityType,
    pub operation: String,
    pub parameters: serde_json::Value,
    pub requirements: ServiceRequirements,
    pub timeout_ms: u64,
    pub priority: u8,
    pub metadata: std::collections::HashMap<String, String>,
}

impl CapabilityRequest {
    pub fn new(capability: CapabilityType, operation: &str, parameters: serde_json::Value) -> Self {
        Self {
            capability,
            operation: operation.to_string(),
            parameters,
            requirements: ServiceRequirements {
                min_success_rate: 80.0,
                max_response_time_ms: 10000.0,
                max_load: 90.0,
                required_features: Vec::new(),
            },
            timeout_ms: 10000,
            priority: 128, // Medium priority
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set specific requirements for this request
    pub fn with_requirements(mut self, requirements: ServiceRequirements) -> Self {
        self.timeout_ms = requirements.max_response_time_ms as u64;
        self.requirements = requirements;
        self
    }

    /// Set timeout for this request
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set priority for this request (lower number = higher priority)
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata to this request
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Universal capability response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResponse {
    pub request_id: Uuid,
    pub provider_id: Uuid,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub processing_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics {
            avg_response_time_ms: 150.0,
            success_rate: 0.95,
            current_load: 0.7,
            last_updated: Some(Utc::now()),
        };

        assert_eq!(metrics.avg_response_time_ms, 150.0);
        assert_eq!(metrics.success_rate, 0.95);
        assert_eq!(metrics.current_load, 0.7);
    }

    #[test]
    fn test_capability_provider_creation() {
        let provider = CapabilityProvider {
            id: Uuid::new_v4(),
            name: "Test Provider".to_string(),
            capabilities: vec![CapabilityType::Compute {
                cpu_intensive: true,
                memory_intensive: false,
                gpu_capable: false,
            }],
            endpoint: "http://localhost:{}".to_string(),
            health_status: HealthStatus::Healthy,
            performance_metrics: PerformanceMetrics {
                avg_response_time_ms: 0.0,
                success_rate: 1.0,
                current_load: 0.0,
                last_updated: Some(Utc::now()),
            },
            priority: 5,
        };

        // Fix assertion: compare UUID to UUID, not to string
        assert_eq!(provider.name, "Test Provider");
        assert_eq!(provider.priority, 5);
        assert_eq!(provider.capabilities.len(), 1);
        assert!(matches!(
            provider.capabilities[0],
            CapabilityType::Compute { .. }
        ));
    }
}
