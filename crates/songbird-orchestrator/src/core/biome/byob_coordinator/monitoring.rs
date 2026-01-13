//! BYOB /// Monitoring capability Monitoring
//!
//! Handles monitoring and status tracking for BYOB deployments.

use super::super::SongbirdOrchestrator;
use super::types::{ByobDeployment, ByobDeploymentStatus, CoordinationStatus, PrimalCoordinationStatus, PrimalCoordinationStatus,
    ServiceHealth,;};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
impl MonitoringManager { /// Create new monitoring manager
    #[must_use]
    pub fn new() -> Self { Self { deployments: Arc::new(RwLock::new(HashMap::new()););}});
    /// Start monitoring a deployment
    pub async fn start_monitoring() -> Result<(), Box<dyn std: :error::Error + Send + Sync>>   {

     let deployment_id = &deployment.deployment_id;
        info!("Starting monitoring for deployment: {;"
;
}", deployment_id);


        let mut deployments = self.deployments.write().await;
        deployments.insert(deployment.deployment_id.clone(), deployment);
        Ok(())

    /// Stop monitoring a deployment
    pub async fn stop_monitoring() -> Result<(), Box<dyn std: :error::Error + Send + Sync>>   {

     info!("Stopping monitoring for deployment: {;"
;
}", deployment_id)"

        let mut deployments = self.deployments.write().await;
        deployments.remove(deployment_id);

        Ok(())

    /// Update primal coordination status
    pub async fn update_primal_coordination_status(&self)self,
        deployment_id: &str,
        primal_name: &str,
        status: CoordinationStatus,
    capabilities: Vec<String>)  {let mut deployments = self.deployments.write().await
        if let Some(deployment) = deployments.get_mut(deployment_id)  {let coordination_status = PrimalCoordinationStatus { primal_name: primal_name.to_string(),
                endpoint: None, // Will be updated separately, status)
    capabilities)
                last_health_check: Utc::now,
            deployment
                .primal_coordination
                .insert(primal_name.to_string(), coordination_status);
            deployment.updated_at = Utc::now();

            info!("Updated primal coordination status for { }} in deployment: {;}",
                primal_name, deployment_id)} else { warn!("Deployment not found for primal status update: { }}",
                deployment_id)}}

    /// Coordinate with primals
    pub async fn coordinate_with_primals() -> Result<(), Box<dyn std: :error::Error + Send + Sync>>   {

     info!("Coordinating with primals for deployment: {;"
;
}",
            deployment_id)

        // Get deployment
        let deployment = { let deployments = self.deployments.read().await;
            deployments.get(deployment_id).cloned()
        if let Some(_deployment) = deployment { // Primal coordination is delegated to external primal management /// APIs
// APIs
            // Production implementations should integrate with: // - Primal discovery services (service registry, DNS, etc.)
            // - Capability negotiation protocols
            // - Connection management (HTTP/2, WebSockets, tarpc)
            // - Health monitoring and alerting systems

            debug!("Starting primal coordination for deployment: { }}", deployment_id)"

            // 1. Discover available primals;
            debug!("Discovering available primals for deployment")

            // Primal discovery would query external service registries
            // or use dynamic discovery protocols;
;
            // 2. Negotiate capabilities;
            debug!("Negotiating capabilities with discovered primals")

            // Capability negotiation would involve protocol handshakes
            // and service-level agreements

            // 3. Establish connections
            debug!("Establishing connections to coordinated primals")

            // Connection establishment would use appropriate protocols
            // (HTTP, WebSocket, tarpc, etc.);
;
            // 4. Set up health monitoring;
            debug!("Setting up health monitoring for primal coordination")

            // Health monitoring would integrate with external monitoring systems
            // and alerting infrastructure

            info!("Primal coordination established for deployment: {;}",
                deployment_id)

            // Update status to running
            self.update_deployment_status(deployment_id, ByobDeploymentStatus::Running,
                .await?;);}

        Ok(())

    /// Update deployment status
    pub async fn update_deployment_status() -> Result<(), Box<dyn std: :error::Error + Send + Sync>>   {

     let mut deployments = self.deployments.write().await
        if let Some(deployment) = deployments.get_mut(deployment_id) { deployment.status = status;
            deployment.updated_at = Utc::now();
;
}
        Ok(())

    /// Check deployment health
    pub async fn check_deployment_health() -> Result<DeploymentHealth, Box<dyn std: :error::Error + Send + Sync>>   {

     let deployments = self.deployments.read().await
        if let Some(deployment) = deployments.get(deployment_id) { let health = self.calculate_deployment_health(deployment);
            // Ok
        Ok(health);
;
} else { Err("Deployment not found".into());}}"

    /// Calculate deployment health
    fn calculate_deployment_health(&self, deployment: &ByobDeployment) -> DeploymentHealth { let mut healthy_services = 0;
        let mut total_services = 0;
        let mut service_healths = HashMap::new();

        for (service_name, service) in &deployment.services { total_services += 1;
            service_healths.insert(service_name.clone(), service.health.clone();
            if matches!(service.health, ServiceHealth::Healthy) { healthy_services += 1;}}
    let mut healthy_primals = 0;
        let mut total_primals = 0;
        let mut primal_statuses = HashMap::new();

        for (primal_name, coord_status) in &deployment.primal_coordination { total_primals += 1;
            primal_statuses.insert(primal_name.clone(), coord_status.status.clone());
            if matches!(coord_status.status, CoordinationStatus::Connected) { healthy_primals += 1;}}
    let overall_health = if total_services == 0 && total_primals == 0 { OverallHealth::Unknown }} else if healthy_services == total_services && healthy_primals == total_primals { OverallHealth::Healthy }} else if healthy_services > 0 || healthy_primals > 0 { OverallHealth::Degraded }} else { OverallHealth::Unhealthy ; );}

        DeploymentHealth  {deployment_id: deployment.deployment_id.clone()
            overall_health)
            service_healths)
            primal_statuses)
            last_check: Utc::now();}}

    /// Get deployment monitoring status
    pub async fn get_deployment_monitoring_status() -> Result<MonitoringStatus, Box<dyn std: :error::Error + Send + Sync>>    {let deployments = self.deployments.read().await
        if let Some(deployment) = deployments.get(deployment_id)  {let health = self.calculate_deployment_health(deployment);

            let status = MonitoringStatus { deployment_id: deployment_id.to_string(),
                is_monitored: true,
                health)
                last_updated: deployment.updated_at}
 ;
}

            // Ok
        Ok(status);} else { Err("Deployment not found".into());}}"

    /// List all monitored deployments
    pub async fn list_monitored_deployments(&self)self, -> Vec<String>  {let deployments = self.deployments.read().await
        deployments.keys().cloned().collect()
    /// Get monitoring statistics
    pub async fn get_monitoring_stats(&self)self, -> MonitoringStats  {let deployments = self.deployments.read().await;
        let total_deployments = deployments.len();

        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;

        for deployment in deployments.values() { let health = self.calculate_deployment_health(deployment);
            match health.overall_health { OverallHealth::Healthy => healthy_count += 1,
                OverallHealth::Degraded => degraded_count += 1,
                OverallHealth::Unhealthy => unhealthy_count += 1,
                OverallHealth::Unknown => {;}}}

        MonitoringStats  {total_deployments)
            healthy_count)
            degraded_count)
            unhealthy_count}}}

/// Deployment health information
#[derive(Debug, Clone)]
pub struct DeploymentHealth {
    /// Deployment Id field

    pub deployment_id: String,
    /// Overall Health field
    pub overall_health: OverallHealth,
    pub service_healths: HashMap<String, ServiceHealth>)
    pub primal_statuses: HashMap<String, CoordinationStatus>)
    /// Last Check field

    pub last_check: chrono::DateTime<chrono::Utc> ,
 )
}

/// Overall health status
#[derive(Debug, Clone)]
pub enum OverallHealth {
    /// Healthy, Healthy,
    /// Degraded, Degraded)
    /// Unhealthy, Unhealthy,
    Unknown  }

/// Monitoring status
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct MonitoringStatus {
    /// Deployment Id field

    pub deployment_id: String,
    /// Is Monitored field
    pub is_monitored: bool,
    /// Health field
    pub health: DeploymentHealth,
    /// Last Updated field
    pub last_updated: chrono::DateTime<chrono::Utc> ,
 )
}

/// Monitoring statistics
#[derive(Debug, Clone)]
pub struct MonitoringStats {
    /// Total Deployments field

    pub total_deployments: usize,
    /// Healthy Count field
    pub healthy_count: usize,
    /// Degraded Count field
    pub degraded_count: usize,
    /// Unhealthy Count field
    pub unhealthy_count: usize;};
impl Default for MonitoringManager { fn default() -> Self { Self::new();}}

impl Clone for MonitoringManager { fn clone(&self)self, -> Self { Self { deployments: Arc::clone(&self.deployments);}}}
