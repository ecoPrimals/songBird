// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::detection::{detect_capabilities, detect_resources};
use super::handlers::bridge_router;
use super::types::{
    Args, BridgeConfig, BridgeState, ServiceInfo, ServiceRegistration, WorkloadRequest,
};
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use clap::Parser;
use songbird_http_client::IpcHttpClient;
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt;

impl Args {
    pub(crate) fn host(&self) -> &str {
        &self.host
    }

    pub(crate) fn port(&self) -> u16 {
        self.port
    }

    pub(crate) fn service_name(&self) -> &str {
        &self.service_name
    }

    pub(crate) fn service_type(&self) -> &str {
        &self.service_type
    }

    pub(crate) fn songbird_endpoint(&self) -> Option<&str> {
        self.songbird_endpoint.as_deref()
    }

    pub(crate) fn node_id(&self) -> Option<&str> {
        self.node_id.as_deref()
    }

    pub(crate) fn tower_id(&self) -> Option<&str> {
        self.tower_id.as_deref()
    }

    pub(crate) fn capabilities(&self) -> Option<&str> {
        self.capabilities.as_deref()
    }

    pub(crate) fn backend_url(&self) -> Option<&str> {
        self.backend_url.as_deref()
    }
}

async fn sample_bridge_state() -> BridgeState {
    let http_client = IpcHttpClient::new().await.unwrap();
    let config = Arc::new(BridgeConfig {
        host: "127.0.0.1".to_string(),
        port: 9000,
        service_name: "Test Compute".to_string(),
        service_type: "compute".to_string(),
        node_id: "test-node-1".to_string(),
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

#[test]
fn args_parse_defaults_match_documented_defaults() {
    let args = Args::try_parse_from(["songbird-compute-bridge"]).unwrap();
    assert_eq!(args.host(), "0.0.0.0");
    assert_eq!(args.port(), 9000);
    assert_eq!(args.service_name(), "Compute Service");
    assert_eq!(args.service_type(), "compute");
    assert!(args.songbird_endpoint().is_none());
    assert!(args.node_id().is_none());
    assert!(args.tower_id().is_none());
    assert!(args.capabilities().is_none());
    assert!(args.backend_url().is_none());
}

#[test]
fn args_parse_overrides_from_argv() {
    let args = Args::try_parse_from([
        "songbird-compute-bridge",
        "--host",
        "10.0.0.1",
        "--port",
        "7777",
        "--service-name",
        "Edge Node",
        "--service-type",
        "gpu",
        "--songbird-endpoint",
        "http://sb:8080",
        "--node-id",
        "n-1",
        "--tower-id",
        "t-1",
        "--capabilities",
        "a,b",
        "--backend-url",
        "http://backend/",
    ])
    .unwrap();
    assert_eq!(args.host(), "10.0.0.1");
    assert_eq!(args.port(), 7777);
    assert_eq!(args.service_name(), "Edge Node");
    assert_eq!(args.service_type(), "gpu");
    assert_eq!(args.songbird_endpoint(), Some("http://sb:8080"));
    assert_eq!(args.node_id(), Some("n-1"));
    assert_eq!(args.tower_id(), Some("t-1"));
    assert_eq!(args.capabilities(), Some("a,b"));
    assert_eq!(args.backend_url(), Some("http://backend/"));
}

#[tokio::test]
async fn detect_resources_returns_consistent_shape() {
    let info = detect_resources().await;
    assert!(info.cpu_cores >= 1);
    assert!(info.memory_gb >= 1);
    assert!(info.gpu_count < 1_000);
    assert!(info.platform.contains('-'));
    assert!(
        info.storage_gb.is_some(),
        "storage_gb should be detected from disk or COMPUTE_STORAGE_GB env"
    );
}

#[test]
fn detect_capabilities_reflects_gpu_and_cpu_tiers() {
    let base = ServiceInfo {
        cpu_cores: 4,
        memory_gb: 8,
        gpu_count: 0,
        gpu_model: None,
        storage_gb: None,
        platform: "linux-x86_64".to_string(),
    };
    let s = detect_capabilities(&base);
    assert!(s.contains("compute"));
    assert!(s.contains("cpu"));
    assert!(!s.contains("gpu"));

    let with_gpu = ServiceInfo {
        gpu_count: 1,
        gpu_model: Some("Test GPU".to_string()),
        ..base.clone()
    };
    assert!(detect_capabilities(&with_gpu).contains("gpu"));
    assert!(detect_capabilities(&with_gpu).contains("ml-inference"));

    let batch = ServiceInfo {
        cpu_cores: 8,
        ..base.clone()
    };
    assert!(detect_capabilities(&batch).contains("batch-processing"));

    let parallel = ServiceInfo {
        cpu_cores: 32,
        ..base
    };
    assert!(detect_capabilities(&parallel).contains("parallel-computing"));
}

#[tokio::test]
async fn health_endpoint_ok_plain_text() {
    let app = bridge_router(sample_bridge_state().await);
    let res =
        app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"OK");
}

#[tokio::test]
async fn info_endpoint_json_shape() {
    let app = bridge_router(sample_bridge_state().await);
    let res =
        app.oneshot(Request::builder().uri("/info").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["node_id"], "test-node-1");
    assert_eq!(v["service_name"], "Test Compute");
    assert_eq!(v["service_type"], "compute");
    assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    let caps = v["capabilities"].as_array().unwrap();
    assert_eq!(caps.len(), 2);
    assert!(caps.iter().any(|c| c == "compute"));
}

#[tokio::test]
async fn router_unknown_route_returns_404() {
    let app = bridge_router(sample_bridge_state().await);
    let res =
        app.oneshot(Request::builder().uri("/nope").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn resources_endpoint_matches_service_info_json() {
    let app = bridge_router(sample_bridge_state().await);
    let res = app
        .oneshot(Request::builder().uri("/resources").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["cpu_cores"], 4);
    assert_eq!(v["memory_gb"], 8);
    assert_eq!(v["gpu_count"], 0);
    assert_eq!(v["platform"], "linux-x86_64");
}

#[test]
fn service_registration_serde_roundtrip() {
    let mut metadata = HashMap::new();
    metadata.insert("cpu_cores".into(), "4".into());
    let reg = ServiceRegistration {
        service_id: "svc-1".into(),
        service_name: "N".into(),
        service_type: "compute".into(),
        tower_id: "t1".into(),
        tower_name: "t1".into(),
        endpoint: "http://127.0.0.1:9000".into(),
        capabilities: vec!["compute".into(), "cpu".into()],
        metadata,
        health_status: "healthy".into(),
        registered_at: "2025-01-01T00:00:00Z".into(),
        last_seen: "2025-01-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&reg).unwrap();
    let back: ServiceRegistration = serde_json::from_str(&json).unwrap();
    assert_eq!(back.service_id, reg.service_id);
    assert_eq!(back.capabilities, reg.capabilities);
}

#[test]
fn workload_request_serde_roundtrip() {
    let w = WorkloadRequest {
        name: "job".into(),
        payload: serde_json::json!({ "x": 1 }),
    };
    let json = serde_json::to_string(&w).unwrap();
    let back: WorkloadRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, w.name);
    assert_eq!(back.payload, w.payload);
}

#[test]
fn detect_capabilities_edge_cpu_only() {
    let info = ServiceInfo {
        cpu_cores: 4,
        memory_gb: 8,
        gpu_count: 0,
        gpu_model: None,
        storage_gb: None,
        platform: "linux-x86_64".into(),
    };
    let s = detect_capabilities(&info);
    assert!(s.contains("compute"));
    assert!(!s.contains("batch-processing"));
}

#[tokio::test]
async fn capabilities_endpoint_returns_config_capabilities() {
    let app = bridge_router(sample_bridge_state().await);
    let res = app
        .oneshot(Request::builder().uri("/capabilities").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v["capabilities"].as_array().unwrap().contains(&serde_json::json!("compute")));
}

#[tokio::test]
async fn get_workload_returns_no_backend() {
    let app = bridge_router(sample_bridge_state().await);
    let res = app
        .oneshot(Request::builder().uri("/api/v1/workloads/job-1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["error"], "no_backend");
}

#[tokio::test]
async fn submit_workload_without_backend_returns_service_unavailable() {
    let app = bridge_router(sample_bridge_state().await);
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/workloads")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"job","payload":{"k":"v"}}"#.as_bytes().to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["error"], "no_backend");
}
