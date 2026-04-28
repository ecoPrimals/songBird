// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Binary upload, filesystem layout, and process lifecycle for deployments.

use anyhow::Context as _;
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

fn default_service_name_for_deployment(deployment_id: &str) -> String {
    let suffix = deployment_id.get(7..15).filter(|s| !s.is_empty()).unwrap_or(deployment_id);
    format!("service-{suffix}")
}

fn port_from_env_vars(env_vars: &HashMap<String, String>) -> Option<u16> {
    env_vars
        .iter()
        .find(|(k, _)| k.to_uppercase().contains("PORT"))
        .and_then(|(_, v)| v.parse::<u16>().ok())
}

fn service_url_from_deployment_env(
    env_vars: &HashMap<String, String>,
    port: Option<u16>,
) -> Option<String> {
    let host = env_vars.get("COMPUTE_HOST").or_else(|| env_vars.get("SERVICE_HOST"))?;
    let port = port?;
    Some(format!("http://{host}:{port}"))
}

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
    let service_name =
        service_name.unwrap_or_else(|| default_service_name_for_deployment(&deployment_id));

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

    let port = port_from_env_vars(&env_vars);

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

    let service_url = service_url_from_deployment_env(&env_vars, port);

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
) -> anyhow::Result<u32>
where
    S: std::hash::BuildHasher + Send + Sync,
{
    debug!("🎬 Starting service: {}", binary_path);

    let mut command = Command::new(binary_path);

    for (key, value) in env_vars {
        command.env(key, value);
    }

    command.stdout(Stdio::null()).stderr(Stdio::null()).stdin(Stdio::null());

    let child =
        command.spawn().with_context(|| format!("Failed to spawn process: {binary_path}"))?;

    let pid = child.id();
    debug!("✅ Service started with PID: {}", pid);

    Ok(pid)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::{
        default_service_name_for_deployment, port_from_env_vars, service_url_from_deployment_env,
    };
    use std::collections::HashMap;

    #[test]
    fn default_service_name_uses_deployment_id_suffix() {
        assert_eq!(default_service_name_for_deployment("deploy-abcdef12"), "service-abcdef12");
    }

    #[test]
    fn default_service_name_falls_back_to_full_id_when_short() {
        assert_eq!(default_service_name_for_deployment("deploy"), "service-deploy");
    }

    #[test]
    fn port_from_env_vars_matches_port_substring_case_insensitive() {
        let mut m = HashMap::new();
        m.insert("APP_PORT".into(), "8080".into());
        assert_eq!(port_from_env_vars(&m), Some(8080));
        m.insert("not-a-number".into(), "xyz".into());
        assert_eq!(port_from_env_vars(&m), Some(8080));
    }

    #[test]
    fn port_from_env_vars_ignores_unparseable_port_value() {
        let mut m = HashMap::new();
        m.insert("PORT".into(), "nope".into());
        assert_eq!(port_from_env_vars(&m), None);
    }

    #[test]
    fn service_url_prefers_compute_host_then_service_host() {
        let mut m = HashMap::new();
        m.insert("SERVICE_HOST".into(), "10.0.0.1".into());
        assert_eq!(
            service_url_from_deployment_env(&m, Some(443)),
            Some("http://10.0.0.1:443".into())
        );
        m.insert("COMPUTE_HOST".into(), "192.168.0.2".into());
        assert_eq!(
            service_url_from_deployment_env(&m, Some(80)),
            Some("http://192.168.0.2:80".into())
        );
    }

    #[test]
    fn service_url_none_without_host_or_port() {
        let m: HashMap<String, String> = HashMap::new();
        assert_eq!(service_url_from_deployment_env(&m, Some(1)), None);
        let mut m = HashMap::new();
        m.insert("COMPUTE_HOST".into(), "h".into());
        assert_eq!(service_url_from_deployment_env(&m, None), None);
    }

    #[test]
    fn deployment_response_serializes() {
        use super::DeploymentResponse;
        let r = DeploymentResponse {
            deployment_id: "d1".into(),
            status: "deployed".into(),
            message: "ok".into(),
            service_url: Some("http://localhost:1".into()),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["deployment_id"], "d1");
        assert_eq!(v["service_url"], "http://localhost:1");
    }
}
