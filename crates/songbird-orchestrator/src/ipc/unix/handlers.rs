//! JSON-RPC Method Handlers
//!
//! This module implements all JSON-RPC method handlers for inter-primal communication.
//! Each handler is responsible for a specific capability or service.
//!
//! ## Handler Categories
//!
//! - **Primal Registration**: Register, unregister, query capabilities
//! - **Discovery**: Peer discovery, peer listing, peer health
//! - **HTTP Delegation**: Forward HTTP/HTTPS requests to external services
//! - **Capability Discovery**: Announce Songbird's capabilities to other primals

use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::jsonrpc::JsonRpcError;
use crate::app::connection_manager::ConnectionManager;
use crate::ipc::primal_registry::PrimalRegistry;
use songbird_http_client::SongbirdHttpClient;

// ============================================================================
// PRIMAL REGISTRATION HANDLERS
// ============================================================================

/// Handle primal.register - Register a primal with capabilities
pub async fn handle_primal_register(
    registry: Arc<RwLock<PrimalRegistry>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct RegisterParams {
        primal_id: String,
        capabilities: Vec<String>,
        endpoint: Option<String>,
        metadata: Option<serde_json::Map<String, Value>>,
    }
    
    let params: RegisterParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    let info = crate::ipc::primal_registry::PrimalInfo {
        primal_id: params.primal_id.clone(),
        capabilities: params.capabilities,
        endpoint: params.endpoint,
        metadata: params.metadata.unwrap_or_default(),
    };
    
    let mut reg = registry.write().await;
    reg.register(info).await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    info!("✅ Registered primal: {}", params.primal_id);
    
    Ok(serde_json::json!({
        "success": true,
        "primal_id": params.primal_id
    }))
}

/// Handle primal.unregister - Unregister a primal
pub async fn handle_primal_unregister(
    registry: Arc<RwLock<PrimalRegistry>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct UnregisterParams {
        primal_id: String,
    }
    
    let params: UnregisterParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    let mut reg = registry.write().await;
    reg.unregister(&params.primal_id).await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    info!("✅ Unregistered primal: {}", params.primal_id);
    
    Ok(serde_json::json!({
        "success": true,
        "primal_id": params.primal_id
    }))
}

/// Handle primal.get_provider - Get a provider for a specific capability
pub async fn handle_get_provider(
    registry: Arc<RwLock<PrimalRegistry>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct GetProviderParams {
        capability: String,
    }
    
    let params: GetProviderParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    let reg = registry.read().await;
    let provider = reg.get_provider(&params.capability).await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    match provider {
        Some(info) => {
            debug!("🎯 Found provider for '{}': {}", params.capability, info.primal_id);
            Ok(serde_json::to_value(&info)
                .map_err(|e| JsonRpcError::internal_error(e.to_string()))?)
        }
        None => {
            debug!("🔍 No provider found for capability: {}", params.capability);
            Ok(Value::Null)
        }
    }
}

/// Handle primal.list_providers - List all providers for a capability
pub async fn handle_list_providers(
    registry: Arc<RwLock<PrimalRegistry>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct ListProvidersParams {
        capability: String,
    }
    
    let params: ListProvidersParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    let reg = registry.read().await;
    let providers = reg.list_providers(&params.capability).await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    debug!("📋 Found {} providers for '{}'", providers.len(), params.capability);
    
    Ok(serde_json::to_value(&providers)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?)
}

/// Handle primal.list_all - List all registered primals
pub async fn handle_list_all_primals(
    registry: Arc<RwLock<PrimalRegistry>>,
) -> Result<Value, JsonRpcError> {
    let reg = registry.read().await;
    let primals = reg.list_all().await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    debug!("📋 Listing {} registered primals", primals.len());
    
    Ok(serde_json::to_value(&primals)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?)
}

// ============================================================================
// HEALTH & DIAGNOSTICS HANDLERS
// ============================================================================

/// Handle primal.health - Get health status
pub async fn handle_health(
    registry: Arc<RwLock<PrimalRegistry>>,
) -> Result<Value, JsonRpcError> {
    let reg = registry.read().await;
    let primal_count = reg.list_all().await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?
        .len();
    
    Ok(serde_json::json!({
        "status": "healthy",
        "registered_primals": primal_count,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Handle primal.ping - Simple ping/pong
pub async fn handle_ping() -> Result<Value, JsonRpcError> {
    Ok(serde_json::json!({
        "pong": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// ============================================================================
// PEER DISCOVERY HANDLERS
// ============================================================================

/// Handle discovery.list_peers - List all discovered peers
pub async fn handle_discovery_list_peers(
    connection_manager: Option<Arc<ConnectionManager>>,
    _params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Peer discovery not initialized"))?;
    
    let peers = manager.get_all_peers().await;
    
    debug!("📡 Discovered {} peers", peers.len());
    
    Ok(serde_json::json!({
        "total": peers.len(),
        "peers": peers
    }))
}

/// Handle discovery.peer_count - Get count of discovered peers
pub async fn handle_discovery_peer_count(
    connection_manager: Option<Arc<ConnectionManager>>,
) -> Result<Value, JsonRpcError> {
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Peer discovery not initialized"))?;
    
    let count = manager.get_peer_count().await;
    
    debug!("📊 Peer count: {}", count);
    
    Ok(serde_json::json!({"count": count}))
}

/// Handle discovery.rejected_peers - Get list of rejected peers (diagnostics)
pub async fn handle_discovery_rejected_peers(
    connection_manager: Option<Arc<ConnectionManager>>,
) -> Result<Value, JsonRpcError> {
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Peer discovery not initialized"))?;
    
    let rejected = manager.get_rejected_peers().await;
    let rejected_list: Vec<_> = rejected.iter()
        .map(|(peer_id, reason)| serde_json::json!({
            "peer_id": peer_id,
            "reason": reason
        }))
        .collect();
    
    debug!("🚫 Rejected {} peers", rejected_list.len());
    
    Ok(serde_json::json!({
        "rejected": rejected_list,
        "total": rejected_list.len()
    }))
}

/// Handle peer.ping - Ping a specific peer
pub async fn handle_peer_ping(
    connection_manager: Option<Arc<ConnectionManager>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Peer discovery not initialized"))?;
    
    let params = params
        .ok_or_else(|| JsonRpcError::invalid_params("Missing params"))?;
    
    let target: String = params.get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError::invalid_params("Missing 'target' parameter"))?
        .to_string();
    
    // Check if peer exists
    let peer = manager.get_peer_metadata(&target).await
        .ok_or_else(|| JsonRpcError::internal_error(&format!("Peer '{}' not found", target)))?;
    
    // Measure latency (actual ping would go here in future)
    let start = std::time::Instant::now();
    
    // For now, just verify we have the peer in our metadata
    // TODO: Add actual RPC call to peer's endpoint
    
    let latency_ms = start.elapsed().as_millis() as u64;
    
    debug!("🏓 Pinged peer '{}' ({} ms)", target, latency_ms);
    
    Ok(serde_json::json!({
        "pong": true,
        "peer_id": target,
        "endpoint": peer.endpoint,
        "latency_ms": latency_ms,
        "trust_level": peer.trust_level.as_u8()
    }))
}

/// Handle discovery.status - Get complete discovery status and statistics
///
/// NEW (Jan 5, 2026): Provides observability without relying on logs.
/// Critical for when Tower redirects stdout/stderr to /dev/null.
pub async fn handle_discovery_status(
    discovery_status_manager: Option<Arc<songbird_discovery::DiscoveryStatusManager>>,
) -> Result<Value, JsonRpcError> {
    let manager = discovery_status_manager
        .ok_or_else(|| JsonRpcError::internal_error("Discovery status manager not initialized"))?;
    
    let status = manager.get_status().await;
    
    debug!("📊 Discovery status: enabled={}, running={}, broadcasts={}, peers={}", 
        status.enabled, status.running, status.stats.broadcasts_sent, status.stats.peers_active);
    
    // Convert to JSON
    Ok(serde_json::to_value(status)
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to serialize status: {}", e)))?)
}

// ============================================================================
// CAPABILITY DISCOVERY HANDLERS
// ============================================================================

/// Handle discover_capabilities - Return Songbird's capabilities
///
/// NEW (Jan 20, 2026): Upstream integration from biomeOS.
/// Allows Squirrel to discover that Songbird provides HTTP delegation.
///
/// **Response Format**:
/// ```json
/// {
///   "capabilities": ["http.post", "http.get", "http.request", "discovery.announce", "discovery.query"],
///   "metadata": {
///     "primal_name": "songbird",
///     "version": "4.3.0",
///     "family_id": "nat0"
///   }
/// }
/// ```
pub async fn handle_discover_capabilities() -> Result<Value, JsonRpcError> {
    info!("🔍 Capability discovery request received");
    
    // Get family ID from environment or use default
    let family_id = std::env::var("SONGBIRD_FAMILY_ID").unwrap_or_else(|_| "nat0".to_string());
    
    // Songbird's capabilities for inter-primal communication
    let capabilities = vec![
        "http.post",           // POST requests
        "http.get",            // GET requests
        "http.request",        // Generic HTTP requests
        "discovery.announce",  // Service announcement
        "discovery.query",     // Service discovery
        "security.verify",     // JWT verification (via BearDog delegation)
    ];
    
    Ok(serde_json::json!({
        "capabilities": capabilities,
        "metadata": {
            "primal_name": "songbird",
            "version": env!("CARGO_PKG_VERSION"),
            "family_id": family_id
        }
    }))
}

// ============================================================================
// HTTP DELEGATION HANDLERS
// ============================================================================

/// Handle http.request - Delegate HTTP requests to external services
///
/// NEW (Jan 20, 2026): Upstream integration from biomeOS.
/// Enables Squirrel's Anthropic adapter to delegate HTTP requests through Songbird.
///
/// **Request Format**:
/// ```json
/// {
///   "method": "POST",
///   "url": "https://api.anthropic.com/v1/messages",
///   "headers": {
///     "anthropic-version": "2023-06-01",
///     "content-type": "application/json",
///     "x-api-key": "sk-ant-..."
///   },
///   "body": { ... }
/// }
/// ```
///
/// **Response Format**:
/// ```json
/// {
///   "status": 200,
///   "headers": { "content-type": "application/json" },
///   "body": { ... }
/// }
/// ```
pub async fn handle_http_request(params: Option<Value>) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct HttpRequestParams {
        method: String,
        url: String,
        #[serde(default)]
        headers: std::collections::HashMap<String, String>,
        #[serde(default)]
        body: Option<Value>,
    }
    
    let params: HttpRequestParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    info!("🌐 HTTP delegation (Pure Rust): {} {}", params.method, params.url);
    
    // ✅ NEW: Use Pure Rust HTTP client with capability-based crypto discovery (TRUE PRIMAL)
    let crypto_socket = crate::primal_discovery::discover_crypto_provider().await
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to discover crypto provider: {}", e)))?;
    
    let client = SongbirdHttpClient::new(crypto_socket);
    
    // Make request via Pure Rust client
    let response = client
        .request(
            &params.method,
            &params.url,
            params.headers,
            params.body,
        )
        .await
        .map_err(|e| JsonRpcError::internal_error(&format!("HTTP request failed: {}", e)))?;
    
    info!("✅ HTTP delegation complete (Pure Rust): {} (status: {})", params.url, response.status);
    
    Ok(serde_json::json!({
        "status": response.status,
        "headers": response.headers,
        "body": response.body
    }))
}

