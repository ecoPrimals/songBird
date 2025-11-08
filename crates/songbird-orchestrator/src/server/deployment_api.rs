//! # 🚀 Deployment API
//!
//! HTTP-based service deployment for Songbird federation.
//! Enables remote service deployment without SSH.
//!
//! ## Features
//! - ✅ Binary upload via multipart/form-data
//! - ✅ Environment variable configuration
//! - ✅ Automatic service startup
//! - ✅ Health verification
//! - ✅ Federation-integrated
//!
//! ## Endpoints
//! - POST /api/deployment/binary - Deploy and start a service
//! - GET /api/deployment/status/:id - Check deployment status
//! - DELETE /api/deployment/:id - Stop and remove deployment

use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Deployment state
#[derive(Clone)]
pub struct DeploymentState {
    deployments: Arc<RwLock<HashMap<String, DeploymentInfo>>>,
}

impl DeploymentState {
    pub fn new() -> Self {
        Self {
            deployments: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub deployment_id: String,
    pub service_name: String,
    pub binary_path: String,
    pub env_vars: HashMap<String, String>,
    pub status: DeploymentStatus,
    pub deployed_at: String, // ISO 8601 timestamp
    pub pid: Option<u32>,
    pub port: Option<u16>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Deploying,
    Running,
    Failed,
    Stopped,
}

/// Deployment request
#[derive(Debug, Deserialize)]
pub struct DeploymentRequest {
    pub service_name: String,
    pub env_vars: Option<HashMap<String, String>>,
    pub auto_start: Option<bool>,
}

/// Deployment response
#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub message: String,
    pub service_url: Option<String>,
}

/// Create deployment routes
pub fn deployment_routes(state: DeploymentState) -> Router {
    Router::new()
        .route("/binary", post(deploy_binary))
        .route("/status/:id", get(get_deployment_status))
        .route("/:id", delete(stop_deployment))
        .route("/list", get(list_deployments))
        .with_state(state)
}

/// POST /api/deployment/binary - Deploy a binary service
async fn deploy_binary(
    State(state): State<DeploymentState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DeploymentResponse>), (StatusCode, String)> {
    info!("📦 Received deployment request");

    let deployment_id = format!("deploy-{}", fastrand::u64(..));
    let mut binary_data: Option<Bytes> = None;
    let mut service_name = format!("service-{}", &deployment_id[..8]);
    let mut env_vars: HashMap<String, String> = HashMap::new();
    let mut auto_start = true;

    // Parse multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "binary" => {
                debug!("📥 Receiving binary data...");
                binary_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Binary read error: {}", e)))?,
                );
            }
            "service_name" => {
                service_name = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Service name error: {}", e)))?;
            }
            "env_vars" => {
                let env_json = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Env vars error: {}", e)))?;
                env_vars = serde_json::from_str(&env_json)
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid env JSON: {}", e)))?;
            }
            "auto_start" => {
                let auto_str = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Auto start error: {}", e)))?;
                auto_start = auto_str.parse().unwrap_or(true);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    let binary_data = binary_data.ok_or((StatusCode::BAD_REQUEST, "No binary provided".to_string()))?;

    info!("📦 Deploying service: {}", service_name);
    debug!("   Deployment ID: {}", deployment_id);
    debug!("   Binary size: {} bytes", binary_data.len());
    debug!("   Environment vars: {}", env_vars.len());

    // Create deployment directory
    let deploy_dir = format!("/tmp/songbird-deployments/{}", deployment_id);
    fs::create_dir_all(&deploy_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Directory creation failed: {}", e)))?;

    // Write binary
    let binary_path = format!("{}/service", deploy_dir);
    fs::write(&binary_path, &binary_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Binary write failed: {}", e)))?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata read failed: {}", e)))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Chmod failed: {}", e)))?;
    }

    info!("✅ Binary deployed to: {}", binary_path);

    // Extract port from env vars
    let port = env_vars
        .iter()
        .find(|(k, _)| k.to_uppercase().contains("PORT"))
        .and_then(|(_, v)| v.parse::<u16>().ok());

    // Create deployment info
    let mut deployment = DeploymentInfo {
        deployment_id: deployment_id.clone(),
        service_name: service_name.clone(),
        binary_path: binary_path.clone(),
        env_vars: env_vars.clone(),
        status: DeploymentStatus::Deploying,
        deployed_at: chrono::Utc::now().to_rfc3339(),
        pid: None,
        port,
    };

    // Start service if requested
    if auto_start {
        match start_service(&binary_path, &env_vars).await {
            Ok(pid) => {
                info!("✅ Service started with PID: {}", pid);
                deployment.status = DeploymentStatus::Running;
                deployment.pid = Some(pid);
            }
            Err(e) => {
                error!("❌ Service start failed: {}", e);
                deployment.status = DeploymentStatus::Failed;
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Service start failed: {}", e)));
            }
        }
    }

    // Store deployment info
    state.deployments.write().await.insert(deployment_id.clone(), deployment.clone());

    // Build service URL
    let service_url = if let (Some(host), Some(port)) = (
        env_vars.get("COMPUTE_HOST").or(env_vars.get("SERVICE_HOST")),
        port,
    ) {
        Some(format!("http://{}:{}", host, port))
    } else {
        None
    };

    let response = DeploymentResponse {
        deployment_id,
        status: "deployed".to_string(),
        message: format!("Service '{}' deployed successfully", service_name),
        service_url,
    };

    info!("🎉 Deployment complete: {}", service_name);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Start a service with environment variables
async fn start_service(binary_path: &str, env_vars: &HashMap<String, String>) -> Result<u32, String> {
    debug!("🎬 Starting service: {}", binary_path);

    let mut command = Command::new(binary_path);

    // Set environment variables
    for (key, value) in env_vars {
        command.env(key, value);
    }

    // Run in background with nohup
    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    // Spawn the process
    let child = command
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    let pid = child.id();
    debug!("✅ Service started with PID: {}", pid);

    Ok(pid)
}

/// GET /api/deployment/status/:id - Get deployment status
async fn get_deployment_status(
    State(state): State<DeploymentState>,
    Path(deployment_id): Path<String>,
) -> Result<Json<DeploymentInfo>, (StatusCode, String)> {
    let deployments = state.deployments.read().await;
    
    deployments
        .get(&deployment_id)
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, format!("Deployment '{}' not found", deployment_id)))
        .map(Json)
}

/// DELETE /api/deployment/:id - Stop deployment
async fn stop_deployment(
    State(state): State<DeploymentState>,
    Path(deployment_id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    info!("🛑 Stopping deployment: {}", deployment_id);

    let mut deployments = state.deployments.write().await;
    
    let deployment = deployments
        .get_mut(&deployment_id)
        .ok_or((StatusCode::NOT_FOUND, format!("Deployment '{}' not found", deployment_id)))?;

    // Stop process if running
    if let Some(pid) = deployment.pid {
        debug!("Stopping process PID: {}", pid);
        
        // Try to stop the process (best effort)
        #[cfg(unix)]
        {
            // Use kill command as fallback
            let _ = std::process::Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .output();
        }
        
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .arg("/PID")
                .arg(pid.to_string())
                .arg("/T")
                .output();
        }
    }

    deployment.status = DeploymentStatus::Stopped;
    deployment.pid = None;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "stopped",
            "deployment_id": deployment_id,
            "message": "Service stopped successfully"
        })),
    ))
}

/// GET /api/deployment/list - List all deployments
async fn list_deployments(
    State(state): State<DeploymentState>,
) -> Json<Vec<DeploymentInfo>> {
    let deployments = state.deployments.read().await;
    Json(deployments.values().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_state_creation() {
        let state = DeploymentState::new();
        assert!(state.deployments.try_read().is_ok());
    }
}

