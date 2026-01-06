//! Unix Socket IPC Server for Inter-Primal Communication
//!
//! This module implements a Unix socket-based IPC server that allows other primals
//! (BearDog, ToadStool, Gorilla, etc.) to communicate with Songbird for:
//! - Capability-based registration
//! - Peer discovery notifications
//! - Service lookup
//! - Event subscriptions
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐     Unix Socket      ┌──────────────┐
//! │   BearDog   │────/tmp/songbird.sock│  Songbird    │
//! │  (Client)   │<────JSON-RPC 2.0─────│  (Server)    │
//! └─────────────┘                      └──────────────┘
//!        │
//!        │ registers: capabilities=["security", "encryption"]
//!        │ subscribes: events=["peer_discovered"]
//!        │
//!        └─→ Songbird routes requests by capability
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_orchestrator::ipc::UnixSocketIpcServer;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let server = UnixSocketIpcServer::new("/tmp/songbird-nat0.sock").await?;
//! server.start().await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::primal_registry::PrimalRegistry;
use crate::app::connection_manager::ConnectionManager;

/// Unix socket IPC server for inter-primal communication
pub struct UnixSocketIpcServer {
    /// Path to the Unix socket file
    socket_path: PathBuf,
    
    /// Unix socket listener
    listener: Option<UnixListener>,
    
    /// Primal capability registry
    registry: Arc<RwLock<PrimalRegistry>>,
    
    /// Connection manager for peer discovery API (optional)
    connection_manager: Option<Arc<ConnectionManager>>,
    
    /// Discovery status manager for observability (optional, Jan 5, 2026)
    discovery_status_manager: Option<Arc<songbird_discovery::DiscoveryStatusManager>>,
    
    /// Atomic flag indicating server is ready to accept connections
    /// This allows other components to wait for readiness without polling the filesystem
    is_ready: Arc<AtomicBool>,
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    
    /// Error message
    pub message: String,
    
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Standard error codes
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    
    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: Self::PARSE_ERROR,
            message: message.into(),
            data: None,
        }
    }
    
    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_REQUEST,
            message: message.into(),
            data: None,
        }
    }
    
    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: Self::METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }
    
    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }
    
    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: Self::INTERNAL_ERROR,
            message: message.into(),
            data: None,
        }
    }
}

impl UnixSocketIpcServer {
    /// Create a new Unix socket IPC server
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to the Unix socket file (e.g., "/tmp/songbird-nat0.sock")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::UnixSocketIpcServer;
    /// # async fn example() -> anyhow::Result<()> {
    /// let server = UnixSocketIpcServer::new("/tmp/songbird.sock").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(socket_path: impl Into<PathBuf>) -> Result<Self> {
        let socket_path = socket_path.into();
        
        // Remove existing socket file if it exists (handles crashes/unclean shutdowns)
        if socket_path.exists() {
            info!("🧹 Removing existing socket: {}", socket_path.display());
            std::fs::remove_file(&socket_path)
                .context("Failed to remove existing socket")?;
        }
        
        // Create Unix socket listener
        let listener = UnixListener::bind(&socket_path)
            .context("Failed to bind Unix socket")?;
        
        info!("🎧 Unix socket IPC server bound to: {}", socket_path.display());
        
        Ok(Self {
            socket_path,
            listener: Some(listener),
            registry: Arc::new(RwLock::new(PrimalRegistry::new())),
            connection_manager: None, // Initially none, can be set via set_connection_manager()
            discovery_status_manager: None, // Initially none, can be set via set_discovery_status_manager()
            is_ready: Arc::new(AtomicBool::new(false)),
        })
    }
    
    /// Set the connection manager (for peer discovery APIs)
    pub fn set_connection_manager(&mut self, manager: Arc<ConnectionManager>) {
        self.connection_manager = Some(manager);
    }
    
    /// Set the discovery status manager (for discovery observability)
    pub fn set_discovery_status_manager(&mut self, manager: Arc<songbird_discovery::DiscoveryStatusManager>) {
        self.discovery_status_manager = Some(manager);
        info!("🔗 Wired DiscoveryStatusManager into IPC server for discovery.status API");
    }
    
    /// Get the socket path
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
    
    /// Get a reference to the primal registry
    pub fn registry(&self) -> Arc<RwLock<PrimalRegistry>> {
        Arc::clone(&self.registry)
    }
    
    /// Get a clone of the readiness flag
    ///
    /// This allows checking readiness even after the server has been moved
    /// into a spawn task.
    pub fn readiness_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_ready)
    }
    
    /// Check if the server is ready to accept connections
    /// 
    /// This is an atomic, lock-free operation that can be safely called
    /// from any thread without blocking.
    pub fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::Acquire)
    }
    
    /// Wait for the server to be ready
    /// 
    /// This is a non-blocking async wait that checks readiness without
    /// filesystem polling. Use this instead of `sleep` loops!
    pub async fn wait_ready(&self, timeout: std::time::Duration) -> bool {
        let start = std::time::Instant::now();
        while !self.is_ready() {
            if start.elapsed() > timeout {
                return false;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        true
    }
    
    /// Wait for readiness using a readiness flag
    ///
    /// This is a standalone function for use after the server has been moved.
    pub async fn wait_ready_flag(flag: &Arc<AtomicBool>, timeout: std::time::Duration) -> bool {
        let start = std::time::Instant::now();
        while !flag.load(Ordering::Acquire) {
            if start.elapsed() > timeout {
                return false;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        true
    }
    
    /// Start the IPC server
    ///
    /// This will accept connections and handle JSON-RPC requests in a loop.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::ipc::UnixSocketIpcServer;
    /// # async fn example() -> anyhow::Result<()> {
    /// let server = UnixSocketIpcServer::new("/tmp/songbird.sock").await?;
    /// server.start().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start(mut self) -> Result<()> {
        let listener = self.listener.take()
            .context("Server already started or listener not initialized")?;
        
        // Mark server as ready atomically (no locks needed!)
        self.is_ready.store(true, Ordering::Release);
        
        info!("🚀 Unix socket IPC server starting...");
        info!("   Socket: {}", self.socket_path.display());
        info!("   Protocol: JSON-RPC 2.0");
        info!("   Status: READY ✅");
        info!("   Listening for primal connections...");
        
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let registry = Arc::clone(&self.registry);
                    let conn_mgr = self.connection_manager.clone();
                    let disc_status = self.discovery_status_manager.clone();
                    
                    // Spawn a task to handle this connection concurrently
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, registry, conn_mgr, disc_status).await {
                            error!("❌ Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("❌ Error accepting connection: {}", e);
                }
            }
        }
    }
    
    /// Stop the IPC server gracefully
    ///
    /// Cleans up the socket file. The actual server loop will continue
    /// until the task is cancelled by the orchestrator.
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Unix socket IPC server...");
        
        // Mark as not ready
        self.is_ready.store(false, Ordering::Release);
        
        // Remove socket file
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .context("Failed to remove socket file")?;
            info!("🧹 Removed socket: {}", self.socket_path.display());
        }
        
        Ok(())
    }
}

/// Handle a single Unix socket connection
async fn handle_connection(
    stream: UnixStream,
    registry: Arc<RwLock<PrimalRegistry>>,
    connection_manager: Option<Arc<ConnectionManager>>,
    discovery_status_manager: Option<Arc<songbird_discovery::DiscoveryStatusManager>>,
) -> Result<()> {
    debug!("📞 New primal connection established");
    
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    
    loop {
        line.clear();
        
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // Connection closed
                debug!("👋 Primal disconnected");
                break;
            }
            Ok(n) => {
                debug!("📥 Received {} bytes", n);
                
                // Parse JSON-RPC request
                let request: JsonRpcRequest = match serde_json::from_str(&line) {
                    Ok(req) => req,
                    Err(e) => {
                        warn!("⚠️  Invalid JSON-RPC request: {}", e);
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError::parse_error(e.to_string())),
                            id: Value::Null,
                        };
                        
                        send_response(&mut writer, &response).await?;
                        continue;
                    }
                };
                
                // Handle request
                let response = handle_request(request, Arc::clone(&registry), connection_manager.clone(), discovery_status_manager.clone()).await;
                
                // Send response
                send_response(&mut writer, &response).await?;
            }
            Err(e) => {
                error!("❌ Error reading from stream: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

/// Handle a JSON-RPC request
async fn handle_request(
    request: JsonRpcRequest,
    registry: Arc<RwLock<PrimalRegistry>>,
    connection_manager: Option<Arc<ConnectionManager>>,
    discovery_status_manager: Option<Arc<songbird_discovery::DiscoveryStatusManager>>,
) -> JsonRpcResponse {
    debug!("🔧 Handling method: {}", request.method);
    
    // Validate JSON-RPC version
    if request.jsonrpc != "2.0" {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError::invalid_request("jsonrpc must be '2.0'")),
            id: request.id.unwrap_or(Value::Null),
        };
    }
    
    // Route to appropriate handler
    let result = match request.method.as_str() {
        // Primal registration methods
        "primal.register" => handle_primal_register(registry, request.params).await,
        "primal.unregister" => handle_primal_unregister(registry, request.params).await,
        "primal.get_provider" => handle_get_provider(registry, request.params).await,
        "primal.list_providers" => handle_list_providers(registry, request.params).await,
        "primal.list_all" => handle_list_all_primals(registry).await,
        
        // Health and diagnostics
        "primal.health" => handle_health(registry).await,
        "primal.ping" => handle_ping().await,
        
        // Discovery methods (NEW!)
        "discovery.list_peers" => handle_discovery_list_peers(connection_manager, request.params).await,
        "discovery.peer_count" => handle_discovery_peer_count(connection_manager).await,
        "discovery.rejected_peers" => handle_discovery_rejected_peers(connection_manager).await,
        "discovery.status" => handle_discovery_status(discovery_status_manager).await,
        "peer.ping" => handle_peer_ping(connection_manager, request.params).await,
        
        // Unknown method
        _ => {
            warn!("⚠️  Unknown method: {}", request.method);
            Err(JsonRpcError::method_not_found(&request.method))
        }
    };
    
    // Build response
    match result {
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
    }
}

/// Send a JSON-RPC response to the client
async fn send_response(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    response: &JsonRpcResponse,
) -> Result<()> {
    let json = serde_json::to_string(response)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

// Method handlers

/// Handle primal.register
async fn handle_primal_register(
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

/// Handle primal.unregister
async fn handle_primal_unregister(
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

/// Handle primal.get_provider
async fn handle_get_provider(
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

/// Handle primal.list_providers
async fn handle_list_providers(
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

/// Handle primal.list_all
async fn handle_list_all_primals(
    registry: Arc<RwLock<PrimalRegistry>>,
) -> Result<Value, JsonRpcError> {
    let reg = registry.read().await;
    let primals = reg.list_all().await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    
    debug!("📋 Listing {} registered primals", primals.len());
    
    Ok(serde_json::to_value(&primals)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?)
}

/// Handle primal.health
async fn handle_health(
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

/// Handle primal.ping
async fn handle_ping() -> Result<Value, JsonRpcError> {
    Ok(serde_json::json!({
        "pong": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Handle discovery.list_peers - List all discovered peers
async fn handle_discovery_list_peers(
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
async fn handle_discovery_peer_count(
    connection_manager: Option<Arc<ConnectionManager>>,
) -> Result<Value, JsonRpcError> {
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Peer discovery not initialized"))?;
    
    let count = manager.get_peer_count().await;
    
    debug!("📊 Peer count: {}", count);
    
    Ok(serde_json::json!({"count": count}))
}

/// Handle discovery.rejected_peers - Get list of rejected peers (diagnostics)
async fn handle_discovery_rejected_peers(
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
async fn handle_peer_ping(
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
async fn handle_discovery_status(
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_server_creation() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test.sock");
        
        let server = UnixSocketIpcServer::new(socket_path.clone()).await.unwrap();
        assert_eq!(server.socket_path(), socket_path);
    }
    
    #[tokio::test]
    async fn test_jsonrpc_error_codes() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
    }
}

