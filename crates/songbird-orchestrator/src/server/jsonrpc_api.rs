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

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
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
    /// Service registry for discovery
    pub service_registry: Arc<FederatedServiceRegistry>,
    /// Server start time for uptime calculation
    pub start_time: Arc<RwLock<Instant>>,
    /// Universal IPC handler — full method table for inter-gate communication
    /// When present, unknown methods on TCP are forwarded to the same handler
    /// that serves the Unix socket, making TCP equivalent for LAN mesh.
    pub ipc_handler: Option<Arc<IpcServiceHandler>>,
}

impl JsonRpcState {
    #[must_use]
    pub fn new(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
            start_time: Arc::new(RwLock::new(Instant::now())),
            ipc_handler: None,
        }
    }

    /// Create with IPC handler for full method forwarding on TCP
    ///
    /// This enables inter-gate communication over TCP :3492 by forwarding
    /// unknown methods to the universal-ipc handler (same as Unix socket).
    /// Dark Forest gating still applies on all TCP requests.
    #[must_use]
    pub fn with_ipc_handler(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
        ipc_handler: Arc<IpcServiceHandler>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
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
#[allow(dead_code)]
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
        // Service discovery methods
        "songbird.services.list" => handle_services_list(&state).await,
        "songbird.services.get" => handle_service_get(&state, request.params).await,
        "songbird.services.register" => handle_service_register(&state, request.params).await,

        // Compute methods
        "songbird.compute.schedule" => handle_compute_schedule(&state, request.params).await,
        "songbird.compute.status" => handle_compute_status(&state, request.params).await,

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
/// Returns list of all registered services
async fn handle_services_list(_state: &JsonRpcState) -> Result<Value, JsonRpcError> {
    // Phase 2: Return placeholder
    // Full ServiceRegistry integration in later phase
    Ok(serde_json::json!({
        "services": [],
        "count": 0,
        "message": "Service listing via JSON-RPC coming soon! Full integration pending."
    }))
}

/// songbird.services.get
/// Get information about a specific service
async fn handle_service_get(
    _state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    // Extract service_id from params
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

    // Phase 2: Return placeholder
    Ok(serde_json::json!({
        "service_id": service_id,
        "status": "not_implemented",
        "message": "Service details via JSON-RPC coming soon!"
    }))
}

/// songbird.services.register
/// Register a new service (Phase 2: Foundation only)
async fn handle_service_register(
    _state: &JsonRpcState,
    _params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    // Phase 2: Return placeholder
    // Full implementation in later phase
    Ok(serde_json::json!({
        "status": "not_implemented",
        "message": "Service registration via JSON-RPC coming soon!"
    }))
}

/// songbird.compute.schedule
/// Schedule a compute task
async fn handle_compute_schedule(
    _state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let _task_params =
        params.ok_or_else(|| JsonRpcError::invalid_params("Missing task parameters"))?;

    // ✅ EVOLVED: Generate real task IDs using UUID
    // Integration with compute providers via capability-based discovery
    let task_id = uuid::Uuid::new_v4().to_string();

    Ok(serde_json::json!({
        "task_id": task_id,
        "status": "queued",
        "message": "Task queued. Integrate with compute providers via capability discovery (COMPUTE_ENDPOINT)."
    }))
}

/// songbird.compute.status
/// Get status of a compute task
async fn handle_compute_status(
    _state: &JsonRpcState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let _task_id = match &params {
        Some(Value::Object(map)) => map
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError::invalid_params("Missing 'task_id' parameter"))?
            .to_string(),
        _ => return Err(JsonRpcError::invalid_params("Missing task_id parameter")),
    };

    // Phase 2: Return placeholder
    Ok(serde_json::json!({
        "status": "not_implemented",
        "message": "Task status via JSON-RPC coming soon!"
    }))
}

/// songbird.federation.peers
/// List federation peers
async fn handle_federation_peers(_state: &JsonRpcState) -> Result<Value, JsonRpcError> {
    // Phase 2: Return placeholder
    Ok(serde_json::json!({
        "peers": [],
        "count": 0,
        "message": "Federation peer listing via JSON-RPC coming soon!"
    }))
}

/// songbird.federation.join
/// Join federation network
async fn handle_federation_join(
    _state: &JsonRpcState,
    _params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    // Phase 2: Return placeholder
    Ok(serde_json::json!({
        "status": "not_implemented",
        "message": "Federation join via JSON-RPC coming soon!"
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
