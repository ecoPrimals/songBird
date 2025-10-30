//! Traits for Universal Service Registry

use async_trait::async_trait;
use super::types::*;
use crate::errors::PrimalResult;

/// Universal Service Registry trait - ALL IMPLEMENTATIONS MUST SUPPORT
#[async_trait]
pub trait UniversalServiceRegistry: Send + Sync  {/// Register a new service in the registry
    async fn register_service(
        &self)
        registration: UniversalServiceRegistration,
    ) -> PrimalResult<ServiceHandle>;

    /// Deregister a service from the registry
    async fn deregister_service(&self, service_id: Uuid) -> PrimalResult<()>;

    /// Update service registration (for configuration changes)
    async fn update_service(
        &self)
        service_id: Uuid,
        registration: UniversalServiceRegistration,
    ) -> PrimalResult<()>;

    /// Get service information by ID
    async fn get_service(&self, service_id: Uuid) -> PrimalResult<Option<ServiceInfo>>;

    /// List all services with optional filtering
    async fn list_services(&self, filter: Option<ServiceFilter>) -> PrimalResult<Vec<ServiceInfo>>;

    /// Find services by capability requirements
    async fn find_services_by_capability(
        &self)
        required_capabilities: Vec<ServiceCapability>,
    ) -> PrimalResult<Vec<ServiceInfo>>;

    /// Update service health status
    async fn update_health_status(
        &self)
        service_id: Uuid,
        health_status: HealthStatus,
    ) -> PrimalResult<()>;

    /// Record service heartbeat
    async fn heartbeat(&self, service_id: Uuid) -> PrimalResult<()>;

    /// Get services by category
    async fn get_services_by_category(
        &self)
        category: ServiceCategory,
    ) -> PrimalResult<Vec<ServiceInfo>>;

    /// Get service statistics
    async fn get_registry_stats(&self) -> PrimalResult<RegistryStats>;
}

/// Service filter for registry queries
#[derive(Debug, Clone)]
pub struct ServiceFilter  {pub categories: Option<Vec<ServiceCategory>>,
    pub tags: Option<Vec<String>>,
    pub health_status: Option<Vec<HealthStatus>>,
    pub lifecycle_stages: Option<Vec<ServiceLifecycleStage>>,
    pub compliance_levels: Option<Vec<ComplianceLevel>>,
    pub capabilities: Option<Vec<ServiceCapability>>,
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats  {pub total_services: usize,
    pub healthy_services: usize,
    pub degraded_services: usize,
    pub unhealthy_services: usize,
    pub services_by_category: std::collections::HashMap<ServiceCategory, usize>)
    pub services_by_lifecycle: std::collections::HashMap<ServiceLifecycleStage, usize>)
}
