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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::bridge_router;
    use crate::service::types::{BridgeConfig, BridgeState, ServiceInfo};
    use axum::body::{Body, to_bytes};
    use axum::http::{Method, Request, StatusCode};
    use songbird_http_client::IpcHttpClient;
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn sample_bridge_state() -> BridgeState {
        let http_client = IpcHttpClient::new().await.expect("IPC HTTP client for tests");
        let config = Arc::new(BridgeConfig {
            host: "127.0.0.1".to_string(),
            port: 9000,
            service_name: "Test Compute".to_string(),
            service_type: "compute".to_string(),
            node_id: "test-node-handlers".to_string(),
            tower_id: "tower-test".to_string(),
            songbird_endpoint: None,
            capabilities: vec!["compute".to_string(), "cpu".to_string()],
            backend_url: None,
        });
        let service_info = Arc::new(ServiceInfo {
            cpu_cores: 4,
            memory_gb: 8,
            gpu_count: 0,
            gpu_model: None,
            storage_gb: Some(100),
            platform: "linux-x86_64".to_string(),
        });
        BridgeState {
            config,
            http_client,
            service_info,
        }
    }

    #[tokio::test]
    async fn submit_workload_invalid_json_returns_bad_request() {
        let app = bridge_router(sample_bridge_state().await);
        let res = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/workloads")
                    .header("content-type", "application/json")
                    .body(Body::from(b"{not json".as_slice().to_vec()))
                    .expect("valid test request"),
            )
            .await
            .expect("router should accept request");
        assert_eq!(
            res.status(),
            StatusCode::BAD_REQUEST,
            "malformed JSON body should fail Axum Json extraction before handler"
        );
    }

    #[tokio::test]
    async fn post_to_health_returns_method_not_allowed() {
        let app = bridge_router(sample_bridge_state().await);
        let res = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/health")
                    .body(Body::empty())
                    .expect("valid test request"),
            )
            .await
            .expect("router should accept request");
        assert_eq!(
            res.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "/health only allows GET"
        );
    }

    #[tokio::test]
    async fn workload_submission_accepts_minimal_json_body() {
        let app = bridge_router(sample_bridge_state().await);
        let res = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/workloads")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        br#"{"name":"n","payload":null}"#.to_vec(),
                    ))
                    .expect("valid test request"),
            )
            .await
            .expect("router should accept request");
        assert_eq!(
            res.status(),
            StatusCode::SERVICE_UNAVAILABLE,
            "without backend_url handler returns service unavailable"
        );
        let body = to_bytes(res.into_body(), usize::MAX)
            .await
            .expect("read body");
        let v: serde_json::Value = serde_json::from_slice(&body).expect("json body");
        assert_eq!(v["error"], "no_backend");
    }

    #[tokio::test]
    async fn get_workload_by_id_returns_no_backend_message() {
        let app = bridge_router(sample_bridge_state().await);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workloads/w-1")
                    .body(Body::empty())
                    .expect("valid test request"),
            )
            .await
            .expect("router should accept request");
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = to_bytes(res.into_body(), usize::MAX)
            .await
            .expect("read body");
        let v: serde_json::Value = serde_json::from_slice(&body).expect("json body");
        assert!(
            v["message"]
                .as_str()
                .expect("message field")
                .contains("compute backend"),
            "expected guidance about backend; got {v}"
        );
    }

    #[tokio::test]
    async fn proxy_path_when_backend_configured_returns_bad_gateway_or_error() {
        let http_client = IpcHttpClient::new().await.expect("IPC HTTP client");
        let config = Arc::new(BridgeConfig {
            host: "127.0.0.1".to_string(),
            port: 9000,
            service_name: "Test".to_string(),
            service_type: "compute".to_string(),
            node_id: "n".to_string(),
            tower_id: "t".to_string(),
            songbird_endpoint: None,
            capabilities: vec!["compute".into()],
            backend_url: Some("http://127.0.0.1:1".into()),
        });
        let service_info = Arc::new(ServiceInfo {
            cpu_cores: 1,
            memory_gb: 1,
            gpu_count: 0,
            gpu_model: None,
            storage_gb: None,
            platform: "linux-x86_64".into(),
        });
        let state = BridgeState {
            config,
            http_client,
            service_info,
        };
        let app = bridge_router(state);
        let res = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/workloads")
                    .header("content-type", "application/json")
                    .body(Body::from(br#"{"name":"j","payload":{}}"#.to_vec()))
                    .expect("valid test request"),
            )
            .await
            .expect("router should accept request");
        assert!(
            res.status() == StatusCode::BAD_GATEWAY || res.status() == StatusCode::INTERNAL_SERVER_ERROR,
            "unreachable backend should yield gateway or build error, got {}",
            res.status()
        );
    }
}
