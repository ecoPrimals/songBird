// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # Service Registry API Endpoints
//!
//! **HTTP API for Primal Service Registration**
//!
//! This module provides the HTTP endpoints for primals to register with Songbird.
//! It implements the Universal Port Authority principle.
//!
//! ## Endpoints
//!
//! - `POST /api/v1/services/register` - Register a new service
//! - `POST /api/v1/services/{id}/heartbeat` - Send heartbeat
//! - `DELETE /api/v1/services/{id}` - Deregister service
//! - `GET /api/v1/services` - List all services
//! - `GET /api/v1/services/{id}` - Get service by ID
//! - `GET /api/v1/services/query/{capability}` - Query by capability
//! - `GET /api/v1/info` - Get orchestrator info

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use serde_json::json;
use tracing::{error, info};

use crate::service_registry::{
    DeregistrationRequest, HeartbeatRequest, RegistrationRequest, ServiceRegistry,
};

/// Service registry API state
#[derive(Clone)]
pub struct ServiceRegistryApiState {
    pub registry: ServiceRegistry,
}

impl ServiceRegistryApiState {
    #[must_use]
    pub const fn new(registry: ServiceRegistry) -> Self {
        Self {
            registry,
        }
    }
}

/// Create service registry routes
pub fn service_registry_routes(registry: ServiceRegistry) -> Router {
    let state = ServiceRegistryApiState::new(registry);

    Router::new()
        .route("/register", post(register_service))
        .route("/:id/heartbeat", post(heartbeat_service))
        .route("/:id", delete(deregister_service))
        .route("/:id", get(get_service))
        .route("/", get(list_services))
        .route("/query/:capability", get(query_by_capability))
        .with_state(state)
}

/// Create info routes (for orchestrator discovery)
pub fn info_routes() -> Router {
    Router::new().route("/info", get(get_orchestrator_info))
}

// ============================================================================
// HANDLERS
// ============================================================================

/// POST /api/v1/services/register
async fn register_service(
    State(state): State<ServiceRegistryApiState>,
    Json(request): Json<RegistrationRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("📝 Registration request from {}", request.primal_name);

    let response = state.registry.register(request).await?;

    Ok(Json(json!({
        "status": response.status,
        "service_id": response.service_id,
        "assigned_endpoint": {
            "protocol": response.assigned_endpoint.protocol,
            "host": response.assigned_endpoint.host,
            "port": response.assigned_endpoint.port,
            "full_url": response.assigned_endpoint.full_url,
        },
        "fallback_endpoint": response.fallback_endpoint.as_ref().map(|e| json!({
            "protocol": e.protocol,
            "host": e.host,
            "port": e.port,
            "full_url": e.full_url,
        })),
        "token": response.registration_token,
        "heartbeat_interval_sec": response.heartbeat_interval_sec,
        "trust_level": response.trust_level,
    })))
}

/// POST /api/v1/services/:id/heartbeat
async fn heartbeat_service(
    State(state): State<ServiceRegistryApiState>,
    Path(service_id): Path<String>,
    Json(mut request): Json<HeartbeatRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Override service_id from path (more trustworthy than body)
    request.service_id = service_id;

    let response = state.registry.heartbeat(request).await?;

    Ok(Json(json!({
        "status": response.status,
        "next_heartbeat_sec": response.next_heartbeat_sec,
        "commands": response.commands,
    })))
}

/// DELETE /api/v1/services/:id
async fn deregister_service(
    State(state): State<ServiceRegistryApiState>,
    Path(service_id): Path<String>,
    Json(mut request): Json<DeregistrationRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Override service_id from path
    request.service_id = service_id;

    state.registry.deregister(request).await?;

    Ok(Json(json!({
        "status": "deregistered",
        "message": "Service successfully deregistered"
    })))
}

/// GET /api/v1/services/:id
async fn get_service(
    State(state): State<ServiceRegistryApiState>,
    Path(service_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let service = state
        .registry
        .get_service(&service_id)
        .await
        .ok_or_else(|| ApiError::NotFound(format!("Service not found: {service_id}")))?;

    Ok(Json(serde_json::to_value(service)?))
}

/// GET /api/v1/services
async fn list_services(
    State(state): State<ServiceRegistryApiState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let services = state.registry.list_services().await;
    let stats = state.registry.get_stats().await;

    Ok(Json(json!({
        "services": services,
        "stats": stats,
    })))
}

/// GET /api/v1/services/query/:capability
async fn query_by_capability(
    State(state): State<ServiceRegistryApiState>,
    Path(capability): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let services = state.registry.query_by_capability(&capability).await;

    Ok(Json(json!({
        "capability": capability,
        "services": services,
        "count": services.len(),
    })))
}

/// GET /api/v1/info
async fn get_orchestrator_info() -> Json<serde_json::Value> {
    // ✅ MIGRATED: Use environment-based configuration
    let base_url = songbird_process_env::var("SONGBIRD_BASE_URL").unwrap_or_else(|_| {
        let port =
            songbird_process_env::var("SONGBIRD_PORT").unwrap_or_else(|_| "8080".to_string());
        format!("https://[::]:{port}")
    });

    Json(json!({
        "name": "Songbird",
        "url": base_url,
        "capabilities": [
            "service_registry",
            "federation",
            "compute_orchestration",
            "task_management"
        ],
        "protocols": ["https", "tarpc", "jsonrpc", "websocket"],
        "version": env!("CARGO_PKG_VERSION"),
        "metadata": {
            "universal_port_authority": true,
            "capability_based_discovery": true,
            "graduated_information_disclosure": true
        }
    }))
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

/// API error type
#[derive(Debug)]
pub enum ApiError {
    Internal(anyhow::Error),
    NotFound(String),
    BadRequest(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Internal(err) => {
                error!("Internal error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {err}"))
            }
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_orchestrator_info() {
        let app = info_routes();

        let response = app
            .oneshot(Request::builder().uri("/info").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
