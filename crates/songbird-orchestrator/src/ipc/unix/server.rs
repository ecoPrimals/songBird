// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Unix Socket IPC Server Infrastructure
//!
//! This module implements the core server infrastructure for Unix socket-based
//! inter-primal communication. It provides:
//!
//! - Server lifecycle management (bind, start, stop)
//! - Event-driven readiness notification (NO POLLING!)
//! - Concurrent connection handling
//! - Request routing to handlers
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐     Unix Socket      ┌──────────────┐
//! │   Primal    │────/tmp/songbird.sock│  Songbird    │
//! │  (Client)   │<────JSON-RPC 2.0─────│  (Server)    │
//! └─────────────┘                      └──────────────┘
//! ```

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{Notify, RwLock};
use tracing::{debug, error, info, warn};

use songbird_types::json_rpc_method::{
    DiscoveryMethod, EncryptionDiscoveryMethod, HealthMethod, HttpMethod, JsonRpcMethod,
    NetworkMethod, PeerMethod, PrimalMethod, RpcMethod,
};
use super::handlers;
use super::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::app::connection_manager::ConnectionManager;
use crate::ipc::primal_registry::PrimalRegistry;

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
    
    /// Server start time for uptime tracking (Phase 5A, Feb 4, 2026)
    start_time: Arc<RwLock<std::time::Instant>>,
    
    /// Atomic flag indicating server is ready to accept connections
    /// This allows other components to wait for readiness without polling the filesystem
    is_ready: Arc<AtomicBool>,
    
    /// Event notification for event-driven readiness waiting (NO POLLING!)
    /// ✅ NEW: Eliminates CPU waste from polling loops
    ready_notify: Arc<Notify>,
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
            start_time: Arc::new(RwLock::new(std::time::Instant::now())),  // ✅ NEW: Track startup time (Phase 5A)
            is_ready: Arc::new(AtomicBool::new(false)),
            ready_notify: Arc::new(Notify::new()),  // ✅ NEW: Event notification
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
    
    /// Get a clone of the start time for uptime calculation
    ///
    /// NEW (Phase 5A, Feb 4, 2026): Enables real uptime tracking in health checks
    pub fn start_time(&self) -> Arc<RwLock<std::time::Instant>> {
        Arc::clone(&self.start_time)
    }
    
    /// Check if the server is ready to accept connections
    /// 
    /// This is an atomic, lock-free operation that can be safely called
    /// from any thread without blocking.
    pub fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::Acquire)
    }
    
    /// Wait for the server to be ready (EVENT-DRIVEN, NO POLLING!)
    /// 
    /// ✅ NEW: Uses event notification instead of polling
    /// ⚡ Performance: ~1000x better (no CPU waste)
    /// 🔒 Thread-safe: Multiple waiters supported
    /// 
    /// This is a non-blocking async wait that gets notified when ready.
    /// NO polling, NO busy-waiting, NO CPU waste!
    pub async fn wait_ready(&self, timeout: std::time::Duration) -> bool {
        // Fast path: Already ready?
        if self.is_ready() {
            return true;
        }
        
        // Wait for notification with timeout
        tokio::select! {
            _ = self.ready_notify.notified() => {
                // Got notified, server is ready!
                true
            }
            _ = tokio::time::sleep(timeout) => {
                // Timeout - check one more time in case of race
                self.is_ready()
            }
        }
    }
    
    /// Wait for readiness using notification (EVENT-DRIVEN)
    ///
    /// ✅ NEW: Event-driven wait instead of polling
    /// 
    /// This is a standalone function for use after the server has been moved.
    pub async fn wait_ready_notify(
        notify: &Arc<Notify>,
        flag: &Arc<AtomicBool>,
        timeout: std::time::Duration
    ) -> bool {
        // Fast path: Already ready?
        if flag.load(Ordering::Acquire) {
            return true;
        }
        
        // Wait for notification with timeout
        tokio::select! {
            _ = notify.notified() => {
                // Got notified!
                true
            }
            _ = tokio::time::sleep(timeout) => {
                // Timeout - check one more time
                flag.load(Ordering::Acquire)
            }
        }
    }
    
    /// Get the readiness notification handle
    /// 
    /// Use this to wait for readiness after moving the server.
    pub fn ready_notify(&self) -> Arc<Notify> {
        Arc::clone(&self.ready_notify)
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
        
        // ✅ NEW: Notify all waiters that server is ready (EVENT-DRIVEN!)
        self.ready_notify.notify_waiters();
        
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
                    let start_time = Arc::clone(&self.start_time);
                    
                    // Spawn a task to handle this connection concurrently
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, registry, conn_mgr, disc_status, start_time).await {
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
    start_time: Arc<RwLock<std::time::Instant>>,
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
                
                let is_notification = request.id.is_none();
                let response = handle_request(request, Arc::clone(&registry), connection_manager.clone(), discovery_status_manager.clone(), Arc::clone(&start_time)).await;
                
                if !is_notification {
                    send_response(&mut writer, &response).await?;
                }
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
    start_time: Arc<RwLock<std::time::Instant>>,
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
    
    // Route to appropriate handler (raw wire names; bare `"health"` stays distinct from `health.check`)
    let method = match JsonRpcMethod::from_wire_str(request.method.as_str()) {
        Ok(m) => m,
        Err(_) => {
            warn!("⚠️  Unknown method: {}", request.method);
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::method_not_found(&request.method)),
                id: request.id.unwrap_or(Value::Null),
            };
        }
    };

    let result = match method {
        // ========================================================================
        // biomeOS Standard Methods (Feb 4, 2026)
        // ========================================================================
        JsonRpcMethod::BiomeOsHealth => handlers::handle_health_standard(Arc::clone(&registry), connection_manager.clone(), Some(Arc::clone(&start_time))).await,
        JsonRpcMethod::Health(HealthMethod::Liveness) => Ok(songbird_universal_ipc::introspection::health_liveness()),
        JsonRpcMethod::Health(HealthMethod::Readiness) => Ok(songbird_universal_ipc::introspection::health_readiness()),
        JsonRpcMethod::Health(HealthMethod::Check) => handlers::handle_health_standard(Arc::clone(&registry), connection_manager.clone(), Some(Arc::clone(&start_time))).await,
        JsonRpcMethod::Identity => handlers::handle_identity().await,
        JsonRpcMethod::Rpc(RpcMethod::Discover) => handlers::handle_rpc_discover().await,
        
        // ========================================================================
        // Primal registration methods
        // ========================================================================
        JsonRpcMethod::Primal(PrimalMethod::Register) => handlers::handle_primal_register(registry, request.params).await,
        JsonRpcMethod::Primal(PrimalMethod::Unregister) => handlers::handle_primal_unregister(registry, request.params).await,
        JsonRpcMethod::Primal(PrimalMethod::GetProvider) => handlers::handle_get_provider(registry, request.params).await,
        JsonRpcMethod::Primal(PrimalMethod::ListProviders) => handlers::handle_list_providers(registry, request.params).await,
        JsonRpcMethod::Primal(PrimalMethod::ListAll) => handlers::handle_list_all_primals(registry).await,
        
        // Health and diagnostics (legacy)
        JsonRpcMethod::Primal(PrimalMethod::Health) => handlers::handle_health(registry).await,
        JsonRpcMethod::Primal(PrimalMethod::Ping) => handlers::handle_ping().await,
        
        // ========================================================================
        // Discovery methods
        // ========================================================================
        JsonRpcMethod::Discovery(DiscoveryMethod::ListPeers) => handlers::handle_discovery_list_peers(connection_manager, request.params).await,
        JsonRpcMethod::Discovery(DiscoveryMethod::PeerCount) => handlers::handle_discovery_peer_count(connection_manager).await,
        JsonRpcMethod::Discovery(DiscoveryMethod::RejectedPeers) => handlers::handle_discovery_rejected_peers(connection_manager).await,
        JsonRpcMethod::Discovery(DiscoveryMethod::Status) => handlers::handle_discovery_status(discovery_status_manager).await,
        JsonRpcMethod::Peer(PeerMethod::Ping) => handlers::handle_peer_ping(connection_manager, request.params).await,
        
        // ========================================================================
        // Capability discovery (legacy - backward compat)
        // ========================================================================
        JsonRpcMethod::DiscoverCapabilities => handlers::handle_discover_capabilities().await,
        
        // ========================================================================
        // Encryption wrappers (biomeOS integration - Feb 4, 2026)
        // ========================================================================
        JsonRpcMethod::EncryptionDiscovery(EncryptionDiscoveryMethod::Encrypt) => handlers::handle_encrypt_discovery(request.params).await,
        JsonRpcMethod::EncryptionDiscovery(EncryptionDiscoveryMethod::Decrypt) => handlers::handle_decrypt_discovery(request.params).await,
        
        // ========================================================================
        // Network methods (biomeOS integration - Feb 4, 2026)
        // ========================================================================
        JsonRpcMethod::Network(NetworkMethod::BeaconExchange) => handlers::handle_beacon_exchange(connection_manager.clone(), request.params).await,
        JsonRpcMethod::Network(NetworkMethod::Broadcast) => handlers::handle_network_broadcast(request.params).await,
        JsonRpcMethod::Network(NetworkMethod::Listen) => handlers::handle_network_listen(request.params).await,
        
        // ========================================================================
        // HTTP delegation (for AI coordination / Anthropic-style adapters — Jan 20, 2026)
        // ========================================================================
        JsonRpcMethod::Http(HttpMethod::Request) => handlers::handle_http_request(request.params).await,
        
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

/// Send a JSON-RPC response to the client (zero intermediate String allocation).
async fn send_response(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    response: &JsonRpcResponse,
) -> Result<()> {
    let bytes = serde_json::to_vec(response)?;
    writer.write_all(&bytes).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
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
}

