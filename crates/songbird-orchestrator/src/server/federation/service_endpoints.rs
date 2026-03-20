// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Federation Service Registry Endpoints
//!
//! Handles service registration, discovery, and statistics operations

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use songbird_network_federation::service_registry::ServiceRegistration;
use std::sync::Arc;
use tracing::{debug, info};

use super::types::FederationAppState;

/// GET /api/federation/services - List all services
pub async fn list_services(State(state): State<Arc<FederationAppState>>) -> impl IntoResponse {
    debug!("📋 Service list requested");

    let services = state.service_registry.get_all_services().await;
    (StatusCode::OK, Json(services))
}

/// POST /api/federation/services - Register a service
pub async fn register_service(
    State(state): State<Arc<FederationAppState>>,
    Json(service): Json<ServiceRegistration>,
) -> impl IntoResponse {
    info!("📝 Service registration request: {} ({})", service.service_name, service.service_type);

    // Determine if this is a local or remote service
    // (For now, treat all as remote since they come via API)
    state.service_registry.register_remote(service).await;

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "registered",
            "message": "Service registered successfully"
        })),
    )
}

/// GET /`api/federation/services/:service_id` - Get specific service
pub async fn get_service(
    State(state): State<Arc<FederationAppState>>,
    axum::extract::Path(service_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    debug!("🔍 Service lookup: {}", service_id);

    match state.service_registry.find_by_id(&service_id).await {
        Some(service) => (StatusCode::OK, Json(service)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Service not found",
                "service_id": service_id
            })),
        )
            .into_response(),
    }
}

/// GET /`api/federation/services/type/:service_type` - Find services by type
pub async fn find_services_by_type(
    State(state): State<Arc<FederationAppState>>,
    axum::extract::Path(service_type): axum::extract::Path<String>,
) -> impl IntoResponse {
    debug!("🔍 Services by type: {}", service_type);

    let services = state.service_registry.find_by_type(&service_type).await;

    (StatusCode::OK, Json(services))
}

/// GET /api/federation/services/stats - Get service registry statistics
#[allow(clippy::similar_names)] // `state` and `stats` are semantically different despite similar names
pub async fn service_stats(State(state): State<Arc<FederationAppState>>) -> impl IntoResponse {
    debug!("📊 Service stats requested");

    let stats = state.service_registry.get_stats().await;
    (StatusCode::OK, Json(stats))
}
