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
use tracing::{info, warn};

/// JSON-RPC 2.0 routes
pub fn jsonrpc_routes() -> Router<JsonRpcState> {
    Router::new()
        .route("/", post(handle_jsonrpc_request))
        .route("/rpc", post(handle_jsonrpc_request))
}

use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
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
}

impl JsonRpcState {
    pub fn new(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
            start_time: Arc::new(RwLock::new(Instant::now())),
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
    if request.jsonrpc != "2.0" {
        return Ok(Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError::invalid_request("jsonrpc must be '2.0'")),
            id: request.id.unwrap_or(Value::Null),
        }));
    }

    info!("📞 JSON-RPC request: method={}", request.method);

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

        // Unknown method
        _ => {
            warn!("⚠️  Unknown JSON-RPC method: {}", request.method);
            Err(JsonRpcError::method_not_found(&request.method))
        }
    };

    // Build response
    let response = match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(value),
            error: None,
            id: request.id.unwrap_or(Value::Null),
        },
        Err(error) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
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
