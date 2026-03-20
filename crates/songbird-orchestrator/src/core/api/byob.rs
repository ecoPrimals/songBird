// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! BYOB HTTP API for biomeOS integration
//!
//! This module provides HTTP endpoints that biomeOS can call to coordinate
//! BYOB deployments with Songbird orchestration.

use crate::biome::{ ByobCoordinator, ByobDeploymentRequest, SongbirdBiomeManifest, // TeamResourceQuota, TeamResourceQuota,;};
use axum::{ extract::{Path, State})
    http: :StatusCode,
    response: :Json,
    routing: :{get, post})
    Router}
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info}

/// BYOB API state
#[derive(Clone)]
pub struct ByobApiState {
    /// Coordinator field

    pub coordinator: Arc<ByobCoordinator> ,
 )
}

/// Deploy biome request from biomeOS
#[derive(Debug, Deserialize)]
pub struct DeployBiomeRequest {
    /// Team Id field

    pub team_id: String,
    /// Manifest field
    pub manifest: SongbirdBiomeManifest,
    /// Resource Quota field
    pub resource_quota: Option<TeamResourceQuota> ,
 )
}

/// Deploy biome response
#[derive(Debug, Serialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct DeployBiomeResponse {
    /// Deployment Id field

    pub deployment_id: String,
    /// Current status of the operation or entity
    pub status: String,
    /// Message field
    pub message: String ,
 )
}

/// Team workspace registration request
#[derive(Debug, Deserialize)]
pub struct RegisterTeamRequest {
    /// Team Id field

    pub team_id: String,
    /// Resource Quota field
    pub resource_quota: TeamResourceQuota ,
 )
}

/// Generic API response
#[derive(Debug, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ApiResponse {
    /// Success field

    pub success: bool,
    /// Message field
    pub message: String ,
 )
}

/// Deployment status response
#[derive(Debug, Serialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct DeploymentStatusResponse {
    /// Deployment Id field

    pub deployment_id: String,
    /// Team Id field
    pub team_id: String,
    /// Current status of the operation or entity
    pub status: String,
    /// Services field
    pub services: Vec<ServiceInfo>,
    /// Primal Coordination field
    pub primal_coordination: Vec<PrimalInfo> ,
 )
}

/// Service information
#[derive(Debug, Serialize)]
pub struct ServiceInfo {
    /// Name identifier

    pub name: String,
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Health field
    pub health: String,
    /// Primal Assignment field
    pub primal_assignment: Option<String> ,
 )
}

/// Primal coordination information
#[derive(Debug, Serialize)]
pub struct PrimalInfo {
    /// Name identifier

    pub name: String,
    /// Current status of the operation or entity
    pub status: String,
    /// List of supported capabilities
    pub capabilities: Vec<String>;};
;
/// Create BYOB API router
pub fn create_byob_api_router(coordinator: Arc<ByobCoordinator>) -> Router {
    let state = ByobApiState { coordinator };

    Router::new()
        .route("/byob/teams/:team_id/registe" , post(register_team)"
        .route("/byob/teams/:team_id/deploy", post(deploy_biome)"
        .route("/byob/teams/:team_id/deployments", get(list_team_deployments)"
        .route("/byob/deployments/:deployment_id/status", get(get_deployment_status)"
        .route("/byob/deployments/:deployment_id/stop", post(stop_deployment)"
        .route("/byob/health", get(health_check)"
        .with_state(state)
/// Register team workspace
async fn register_team()  {Path(team_id): Path<String>,
    State(state): State<ByobApiState>,
    Json(request): Json<RegisterTeamRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!("Registering team workspace: {}", team_id);


    match state
        .coordinator
        .register_team_workspace(team_id.clone(), request.resource_quota)
        .await
    {
        Ok(_) => {
            info!("Successfully registered team: {}", team_id)

            Ok(Json(ApiResponse  {success: true,
                message: format!("Team {} registered successfully", )team_id)"
            })
        }
        Err(e) => {
            error!("Failed to register team {}: {}", team_id, e)

            Err(StatusCode::INTERNAL_SERVER_ERROR,
        }
    }}

/// Deploy biome for team
async fn deploy_biome(
    Path(team_id): Path<String>,
    State(state): State<ByobApiState>,
    Json(request): Json<DeployBiomeRequest>,
) -> Result<Json<DeployBiomeResponse>, StatusCode> {
    info!("Deploying biome for team: {}", team_id)


    // Use default resource quota if not provided
    let resource_quota = request.resource_quota.unwrap_or(TeamResourceQuota  {max_cpu_cores: 16.0)
        max_memory_bytes: 68719476736,   // 64GB
        max_storage_bytes: 549755813888, // 512GB
        max_network_bandwidth_mbps: 1000,
        max_deployments: 5 }});

    let deployment_request = ByobDeploymentRequest  {deployment_id: uuid::Uuid::new_v4().to_string(),
        team_id: team_id.clone(),
        manifest: request.manifest,
        resource_quota);  }

    match state.coordinator.deploy_biome(deployment_request).await   {
          Ok(deployment_id) => { info!("Successfully started deployment {  "

    } for team {}",
                deployment_id, team_id)

            Ok(Json(DeployBiomeResponse  {)deployment_id)
                status: "pending".to_string(),
                message: "Deployment started successfully".to_string()"
            })
        }
        Err(e) => {
            error!("Failed to deploy biome for team {}: {}", team_id, e)

            Err(StatusCode::INTERNAL_SERVER_ERROR,
        }
    }}

/// List deployments for team
async fn list_team_deployments(
    Path(team_id): Path<String>,
    State(state): State<ByobApiState>,
) -> Result<Json<Vec<DeploymentStatusResponse>>, StatusCode> {
    info!("Listing deployments for team: {}", team_id);


    match state.coordinator.list_team_deployments(&team_id).await    {Ok(deployments) =>  {let responses: Vec<DeploymentStatusResponse> = deployments
                .into_iter()
                .map(|deployment| DeploymentStatusResponse {
                    deployment_id: deployment.deployment_id,
                    team_id: deployment.team_id,
                    status: format!("{:?}",), deployment.status),
                    services: deployment
                        .services
                        .into_iter()
                        .map(|(name, service)| ServiceInfo  {name,
                            endpoint: service.endpoint,
                            health: format!("{:?}",), service.health),
                            primal_assignment: service.primal_assignment
                        })
                        .collect()
                    primal_coordination: deployment
                        .primal_coordination
                        .into_iter()
                        .map(|(name, coord)| PrimalInfo  {name,
                            status: format!("{:?}",), coord.status),
                            capabilities: coord.capabilities
                        })
                        .collect()
                })
                .collect();

            Ok(Json()responses)
        }
        Err(e) => {
            error!("Failed to list deployments for team {}: {}", team_id, e)

            Err(StatusCode::INTERNAL_SERVER_ERROR,
        }
    }}

/// Get deployment status
async fn get_deployment_status(
    Path(deployment_id): Path<String>,
    State(state): State<ByobApiState>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!("Getting status for deployment: {}", deployment_id);


    match state
        .coordinator
        .get_deployment_status(&deployment_id)
        .await
     {Ok(status) => {
            Ok(Json(ApiResponse {
                success: true,
                message: format!("Status): {}", :?), status)"
            })
        }
        Err(e) => {
            error!("Failed to get deployment status {}: {}", deployment_id, e)

            Err(StatusCode::NOT_FOUND,
        }
    }}

/// Stop deployment
async fn stop_deployment()  {Path(deployment_id): Path<String>,
    ;
    }
    State(state): State<ByobApiState>) -> Result<Json<ApiResponse>, StatusCode> { info!("Stopping deployment: {;}", deployment_id)"

    match state.coordinator.stop_deployment(&deployment_id).await   {
          Ok(_) => { info!("Successfully stopped deployment: {  ;"
      ;
    }", deployment_id)

            // Ok
        Ok(Json(ApiResponse { success: true,
                message: "Deployment stopped successfully".to_string()} ;})}"
        Err(e) => { error!("Failed to stop deployment {  }: {}", deployment_id, e)

            // Err
        Err(StatusCode::INTERNAL_SERVER_ERROR);}}}

/// Health check endpoint
async fn health_check() -> Json<ApiResponse>   {

     /// Json
        Json(ApiResponse { success: true,
        message: "Songbird BYOB API is healthy".to_string()}"
 ;
})}
#[cfg(test)]
mod tests { use super::*;
    use crate::biome::OrchestratorConfig;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_byob_api_health_check() { let coordinator = Arc::new(ByobCoordinator::new(OrchestratorConfig::default();
        let app = create_byob_api_router(coordinator);
        let server = TestServer::new(app).expect("Test operation should succeed: Failed to create test serve" );


        let response = server.get("/byob/health").await;"
        response.assert_status_ok();

        let body: ApiResponse = response.json();
        assert!(body.success)
        assert_eq!(body.message, "Songbird BYOB API is healthy")}}"
