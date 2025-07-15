//! BYOB (Bring Your Own Biome) Coordinator
//!
//! This module coordinates BYOB deployments from biomeOS teams with Songbird's orchestration
//! capabilities. It enables teams to deploy independently while leveraging the shared Primal
//! ecosystem for network effects.

pub mod deployment;
pub mod integration;
pub mod monitoring;
pub mod types;
pub mod workspace;

use self::deployment::DeploymentManager;
use self::integration::IntegrationManager;
use self::monitoring::MonitoringManager;
use self::workspace::WorkspaceManager;

use super::{NestGateConfig, OrchestratorConfig};
use std::collections::HashMap;
use tracing::{error, info};

/// BYOB Coordinator - manages team deployments with Songbird orchestration
pub struct ByobCoordinator {
    /// Workspace manager
    workspace_manager: WorkspaceManager,
    /// Deployment manager
    deployment_manager: DeploymentManager,
    /// Monitoring manager
    monitoring_manager: MonitoringManager,
    /// Integration manager
    integration_manager: IntegrationManager,
}

impl ByobCoordinator {
    /// Create new BYOB coordinator
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            workspace_manager: WorkspaceManager::new(),
            deployment_manager: DeploymentManager::new(config),
            monitoring_manager: MonitoringManager::new(),
            integration_manager: IntegrationManager::new(),
        }
    }

    /// Configure with NestGate
    pub fn with_nestgate(mut self, config: NestGateConfig) -> Self {
        self.integration_manager = self.integration_manager.with_nestgate(config);
        self
    }

    /// Register team workspace for BYOB deployments
    pub async fn register_team_workspace(
        &self,
        team_id: String,
        resource_quota: types::TeamResourceQuota,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Registering team workspace: {}", team_id);
        self.workspace_manager
            .register_team_workspace(team_id, resource_quota)
            .await
    }

    /// Deploy biome
    pub async fn deploy_biome(
        &self,
        request: types::ByobDeploymentRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        info!("Deploying biome for team: {}", request.team_id);

        // Check team capacity
        let has_capacity = match self
            .workspace_manager
            .check_team_capacity(&request.team_id, &request.resource_quota)
            .await
        {
            Ok(capacity) => capacity,
            Err(e) => {
                error!("Failed to check team capacity: {}", e);
                return Err(format!("Failed to check team capacity: {e}").into());
            }
        };

        if !has_capacity {
            return Err("Team has insufficient capacity for new deployment".into());
        }

        // Deploy through deployment manager
        let deployment_id = self
            .deployment_manager
            .deploy_biome(request.clone())
            .await?;

        // Get deployment for workspace tracking
        if let Some(deployment) = self.deployment_manager.get_deployment(&deployment_id).await {
            // Add to workspace
            if let Err(e) = self
                .workspace_manager
                .add_deployment_to_workspace(&request.team_id, deployment.clone())
                .await
            {
                error!("Failed to add deployment to workspace: {}", e);
                return Err(format!("Failed to add deployment to workspace: {e}").into());
            }

            // Start monitoring
            if let Err(e) = self.monitoring_manager.start_monitoring(deployment).await {
                error!("Failed to start monitoring: {}", e);
                return Err(format!("Failed to start monitoring: {e}").into());
            }

            // Coordinate with primals
            if let Some(deployment) = self.deployment_manager.get_deployment(&deployment_id).await {
                if let Some(orchestrator) = &deployment.orchestrator {
                    if let Err(e) = self
                        .monitoring_manager
                        .coordinate_with_primals(orchestrator, &deployment_id)
                        .await
                    {
                        error!("Failed to coordinate with primals: {}", e);
                        return Err(format!("Failed to coordinate with primals: {e}").into());
                    }
                }
            }
        }

        Ok(deployment_id)
    }

    /// Get deployment status
    pub async fn get_deployment_status(
        &self,
        deployment_id: &str,
    ) -> Result<types::ByobDeploymentStatus, Box<dyn std::error::Error>> {
        self.deployment_manager
            .get_deployment_status(deployment_id)
            .await
    }

    /// List team deployments
    pub async fn list_team_deployments(
        &self,
        team_id: &str,
    ) -> Result<Vec<types::ByobDeployment>, Box<dyn std::error::Error>> {
        self.deployment_manager.list_team_deployments(team_id).await
    }

    /// Stop deployment
    pub async fn stop_deployment(
        &self,
        deployment_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping deployment: {}", deployment_id);

        // Get deployment info before stopping
        let deployment = self.deployment_manager.get_deployment(deployment_id).await;

        // Stop deployment
        self.deployment_manager
            .stop_deployment(deployment_id)
            .await?;

        // Remove from workspace if we have deployment info
        if let Some(deployment) = deployment {
            if let Err(e) = self
                .workspace_manager
                .remove_deployment_from_workspace(&deployment.team_id, deployment_id)
                .await
            {
                error!("Failed to remove deployment from workspace: {}", e);
                return Err(e);
            }
        }

        // Stop monitoring
        if let Err(e) = self.monitoring_manager.stop_monitoring(deployment_id).await {
            error!("Failed to stop monitoring: {}", e);
            return Err(e);
        }

        Ok(())
    }

    /// Storage operations are handled by the universal primal adapter system
    /// Storage provisioning, removal, and status checking are managed through
    /// primal discovery and coordination mechanisms

    /// Get team workspace
    pub async fn get_team_workspace(
        &self,
        team_id: &str,
    ) -> Result<Option<types::ByobTeamWorkspace>, Box<dyn std::error::Error + Send + Sync>> {
        self.workspace_manager.get_team_workspace(team_id).await
    }

    /// Get workspace statistics
    pub async fn get_workspace_stats(
        &self,
        team_id: &str,
    ) -> Result<workspace::WorkspaceStats, Box<dyn std::error::Error + Send + Sync>> {
        self.workspace_manager.get_workspace_stats(team_id).await
    }

    /// Add primal discovery endpoint
    pub async fn add_primal_discovery_endpoint(
        &self,
        primal_name: String,
        endpoint: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.integration_manager
            .add_primal_discovery_endpoint(primal_name, endpoint)
            .await
    }

    /// List primal discovery endpoints
    pub async fn list_primal_discovery_endpoints(&self) -> HashMap<String, String> {
        self.integration_manager
            .list_primal_discovery_endpoints()
            .await
    }

    /// Check deployment health
    pub async fn check_deployment_health(
        &self,
        deployment_id: &str,
    ) -> Result<monitoring::DeploymentHealth, Box<dyn std::error::Error + Send + Sync>> {
        self.monitoring_manager
            .check_deployment_health(deployment_id)
            .await
    }

    /// Get monitoring statistics
    pub async fn get_monitoring_stats(&self) -> monitoring::MonitoringStats {
        self.monitoring_manager.get_monitoring_stats().await
    }

    /// List all team workspaces
    pub async fn list_team_workspaces(
        &self,
    ) -> Result<Vec<types::ByobTeamWorkspace>, Box<dyn std::error::Error + Send + Sync>> {
        self.workspace_manager.list_team_workspaces().await
    }

    /// List all deployments
    pub async fn list_all_deployments(&self) -> Vec<types::ByobDeployment> {
        self.deployment_manager.list_all_deployments().await
    }

    /// Update primal coordination status
    pub async fn update_primal_coordination_status(
        &self,
        deployment_id: &str,
        primal_name: &str,
        status: types::CoordinationStatus,
        capabilities: Vec<String>,
    ) {
        self.monitoring_manager
            .update_primal_coordination_status(deployment_id, primal_name, status, capabilities)
            .await
    }
}

impl Clone for ByobCoordinator {
    fn clone(&self) -> Self {
        Self {
            workspace_manager: self.workspace_manager.clone(),
            deployment_manager: self.deployment_manager.clone(),
            monitoring_manager: self.monitoring_manager.clone(),
            integration_manager: self.integration_manager.clone(),
        }
    }
}

// Re-export types for convenience
pub use integration::PrimalInfo;
pub use monitoring::{DeploymentHealth, MonitoringStats, OverallHealth};
pub use types::*;
pub use workspace::WorkspaceStats;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_byob_coordinator_creation() -> Result<(), Box<dyn std::error::Error>> {
        let config = OrchestratorConfig::default();
        let coordinator = ByobCoordinator::new(config);

        // Test basic functionality
        assert!(coordinator.list_all_deployments().await.is_empty());

        Ok(())
    }
}
