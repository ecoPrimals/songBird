// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
//! - ✅ Capability discovery (auto-detection)
//!
//! ## Endpoints
//! - GET /api/deployment/capabilities - Discover node capabilities
//! - POST /api/deployment/binary - Deploy and start a service (single upload)
//! - POST /api/deployment/negotiate - Start chunked upload negotiation
//! - POST /`api/deployment/chunk/:neg_id/:index` - Upload chunk
//! - POST /`api/deployment/finalize/:neg_id` - Finalize chunked upload
//! - GET /api/deployment/status/:id - Check deployment status
//! - DELETE /api/deployment/:id - Stop and remove deployment

mod binary;
mod capabilities;
mod types;

pub use super::chunked_upload::{finalize_chunked_upload, negotiate_chunked_upload, upload_chunk};
pub use binary::{deploy_binary, deploy_binary_bytes, start_service};
pub use capabilities::get_capabilities;
pub use types::*;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use tracing::{debug, info};

/// Create deployment routes
pub fn deployment_routes(state: DeploymentState) -> Router {
    Router::new()
        .route("/capabilities", get(get_capabilities))
        .route("/binary", post(deploy_binary))
        .route("/negotiate", post(negotiate_chunked_upload))
        .route("/chunk/{neg_id}/{index}", post(upload_chunk))
        .route("/finalize/{neg_id}", post(finalize_chunked_upload))
        .route("/status/{id}", get(get_deployment_status))
        .route("/{id}", delete(stop_deployment))
        .route("/list", get(list_deployments))
        .with_state(state)
}

/// GET /api/deployment/status/:id - Get deployment status
pub(crate) async fn get_deployment_status(
    State(state): State<DeploymentState>,
    Path(deployment_id): Path<String>,
) -> Result<Json<DeploymentInfo>, (StatusCode, String)> {
    let deployments = state.deployments.read().await;

    deployments
        .get(&deployment_id)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Deployment '{deployment_id}' not found")))
        .map(Json)
}

/// DELETE /api/deployment/:id - Stop deployment
async fn stop_deployment(
    State(state): State<DeploymentState>,
    Path(deployment_id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    info!("🛑 Stopping deployment: {}", deployment_id);

    let mut deployments = state.deployments.write().await;

    let deployment = deployments.get_mut(&deployment_id).ok_or_else(|| {
        (StatusCode::NOT_FOUND, format!("Deployment '{deployment_id}' not found"))
    })?;

    if let Some(pid) = deployment.pid {
        debug!("Stopping process PID: {}", pid);

        #[cfg(unix)]
        {
            let _ = std::process::Command::new("kill").arg("-TERM").arg(pid.to_string()).output();
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
async fn list_deployments(State(state): State<DeploymentState>) -> Json<Vec<DeploymentInfo>> {
    let deployments = state.deployments.read().await;
    Json(deployments.values().cloned().collect())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::capabilities::{calculate_max_concurrent, detect_network_type, estimate_bandwidth};
    use super::*;

    #[test]
    fn test_deployment_state_creation() {
        let state = DeploymentState::new();
        assert!(state.deployments.try_read().is_ok());
    }

    #[test]
    fn test_detect_network_type() {
        let network_type = detect_network_type();
        assert!(["lan", "vpn", "internet"].contains(&network_type.as_str()));
    }

    #[test]
    fn test_calculate_max_concurrent() {
        assert_eq!(calculate_max_concurrent(0), 1);
        assert_eq!(calculate_max_concurrent(5), 5);
        assert_eq!(calculate_max_concurrent(20), 10);
    }

    #[test]
    fn deployment_status_serde_lowercase() {
        let s = serde_json::to_string(&DeploymentStatus::Running).unwrap();
        assert!(s.contains("running"));
        let back: DeploymentStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(back, DeploymentStatus::Running);
    }

    #[test]
    fn deployment_status_all_variants_roundtrip() {
        for status in [
            DeploymentStatus::Deploying,
            DeploymentStatus::Running,
            DeploymentStatus::Failed,
            DeploymentStatus::Stopped,
        ] {
            let j = serde_json::to_string(&status).unwrap();
            let back: DeploymentStatus = serde_json::from_str(&j).unwrap();
            assert_eq!(back, status);
        }
    }

    #[test]
    fn deployment_info_roundtrip() {
        let mut env = std::collections::HashMap::new();
        env.insert("A".to_string(), "b".to_string());
        let info = DeploymentInfo {
            deployment_id: "d1".to_string(),
            service_name: "svc".to_string(),
            binary_path: "/tmp/svc".to_string(),
            env_vars: env,
            status: DeploymentStatus::Deploying,
            deployed_at: "2026-01-01T00:00:00Z".to_string(),
            pid: Some(42),
            port: Some(8080),
        };
        let j = serde_json::to_string(&info).unwrap();
        let back: DeploymentInfo = serde_json::from_str(&j).unwrap();
        assert_eq!(back.deployment_id, info.deployment_id);
        assert_eq!(back.status, info.status);
        assert_eq!(back.port, Some(8080));
    }

    #[test]
    fn negotiation_response_json_shape() {
        let r = NegotiationResponse {
            negotiation_id: "neg-1".to_string(),
            accepted_method: "chunked".to_string(),
            chunk_size_mb: 10,
            total_chunks: 3,
            chunk_upload_path: "/api/deployment/chunk/neg-1/{index}".to_string(),
            finalize_path: "/api/deployment/finalize/neg-1".to_string(),
            timeout_seconds: 300,
        };
        let j = serde_json::to_string(&r).unwrap();
        assert!(j.contains("negotiation_id"));
        assert!(j.contains("chunk_upload_path"));
    }

    #[test]
    fn estimate_bandwidth_lan_vs_default() {
        let lan = estimate_bandwidth("lan");
        assert_eq!(lan.confidence, "high");
        let other = estimate_bandwidth("satellite");
        assert_eq!(other.confidence, "low");
    }

    #[test]
    fn estimate_bandwidth_vpn() {
        let vpn = estimate_bandwidth("vpn");
        assert_eq!(vpn.confidence, "medium");
        assert!(vpn.latency_ms > lan_latency());
    }

    fn lan_latency() -> u32 {
        estimate_bandwidth("lan").latency_ms
    }

    #[test]
    fn deployment_response_fields() {
        let r = DeploymentResponse {
            deployment_id: "deploy-x".to_string(),
            status: "deployed".to_string(),
            message: "ok".to_string(),
            service_url: Some("http://127.0.0.1:8080".to_string()),
        };
        let j = serde_json::to_string(&r).unwrap();
        assert!(j.contains("deploy-x"));
        assert!(j.contains("service_url"));
    }
}
