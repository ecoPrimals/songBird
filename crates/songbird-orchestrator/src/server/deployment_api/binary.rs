// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Binary upload, filesystem layout, and process lifecycle for deployments.

use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use tokio::fs;
use tracing::{debug, error, info};

use super::types::{DeploymentInfo, DeploymentResponse, DeploymentState, DeploymentStatus};

/// POST /api/deployment/binary - Deploy a binary service
pub async fn deploy_binary(
    State(state): State<DeploymentState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DeploymentResponse>), (StatusCode, String)> {
    info!("📦 Received deployment request");

    let mut binary_data: Option<Bytes> = None;
    let mut service_name: Option<String> = None;
    let mut env_vars: HashMap<String, String> = HashMap::new();
    let mut auto_start = true;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "binary" => {
                debug!("📥 Receiving binary data...");
                binary_data =
                    Some(field.bytes().await.map_err(|e| {
                        (StatusCode::BAD_REQUEST, format!("Binary read error: {e}"))
                    })?);
            }
            "service_name" => {
                service_name =
                    Some(field.text().await.map_err(|e| {
                        (StatusCode::BAD_REQUEST, format!("Service name error: {e}"))
                    })?);
            }
            "env_vars" => {
                let env_json = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Env vars error: {e}")))?;
                env_vars = serde_json::from_str(&env_json)
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid env JSON: {e}")))?;
            }
            "auto_start" => {
                let auto_str = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Auto start error: {e}")))?;
                auto_start = auto_str.parse().unwrap_or(true);
            }
            _ => {}
        }
    }

    let binary_data =
        binary_data.ok_or_else(|| (StatusCode::BAD_REQUEST, "No binary provided".to_string()))?;

    let (status, response) =
        deploy_binary_bytes(&state, binary_data, service_name, env_vars, auto_start).await?;

    Ok((status, Json(response)))
}

/// Deploy raw binary bytes — shared by REST multipart and JSON-RPC `deployment.create`.
#[expect(
    clippy::implicit_hasher,
    reason = "Public API uses std HashMap for serde/multipart and JSON-RPC callers"
)]
pub async fn deploy_binary_bytes(
    state: &DeploymentState,
    binary_data: Bytes,
    service_name: Option<String>,
    env_vars: HashMap<String, String>,
    auto_start: bool,
) -> Result<(StatusCode, DeploymentResponse), (StatusCode, String)> {
    let deployment_id = format!("deploy-{}", fastrand::u64(..));
    let service_name = service_name.unwrap_or_else(|| {
        let suffix =
            deployment_id.get(7..15).filter(|s| !s.is_empty()).unwrap_or(deployment_id.as_str());
        format!("service-{suffix}")
    });

    info!("📦 Deploying service: {}", service_name);
    debug!("   Deployment ID: {}", deployment_id);
    debug!("   Binary size: {} bytes", binary_data.len());
    debug!("   Environment vars: {}", env_vars.len());

    let base_deploy_dir = crate::env_config::deployment_dir();
    let deploy_dir = base_deploy_dir.join(&deployment_id);
    fs::create_dir_all(&deploy_dir).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Directory creation failed: {e}"))
    })?;

    let binary_path = deploy_dir.join("service");
    fs::write(&binary_path, &binary_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Binary write failed: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata read failed: {e}")))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Chmod failed: {e}")))?;
    }

    info!("✅ Binary deployed to: {}", binary_path.display());

    let port = env_vars
        .iter()
        .find(|(k, _)| k.to_uppercase().contains("PORT"))
        .and_then(|(_, v)| v.parse::<u16>().ok());

    let mut deployment = DeploymentInfo {
        deployment_id: deployment_id.clone(),
        service_name: service_name.clone(),
        binary_path: binary_path.to_string_lossy().to_string(),
        env_vars: env_vars.clone(),
        status: DeploymentStatus::Deploying,
        deployed_at: chrono::Utc::now().to_rfc3339(),
        pid: None,
        port,
    };

    if auto_start {
        match start_service(&binary_path.to_string_lossy(), &env_vars).await {
            Ok(pid) => {
                info!("✅ Service started with PID: {}", pid);
                deployment.status = DeploymentStatus::Running;
                deployment.pid = Some(pid);
            }
            Err(e) => {
                error!("❌ Service start failed: {}", e);
                deployment.status = DeploymentStatus::Failed;
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Service start failed: {e}"),
                ));
            }
        }
    }

    state.deployments.write().await.insert(deployment_id.clone(), deployment.clone());

    let service_url = if let (Some(host), Some(port)) =
        (env_vars.get("COMPUTE_HOST").or_else(|| env_vars.get("SERVICE_HOST")), port)
    {
        Some(format!("http://{host}:{port}"))
    } else {
        None
    };

    let response = DeploymentResponse {
        deployment_id,
        status: "deployed".to_string(),
        message: format!("Service '{service_name}' deployed successfully"),
        service_url,
    };

    info!("🎉 Deployment complete: {}", service_name);

    Ok((StatusCode::CREATED, response))
}

/// Start a service with environment variables
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn start_service<S>(
    binary_path: &str,
    env_vars: &std::collections::HashMap<String, String, S>,
) -> Result<u32, String>
where
    S: std::hash::BuildHasher + Send + Sync,
{
    debug!("🎬 Starting service: {}", binary_path);

    let mut command = Command::new(binary_path);

    for (key, value) in env_vars {
        command.env(key, value);
    }

    command.stdout(Stdio::null()).stderr(Stdio::null()).stdin(Stdio::null());

    let child = command.spawn().map_err(|e| format!("Failed to spawn process: {e}"))?;

    let pid = child.id();
    debug!("✅ Service started with PID: {}", pid);

    Ok(pid)
}
