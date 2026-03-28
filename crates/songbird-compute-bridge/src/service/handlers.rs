// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP routes and request handlers for the compute bridge.

use super::types::{BridgeState, ServiceInfo, WorkloadRequest};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use tracing::{error, warn};

pub fn bridge_router(state: BridgeState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/capabilities", get(capabilities_handler))
        .route("/resources", get(resources_handler))
        .route("/api/v1/workloads", post(submit_workload_handler))
        .route("/api/v1/workloads/:id", get(get_workload_handler))
        .with_state(state)
}

async fn health_handler() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

async fn info_handler(State(state): State<BridgeState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "node_id": state.config.node_id,
        "service_name": state.config.service_name,
        "service_type": state.config.service_type,
        "capabilities": state.config.capabilities,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn capabilities_handler(State(state): State<BridgeState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "capabilities": state.config.capabilities,
    }))
}

async fn resources_handler(State(state): State<BridgeState>) -> Json<ServiceInfo> {
    Json((*state.service_info).clone())
}

async fn submit_workload_handler(
    State(state): State<BridgeState>,
    Json(request): Json<WorkloadRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // If backend_url is configured, proxy to it
    if let Some(ref backend_url) = state.config.backend_url {
        let request_result =
            state.http_client.post(&format!("{backend_url}/api/v1/workloads")).await.json(&request);

        match request_result {
            Ok(request) => match request.send().await {
                Ok(response) => {
                    let status_code = StatusCode::from_u16(response.status())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                    response.json::<serde_json::Value>().await.map_or_else(
                        |_| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(
                                    serde_json::json!({"error": "Backend response parsing failed"}),
                                ),
                            )
                        },
                        |body| (status_code, Json(body)),
                    )
                }
                Err(e) => {
                    error!("Backend request failed: {e}");
                    (
                        StatusCode::BAD_GATEWAY,
                        Json(serde_json::json!({"error": format!("Backend unavailable: {e}")})),
                    )
                }
            },
            Err(e) => {
                error!("Failed to build request: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Request build failed: {e}")})),
                )
            }
        }
    } else {
        warn!("No compute backend configured — rejecting workload submission");
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "no_backend",
                "message": "No compute backend configured. Set COMPUTE_BACKEND_URL or register a compute capability provider."
            })),
        )
    }
}

async fn get_workload_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "no_backend",
            "message": "Workload status requires a compute backend. Register a compute capability provider."
        })),
    )
}
