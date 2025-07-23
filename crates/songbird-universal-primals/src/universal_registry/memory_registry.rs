//! In-memory implementation of Universal Service Registry

use async_trait::async_trait;
use chrono::Utc;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::traits::*;
use super::types::*;
use crate::errors::{PrimalError, PrimalResult};

/// In-memory service registry implementation
#[derive(Debug)]
pub struct MemoryServiceRegistry {
    services: Arc<RwLock<HashMap<Uuid, ServiceInfo>>>,
}

impl MemoryServiceRegistry {
    /// Create a new memory-based service registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UniversalServiceRegistry for MemoryServiceRegistry {
    async fn register_service(
        &self,
        registration: UniversalServiceRegistration,
    ) -> PrimalResult<ServiceHandle> {
        let mut services = self.services.write().await;

        let service_info = ServiceInfo {
            registration: registration.clone(),
            health_status: HealthStatus::Unknown,
            performance_metrics: HashMap::new(),
        };

        services.insert(registration.service_id, service_info);

        Ok(ServiceHandle {
            service_id: registration.service_id,
            last_heartbeat: Utc::now(),
        })
    }

    async fn deregister_service(&self, service_id: Uuid) -> PrimalResult<()> {
        let mut services = self.services.write().await;
        services.remove(&service_id);
        Ok(())
    }

    async fn update_service(
        &self,
        service_id: Uuid,
        registration: UniversalServiceRegistration,
    ) -> PrimalResult<()> {
        let mut services = self.services.write().await;

        if let Some(service_info) = services.get_mut(&service_id) {
            service_info.registration = registration;
            Ok(())
        } else {
            Err(PrimalError::service_not_found(service_id))
        }
    }

    async fn get_service(&self, service_id: Uuid) -> PrimalResult<Option<ServiceInfo>> {
        let services = self.services.read().await;
        Ok(services.get(&service_id).cloned())
    }

    async fn list_services(&self, filter: Option<ServiceFilter>) -> PrimalResult<Vec<ServiceInfo>> {
        let services = self.services.read().await;
        let mut result: Vec<ServiceInfo> = services.values().cloned().collect();

        if let Some(filter) = filter {
            result.retain(|service| {
                // Apply category filter
                if let Some(ref categories) = filter.categories {
                    if !categories.contains(&service.registration.metadata.category) {
                        return false;
                    }
                }

                // Apply health status filter
                if let Some(ref health_statuses) = filter.health_status {
                    if !health_statuses.contains(&service.health_status) {
                        return false;
                    }
                }

                // Apply lifecycle stage filter
                if let Some(ref lifecycle_stages) = filter.lifecycle_stages {
                    if !lifecycle_stages.contains(&service.registration.metadata.lifecycle_stage) {
                        return false;
                    }
                }

                // Apply compliance level filter
                if let Some(ref compliance_levels) = filter.compliance_levels {
                    if !compliance_levels.contains(&service.registration.metadata.compliance_level)
                    {
                        return false;
                    }
                }

                true
            });
        }

        Ok(result)
    }

    async fn find_services_by_capability(
        &self,
        required_capabilities: Vec<ServiceCapability>,
    ) -> PrimalResult<Vec<ServiceInfo>> {
        let services = self.services.read().await;
        let mut matching_services = Vec::new();

        for service_info in services.values() {
            let mut matches = true;

            for required_cap in &required_capabilities {
                if !service_info
                    .registration
                    .capabilities
                    .contains(required_cap)
                {
                    matches = false;
                    break;
                }
            }

            if matches {
                matching_services.push(service_info.clone());
            }
        }

        // Randomize order for load balancing
        let mut rng = rand::thread_rng();
        matching_services.shuffle(&mut rng);

        Ok(matching_services)
    }

    async fn update_health_status(
        &self,
        service_id: Uuid,
        health_status: HealthStatus,
    ) -> PrimalResult<()> {
        let mut services = self.services.write().await;

        if let Some(service_info) = services.get_mut(&service_id) {
            service_info.health_status = health_status;
            Ok(())
        } else {
            Err(PrimalError::service_not_found(service_id))
        }
    }

    async fn heartbeat(&self, service_id: Uuid) -> PrimalResult<()> {
        // In a real implementation, this would update last_heartbeat timestamp
        // For now, just verify the service exists
        let services = self.services.read().await;

        if services.contains_key(&service_id) {
            Ok(())
        } else {
            Err(PrimalError::service_not_found(service_id))
        }
    }

    async fn get_services_by_category(
        &self,
        category: ServiceCategory,
    ) -> PrimalResult<Vec<ServiceInfo>> {
        let services = self.services.read().await;
        let result: Vec<ServiceInfo> = services
            .values()
            .filter(|service| service.registration.metadata.category == category)
            .cloned()
            .collect();

        Ok(result)
    }

    async fn get_registry_stats(&self) -> PrimalResult<RegistryStats> {
        let services = self.services.read().await;

        let total_services = services.len();
        let mut healthy_services = 0;
        let mut degraded_services = 0;
        let mut unhealthy_services = 0;
        let mut services_by_category = HashMap::new();
        let mut services_by_lifecycle = HashMap::new();

        for service_info in services.values() {
            match service_info.health_status {
                HealthStatus::Healthy => healthy_services += 1,
                HealthStatus::Degraded => degraded_services += 1,
                HealthStatus::Unhealthy => unhealthy_services += 1,
                HealthStatus::Unknown => {} // Don't count unknown
            }

            let category = &service_info.registration.metadata.category;
            *services_by_category.entry(category.clone()).or_insert(0) += 1;

            let lifecycle = &service_info.registration.metadata.lifecycle_stage;
            *services_by_lifecycle.entry(lifecycle.clone()).or_insert(0) += 1;
        }

        Ok(RegistryStats {
            total_services,
            healthy_services,
            degraded_services,
            unhealthy_services,
            services_by_category,
            services_by_lifecycle,
        })
    }
}
