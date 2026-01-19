//! Pure Rust JSON-RPC 2.0 Request Handler
//!
//! Manual implementation of JSON-RPC request handling using only Rust standard library
//! and `serde_json`. This eliminates heavy RPC libraries and their C dependencies.
//!
//! Inspired by BearDog's proven production implementation.
//!
//! ## Architecture
//! - ✅ Simple routing: Method name → Handler function
//! - ✅ Type-safe: Compile-time checks for all handlers
//! - ✅ Async/await: Modern Rust concurrency
//! - ✅ Error handling: Proper JSON-RPC error codes
//! - ✅ Extensible: Easy to add new methods

use super::pure_jsonrpc_types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, warn};

/// Handle a JSON-RPC 2.0 request
///
/// This is the main entry point for all JSON-RPC requests. It validates the request,
/// routes to the appropriate handler, and builds the response.
///
/// # Arguments
/// * `request` - The JSON-RPC request to handle
///
/// # Returns
/// A JSON-RPC response with either a result or an error
///
/// # Philosophy
/// - Validate early: Check JSON-RPC version before routing
/// - Fail gracefully: Return proper error codes
/// - Log appropriately: Debug for requests, warn for errors
pub async fn handle_jsonrpc_request(request: &JsonRpcRequest) -> JsonRpcResponse {
    debug!("→ JSON-RPC Request: method={}", request.method);

    // Validate JSON-RPC version
    if request.jsonrpc != "2.0" {
        warn!("Invalid JSON-RPC version: {}", request.jsonrpc);
        return JsonRpcResponse::error(
            JsonRpcError::invalid_request("Invalid JSON-RPC version (must be 2.0)"),
            request.id.clone().unwrap_or(serde_json::Value::Null),
        );
    }

    // Route to handler
    let result = route_method(&request.method, request.params.as_ref()).await;

    // Build response
    let id = request.id.clone().unwrap_or(serde_json::Value::Null);
    match result {
        Ok(value) => {
            debug!("✅ JSON-RPC Success: method={}", request.method);
            JsonRpcResponse::success(value, id)
        }
        Err(e) => {
            warn!("❌ JSON-RPC Error: method={}, error={}", request.method, e);
            
            // Detect error type and use appropriate error code
            let error = if e.contains("Unknown method") || e.contains("Method not found") {
                JsonRpcError::method_not_found(&request.method)
            } else if e.contains("Invalid params") || e.contains("Missing required") {
                JsonRpcError::invalid_params(e)
            } else if e.contains("Unauthorized") {
                JsonRpcError::unauthorized(e)
            } else if e.contains("Forbidden") {
                JsonRpcError::forbidden(e)
            } else if e.contains("Not found") {
                JsonRpcError::not_found(e)
            } else if e.contains("Timeout") || e.contains("timed out") {
                JsonRpcError::timeout(e)
            } else {
                JsonRpcError::internal_error(e)
            };
            
            JsonRpcResponse::error(error, id)
        }
    }
}

/// Route method to appropriate handler
///
/// This function is the core of the JSON-RPC routing logic. It maps method names
/// to handler functions and extracts parameters.
///
/// # Arguments
/// * `method` - The method name to route
/// * `params` - Optional parameters for the method
///
/// # Returns
/// Result value or error string
async fn route_method(
    method: &str,
    params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    match method {
        // Health & diagnostics
        "ping" => handle_ping(params).await,
        "health" => handle_health(params).await,
        "version" => handle_version(params).await,
        
        // Service discovery
        "discover_services" => handle_discover_services(params).await,
        "register_service" => handle_register_service(params).await,
        "unregister_service" => handle_unregister_service(params).await,
        "list_services" => handle_list_services(params).await,
        
        // Connection management
        "get_connection_status" => handle_get_connection_status(params).await,
        "list_connections" => handle_list_connections(params).await,
        
        // Configuration
        "get_config" => handle_get_config(params).await,
        "validate_config" => handle_validate_config(params).await,
        
        // Metrics
        "get_metrics" => handle_metrics(params).await,
        
        // Unknown method
        _ => Err(format!("Unknown method: {}", method)),
    }
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Handle ping request
async fn handle_ping(_params: Option<&serde_json::Value>) -> Result<serde_json::Value, String> {
    Ok(json!({
        "pong": true,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Handle health check request
async fn handle_health(_params: Option<&serde_json::Value>) -> Result<serde_json::Value, String> {
    Ok(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime_seconds": 0, // TODO: Track actual uptime
    }))
}

/// Handle version request
async fn handle_version(_params: Option<&serde_json::Value>) -> Result<serde_json::Value, String> {
    Ok(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": env!("CARGO_PKG_NAME"),
        "architecture": "UniBin",
        "ecobin_compliance": "98%",
    }))
}

/// Handle discover services request
async fn handle_discover_services(
    _params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement service discovery
    Ok(json!({
        "services": [],
        "discovered_at": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Handle register service request
async fn handle_register_service(
    params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let params = params.ok_or("Missing required params")?;
    
    // Extract service name (example - adapt to actual needs)
    let _service_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing required param: name")?;
    
    // TODO: Implement actual registration
    Ok(json!({
        "registered": true,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Handle unregister service request
async fn handle_unregister_service(
    params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let params = params.ok_or("Missing required params")?;
    
    let _service_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing required param: name")?;
    
    // TODO: Implement actual unregistration
    Ok(json!({
        "unregistered": true,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Handle list services request
async fn handle_list_services(
    _params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement actual service listing
    Ok(json!({
        "services": [],
        "count": 0,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Handle get connection status request
async fn handle_get_connection_status(
    _params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement actual connection status
    Ok(json!({
        "status": "connected",
        "active_connections": 0,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Handle list connections request
async fn handle_list_connections(
    _params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement actual connection listing
    Ok(json!({
        "connections": [],
        "count": 0,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Handle get config request
async fn handle_get_config(
    _params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement actual config retrieval
    Ok(json!({
        "config": {},
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Handle validate config request
async fn handle_validate_config(
    _params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement actual config validation
    Ok(json!({
        "valid": true,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Handle metrics request
async fn handle_metrics(
    _params: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // TODO: Implement actual metrics
    Ok(json!({
        "requests_total": 0,
        "requests_success": 0,
        "requests_error": 0,
        "uptime_seconds": 0,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping_handler() {
        let result = handle_ping(None).await.unwrap();
        assert_eq!(result["pong"], true);
        assert!(result["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_health_handler() {
        let result = handle_health(None).await.unwrap();
        assert_eq!(result["status"], "healthy");
    }

    #[tokio::test]
    async fn test_version_handler() {
        let result = handle_version(None).await.unwrap();
        assert!(result["version"].is_string());
        assert_eq!(result["architecture"], "UniBin");
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let result = route_method("unknown_method", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown method"));
    }

    #[tokio::test]
    async fn test_jsonrpc_request_handling() {
        let req = JsonRpcRequest::new("ping", None, 1);
        let resp = handle_jsonrpc_request(&req).await;
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[tokio::test]
    async fn test_invalid_jsonrpc_version() {
        let mut req = JsonRpcRequest::new("ping", None, 1);
        req.jsonrpc = "1.0".to_string();
        let resp = handle_jsonrpc_request(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, JsonRpcError::INVALID_REQUEST);
    }

    #[tokio::test]
    async fn test_method_not_found_error() {
        let req = JsonRpcRequest::new("nonexistent", None, 1);
        let resp = handle_jsonrpc_request(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, JsonRpcError::METHOD_NOT_FOUND);
    }
}

