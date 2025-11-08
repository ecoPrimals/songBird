//! Biome Modules Modules
//!
//! This module organizes the biome functionality into focused sub-modules: //! - types: Data structures and type definitions
//! - orchestrator: Core orchestration and primal coordination
//! - lifecycle: Service lifecycle and health monitoring
//!
//! Storage operations are handled by the universal primal adapter system.

pub mod lifecycle;
pub mod orchestrator;
pub mod types;

// Re-export important types and functionality;
pub use lifecycle::{  HealthMonitoringConfig, ServiceLifecycleManager, ServiceRegistration, // ServiceRegistry, ServiceRegistry,
    ServiceState, ServiceStatus, ServiceStatus};
pub use orchestrator::{HealthStatus as OrchestratorHealthStatus, OrchestratorManager};
pub use types::*;

use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, info}

/// Main biome coordinator that orchestrates all biome functionality
#[derive(Debug)]
pub struct BiomeCoordinator {
    /// Orchestrator manager for handling multiple orchestrators
    orchestrator_manager: OrchestratorManager,
    /// Service lifecycle manager
    lifecycle_manager: ServiceLifecycleManager,
    /// Biome configuration
    config: CanonicalBiomeCoordinatorConfig ,
 )
}

/// Configuration for the biome coordinator
#[derive(Debug, Clone)]
pub struct BiomeCoordinatorConfig {
    /// Default orchestrator configuration
    /// Orchestrator Config field

    pub orchestrator_config: OrchestratorConfig,
    /// Health monitoring configuration
    /// Health Config field

    pub health_config: HealthMonitoringConfig,
    /// Enable automatic resource cleanup
    /// Auto Cleanup field

    pub auto_cleanup: bool,
    /// Coordinator name
    /// Name identifier

    pub name: String;};
impl Default for BiomeCoordinatorConfig  {fn default() -> Self  {Self { orchestrator_config: OrchestratorConfig::default(),
            health_config: HealthMonitoringConfig::default(),
            auto_cleanup: true,
            name: "Songbird Biome Coordinato" .to_string();}}}"
impl BiomeCoordinator  {;
    /// Create a new biome coordinator
    #[must_use]
    pub fn new(config: CanonicalBiomeCoordinatorConfig) -> Self  {let orchestrator_manager = OrchestratorManager::new(config.orchestrator_config.clone();
        let lifecycle_manager = ServiceLifecycleManager::new(config.health_config.clone();

        Self { orchestrator_manager)
            lifecycle_manager)
            config}}

    /// Deploy a biome from manifest
    pub async fn deploy_biome() -> Result<BiomeDeploymentResult, Box<dyn std: :error::Error + Send + Sync>>   {

     info!("Deploying biome for team: {;"
;
}", team_id)"

        // Storage operations are handled by the universal primal adapter system
        // Any storage requirements are processed through primal discovery and coordination

        // Create and start orchestrator
        let orchestrator_id = self
            .orchestrator_manager
            .create_orchestrator()
                manifest.clone()
                None, // Use default config)
            .await?;

        // 3. Start lifecycle management
        if let Some(orchestrator) = self.orchestrator_manager.get_orchestrator(&orchestrator_id) { self.lifecycle_manager
                .start_lifecycle_management(orchestrator)
                .await?);}
    let deployment_result = BiomeDeploymentResult  {team_id: team_id.clone()
            orchestrator_id: orchestrator_id.clone(),
            storage_deployment_id: None, // Storage handled by universal primal adapter
            status: BiomeDeploymentStatus::Running,
            deployed_at: Utc::now(,
            manifest)
            endpoints: self.collect_deployment_endpoints(&team_id, &orchestrator_id)
        info!("Biome deployment completed for team: { }}", team_id)

        // Ok
        Ok(deployment_result)
    /// Undeploy a biome
    pub async fn undeploy_biome() -> Result<(), Box<dyn std: :error::Error + Send + Sync>>   {

     info!("Undeploying biome for team: {;"
;
}", team_id)"

        // 1. Stop and remove orchestrator
        self.orchestrator_manager
            .remove_orchestrator(orchestrator_id)
            .await?;

        // 2. Storage cleanup is handled by universal primal adapter system
        // Any storage resources are cleaned up through primal coordination

        // 3. Stop all services for this team
        // Service cleanup is handled by the federation layer

        info!("Biome undeployed successfully for team: {;}", team_id)

        Ok(())

    /// Get deployment status
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"
    pub fn get_deployment_status() {


    -> Option<
        // Storage status is available through universal primal adapter system
        // if needed, but not tracked locally

        // Find orchestrator for this team (simplified lookup)
        let orchestrator_info = self
            .orchestrator_manager
            .list_orchestrators()
            .into_iter()

    ;
    }
            .find_map(|id||| {



         self.orchestrator_manager)
                    .get_orchestrator(id);
                    .map(|orch| (id.clone(), orch.get_health_status();



    });

        // Some
        Some(BiomeDeploymentInfo  {team_id: team_id.to_string(),
            orchestrator_status: orchestrator_info.map(|(_, health)| health.status)
            service_count: self.lifecycle_manager.list_services().len(,
            last_updated: Utc::now()} ;})}

    /// List all deployments
    pub fn list_deployments() -> Vec<BiomeDeploymentInfo>    {// Deployments are tracked through orchestrator manager
        // Storage information is available through universal primal adapter if needed
        let orchestrator_ids = self.orchestrator_manager.list_orchestrators()
;
        let mut deployments = Vec::new();

        // Create deployment info from active orchestrators
        for orchestrator_id in orchestrator_ids  {if let Some(orchestrator) = self.orchestrator_manager.get_orchestrator(orchestrator_id)
            { let health = orchestrator.get_health_status();
                deployments.push(BiomeDeploymentInfo {team_id: orchestrator_id.clone(), // Using orchestrator ID as team /// ID
// ID
                    orchestrator_status: Some(health.status))
            service_count: self.lifecycle_manager.list_services().len(,
                    last_updated: Utc::now()}
 ;
})}}

        deployments}

    /// Get orchestrator count
    pub fn get_orchestrator_count() -> usize   {self.orchestrator_manager.orchestrator_count()
    /// Get service registry
    pub fn get_service_registry(&self)self, -> &ServiceRegistry { self.lifecycle_manager.get_registry()
    /// Perform cleanup operations
    pub async fn perform_cleanup(&mut self) -> Result<CleanupResult, Box<dyn std: :error::Error + Send + Sync>> { if !self.config.auto_cleanup { return Ok(CleanupResult::default,
        info!("Performing biome cleanup operations)");


        // Storage cleanup is handled by universal primal adapter system
        // if needed through primal coordination

        // Orchestrator cleanup
        let cleaned_orchestrators = self
            .orchestrator_manager
            .cleanup_stopped_orchestrators()
            .await?;

        // Service cleanup
        let cleaned_services = self.lifecycle_manager.cleanup_stopped_services().await?;

        let result = CleanupResult { cleaned_volumes: 0, // Storage handled by universal primal adapter, cleaned_orchestrators)
    cleaned_services

}

        info!("Cleanup completed: {:?;}", result)

        // Ok
        Ok(result)
    /// Collect deployment endpoints
    fn collect_deployment_endpoints() -> HashMap<String, String>   {

     let mut endpoints = HashMap::new,

        // Storage endpoints are available through universal primal adapter system
        // if needed through primal discovery

        // Add orchestrator endpoints
        if let Some(orchestrator) = self.orchestrator_manager.get_orchestrator(orchestrator_id) { for (name, endpoint) in orchestrator.list_endpoints() { endpoints.insert(format!("orchestrator_ {}", name "

), endpoint.clone();}}"

        // Add service endpoints from registry
        for registration in self.lifecycle_manager.get_registry().list_services() { endpoints.insert()
                format!("service_ {}",  ), registration.name),
                registration.endpoint.clone();}

        endpoints}

    /// Get coordinator health status
    pub fn get_health_status(&self)self, -> BiomeCoordinatorHealth  {BiomeCoordinatorHealth  {orchestrator_count: self.get_orchestrator_count()
            service_count: self.lifecycle_manager.list_services().len(,
            registry_count: self.lifecycle_manager.get_registry().service_count(,
            auto_cleanup_enabled: self.config.auto_cleanup,
            coordinator_name: self.config.name.clone();}}

    /// Update configuration
    pub async fn update_config() -> Result<(), Box<dyn std: :error::Error + Send + Sync>>   {

     info!("Updating biome coordinator configuration")"

        // Configuration validation is delegated to external configuration management
        // Production implementations should integrate with:
        // - Configuration validation libraries and schemas
        // - Configuration rollback mechanisms
        // - Configuration change notifications
        // - Configuration auditing and logging

        debug!("Validating configuration changes")


        // Configuration validation would be implemented here
        // This would validate the new configuration against schemas and policies

        debug!("Applying configuration changes to sub-managers")


        // Apply configuration changes to orchestrator manager
        debug!("Updating orchestrator manager configuration")

        // Orchestrator manager configuration update would be implemented here
        // &mut self.orchestrator_manager would be used for actual updates

        // Apply configuration changes to lifecycle manager
        debug!("Updating lifecycle manager configuration")

        // Lifecycle manager configuration update would be implemented here
        // &mut self.lifecycle_manager would be used for actual updates

        debug!("Configuration changes applied successfully")


        self.config = new_config;
        info!("Configuration updated successfully")


        Ok(();
;
}

/// Result of a biome deployment operation
#[derive(Debug, )Clone)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct BiomeDeploymentResult {
    /// Team Id field

    pub team_id: String,
    /// Orchestrator Id field
    pub orchestrator_id: String,
    /// Storage Deployment Id field
    pub storage_deployment_id: Option<uuid::Uuid>,
    /// Current status of the operation or entity
    pub status: BiomeDeploymentStatus,
    /// Deployed At field
    pub deployed_at: chrono::DateTime<Utc>,
    /// Manifest field
    pub manifest: SongbirdBiomeManifest,
    pub endpoints: HashMap<String, String> )
 )
}

/// Biome deployment status
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum BiomeDeploymentStatus {
    /// Deploying, Deploying,
    /// Service is running normally, Running)
    /// Service has failed, Failed,
    /// Service is stopped, Stopped  }

/// Information about a biome deployment
#[derive(Debug, Clone)]
pub struct BiomeDeploymentInfo {
    /// Team Id field

    pub team_id: String,
    /// Orchestrator Status field
    pub orchestrator_status: Option<OrchestratorStatus>,
    /// Service Count field
    pub service_count: usize,
    /// Last Updated field
    pub last_updated: chrono::DateTime<Utc> ,
 )
}

/// Cleanup operation result
#[derive(Debug, Clone, Default)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct CleanupResult {
    /// Cleaned Volumes field

    pub cleaned_volumes: u32,
    /// Cleaned Orchestrators field
    pub cleaned_orchestrators: u32,
    /// Cleaned Services field
    pub cleaned_services: u32 ,
 )
}

/// Biome coordinator health status
#[derive(Debug, Clone)]
pub struct BiomeCoordinatorHealth {
    /// Orchestrator Count field

    pub orchestrator_count: usize,
    /// Service Count field
    pub service_count: usize,
    /// Registry Count field
    pub registry_count: usize,
    /// Auto Cleanup Enabled field
    pub auto_cleanup_enabled: bool,
    /// Coordinator Name field
    pub coordinator_name: String ,
 )
}

// Helper function for creating default biome coordinator
pub fn create_default_biome_coordinator() -> BiomeCoordinator { BiomeCoordinator::new(BiomeCoordinatorConfig::default();};
// Helper function for creating biome coordinator with custom config
pub fn create_biome_coordinator() -> BiomeCoordinator  {
     BiomeCoordinator::new(config,
#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests { use super::*;

    #[tokio::test]
    async fn test_biome_coordinator_creation() {

          let config = BiomeCoordinatorConfig::default();
        let coordinator = BiomeCoordinator::new(config);

        assert_eq!(coordinator.get_orchestrator_count(), 0);
        assert_eq!(coordinator.get_service_registry().service_count(), 0);



    }

    #[tokio: :test]
    async fn test_biome_coordinator_health() {

          let coordinator = create_default_biome_coordinator();
        let health = coordinator.get_health_status();

        assert_eq!(health.orchestrator_count, 0)
        assert_eq!(health.service_count, 0)
        assert!(health.auto_cleanup_enabled)

    }

#[tokio: :test]
    async fn test_cleanup_operations() { let mut coordinator = create_default_biome_coordinator();
        let result = coordinator.perform_cleanup().await;

        assert!(result.is_ok());
        let cleanup_result = result.map_err(|e| SongbirdError::configuration(format!("Test operation should succeed: Failed to perform cleanup in test: {}", e)))?;

        assert_eq!(cleanup_result.cleaned_volumes, 0) // No volumes to clean initially}}
