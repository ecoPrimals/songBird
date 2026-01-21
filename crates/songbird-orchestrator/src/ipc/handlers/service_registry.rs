//! Service Registry Handlers
//!
//! Handles registration, discovery, and health checking of primal services.
//! v3.20.0: Service registry for capability-based discovery
//! v3.22.1: Extracted to focused module (Jan 12, 2026)
//! v3.34.0: Migrated to Pure Rust types (Phase 4B)

use std::time::SystemTime;
use tracing::{debug, info};

use crate::ipc::handlers::IpcHandlers;
use crate::ipc::pure_rust_server::JsonRpcError;
use crate::ipc::types::{
    system_time_to_iso8601, DiscoverByCapabilityRequest, DiscoverByCapabilityResponse,
    GetServiceHealthRequest, GetServiceHealthResponse, HealthCheckResponse, HealthStatus,
    RegisterServiceRequest, RegisterServiceResponse,
};

// ============================================================================
// jsonrpsee Handlers (Original API)
// ============================================================================

/// Handle `register_service` RPC call
///
/// Registers a primal service with Songbird for capability-based discovery.
///
/// ## Example Request
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "register_service",
///   "params": {
///     "primal_name": "BearDog",
///     "capabilities": ["encryption", "identity"],
///     "endpoint": "/run/user/1000/beardog-nat0.sock",
///     "protocol": "json-rpc",
///     "health_check_interval": 30
///   },
///   "id": 4
/// }
/// ```
pub async fn register_service(
    handlers: &IpcHandlers,
    params: serde_json::Value,
) -> Result<RegisterServiceResponse, JsonRpcError> {
    debug!("📝 IPC: register_service called");

    // Parse request parameters
    let request: RegisterServiceRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError::invalid_params(format!("Invalid params: {}", e)))?;

    info!(
        "📋 Registering service: {} with capabilities: {:?}",
        request.primal_name, request.capabilities
    );

    // Register in service registry
    let service_id = handlers
        .service_registry
        .register_service(
            request.primal_name,
            request.capabilities,
            request.endpoint,
            request.protocol,
            request.health_check_interval,
        )
        .await
        .map_err(|e| {
            JsonRpcError::custom(-32603, "Failed to register service", Some(format!("{}", e)))
        })?;

    info!("✅ Service registered: {}", service_id);

    Ok(RegisterServiceResponse {
        service_id,
        status: "registered".to_string(),
        registered_at: system_time_to_iso8601(SystemTime::now()),
    })
}

/// Handle `discover_by_capability` RPC call
///
/// Discovers primals by capability (e.g., "encryption", "storage", "*" for all).
///
/// ## Example Request
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "discover_by_capability",
///   "params": {
///     "capability": "encryption",
///     "protocol": "json-rpc"
///   },
///   "id": 5
/// }
/// ```
pub async fn discover_by_capability(
    handlers: &IpcHandlers,
    params: serde_json::Value,
) -> Result<DiscoverByCapabilityResponse, JsonRpcError> {
    debug!("🔍 IPC: discover_by_capability called");

    // Parse request parameters
    let request: DiscoverByCapabilityRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError::invalid_params(format!("Invalid params: {}", e)))?;

    info!("🔎 Discovering primals with capability: {}", request.capability);

    // Query service registry
    let primals = handlers
        .service_registry
        .discover_by_capability(&request.capability, request.protocol.as_deref())
        .await
        .map_err(|e| {
            JsonRpcError::custom(-32603, "Failed to discover primals", Some(format!("{}", e)))
        })?;

    info!("   Found {} primals", primals.len());

    Ok(DiscoverByCapabilityResponse {
        primals,
    })
}

/// Handle `get_service_health` RPC call
///
/// Gets the health status of a specific registered service.
///
/// ## Example Request
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "get_service_health",
///   "params": {
///     "service_id": "beardog-12345"
///   },
///   "id": 6
/// }
/// ```
pub async fn get_service_health(
    handlers: &IpcHandlers,
    params: serde_json::Value,
) -> Result<GetServiceHealthResponse, JsonRpcError> {
    debug!("🩺 IPC: get_service_health called");

    // Parse request parameters
    let request: GetServiceHealthRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError::invalid_params(format!("Invalid params: {}", e)))?;

    info!("🏥 Checking health for service: {}", request.service_id);

    // Get health from registry
    let (status, message) =
        handlers.service_registry.get_service_health(&request.service_id).await.map_err(|e| {
            JsonRpcError::custom(-32603, "Failed to get health", Some(format!("{}", e)))
        })?;

    let health = HealthStatus {
        service_id: request.service_id,
        status,
        message,
        timestamp: system_time_to_iso8601(SystemTime::now()),
    };

    Ok(GetServiceHealthResponse {
        health,
    })
}

/// Handle `health_check` RPC call
///
/// Returns Songbird's own health status.
///
/// ## Example Request
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "health_check",
///   "params": {},
///   "id": 7
/// }
/// ```
pub async fn health_check(
    _handlers: &IpcHandlers,
    _params: serde_json::Value,
) -> Result<HealthCheckResponse, JsonRpcError> {
    debug!("💓 IPC: health_check called");

    // Songbird's health is always "healthy" if responding to RPC
    let health = HealthStatus {
        service_id: "songbird".to_string(),
        status: "healthy".to_string(),
        message: None,
        timestamp: system_time_to_iso8601(SystemTime::now()),
    };

    Ok(HealthCheckResponse {
        health,
    })
}

// ============================================================================
// Pure JSON Adapters (v3.22.0)
// ============================================================================

/// Service Registry: register_service (pure JSON adapter)
pub async fn register_service_json(
    handlers: &IpcHandlers,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
    let request: RegisterServiceRequest = match params {
        Some(p) => serde_json::from_value(p).map_err(|e| {
            crate::ipc::pure_rust_server::JsonRpcError::invalid_params(format!(
                "Invalid params: {}",
                e
            ))
        })?,
        None => {
            return Err(crate::ipc::pure_rust_server::JsonRpcError::invalid_params(
                "params required",
            ))
        }
    };

    let response = handlers
        .service_registry
        .register_service(
            request.primal_name,
            request.capabilities,
            request.endpoint,
            request.protocol,
            request.health_check_interval,
        )
        .await
        .map_err(|e| {
            crate::ipc::pure_rust_server::JsonRpcError::internal_error(format!(
                "Failed to register service: {}",
                e
            ))
        })?;

    let resp = RegisterServiceResponse {
        service_id: response,
        status: "registered".to_string(),
        registered_at: system_time_to_iso8601(SystemTime::now()),
    };

    serde_json::to_value(resp).map_err(|e| {
        crate::ipc::pure_rust_server::JsonRpcError::internal_error(format!(
            "Failed to serialize response: {}",
            e
        ))
    })
}

/// Service Registry: discover_by_capability (pure JSON adapter)
pub async fn discover_by_capability_json(
    handlers: &IpcHandlers,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
    let request: DiscoverByCapabilityRequest = match params {
        Some(p) => serde_json::from_value(p).map_err(|e| {
            crate::ipc::pure_rust_server::JsonRpcError::invalid_params(format!(
                "Invalid params: {}",
                e
            ))
        })?,
        None => {
            return Err(crate::ipc::pure_rust_server::JsonRpcError::invalid_params(
                "params required",
            ))
        }
    };

    let primals = handlers
        .service_registry
        .discover_by_capability(&request.capability, request.protocol.as_deref())
        .await
        .map_err(|e| {
            crate::ipc::pure_rust_server::JsonRpcError::internal_error(format!(
                "Failed to discover primals: {}",
                e
            ))
        })?;

    let resp = DiscoverByCapabilityResponse {
        primals,
    };

    serde_json::to_value(resp).map_err(|e| {
        crate::ipc::pure_rust_server::JsonRpcError::internal_error(format!(
            "Failed to serialize response: {}",
            e
        ))
    })
}

/// Service Registry: get_service_health (pure JSON adapter)
pub async fn get_service_health_json(
    handlers: &IpcHandlers,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
    let request: GetServiceHealthRequest = match params {
        Some(p) => serde_json::from_value(p).map_err(|e| {
            crate::ipc::pure_rust_server::JsonRpcError::invalid_params(format!(
                "Invalid params: {}",
                e
            ))
        })?,
        None => {
            return Err(crate::ipc::pure_rust_server::JsonRpcError::invalid_params(
                "params required",
            ))
        }
    };

    let (status, message) =
        handlers.service_registry.get_service_health(&request.service_id).await.map_err(|e| {
            crate::ipc::pure_rust_server::JsonRpcError::internal_error(format!(
                "Failed to get health: {}",
                e
            ))
        })?;

    let health = HealthStatus {
        service_id: request.service_id,
        status,
        message,
        timestamp: system_time_to_iso8601(SystemTime::now()),
    };

    let resp = GetServiceHealthResponse {
        health,
    };

    serde_json::to_value(resp).map_err(|e| {
        crate::ipc::pure_rust_server::JsonRpcError::internal_error(format!(
            "Failed to serialize response: {}",
            e
        ))
    })
}

/// Service Registry: health_check (pure JSON adapter)
pub async fn health_check_json(
    _handlers: &IpcHandlers,
) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
    let health = HealthStatus {
        service_id: "songbird".to_string(),
        status: "healthy".to_string(),
        message: Some("Songbird orchestrator is running".to_string()),
        timestamp: system_time_to_iso8601(SystemTime::now()),
    };

    let resp = HealthCheckResponse {
        health,
    };

    serde_json::to_value(resp).map_err(|e| {
        crate::ipc::pure_rust_server::JsonRpcError::internal_error(format!(
            "Failed to serialize response: {}",
            e
        ))
    })
}
