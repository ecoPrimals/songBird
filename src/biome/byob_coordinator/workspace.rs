//! BYOB Workspace Management
//!
//! Handles team workspace registration, management, and resource allocation.

use super::types::{ByobDeployment, ByobTeamWorkspace, TeamResourceQuota};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Team workspace manager
pub struct WorkspaceManager {
    /// Team workspaces
    workspaces: Arc<RwLock<HashMap<String, ByobTeamWorkspace>>>,
}

impl WorkspaceManager {
    /// Create new workspace manager
    pub fn new() -> Self {
        Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register team workspace for BYOB deployments
    pub async fn register_team_workspace(
        &self,
        team_id: String,
        resource_quota: TeamResourceQuota,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut workspaces = self.workspaces.write().await;

        // Check if workspace already exists
        if workspaces.contains_key(&team_id) {
            warn!("Team workspace already exists for team: {}", team_id);
            return Ok(());
        }

        // Create new workspace
        let workspace = ByobTeamWorkspace {
            team_id: team_id.clone(),
            active_deployments: HashMap::new(),
            resource_quota,
            primal_endpoints: HashMap::new(),
        };

        workspaces.insert(team_id.clone(), workspace);
        info!("Registered new team workspace: {}", team_id);

        Ok(())
    }

    /// Get team workspace
    pub async fn get_team_workspace(
        &self,
        team_id: &str,
    ) -> Result<Option<ByobTeamWorkspace>, Box<dyn std::error::Error + Send + Sync>> {
        let workspaces = self.workspaces.read().await;
        Ok(workspaces.get(team_id).cloned())
    }

    /// Update team workspace
    pub async fn update_team_workspace(
        &self,
        team_id: &str,
        workspace: ByobTeamWorkspace,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut workspaces = self.workspaces.write().await;

        if workspaces.contains_key(team_id) {
            workspaces.insert(team_id.to_string(), workspace);
            info!("Updated team workspace: {}", team_id);
        } else {
            warn!("Attempting to update non-existent workspace: {}", team_id);
        }

        Ok(())
    }

    /// Add deployment to team workspace
    pub async fn add_deployment_to_workspace(
        &self,
        team_id: &str,
        deployment: ByobDeployment,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut workspaces = self.workspaces.write().await;

        if let Some(workspace) = workspaces.get_mut(team_id) {
            workspace
                .active_deployments
                .insert(deployment.deployment_id.clone(), deployment);
            info!("Added deployment to workspace for team: {}", team_id);
        } else {
            error!("Team workspace not found: {}", team_id);
            return Err("Team workspace not found".into());
        }

        Ok(())
    }

    /// Remove deployment from team workspace
    pub async fn remove_deployment_from_workspace(
        &self,
        team_id: &str,
        deployment_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut workspaces = self.workspaces.write().await;

        if let Some(workspace) = workspaces.get_mut(team_id) {
            workspace.active_deployments.remove(deployment_id);
            info!(
                "Removed deployment {} from workspace for team: {}",
                deployment_id, team_id
            );
        } else {
            error!("Team workspace not found: {}", team_id);
            return Err("Team workspace not found".into());
        }

        Ok(())
    }

    /// Check if team has capacity for new deployment
    pub async fn check_team_capacity(
        &self,
        team_id: &str,
        required_quota: &TeamResourceQuota,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let workspaces = self.workspaces.read().await;

        if let Some(workspace) = workspaces.get(team_id) {
            let current_deployments = workspace.active_deployments.len() as u32;

            // Check deployment count limit
            if current_deployments >= workspace.resource_quota.max_deployments {
                return Ok(false);
            }

            // Check resource limits (simplified - in real implementation would sum all active deployments)
            let has_capacity = required_quota.max_cpu_cores
                <= workspace.resource_quota.max_cpu_cores
                && required_quota.max_memory_bytes <= workspace.resource_quota.max_memory_bytes
                && required_quota.max_storage_bytes <= workspace.resource_quota.max_storage_bytes
                && required_quota.max_network_bandwidth_mbps
                    <= workspace.resource_quota.max_network_bandwidth_mbps;

            Ok(has_capacity)
        } else {
            error!("Team workspace not found: {}", team_id);
            Err("Team workspace not found".into())
        }
    }

    /// List all team workspaces
    pub async fn list_team_workspaces(
        &self,
    ) -> Result<Vec<ByobTeamWorkspace>, Box<dyn std::error::Error + Send + Sync>> {
        let workspaces = self.workspaces.read().await;
        Ok(workspaces.values().cloned().collect())
    }

    /// Get workspace statistics
    pub async fn get_workspace_stats(
        &self,
        team_id: &str,
    ) -> Result<WorkspaceStats, Box<dyn std::error::Error + Send + Sync>> {
        let workspaces = self.workspaces.read().await;

        if let Some(workspace) = workspaces.get(team_id) {
            let stats = WorkspaceStats {
                team_id: team_id.to_string(),
                active_deployments: workspace.active_deployments.len(),
                total_quota: workspace.resource_quota.clone(),
                primal_endpoints: workspace.primal_endpoints.len(),
            };
            Ok(stats)
        } else {
            error!("Team workspace not found: {}", team_id);
            Err("Team workspace not found".into())
        }
    }
}

/// Workspace statistics
#[derive(Debug, Clone)]
pub struct WorkspaceStats {
    pub team_id: String,
    pub active_deployments: usize,
    pub total_quota: TeamResourceQuota,
    pub primal_endpoints: usize,
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WorkspaceManager {
    fn clone(&self) -> Self {
        Self {
            workspaces: Arc::clone(&self.workspaces),
        }
    }
}
