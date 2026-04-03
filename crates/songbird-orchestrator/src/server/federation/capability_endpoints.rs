// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Federation Capability Registration Endpoints
//!
//! Handles capability provider registration, heartbeat, and discovery operations

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::types::FederationAppState;
use crate::core::registry::types::{
    CapabilityRegistrationRequest, CapabilityRegistrationResponse, HeartbeatData,
    HeartbeatRequest as CapabilityHeartbeatRequest, ProviderListData, ProviderListResponse,
    ProviderSummary, RegistrationData,
};

/// POST /api/v1/federation/register - Register a capability provider
pub async fn register_capability_provider(
    State(state): State<Arc<FederationAppState>>,
    Json(request): Json<CapabilityRegistrationRequest>,
) -> impl IntoResponse {
    info!(
        "🔌 Capability provider '{}' ({}) registering with {} capabilities",
        request.provider_name,
        request.provider_id,
        request.capabilities.len()
    );

    // Modern idiomatic: let...else pattern for early return
    let Some(capability_registry) = &state.capability_registry else {
        warn!("Capability registry not initialized");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(CapabilityRegistrationResponse {
                success: false,
                data: None,
                error: Some("Capability registry not available".to_string()),
                timestamp: Utc::now(),
            }),
        );
    };

    match capability_registry.register(request.clone()).await {
        Ok(registration_id) => {
            info!(
                "✅ Provider '{}' registered successfully with ID: {}",
                request.provider_id, registration_id
            );

            (
                StatusCode::OK,
                Json(CapabilityRegistrationResponse {
                    success: true,
                    data: Some(RegistrationData {
                        provider_id: request.provider_id.clone(),
                        registration_id,
                        status: "registered".to_string(),
                        heartbeat_interval_ms: capability_registry.config().interval.as_millis()
                            as u64,
                        heartbeat_endpoint: "/api/v1/federation/capability/heartbeat".to_string(),
                    }),
                    error: None,
                    timestamp: Utc::now(),
                }),
            )
        }
        Err(e) => {
            warn!("❌ Failed to register provider '{}': {}", request.provider_id, e);

            (
                StatusCode::BAD_REQUEST,
                Json(CapabilityRegistrationResponse {
                    success: false,
                    data: None,
                    error: Some(format!("{e}")),
                    timestamp: Utc::now(),
                }),
            )
        }
    }
}

/// POST /api/v1/federation/capability/heartbeat - Update provider heartbeat
pub async fn capability_provider_heartbeat(
    State(state): State<Arc<FederationAppState>>,
    Json(request): Json<CapabilityHeartbeatRequest>,
) -> impl IntoResponse {
    debug!("💓 Heartbeat from provider '{}'", request.provider_id);

    // Modern idiomatic: let...else pattern for early return
    let Some(capability_registry) = &state.capability_registry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(crate::core::registry::types::HeartbeatResponse {
                success: false,
                data: None,
                error: Some("Capability registry not available".to_string()),
                timestamp: Utc::now(),
            }),
        );
    };

    // Convert heartbeat status to ProviderHealth if provided
    let health = request.health_status.map(|status| {
        use crate::core::registry::types::{HealthStatus, ProviderHealth, ResourceUsage};

        ProviderHealth {
            // Modern idiomatic: combine identical match arms
            status: match status.status.as_str() {
                "degraded" => HealthStatus::Degraded,
                "unhealthy" => HealthStatus::Unhealthy,
                _ => HealthStatus::Healthy, // Default to healthy for unknown statuses
            },
            available_capacity: status.available_capacity,
            resource_usage: ResourceUsage {
                cpu_percent: status.resource_usage.cpu_percent,
                memory_percent: status.resource_usage.memory_percent,
                gpu_utilization: status.resource_usage.gpu_utilization,
            },
        }
    });

    match capability_registry
        .update_heartbeat(&request.provider_id, &request.registration_id, health)
        .await
    {
        Ok(()) => {
            debug!("✅ Heartbeat acknowledged from '{}'", request.provider_id);

            (
                StatusCode::OK,
                Json(crate::core::registry::types::HeartbeatResponse {
                    success: true,
                    data: Some(HeartbeatData {
                        acknowledged: true,
                        next_heartbeat_ms: capability_registry.config().interval.as_millis() as u64,
                    }),
                    error: None,
                    timestamp: Utc::now(),
                }),
            )
        }
        Err(e) => {
            warn!("❌ Heartbeat failed from '{}': {}", request.provider_id, e);

            (
                StatusCode::BAD_REQUEST,
                Json(crate::core::registry::types::HeartbeatResponse {
                    success: false,
                    data: None,
                    error: Some(format!("{e}")),
                    timestamp: Utc::now(),
                }),
            )
        }
    }
}

/// DELETE /`api/v1/federation/register/:provider_id` - Unregister a provider
pub async fn unregister_capability_provider(
    State(state): State<Arc<FederationAppState>>,
    Path(provider_id): Path<String>,
) -> impl IntoResponse {
    info!("🔌 Provider '{}' unregistering", provider_id);

    // Modern idiomatic: let...else pattern for early return
    let Some(capability_registry) = &state.capability_registry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(CapabilityRegistrationResponse {
                success: false,
                data: None,
                error: Some("Capability registry not available".to_string()),
                timestamp: Utc::now(),
            }),
        );
    };

    match capability_registry.unregister(&provider_id).await {
        Ok(()) => {
            info!("✅ Provider '{}' unregistered successfully", provider_id);

            (
                StatusCode::OK,
                Json(CapabilityRegistrationResponse {
                    success: true,
                    data: Some(RegistrationData {
                        provider_id: provider_id.clone(),
                        registration_id: String::new(),
                        status: "unregistered".to_string(),
                        heartbeat_interval_ms: 0,
                        heartbeat_endpoint: String::new(),
                    }),
                    error: None,
                    timestamp: Utc::now(),
                }),
            )
        }
        Err(e) => {
            warn!("❌ Failed to unregister provider '{}': {}", provider_id, e);

            (
                StatusCode::NOT_FOUND,
                Json(CapabilityRegistrationResponse {
                    success: false,
                    data: None,
                    error: Some(format!("{e}")),
                    timestamp: Utc::now(),
                }),
            )
        }
    }
}

/// GET /api/v1/federation/providers - List all registered capability providers
pub async fn list_capability_providers(
    State(state): State<Arc<FederationAppState>>,
) -> impl IntoResponse {
    debug!("📋 Listing all capability providers");

    // Modern idiomatic: let...else pattern for early return
    let Some(capability_registry) = &state.capability_registry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ProviderListResponse {
                success: false,
                data: None,
                error: Some("Capability registry not available".to_string()),
                timestamp: Utc::now(),
            }),
        );
    };

    let providers = capability_registry.list_providers().await;
    let summaries: Vec<ProviderSummary> = providers.iter().map(ProviderSummary::from).collect();

    (
        StatusCode::OK,
        Json(ProviderListResponse {
            success: true,
            data: Some(ProviderListData {
                total_count: summaries.len(),
                providers: summaries,
            }),
            error: None,
            timestamp: Utc::now(),
        }),
    )
}
