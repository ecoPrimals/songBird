//! BYOB (Bring Your Own Biome) Coordinator
//!
//! This module coordinates BYOB deployments from biomeOS teams with Songbird's orchestration
//! capabilities. It enables teams to deploy independently while leveraging the shared Primal
//! ecosystem for network effects.

use super::{
    SongbirdBiomeManifest, SongbirdOrchestrator, ByobError, NestGateConfig, 
    TeamStorageRequirements, StorageDeploymentResponse, StorageEndpoint, VolumeMount, StorageTier, StorageUsage, StorageStatus,
    DeploymentResult, DeploymentStatus, BiomeMetadata, OrchestratorConfig
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

use std::time::Duration;
use chrono::Utc;

/// BYOB deployment request from biomeOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobDeploymentRequest {
    pub deployment_id: String,
    pub team_id: String,
    pub manifest: SongbirdBiomeManifest,
    pub resource_quota: TeamResourceQuota,
}

/// Team resource quota for BYOB isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamResourceQuota {
    pub max_cpu_cores: f64,
    pub max_memory_bytes: u64,
    pub max_storage_bytes: u64,
    pub max_network_bandwidth_mbps: u64,
    pub max_deployments: u32,
}

/// BYOB deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ByobDeploymentStatus {
    Pending,
    Orchestrating,
    CoordinatingPrimals,
    Running,
    Scaling,
    Stopping,
    Stopped,
    Failed(String),
}

/// BYOB team workspace in Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobTeamWorkspace {
    pub team_id: String,
    pub active_deployments: HashMap<String, ByobDeployment>,
    pub resource_quota: TeamResourceQuota,
    pub primal_endpoints: HashMap<String, String>,
}

/// BYOB deployment instance managed by Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByobDeployment {
    pub deployment_id: String,
    pub team_id: String,
    pub orchestrator: Option<SongbirdOrchestrator>,
    pub status: ByobDeploymentStatus,
    pub services: HashMap<String, ServiceStatus>,
    pub primal_coordination: HashMap<String, PrimalCoordinationStatus>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Service status in BYOB deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub version: String,
    pub image: String,
    pub command: Vec<String>,
    pub environment: HashMap<String, String>,
    pub endpoint: Option<String>,
    pub health: ServiceHealth,
    pub primal_assignment: Option<String>,
    pub resources: ServiceResources,
    pub ports: Vec<ServicePort>,
    pub volumes: Vec<ServiceVolume>,
    pub dependencies: Vec<String>,
    pub health_check: Option<ServiceHealthCheck>,
    pub replicas: u32,
    pub service_type: String,
}

/// Service resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResources {
    pub cpu_cores: f64,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub gpu_count: u32,
}

/// Service port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: String,
    pub port: u16,
    pub target_port: u16,
    pub protocol: String,
}

/// Service volume configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceVolume {
    pub name: String,
    pub mount_path: String,
    pub volume_type: String,
}

/// Service health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthCheck {
    pub endpoint: String,
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub retries: u32,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Primal coordination status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordinationStatus {
    pub primal_name: String,
    pub endpoint: Option<String>,
    pub status: CoordinationStatus,
    pub capabilities: Vec<String>,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// Coordination status with Primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationStatus {
    Connected,
    Connecting,
    Disconnected,
    Failed(String),
}

/// BYOB Coordinator - manages team deployments with Songbird orchestration
pub struct ByobCoordinator {
    /// Team workspaces
    workspaces: Arc<RwLock<HashMap<String, ByobTeamWorkspace>>>,
    
    /// Active deployments across all teams
    deployments: Arc<RwLock<HashMap<String, ByobDeployment>>>,
    
    /// Songbird configuration
    config: OrchestratorConfig,
    
    /// NestGate storage configuration
    nestgate_config: Option<NestGateConfig>,
    
    /// Primal discovery endpoints
    primal_discovery: Arc<RwLock<HashMap<String, String>>>,
}

impl ByobCoordinator {
    /// Create new BYOB coordinator
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            deployments: Arc::new(RwLock::new(HashMap::new())),
            config,
            nestgate_config: None,
            primal_discovery: Arc::new(RwLock::new(HashMap::new())),
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
            return Err(format!("Team workspace already exists: {}", team_id).into());
        }
        
        let workspace = ByobTeamWorkspace {
            team_id: team_id.clone(),
            active_deployments: HashMap::new(),
            resource_quota,
            primal_endpoints: HashMap::new(),
        };
        
        workspaces.insert(team_id.clone(), workspace);
        info!("Registered BYOB team workspace: {}", team_id);
        
        Ok(())
    }

    /// Deploy biome for team using Songbird orchestration
    pub async fn deploy_biome(
        &self,
        request: ByobDeploymentRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let deployment_id = request.deployment_id.clone();
        
        info!("Starting BYOB deployment {} for team {}", deployment_id, request.team_id);
        
        // Check team workspace exists and validate quotas
        {
            let workspaces = self.workspaces.read().await;
            let workspace = workspaces.get(&request.team_id)
                .ok_or_else(|| format!("Team workspace not found: {}", request.team_id))?;
            
            // Check deployment quota
            if workspace.active_deployments.len() >= workspace.resource_quota.max_deployments as usize {
                return Err(format!("Deployment quota exceeded for team {}: {}/{}", 
                    request.team_id, 
                    workspace.active_deployments.len(), 
                    workspace.resource_quota.max_deployments).into());
            }
        }
        
        // Create Songbird orchestrator for this deployment
        let orchestrator = self.create_orchestrator_for_deployment(&request).await?;
        
        // Create deployment instance
        let deployment = ByobDeployment {
            deployment_id: deployment_id.clone(),
            team_id: request.team_id.clone(),
            orchestrator: Some(orchestrator),
            status: ByobDeploymentStatus::Pending,
            services: HashMap::new(),
            primal_coordination: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // Store deployment
        {
            let mut deployments = self.deployments.write().await;
            deployments.insert(deployment_id.clone(), deployment);
        }
        
        // Update team workspace
        {
            let mut workspaces = self.workspaces.write().await;
            if let Some(workspace) = workspaces.get_mut(&request.team_id) {
                workspace.active_deployments.insert(deployment_id.clone(), {
                    let deployments = self.deployments.read().await;
                    deployments.get(&deployment_id).cloned()
            .ok_or_else(|| -> Box<dyn std::error::Error> { format!("Deployment not found: {}", deployment_id).into() })?
                });
            }
        }
        
        // Start orchestration in background
        let coordinator = Arc::new(self.clone());
        let deployment_id_clone = deployment_id.clone();
        tokio::spawn(async move {
            if let Err(e) = coordinator.orchestrate_deployment(&deployment_id_clone).await {
                error!("Failed to orchestrate deployment {}: {}", deployment_id_clone, e);
            }
        });
        
        Ok(deployment_id)
    }

    /// Get deployment status
    pub async fn get_deployment_status(
        &self,
        deployment_id: &str,
    ) -> Result<ByobDeploymentStatus, Box<dyn std::error::Error>> {
        let deployments = self.deployments.read().await;
        deployments
            .get(deployment_id)
            .map(|d| d.status.clone())
            .ok_or_else(|| format!("Deployment not found: {}", deployment_id).into())
    }

    /// List deployments for team
    pub async fn list_team_deployments(
        &self,
        team_id: &str,
    ) -> Result<Vec<ByobDeployment>, Box<dyn std::error::Error>> {
        let deployments = self.deployments.read().await;
        let team_deployments: Vec<ByobDeployment> = deployments
            .values()
            .filter(|d| d.team_id == team_id)
            .cloned()
            .collect();
        
        Ok(team_deployments)
    }

    /// Stop deployment
    pub async fn stop_deployment(
        &self,
        deployment_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if deployment exists
        {
            let deployments = self.deployments.read().await;
            if !deployments.contains_key(deployment_id) {
                return Err(format!("Deployment not found: {}", deployment_id).into());
            }
        }
        
        {
            let mut deployments = self.deployments.write().await;
            if let Some(deployment) = deployments.get_mut(deployment_id) {
                deployment.status = ByobDeploymentStatus::Stopping;
                deployment.updated_at = chrono::Utc::now();
            }
        }
        
        // Stop orchestration and clean up resources
        self.cleanup_deployment(deployment_id).await?;
        
        // Update status to stopped
        {
            let mut deployments = self.deployments.write().await;
            if let Some(deployment) = deployments.get_mut(deployment_id) {
                deployment.status = ByobDeploymentStatus::Stopped;
                deployment.updated_at = chrono::Utc::now();
            }
        }
        
        info!("Stopped BYOB deployment: {}", deployment_id);
        Ok(())
    }

    /// Create Songbird orchestrator for BYOB deployment
    async fn create_orchestrator_for_deployment(
        &self,
        request: &ByobDeploymentRequest,
    ) -> Result<SongbirdOrchestrator, Box<dyn std::error::Error>> {
        // Create orchestrator with the team's biome manifest
        let orchestrator = SongbirdOrchestrator {
            id: format!("orchestrator-{}", request.deployment_id),
            config: self.config.clone(),
            status: crate::biome::OrchestratorStatus::Initializing,
            endpoints: HashMap::new(),
            created_at: chrono::Utc::now(),
            manifest: request.manifest.clone(),
        };
        
        info!("Created Songbird orchestrator for deployment {}", request.deployment_id);
        Ok(orchestrator)
    }

    /// Orchestrate BYOB deployment
    async fn orchestrate_deployment(
        &self,
        deployment_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting orchestration for deployment: {}", deployment_id);
        
        // Update status to orchestrating
        {
            let mut deployments = self.deployments.write().await;
            if let Some(deployment) = deployments.get_mut(deployment_id) {
                deployment.status = ByobDeploymentStatus::Orchestrating;
                deployment.updated_at = chrono::Utc::now();
            }
        }
        
        // Get orchestrator
        let orchestrator = {
            let deployments = self.deployments.read().await;
            deployments
                .get(deployment_id)
                .and_then(|d| d.orchestrator.as_ref())
                .cloned()
                .ok_or("No orchestrator found for deployment")?
        };
        
        // Start Songbird orchestration
        orchestrator.orchestrate().await?;
        
        // Update status to coordinating with Primals
        {
            let mut deployments = self.deployments.write().await;
            if let Some(deployment) = deployments.get_mut(deployment_id) {
                deployment.status = ByobDeploymentStatus::CoordinatingPrimals;
                deployment.updated_at = chrono::Utc::now();
            }
        }
        
        // Coordinate with other Primals for network effects
        self.coordinate_with_primals(&orchestrator, deployment_id).await?;
        
        // Update status to running
        {
            let mut deployments = self.deployments.write().await;
            if let Some(deployment) = deployments.get_mut(deployment_id) {
                deployment.status = ByobDeploymentStatus::Running;
                deployment.updated_at = chrono::Utc::now();
            }
        }
        
        info!("Successfully orchestrated deployment: {}", deployment_id);
        Ok(())
    }

    /// Coordinate with other Primals for network effects
    async fn coordinate_with_primals(
        &self,
        orchestrator: &SongbirdOrchestrator,
        deployment_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use universal coordination with all available Primals
        if let Err(e) = orchestrator.coordinate_with_all_primals().await {
            warn!("Universal Primal coordination failed (continuing without): {}", e);
        } else {
            // Update coordination status for all configured Primals
            if let Some(primals) = &orchestrator.manifest.primals {
                for (primal_name, primal_config) in primals {
                    if primal_config.enabled {
                        self.update_primal_coordination_status(
                            deployment_id,
                            primal_name,
                            CoordinationStatus::Connected,
                            primal_config.capabilities.clone(),
                        ).await;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Update Primal coordination status
    async fn update_primal_coordination_status(
        &self,
        deployment_id: &str,
        primal_name: &str,
        status: CoordinationStatus,
        capabilities: Vec<String>,
    ) {
        let mut deployments = self.deployments.write().await;
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            // Get endpoint from discovery registry
            let endpoint = {
                let primal_endpoints = self.primal_discovery.read().await;
                primal_endpoints.get(primal_name).cloned()
            };
            
            let coordination_status = PrimalCoordinationStatus {
                primal_name: primal_name.to_string(),
                endpoint,
                status,
                capabilities,
                last_health_check: chrono::Utc::now(),
            };
            
            deployment.primal_coordination.insert(
                primal_name.to_string(),
                coordination_status,
            );
            deployment.updated_at = chrono::Utc::now();
        }
    }

    /// Create service endpoints for deployment
    fn _create_service_endpoints(&self, deployment: &ByobDeployment) -> HashMap<String, String> {
        let mut endpoints = HashMap::new();
        
        for (service_name, service_status) in &deployment.services {
            if let Some(endpoint) = &service_status.endpoint {
                endpoints.insert(service_name.clone(), endpoint.clone());
            } else {
                // Generate default endpoint using configured port
                let default_port = self.config.default_port.unwrap_or(8080);
                let default_endpoint = format!("http://{}:{}/{}", 
                    deployment.team_id, default_port, service_name);
                endpoints.insert(service_name.clone(), default_endpoint);
            }
        }
        
        // Add orchestrator endpoint using configured port
        let default_port = self.config.default_port.unwrap_or(8080);
        endpoints.insert(
            "orchestrator".to_string(),
            format!("http://{}:{}/orchestrator", deployment.team_id, default_port)
        );
        
        endpoints
    }

    /// Cleanup deployment resources
    async fn cleanup_deployment(
        &self,
        deployment_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting cleanup for deployment: {}", deployment_id);
        
        // Get deployment details for cleanup
        let deployment = {
            let deployments = self.deployments.read().await;
            deployments.get(deployment_id).cloned()
        };
        
        if let Some(deployment) = deployment {
            // 1. Stop orchestrated services
            if let Some(_orchestrator) = &deployment.orchestrator {
                info!("Stopping orchestrated services for deployment: {}", deployment_id);
                
                // Stop services in reverse dependency order
                let service_names: Vec<String> = deployment.services.keys().cloned().collect();
                for service_name in service_names.iter().rev() {
                    info!("Stopping service: {}", service_name);
                    // In a real implementation, this would stop the actual service
                    // For now, we log the action
                }
                
                // Stop orchestrator
                info!("Stopping orchestrator for deployment: {}", deployment_id);
            }
            
            // 2. Release storage resources
            if self.nestgate_config.is_some() {
                info!("Releasing storage resources for deployment: {}", deployment_id);
                let deployment_uuid = uuid::Uuid::parse_str(deployment_id)
                    .map_err(|e| format!("Invalid deployment UUID {}: {}", deployment_id, e))?;
                if let Err(e) = self.remove_storage(deployment_uuid).await {
                    warn!("Failed to cleanup storage for deployment {}: {}", deployment_id, e);
                }
            }
            
            // 3. Notify Primals of cleanup
            info!("Notifying Primals of deployment cleanup: {}", deployment_id);
            for (primal_name, coordination) in &deployment.primal_coordination {
                if let Some(endpoint) = &coordination.endpoint {
                    info!("Notifying {} at {} of cleanup", primal_name, endpoint);
                    // Send cleanup notification to Primal
                    if let Err(e) = self.notify_primal_cleanup(primal_name, endpoint, deployment_id).await {
                        warn!("Failed to notify {} of cleanup: {}", primal_name, e);
                    }
                }
            }
            
            // 4. Remove from team workspace
            {
                let mut workspaces = self.workspaces.write().await;
                if let Some(workspace) = workspaces.get_mut(&deployment.team_id) {
                    workspace.active_deployments.remove(deployment_id);
                    info!("Removed deployment {} from team workspace", deployment_id);
                }
            }
        }
        
        info!("Cleanup completed for deployment: {}", deployment_id);
        Ok(())
    }
    
    /// Notify a Primal service about deployment cleanup
    async fn notify_primal_cleanup(
        &self,
        primal_name: &str,
        endpoint: &str,
        deployment_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
            
        let cleanup_payload = serde_json::json!({
            "action": "deployment_cleanup",
            "deployment_id": deployment_id,
            "primal_name": primal_name,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        let cleanup_url = format!("{}/api/v1/cleanup", endpoint);
        
        match client
            .post(&cleanup_url)
            .json(&cleanup_payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Successfully notified {} of cleanup", primal_name);
                } else {
                    warn!("Primal {} returned non-success status: {}", primal_name, response.status());
                }
            }
            Err(e) => {
                warn!("Failed to notify {} of cleanup: {}", primal_name, e);
            }
        }
        
        Ok(())
    }

    /// Execute team deployment with universal Primal coordination
    async fn _execute_team_deployment(&self, deployment: &ByobDeployment) -> Result<DeploymentResult, String> {
        info!("Executing team deployment: {}", deployment.deployment_id);

        // Use the orchestrator's universal coordination instead of hardcoded methods
        if let Some(ref orchestrator) = deployment.orchestrator {
            if let Err(e) = orchestrator.coordinate_with_all_primals().await {
                warn!("Universal Primal coordination failed (continuing): {}", e);
            }
        }

        // Create deployment result with universal endpoint creation
        let result = DeploymentResult {
            deployment_id: deployment.deployment_id.clone(),
            status: DeploymentStatus::Running,
            endpoints: self._create_service_endpoints(deployment),
            service_endpoints: self._create_service_endpoints(deployment),
            created_at: deployment.created_at,
            manifest: deployment.orchestrator.as_ref()
                .map(|o| o.manifest.clone())
                .unwrap_or_else(|| SongbirdBiomeManifest {
                    metadata: crate::biome::BiomeMetadata {
                        name: deployment.deployment_id.clone(),
                        version: "1.0.0".to_string(),
                        description: Some("BYOB deployment".to_string()),
                    },
                    services: HashMap::new(),
                    networking: None,
                    primals: None,
                }),
        };

        info!("Team deployment executed successfully: {}", deployment.deployment_id);
        Ok(result)
    }

    /// Configure NestGate integration
    pub fn with_nestgate(mut self, config: NestGateConfig) -> Self {
        self.nestgate_config = Some(config);
        self
    }

    /// Provision storage for team deployment
    pub async fn provision_storage(
        &self,
        deployment_id: Uuid,
        team_id: String,
        requirements: TeamStorageRequirements,
    ) -> Result<StorageDeploymentResponse, ByobError> {
        info!("Provisioning storage for deployment: {}", deployment_id);

        if let Some(nestgate_config) = &self.nestgate_config {
            // Use NestGate for storage provisioning
            self.provision_nestgate_storage(deployment_id, team_id, requirements, nestgate_config)
                .await
        } else {
            // Fallback to basic storage provisioning
            self.provision_basic_storage(deployment_id, team_id, requirements)
                .await
        }
    }

    /// Provision storage using NestGate
    async fn provision_nestgate_storage(
        &self,
        deployment_id: Uuid,
        team_id: String,
        requirements: TeamStorageRequirements,
        config: &NestGateConfig,
    ) -> Result<StorageDeploymentResponse, ByobError> {
        let client = reqwest::Client::new();
        
        // Convert requirements to NestGate format
        let mut storage_requirements = HashMap::new();
        for (service_name, spec) in &requirements.service_storage {
            let volumes = vec![serde_json::json!({
                "name": spec.name,
                "mount_path": spec.mount_path,
                "size_bytes": spec.size_bytes,
                "tier": spec.tier,
                "read_only": spec.read_only,
            })];

            storage_requirements.insert(service_name.clone(), serde_json::json!({
                "service_name": service_name,
                "storage_bytes": spec.size_bytes,
                "tier": spec.tier,
                "volumes": volumes,
                "persistence": requirements.persistence,
                "access_mode": if spec.read_only { "ReadOnlyMany" } else { "ReadWriteOnce" },
            }));
        }

        // Create team quotas
        let team_quotas = serde_json::json!({
            "max_total_storage": requirements.total_storage_quota,
            "max_per_tier": {
                "Hot": requirements.total_storage_quota / 4,
                "Warm": requirements.total_storage_quota / 2,
                "Cold": requirements.total_storage_quota,
                "Cache": requirements.total_storage_quota / 8,
            },
            "max_datasets": 20,
            "max_snapshots": 100,
            "max_backup_retention_days": 30,
        });

        let provision_request = serde_json::json!({
            "deployment_id": deployment_id,
            "team_id": team_id,
            "deployment_name": format!("byob-{}", deployment_id),
            "storage_requirements": storage_requirements,
            "team_quotas": team_quotas,
        });

        let response = client
            .post(format!("{}/storage", config.api_endpoint))
            .json(&provision_request)
            .timeout(Duration::from_secs(config.connection_timeout))
            .send()
            .await
            .map_err(|e| ByobError::Storage(format!("NestGate API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ByobError::Storage(format!(
                "NestGate storage provisioning failed: {}",
                error_text
            )));
        }

        let storage_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ByobError::Storage(format!("Failed to parse NestGate response: {}", e)))?;

        // Convert NestGate response to our format
        let endpoints: HashMap<String, StorageEndpoint> = storage_response
            .get("endpoints")
            .and_then(|e| e.as_object())
            .map(|endpoints| {
                endpoints
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            StorageEndpoint {
                                endpoint_url: v.get("url").and_then(|u| u.as_str()).unwrap_or("").to_string(),
                                protocol: v.get("protocol").and_then(|p| p.as_str()).unwrap_or("http").to_string(),
                                port: v.get("port").and_then(|p| p.as_u64()).unwrap_or(8080) as u16,
                                tier: serde_json::from_value(v.get("tier").cloned().unwrap_or(serde_json::json!("Warm"))).unwrap_or(StorageTier::Warm),
                                endpoint_type: v.get("endpoint_type").and_then(|t| t.as_str()).unwrap_or("unknown").to_string(),
                                url: v.get("url").and_then(|u| u.as_str()).unwrap_or("").to_string(),
                                mount_instructions: v.get("mount_instructions").and_then(|m| m.as_str()).unwrap_or("").to_string(),
                                is_secure: v.get("is_secure").and_then(|s| s.as_bool()).unwrap_or(false),
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mounts: HashMap<String, VolumeMount> = storage_response
            .get("mounts")
            .and_then(|m| m.as_object())
            .map(|mounts| {
                mounts
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            VolumeMount {
                                volume_id: v.get("mount_id").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                                mount_path: v.get("mount_point").and_then(|p| p.as_str()).unwrap_or("").to_string(),
                                read_only: v.get("read_only").and_then(|r| r.as_bool()).unwrap_or(false),
                                size_bytes: v.get("size_bytes").and_then(|s| s.as_u64()).unwrap_or(0),
                                name: v.get("mount_id").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                                tier: serde_json::from_value(v.get("tier").cloned().unwrap_or(serde_json::json!("Warm"))).unwrap_or(StorageTier::Warm),
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        let usage = StorageUsage {
            used_bytes: storage_response
                .get("usage")
                .and_then(|u| u.get("total_used"))
                .and_then(|u| u.as_u64())
                .unwrap_or(0),
            available_bytes: storage_response
                .get("usage")
                .and_then(|u| u.get("available_bytes"))
                .and_then(|a| a.as_u64())
                .unwrap_or(0),
            total_bytes: storage_response
                .get("usage")
                .and_then(|u| u.get("total_bytes"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0),
            snapshots_count: storage_response
                .get("usage")
                .and_then(|u| u.get("snapshots_count"))
                .and_then(|s| s.as_u64())
                .unwrap_or(0) as u32,
            total_allocated: storage_response
                .get("usage")
                .and_then(|u| u.get("total_allocated"))
                .and_then(|a| a.as_u64())
                .unwrap_or(0),
            total_used: storage_response
                .get("usage")
                .and_then(|u| u.get("total_used"))
                .and_then(|u| u.as_u64())
                .unwrap_or(0),
            service_usage: storage_response
                .get("usage")
                .and_then(|u| u.get("service_usage"))
                .and_then(|s| s.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| {
                            v.as_u64().map(|usage| (k.clone(), usage))
                        })
                        .collect::<HashMap<String, u64>>()
                })
                .unwrap_or_default(),
        };

        Ok(StorageDeploymentResponse {
            deployment_id,
            team_id,
            status: StorageStatus::Ready,
            endpoints,
            mounts,
            usage,
            created_at: Utc::now(),
            manifest: SongbirdBiomeManifest {
                metadata: BiomeMetadata {
                    name: format!("storage-{}", deployment_id),
                    version: "1.0.0".to_string(),
                    description: Some("Auto-generated storage manifest".to_string()),
                },
                services: HashMap::new(),
                networking: None,
                primals: None,
            },
        })
    }

    /// Provision basic storage (fallback)
    async fn provision_basic_storage(
        &self,
        deployment_id: Uuid,
        team_id: String,
        requirements: TeamStorageRequirements,
    ) -> Result<StorageDeploymentResponse, ByobError> {
        warn!("Using basic storage provisioning for deployment: {}", deployment_id);

        // Basic storage provisioning without NestGate
        // This would create local directories or use a simple storage backend
        
        let mut endpoints = HashMap::new();
        endpoints.insert("local".to_string(), StorageEndpoint {
            endpoint_url: format!("/tmp/byob-storage/{}/{}", team_id, deployment_id),
            protocol: "file".to_string(),
            port: 0,
            tier: StorageTier::Warm,
            endpoint_type: "local".to_string(),
            url: format!("/tmp/byob-storage/{}/{}", team_id, deployment_id),
            mount_instructions: "Local filesystem mount".to_string(),
            is_secure: false,
        });

        let mut mounts = HashMap::new();
        for (service_name, spec) in &requirements.service_storage {
            mounts.insert(
                format!("{}-{}", service_name, spec.name),
                VolumeMount {
                    volume_id: format!("{}-{}", service_name, spec.name),
                    mount_path: spec.mount_path.clone(),
                    read_only: spec.read_only,
                    size_bytes: spec.size_bytes,
                    name: spec.name.clone(),
                    tier: spec.tier.clone(),
                },
            );
        }

        Ok(StorageDeploymentResponse {
            deployment_id,
            team_id,
            status: StorageStatus::Ready,
            endpoints,
            mounts,
            usage: StorageUsage {
                used_bytes: 0,
                available_bytes: requirements.total_storage_quota,
                total_bytes: requirements.total_storage_quota,
                snapshots_count: 0,
                total_allocated: requirements.total_storage_quota,
                total_used: 0,
                service_usage: HashMap::new(),
            },
            created_at: Utc::now(),
            manifest: SongbirdBiomeManifest {
                metadata: BiomeMetadata {
                    name: format!("basic-storage-{}", deployment_id),
                    version: "1.0.0".to_string(),
                    description: Some("Basic storage provision".to_string()),
                },
                services: HashMap::new(),
                networking: None,
                primals: None,
            },
        })
    }

    /// Remove storage for deployment
    pub async fn remove_storage(&self, deployment_id: Uuid) -> Result<(), ByobError> {
        info!("Removing storage for deployment: {}", deployment_id);

        if let Some(nestgate_config) = &self.nestgate_config {
            // Use NestGate for storage removal
            let client = reqwest::Client::new();
            
            let response = client
                .post(format!("{}/storage/{}", nestgate_config.api_endpoint, deployment_id))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({}))
                .timeout(Duration::from_secs(30))
                .send()
                .await
                .map_err(|e| ByobError::Storage(format!("NestGate API request failed: {}", e)))?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(ByobError::Storage(format!(
                    "NestGate storage removal failed: {}",
                    error_text
                )));
            }

            info!("Storage removed successfully via NestGate: {}", deployment_id);
        } else {
            // Basic storage removal
            warn!("Basic storage removal for deployment: {}", deployment_id);
        }

        Ok(())
    }

    /// Get storage status
    pub async fn get_storage_status(&self, deployment_id: Uuid) -> Result<StorageDeploymentResponse, ByobError> {
        if let Some(nestgate_config) = &self.nestgate_config {
            // Use NestGate for storage status
            let client = reqwest::Client::new();
            
            let response = client
                .get(format!("{}/storage/{}", nestgate_config.api_endpoint, deployment_id))
                .timeout(Duration::from_secs(30))
                .send()
                .await
                .map_err(|e| ByobError::Storage(format!("Failed to get storage status for deployment {}: {}", deployment_id, e)))?;

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(ByobError::Storage(format!(
                    "NestGate storage status failed: {}",
                    error_text
                )));
            }

            let storage_response: serde_json::Value = response
                .json()
                .await
                .map_err(|e| ByobError::Storage(format!("Failed to parse NestGate response: {}", e)))?;

            // Extract team_id from deployment context
            let team_id = {
                let deployments = self.deployments.read().await;
                deployments.get(&deployment_id.to_string())
                    .map(|d| d.team_id.clone())
                    .unwrap_or_else(|| "unknown".to_string())
            };

            // Parse status from response
            let status = storage_response
                .get("status")
                .and_then(|s| s.as_str())
                .map(|s| match s {
                    "ready" => StorageStatus::Ready,
                    "provisioning" => StorageStatus::Provisioning,
                    "error" => StorageStatus::Error,
                    _ => StorageStatus::Ready,
                })
                .unwrap_or(StorageStatus::Ready);

            // Parse endpoints from response
            let endpoints: HashMap<String, StorageEndpoint> = storage_response
                .get("endpoints")
                .and_then(|e| e.as_object())
                .map(|endpoints| {
                    endpoints
                        .iter()
                        .map(|(k, v)| {
                            let endpoint = StorageEndpoint {
                                endpoint_url: v.get("endpoint_url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("http://localhost:8080")
                                    .to_string(),
                                tier: match v.get("tier")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("warm") {
                                    "hot" => StorageTier::Hot,
                                    "warm" => StorageTier::Warm,
                                    "cold" => StorageTier::Cold,
                                    "cache" => StorageTier::Cache,
                                    _ => StorageTier::Warm,
                                },
                                endpoint_type: v.get("endpoint_type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("storage")
                                    .to_string(),
                                mount_instructions: v.get("mount_instructions")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                url: v.get("url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("http://localhost:8080")
                                    .to_string(),
                                port: v.get("port")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(8080) as u16,
                                protocol: v.get("protocol")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("http")
                                    .to_string(),
                                is_secure: v.get("is_secure")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                            };
                            (k.clone(), endpoint)
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Parse mounts from response
            let mounts: HashMap<String, VolumeMount> = storage_response
                .get("mounts")
                .and_then(|m| m.as_object())
                .map(|mounts| {
                    mounts
                        .iter()
                        .map(|(k, v)| {
                            let mount = VolumeMount {
                                volume_id: v.get("volume_id").and_then(|id| id.as_str()).unwrap_or("").to_string(),
                                mount_path: v.get("mount_path").and_then(|p| p.as_str()).unwrap_or("").to_string(),
                                read_only: v.get("read_only").and_then(|r| r.as_bool()).unwrap_or(false),
                                size_bytes: v.get("size_bytes").and_then(|s| s.as_u64()).unwrap_or(0),
                                name: v.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                                tier: match v.get("tier").and_then(|t| t.as_str()).unwrap_or("warm") {
                                    "hot" => StorageTier::Hot,
                                    "warm" => StorageTier::Warm,
                                    "cold" => StorageTier::Cold,
                                    "cache" => StorageTier::Cache,
                                    _ => StorageTier::Warm,
                                },
                            };
                            (k.clone(), mount)
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Parse usage from response
            let usage = StorageUsage {
                used_bytes: storage_response
                    .get("usage")
                    .and_then(|u| u.get("used_bytes"))
                    .and_then(|u| u.as_u64())
                    .unwrap_or(0),
                available_bytes: storage_response
                    .get("usage")
                    .and_then(|u| u.get("available_bytes"))
                    .and_then(|u| u.as_u64())
                    .unwrap_or(0),
                total_bytes: storage_response
                    .get("usage")
                    .and_then(|u| u.get("total_bytes"))
                    .and_then(|u| u.as_u64())
                    .unwrap_or(0),
                snapshots_count: storage_response
                    .get("usage")
                    .and_then(|u| u.get("snapshots_count"))
                    .and_then(|u| u.as_u64())
                    .unwrap_or(0) as u32,
                total_allocated: storage_response
                    .get("usage")
                    .and_then(|u| u.get("total_allocated"))
                    .and_then(|u| u.as_u64())
                    .unwrap_or(0),
                total_used: storage_response
                    .get("usage")
                    .and_then(|u| u.get("total_used"))
                    .and_then(|u| u.as_u64())
                    .unwrap_or(0),
                service_usage: storage_response
                    .get("usage")
                    .and_then(|u| u.get("service_usage"))
                    .and_then(|s| s.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| {
                                v.as_u64().map(|usage| (k.clone(), usage))
                            })
                            .collect::<HashMap<String, u64>>()
                    })
                    .unwrap_or_default(),
            };

            // Convert response (similar to provision_nestgate_storage)
            Ok(StorageDeploymentResponse {
                deployment_id,
                team_id,
                status,
                endpoints,
                mounts,
                usage,
                created_at: storage_response
                    .get("created_at")
                    .and_then(|c| c.as_str())
                    .and_then(|c| chrono::DateTime::parse_from_rfc3339(c).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(Utc::now),
                manifest: SongbirdBiomeManifest {
                    metadata: BiomeMetadata {
                        name: format!("status-{}", deployment_id),
                        version: "1.0.0".to_string(),
                        description: Some("Storage status response".to_string()),
                    },
                    services: HashMap::new(),
                    networking: None,
                    primals: None,
                },
            })
        } else {
            Err(ByobError::Storage("Storage status not available without NestGate".to_string()))
        }
    }
}

// Clone trait implementation for background tasks
impl Clone for ByobCoordinator {
    fn clone(&self) -> Self {
        Self {
            workspaces: Arc::clone(&self.workspaces),
            deployments: Arc::clone(&self.deployments),
            config: self.config.clone(),
            nestgate_config: self.nestgate_config.clone(),
            primal_discovery: Arc::clone(&self.primal_discovery),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_byob_coordinator_creation() -> Result<(), Box<dyn std::error::Error>> {
        let config = OrchestratorConfig::default();
        let coordinator = ByobCoordinator::new(config);
        
        // Test team workspace registration
        let team_id = "test-team".to_string();
        let quota = TeamResourceQuota {
            max_cpu_cores: 16.0,
            max_memory_bytes: 68719476736, // 64GB
            max_storage_bytes: 549755813888, // 512GB
            max_network_bandwidth_mbps: 1000,
            max_deployments: 5,
        };
        
        coordinator
            .register_team_workspace(team_id.clone(), quota)
            .await
            .map_err(|e| ByobError::Storage(format!("Failed to parse NestGate response: {}", e)))?;
        
        // Verify workspace was created
        let workspaces = coordinator.workspaces.read().await;
        assert!(workspaces.contains_key(&team_id));
        
        Ok(())
    }
} 