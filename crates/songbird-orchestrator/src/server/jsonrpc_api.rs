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

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, warn};

/// JSON-RPC 2.0 version string (static to avoid allocations)
const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC 2.0 routes
pub fn jsonrpc_routes() -> Router<JsonRpcState> {
    Router::new()
        .route("/", post(handle_jsonrpc_request))
        .route("/rpc", post(handle_jsonrpc_request))
}

use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use songbird_universal_ipc::service::IpcServiceHandler;
use songbird_universal_ipc::tower_atomic::JsonRpcHandler;
use std::time::Instant;
use tokio::sync::RwLock;

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

/// JSON-RPC 2.0 Request
/// <https://www.jsonrpc.org/specification#request_object>
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,

    /// Method name to call
    pub method: String,

    /// Parameters (can be array or object)
    #[serde(default)]
    pub params: Option<Value>,

    /// Request ID (for responses, null for notifications)
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Response
/// <https://www.jsonrpc.org/specification#response_object>
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// Request ID (same as request, or null)
    pub id: Value,
}

/// JSON-RPC 2.0 Error
/// <https://www.jsonrpc.org/specification#error_object>
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard JSON-RPC error codes
#[allow(dead_code, reason = "JSON-RPC error constructors and codes reserved for API completeness")]
impl JsonRpcError {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    #[must_use]
    pub fn parse_error() -> Self {
        Self {
            code: Self::PARSE_ERROR,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_REQUEST,
            message: message.into(),
            data: None,
        }
    }

    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: Self::METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: Self::INTERNAL_ERROR,
            message: message.into(),
            data: None,
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
    // Validate JSON-RPC version
    if request.jsonrpc != JSONRPC_VERSION {
        return Ok(Json(JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(JsonRpcError::invalid_request("jsonrpc must be '2.0'")),
            id: request.id.unwrap_or(Value::Null),
        }));
    }

    debug!("📞 JSON-RPC request: method={}", request.method);

    // Route to appropriate handler based on method
    let result = match request.method.as_str() {
        // Semantic names (wateringHole / PRIMAL_IPC) — mirror REST handlers
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

        // Service discovery methods
        "songbird.services.list" => handle_services_list(&state).await,
        "songbird.services.get" => handle_service_get(&state, request.params).await,
        "songbird.services.register" => handle_service_register(&state, request.params).await,

        // Compute methods
        "songbird.compute.schedule" => handle_compute_route(&state, request.params.clone()).await,
        "songbird.compute.status" => {
            handle_compute_job_status(&state, request.params.clone()).await
        }

        // Federation methods
        "songbird.federation.peers" => handle_federation_peers(&state).await,
        "songbird.federation.join" => handle_federation_join(&state, request.params).await,

        // Protocol methods
        "songbird.protocol.capabilities" => handle_protocol_capabilities().await,

        // Health check
        "songbird.health" => handle_health(&state).await,
        "songbird.version" => handle_version().await,

        // biomeOS standard methods (bare names, no prefix)
        "health" => handle_health_standard(&state).await,
        "identity" => handle_identity().await,
        "network.beacon_exchange" => handle_beacon_exchange(request.params).await,

        // Forward to universal-ipc handler (full method table)
        // This makes TCP /jsonrpc equivalent to the Unix socket for inter-gate comms
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

    // Build response (use static string to avoid allocation)
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

// ============================================================================
// Method Handlers
// ============================================================================

/// songbird.services.list
/// Returns list of all registered services from the federated registry
async fn handle_services_list(state: &JsonRpcState) -> Result<Value, JsonRpcError> {
    let services = state.service_registry.get_all_services().await;
    let service_list: Vec<Value> = services
        .iter()
        .map(|svc| {
            serde_json::json!({
                "service_id": svc.service_id,
                "name": svc.service_name,
                "type": svc.service_type,
                "endpoint": svc.endpoint,
                "tower_id": svc.tower_id,
                "capabilities": svc.capabilities,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "services": service_list,
        "count": services.len(),
    }))
}

/// songbird.services.get
/// Get information about a specific service
async fn handle_service_get(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let service_id = match &params {
        Some(Value::Object(map)) => map
            .get("service_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError::invalid_params("Missing 'service_id' parameter"))?
            .to_string(),
        Some(Value::Array(arr)) if !arr.is_empty() => arr[0]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("First parameter must be string"))?
            .to_string(),
        _ => return Err(JsonRpcError::invalid_params("Missing service_id parameter")),
    };

    match state.service_registry.find_by_id(&service_id).await {
        Some(svc) => Ok(serde_json::json!({
            "service_id": svc.service_id,
            "name": svc.service_name,
            "type": svc.service_type,
            "endpoint": svc.endpoint,
            "tower_id": svc.tower_id,
            "tower_name": svc.tower_name,
            "capabilities": svc.capabilities,
            "status": "active",
        })),
        None => Err(JsonRpcError {
            code: -32001,
            message: format!("Service not found: {service_id}"),
            data: None,
        }),
    }
}

/// songbird.services.register
/// Register a new local service via JSON-RPC
async fn handle_service_register(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let params =
        params.ok_or_else(|| JsonRpcError::invalid_params("Missing registration parameters"))?;

    let obj = params
        .as_object()
        .ok_or_else(|| JsonRpcError::invalid_params("Parameters must be an object"))?;

    let service_id = obj
        .get("service_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError::invalid_params("Missing 'service_id'"))?;
    let service_name = obj.get("name").and_then(|v| v.as_str()).unwrap_or(service_id);
    let service_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("generic");
    let endpoint = obj
        .get("endpoint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError::invalid_params("Missing 'endpoint'"))?;
    let capabilities: Vec<String> = obj
        .get("capabilities")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let now = chrono::Utc::now();
    let registration = songbird_network_federation::service_registry::ServiceRegistration {
        service_id: service_id.to_string(),
        service_name: service_name.to_string(),
        service_type: service_type.to_string(),
        tower_id: "local".to_string(),
        tower_name: "local".to_string(),
        endpoint: endpoint.to_string(),
        capabilities,
        health_status: songbird_network_federation::service_registry::ServiceHealthStatus::Healthy,
        registered_at: now,
        last_seen: now,
        metadata: std::collections::HashMap::new(),
    };

    state.service_registry.register_local(registration).await;

    Ok(serde_json::json!({
        "status": "registered",
        "service_id": service_id,
    }))
}

/// `compute.route` / `songbird.compute.schedule` — same handler as `POST /api/compute/task`
async fn handle_compute_route(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError::invalid_params("Missing parameters"))?;
    let req: crate::server::compute_api::ComputeTaskRequest =
        serde_json::from_value(params).map_err(|e| JsonRpcError::invalid_params(e.to_string()))?;
    let json = crate::server::compute_api::submit_compute_task(
        State(state.compute_state.clone()),
        Json(req),
    )
    .await
    .map_err(jsonrpc_from_compute_error)?;
    serde_json::to_value(json.0).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

/// `songbird.compute.status` — same as `GET /api/compute/task/:job_id`
async fn handle_compute_job_status(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let task_id_str = extract_str_param(params.as_ref(), "task_id")?;
    let job_id = uuid::Uuid::parse_str(&task_id_str)
        .map_err(|_| JsonRpcError::invalid_params("task_id must be a UUID"))?;
    let res = crate::server::compute_api::get_task_status(
        State(state.compute_state.clone()),
        Path(job_id),
    )
    .await
    .map_err(jsonrpc_from_compute_error)?;
    serde_json::to_value(res.0).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

fn jsonrpc_from_compute_error(e: crate::server::compute_api::ApiError) -> JsonRpcError {
    use crate::server::compute_api::ApiError;
    match e {
        ApiError::Routing(msg) | ApiError::Execution(msg) => JsonRpcError::internal_error(msg),
        ApiError::InvalidRequest(msg) => JsonRpcError::invalid_params(msg),
        ApiError::NotFound(msg) => JsonRpcError {
            code: -32001,
            message: msg,
            data: None,
        },
    }
}

/// `deployment.create` — same deployment path as `POST /api/deployment/binary` (body as base64 in JSON)
async fn handle_deployment_create(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let obj = params
        .as_ref()
        .and_then(|p| p.as_object())
        .ok_or_else(|| JsonRpcError::invalid_params("Parameters must be an object"))?;
    let b64 = obj
        .get("binary_base64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError::invalid_params("Missing binary_base64"))?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| JsonRpcError::invalid_params(format!("Invalid base64: {e}")))?;
    let service_name = obj.get("service_name").and_then(|v| v.as_str()).map(String::from);
    let env_vars: std::collections::HashMap<String, String> = obj
        .get("env_vars")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let auto_start = obj.get("auto_start").and_then(Value::as_bool).unwrap_or(true);
    let (status, body) = crate::server::deployment_api::deploy_binary_bytes(
        &state.deployment_state,
        axum::body::Bytes::from(raw),
        service_name,
        env_vars,
        auto_start,
    )
    .await
    .map_err(|(code, msg)| JsonRpcError {
        code: jsonrpc_code_from_http_status(code),
        message: msg,
        data: None,
    })?;
    let mut val =
        serde_json::to_value(&body).map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    if let Some(o) = val.as_object_mut() {
        o.insert("http_status".to_string(), (status.as_u16()).into());
    }
    Ok(val)
}

/// `deployment.status` — same as `GET /api/deployment/status/:id`
async fn handle_deployment_status(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let id = extract_str_param(params.as_ref(), "deployment_id")?;
    match crate::server::deployment_api::get_deployment_status(
        State(state.deployment_state.clone()),
        Path(id),
    )
    .await
    {
        Ok(Json(info)) => {
            serde_json::to_value(info).map_err(|e| JsonRpcError::internal_error(e.to_string()))
        }
        Err((code, msg)) => Err(JsonRpcError {
            code: jsonrpc_code_from_http_status(code),
            message: msg,
            data: None,
        }),
    }
}

/// `task.create` — same as `POST /api/v1/tasks`
async fn handle_task_create(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let req: crate::server::task_api::CreateTaskRequest = serde_json::from_value(
        params.ok_or_else(|| JsonRpcError::invalid_params("Missing parameters"))?,
    )
    .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?;
    let owner = crate::task_lifecycle::UserId::from(req.owner);
    let task_id = state
        .task_manager
        .create_task(owner, req.spec)
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    Ok(serde_json::json!({ "task_id": task_id.to_string() }))
}

/// `task.list` — same as `GET /api/v1/tasks`
async fn handle_task_list(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let owner = params
        .as_ref()
        .and_then(|p| p.get("owner"))
        .and_then(|v| v.as_str())
        .map(crate::task_lifecycle::UserId::from);
    let tower = params
        .as_ref()
        .and_then(|p| p.get("tower"))
        .and_then(|v| v.as_str())
        .map(crate::task_lifecycle::TowerId::from);
    let filter = crate::task_lifecycle::TaskFilter {
        owner,
        tower,
        ..Default::default()
    };
    let tasks = state
        .task_manager
        .list_tasks(&filter)
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    let tasks_json: Vec<Value> = tasks
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    Ok(serde_json::json!({ "tasks": tasks_json }))
}

/// `consent.check` — load consent record (`GET /api/consent/:id`)
async fn handle_consent_check(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let id = extract_str_param(params.as_ref(), "consent_id")?;
    let rec = state.consent_manager.get_consent(&id).await.ok_or_else(|| JsonRpcError {
        code: -32001,
        message: "Consent not found".to_string(),
        data: None,
    })?;
    serde_json::to_value(rec).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

/// `consent.grant` — approve consent (`PUT /api/consent/:id` with approve)
async fn handle_consent_grant(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let obj = params
        .as_ref()
        .and_then(|p| p.as_object())
        .ok_or_else(|| JsonRpcError::invalid_params("Parameters must be an object"))?;
    let id = obj
        .get("consent_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError::invalid_params("Missing consent_id"))?;
    let reason = obj.get("reason").and_then(|v| v.as_str()).map(std::sync::Arc::from);
    if !state.consent_manager.approve(id, reason).await {
        return Err(JsonRpcError {
            code: -32001,
            message: "Consent not found".to_string(),
            data: None,
        });
    }
    Ok(serde_json::json!({
        "status": "approved",
        "consent_id": id,
    }))
}

/// `registry.register` — `POST /api/v1/services/register`
async fn handle_registry_register(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let req: crate::service_registry::RegistrationRequest = serde_json::from_value(
        params.ok_or_else(|| JsonRpcError::invalid_params("Missing parameters"))?,
    )
    .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?;
    let response = state
        .upa_registry
        .register(req)
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    serde_json::to_value(response).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

/// `registry.discover` — list services or query by capability (UPA)
async fn handle_registry_discover(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    if let Some(cap) = params.as_ref().and_then(|p| p.get("capability")).and_then(|v| v.as_str()) {
        let services = state.upa_registry.query_by_capability(cap).await;
        Ok(serde_json::json!({
            "capability": cap,
            "services": services,
            "count": services.len(),
        }))
    } else {
        let services = state.upa_registry.list_services().await;
        let stats = state.upa_registry.get_stats().await;
        Ok(serde_json::json!({ "services": services, "stats": stats }))
    }
}

/// `protocol.negotiate` — same as `POST /api/protocol/negotiate`
async fn handle_protocol_negotiate_semantic(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let req: crate::server::protocol_api::NegotiateRequest = serde_json::from_value(
        params.ok_or_else(|| JsonRpcError::invalid_params("Missing parameters"))?,
    )
    .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?;
    let res = crate::server::protocol_api::protocol_negotiate_result(&state.protocol_state, &req);
    serde_json::to_value(res).map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

fn extract_str_param(params: Option<&Value>, key: &str) -> Result<String, JsonRpcError> {
    params
        .and_then(|p| p.as_object())
        .and_then(|m| m.get(key))
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| JsonRpcError::invalid_params(format!("Missing '{key}'")))
}

fn jsonrpc_code_from_http_status(status: StatusCode) -> i32 {
    match status.as_u16() {
        404 => -32001,
        400..=499 => JsonRpcError::INVALID_PARAMS,
        _ => JsonRpcError::INTERNAL_ERROR,
    }
}

/// songbird.federation.peers
/// List active federation peers from the federation state
async fn handle_federation_peers(state: &JsonRpcState) -> Result<Value, JsonRpcError> {
    let nodes = state.federation_state.active_nodes().await;
    let peers: Vec<Value> = nodes
        .iter()
        .map(|node| {
            serde_json::json!({
                "node_id": node.node_id,
                "name": node.node_name,
                "address": node.node_address,
                "endpoints": node.active_endpoints().iter().map(|ep| &ep.address).collect::<Vec<_>>(),
                "capabilities": node.capabilities,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "peers": peers,
        "count": nodes.len(),
    }))
}

/// songbird.federation.join
/// Register this node with the federation
async fn handle_federation_join(
    state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError::invalid_params("Missing join parameters"))?;

    let obj = params
        .as_object()
        .ok_or_else(|| JsonRpcError::invalid_params("Parameters must be an object"))?;

    let node_id = obj
        .get("node_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError::invalid_params("Missing 'node_id'"))?;
    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or(node_id);

    let now = chrono::Utc::now();
    let registration = songbird_network_federation::state::NodeRegistration {
        node_id: node_id.to_string(),
        node_name: name.to_string(),
        node_address: obj.get("address").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        endpoints: None,
        cpu_cores: 0,
        memory_gb: 0,
        gpu_model: None,
        storage_gb: None,
        capabilities: obj
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
        status: songbird_network_federation::state::NodeStatus::Active,
        joined_at: now,
        last_heartbeat: now,
    };

    state.federation_state.register_node(registration).await;

    let stats = state.federation_state.get_stats().await;

    Ok(serde_json::json!({
        "status": "joined",
        "node_id": node_id,
        "active_peers": stats.active_nodes,
    }))
}

/// songbird.protocol.capabilities
/// Get protocol capabilities (same as REST endpoint)
async fn handle_protocol_capabilities() -> Result<Value, JsonRpcError> {
    Ok(serde_json::json!({
        "songbird_version": env!("CARGO_PKG_VERSION"),
        "protocols": {
            "http": {
                "version": "1.1",
                "available": true
            },
            "jsonrpc": {
                "version": "2.0",
                "available": true
            },
            "tarpc": {
                "version": "0.34",
                "available": false,
                "coming_soon": "Week 3-4"
            }
        },
        "preferred_protocol": "jsonrpc",
        "fallback_protocol": "http"
    }))
}

/// songbird.health
/// Health check with real uptime tracking
async fn handle_health(state: &JsonRpcState) -> Result<Value, JsonRpcError> {
    let start_time = state.start_time.read().await;
    let uptime_seconds = start_time.elapsed().as_secs();

    Ok(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime_seconds
    }))
}

/// songbird.version
/// Get Songbird version
async fn handle_version() -> Result<Value, JsonRpcError> {
    Ok(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": "Songbird Universal Orchestrator",
        "architecture": "100% Rust Core + Universal Compatibility"
    }))
}

// ============================================================================
// biomeOS Standard Methods (bare names, no prefix)
// ============================================================================

/// health - biomeOS-standard health check
///
/// Returns health status with uptime and `BearDog` connectivity.
/// This is the bare `health` method as required by biomeOS Neural API.
async fn handle_health_standard(state: &JsonRpcState) -> Result<Value, JsonRpcError> {
    let start_time = state.start_time.read().await;
    let uptime_seconds = start_time.elapsed().as_secs();

    // Check BearDog connectivity (best effort)
    let beardog_connected = {
        let beardog_socket = std::env::var("BEARDOG_SOCKET")
            .or_else(|_| std::env::var("CRYPTO_PROVIDER_SOCKET"))
            .unwrap_or_else(|_| "/tmp/biomeos/beardog.sock".to_string());
        std::path::Path::new(&beardog_socket).exists()
    };

    Ok(serde_json::json!({
        "status": "healthy",
        "uptime_seconds": uptime_seconds,
        "beardog_connected": beardog_connected,
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// identity - Return Songbird's identity and capabilities
///
/// Required by biomeOS Neural API for primal identification.
async fn handle_identity() -> Result<Value, JsonRpcError> {
    // Get family ID from canonical env_config (proper env chain, default: "default")
    let family_id = crate::env_config::family_id();

    // Songbird's capabilities for biomeOS integration
    let capabilities = vec![
        "network.broadcast",
        "network.listen",
        "network.beacon_exchange",
        "encrypt_discovery",
        "decrypt_discovery",
        "http.post",
        "http.get",
        "http.request",
        "discovery.announce",
        "discovery.query",
        "security.verify",
    ];

    Ok(serde_json::json!({
        "primal": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "family_id": family_id,
        "capabilities": capabilities
    }))
}

/// `network.beacon_exchange` - Exchange encrypted beacons with peers
///
/// Enables secure discovery between family members across networks.
async fn handle_beacon_exchange(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError::invalid_params("Missing params"))?;

    // Extract beacon from params
    let beacon = params
        .get("beacon")
        .ok_or_else(|| JsonRpcError::invalid_params("Missing 'beacon' parameter"))?;

    // Extract optional peer info
    let peer_address = params.get("peer_address").and_then(|v| v.as_str());

    debug!("📡 Beacon exchange request received");
    if let Some(addr) = peer_address {
        debug!("   Peer: {}", addr);
    }

    // For now, acknowledge receipt - full implementation requires ConnectionManager
    // which is not available in HTTP context without significant refactoring
    Ok(serde_json::json!({
        "status": "received",
        "beacon_size": beacon.to_string().len(),
        "peer_address": peer_address,
        "message": "Beacon received. Full peer discovery available via IPC socket."
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_error_codes() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_jsonrpc_error_creation() {
        let error = JsonRpcError::method_not_found("test.method");
        assert_eq!(error.code, JsonRpcError::METHOD_NOT_FOUND);
        assert!(error.message.contains("test.method"));
    }

    #[test]
    fn test_jsonrpc_request_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "method": "songbird.health",
            "id": 1
        }"#;

        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "songbird.health");
        assert!(request.id.is_some());
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
            id: Value::Number(1.into()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }
}
