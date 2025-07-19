//! Universal load balancing types and patterns

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{HealthStatus, ServiceMetrics, ServiceStatus, UniversalServiceRegistration};

/// Universal registered service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService {
    pub service_id: String,
    pub registration: UniversalServiceRegistration,
    pub status: ServiceStatus,
    pub health_status: HealthStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub metrics: ServiceMetrics,
    pub capabilities: Vec<crate::ServiceCapability>,
}

impl RegisteredService {
    /// Create a placeholder service for testing
    pub fn placeholder(service_id: String) -> Self {
        use crate::{
            HealthCheckConfig, PrimalType, ResourceSpec, SecurityConfig, ServiceIdentification,
        };

        Self {
            service_id: service_id.clone(),
            registration: UniversalServiceRegistration {
                service: ServiceIdentification {
                    name: service_id.clone(),
                    version: "1.0.0".to_string(),
                    description: "Placeholder service".to_string(),
                    primal_type: PrimalType::new("placeholder"),
                    instance_id: service_id.clone(),
                },
                primal_type: PrimalType::new("placeholder"),
                biome_id: None,
                capabilities: Vec::new(),
                endpoints: Vec::new(),
                resource_requirements: ResourceSpec {
                    cpu_cores: None,
                    memory_mb: None,
                    disk_mb: None,
                    network_bandwidth_mbps: None,
                    gpu_count: None,
                    custom_resources: HashMap::new(),
                },
                security_config: SecurityConfig {
                    auth_required: false,
                    auth_methods: Vec::new(),
                    encryption_required: false,
                    security_level: crate::SecurityLevel::Public,
                    custom_security: HashMap::new(),
                },
                health_check: HealthCheckConfig {
                    enabled: true,
                    interval_seconds: 30,
                    timeout_seconds: 5,
                    failure_threshold: 3,
                    success_threshold: 2,
                    custom_checks: HashMap::new(),
                },
                metadata: HashMap::new(),
            },
            status: ServiceStatus::Active,
            health_status: HealthStatus::Healthy,
            last_heartbeat: chrono::Utc::now(),
            metrics: ServiceMetrics::default(),
            capabilities: Vec::new(),
        }
    }
}

/// Universal load balancing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStatistics {
    pub total_requests: u64,
    pub strategy_usage: HashMap<String, u64>,
    pub instance_distribution: HashMap<String, u64>,
    pub average_response_time: f64,
    pub error_rates: HashMap<String, f64>,
}

/// Universal capability matcher
pub struct CapabilityMatcher;

impl Default for CapabilityMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityMatcher {
    pub fn new() -> Self {
        Self
    }

    pub fn filter_by_requirements(
        &self,
        services: &[RegisteredService],
        requirements: &[crate::CapabilityRequirement],
    ) -> Result<Vec<RegisteredService>, crate::LoadBalancingError> {
        let mut filtered = Vec::new();

        for service in services {
            let mut matches_all = true;

            for requirement in requirements {
                let mut matches_requirement = false;

                for capability in &service.capabilities {
                    if requirement.is_satisfied_by(capability) {
                        matches_requirement = true;
                        break;
                    }
                }

                if !matches_requirement {
                    matches_all = false;
                    break;
                }
            }

            if matches_all {
                filtered.push(service.clone());
            }
        }

        Ok(filtered)
    }
}
