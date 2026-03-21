// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// JSON-RPC 2.0 Universal Gateway
// Progressive Protocol Enhancement - Phase 2
//
// This module implements a JSON-RPC 2.0 gateway that provides universal
// language-agnostic access to Songbird. Any language with HTTP support
// (Python, JavaScript, Java, Go, C++, etc.) can connect.
//
// Specification: https://www.jsonrpc.org/specification
// Part of: Progressive Protocol Enhancement - Week 2
// Created: November 11, 2025

mod handlers;
mod types;

#[cfg(test)]
#[path = "jsonrpc_api_tests.rs"]
mod tests;

pub use types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use handlers::{
    handle_beacon_exchange, handle_compute_job_status, handle_compute_route, handle_consent_check,
    handle_consent_grant, handle_deployment_create, handle_deployment_status,
    handle_federation_join, handle_federation_peers, handle_health, handle_health_standard,
    handle_identity, handle_protocol_capabilities, handle_protocol_negotiate_semantic,
    handle_registry_discover, handle_registry_register, handle_service_get,
    handle_service_register, handle_services_list, handle_task_create, handle_task_list,
    handle_version,
};
use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use songbird_universal_ipc::service::IpcServiceHandler;
use songbird_universal_ipc::tower_atomic::JsonRpcHandler;
use std::time::Instant;
use types::JSONRPC_VERSION;

/// JSON-RPC 2.0 routes
pub fn jsonrpc_routes() -> Router<JsonRpcState> {
    Router::new()
        .route("/", post(handle_jsonrpc_request))
        .route("/rpc", post(handle_jsonrpc_request))
}

/// Shared state for JSON-RPC API
#[derive(Clone)]
pub struct JsonRpcState {
    /// Federation state for service routing
    pub federation_state: Arc<FederationState>,
    /// Federated service registry (inter-node)
    pub service_registry: Arc<FederatedServiceRegistry>,
    /// Compute API state (same as `/api/compute`)
    pub compute_state: crate::server::compute_api::ComputeApiState,
    /// Deployment API state (same as `/api/deployment`)
    pub deployment_state: crate::server::deployment_api::DeploymentState,
    /// Protocol API state (same as `/api/protocol`)
    pub protocol_state: crate::server::protocol_api::ProtocolApiState,
    /// Universal Port Authority registry (same as `/api/v1/services`)
    pub upa_registry: Arc<crate::service_registry::ServiceRegistry>,
    /// Task lifecycle manager (same as `/api/v1/tasks`)
    pub task_manager: Arc<crate::task_lifecycle::TaskLifecycleManager>,
    /// Consent manager (same as `/api/consent`)
    pub consent_manager: Arc<crate::consent_management::ConsentManager>,
    /// Server start time for uptime calculation
    pub start_time: Arc<RwLock<Instant>>,
    /// Universal IPC handler — full method table for inter-gate communication
    /// When present, unknown methods on TCP are forwarded to the same handler
    /// that serves the Unix socket, making TCP equivalent for LAN mesh.
    pub ipc_handler: Option<Arc<IpcServiceHandler>>,
}

impl JsonRpcState {
    /// Create with IPC handler for full method forwarding on TCP
    ///
    /// This enables inter-gate communication over TCP :3492 by forwarding
    /// unknown methods to the universal-ipc handler (same as Unix socket).
    /// Dark Forest gating still applies on all TCP requests.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Single constructor wires every REST-backed JSON-RPC dependency"
    )]
    pub fn with_ipc_handler(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
        ipc_handler: Arc<IpcServiceHandler>,
        compute_state: crate::server::compute_api::ComputeApiState,
        deployment_state: crate::server::deployment_api::DeploymentState,
        protocol_state: crate::server::protocol_api::ProtocolApiState,
        upa_registry: Arc<crate::service_registry::ServiceRegistry>,
        task_manager: Arc<crate::task_lifecycle::TaskLifecycleManager>,
        consent_manager: Arc<crate::consent_management::ConsentManager>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
            compute_state,
            deployment_state,
            protocol_state,
            upa_registry,
            task_manager,
            consent_manager,
            start_time: Arc::new(RwLock::new(Instant::now())),
            ipc_handler: Some(ipc_handler),
        }
    }
}

/// POST /jsonrpc or POST /jsonrpc/rpc
///
/// Handles JSON-RPC 2.0 requests.
/// Provides universal language-agnostic access to Songbird functionality.
async fn handle_jsonrpc_request(
    State(state): State<JsonRpcState>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, StatusCode> {
    if request.jsonrpc != JSONRPC_VERSION {
        return Ok(Json(JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(JsonRpcError::invalid_request("jsonrpc must be '2.0'")),
            id: request.id.unwrap_or(Value::Null),
        }));
    }

    debug!("📞 JSON-RPC request: method={}", request.method);

    let result = match request.method.as_str() {
        "compute.route" => handle_compute_route(&state, request.params.clone()).await,
        "deployment.create" => handle_deployment_create(&state, request.params.clone()).await,
        "deployment.status" => handle_deployment_status(&state, request.params.clone()).await,
        "task.create" => handle_task_create(&state, request.params.clone()).await,
        "task.list" => handle_task_list(&state, request.params.clone()).await,
        "consent.check" => handle_consent_check(&state, request.params.clone()).await,
        "consent.grant" => handle_consent_grant(&state, request.params.clone()).await,
        "registry.register" => handle_registry_register(&state, request.params.clone()).await,
        "registry.discover" => handle_registry_discover(&state, request.params.clone()).await,
        "protocol.negotiate" => {
            handle_protocol_negotiate_semantic(&state, request.params.clone()).await
        }

        "songbird.services.list" => handle_services_list(&state).await,
        "songbird.services.get" => handle_service_get(&state, request.params).await,
        "songbird.services.register" => handle_service_register(&state, request.params).await,

        "songbird.compute.schedule" => handle_compute_route(&state, request.params.clone()).await,
        "songbird.compute.status" => {
            handle_compute_job_status(&state, request.params.clone()).await
        }

        "songbird.federation.peers" => handle_federation_peers(&state).await,
        "songbird.federation.join" => handle_federation_join(&state, request.params).await,

        "songbird.protocol.capabilities" => handle_protocol_capabilities().await,

        "songbird.health" => handle_health(&state).await,
        "songbird.version" => handle_version().await,

        "health" => handle_health_standard(&state).await,
        "identity" => handle_identity().await,
        "network.beacon_exchange" => handle_beacon_exchange(request.params).await,

        _ => {
            if let Some(ref ipc_handler) = state.ipc_handler {
                debug!(
                    "📡 Forwarding '{}' to universal-ipc handler (TCP→IPC bridge)",
                    request.method
                );
                match ipc_handler
                    .handle(&request.method, request.params.clone().unwrap_or(Value::Null))
                    .await
                {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        warn!("⚠️  IPC handler error for '{}': {}", request.method, e);
                        Err(JsonRpcError::method_not_found(format!("{}: {}", request.method, e)))
                    }
                }
            } else {
                warn!("⚠️  Unknown JSON-RPC method: {} (no IPC handler attached)", request.method);
                Err(JsonRpcError::method_not_found(&request.method))
            }
        }
    };

    let response = match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: Some(value),
            error: None,
            id: request.id.unwrap_or(Value::Null),
        },
        Err(error) => JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(error),
            id: request.id.unwrap_or(Value::Null),
        },
    };

    Ok(Json(response))
}
