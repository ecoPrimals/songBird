//! STUN server & client JSON-RPC handler
//!
//! Provides JSON-RPC methods for NAT traversal via STUN (RFC 5389).
//!
//! **Server Methods**:
//! - `stun.serve` - Start STUN server
//! - `stun.stop` - Stop STUN server
//! - `stun.status` - Get server status
//!
//! **Client Methods** (NAT Traversal):
//! - `stun.get_public_address` - Discover public IP/port via external STUN servers
//! - `stun.bind` - Bind local port and discover NAT mapping

use serde_json::{json, Value};
use songbird_stun::{StunClient, StunServer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// STUN server handler for JSON-RPC integration
///
/// Manages the lifecycle of the integrated STUN server and provides
/// status information via JSON-RPC methods.
///
/// ## Design Principles
///
/// - **Self-Contained**: No external primal dependencies
/// - **Capability-Based**: Exposes capability, not implementation
/// - **Safe**: All operations use safe Rust
/// - **Idiomatic**: Modern async/await patterns
#[derive(Debug)]
pub struct StunHandler {
    /// Currently running server instance
    server_handle: Arc<RwLock<Option<ServerInstance>>>,
}

#[derive(Debug)]
struct ServerInstance {
    /// Tokio task handle for the running server
    handle: JoinHandle<()>,

    /// Bind address the server is listening on
    bind_addr: SocketAddr,

    /// Server start time
    start_time: std::time::Instant,
}

impl StunHandler {
    /// Create new STUN handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            server_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Handle `stun.serve` method - Start STUN server
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "stun.serve",
    ///   "params": {
    ///     "bind_addr": "0.0.0.0:3478"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    ///
    /// # Response Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "result": {
    ///     "status": "started",
    ///     "bind_addr": "0.0.0.0:3478",
    ///     "comment": "STUN server running in background"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_serve(&self, params: Value) -> Result<Value, String> {
        // Check if server is already running
        {
            let instance = self.server_handle.read().await;
            if instance.is_some() {
                return Err("STUN server is already running (use stun.stop first)".to_string());
            }
        }

        // Parse bind address from params (default to standard STUN port)
        let bind_addr_str =
            params.get("bind_addr").and_then(|v| v.as_str()).unwrap_or("0.0.0.0:3478");

        let bind_addr: SocketAddr = bind_addr_str
            .parse()
            .map_err(|e| format!("Invalid bind address '{bind_addr_str}': {e}"))?;

        info!("🌐 Starting STUN server on {}", bind_addr);

        // Create server
        let mut server = StunServer::new(bind_addr);

        // Spawn server in background
        let handle = tokio::spawn(async move {
            match server.run().await {
                Ok(()) => {
                    info!("✅ STUN server shut down gracefully");
                }
                Err(e) => {
                    warn!("⚠️  STUN server error: {}", e);
                }
            }
        });

        // Store server instance
        {
            let mut instance = self.server_handle.write().await;
            *instance = Some(ServerInstance {
                handle,
                bind_addr,
                start_time: std::time::Instant::now(),
            });
        }

        debug!("✅ STUN server started successfully");

        Ok(json!({
            "status": "started",
            "bind_addr": bind_addr.to_string(),
            "comment": "STUN server running in background (use stun.stop to stop)"
        }))
    }

    /// Handle `stun.stop` method - Stop STUN server
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "stun.stop",
    ///   "params": {},
    ///   "id": 2
    /// }
    /// ```
    ///
    /// # Response Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "result": {
    ///     "status": "stopped",
    ///     "uptime_seconds": 3600,
    ///     "bind_addr": "0.0.0.0:3478"
    ///   },
    ///   "id": 2
    /// }
    /// ```
    pub async fn handle_stop(&self, _params: Value) -> Result<Value, String> {
        let mut instance_guard = self.server_handle.write().await;

        if let Some(instance) = instance_guard.take() {
            let uptime = instance.start_time.elapsed().as_secs();
            let bind_addr = instance.bind_addr.to_string();

            info!("🛑 Stopping STUN server (uptime: {}s)", uptime);

            // Abort the server task
            instance.handle.abort();

            Ok(json!({
                "status": "stopped",
                "uptime_seconds": uptime,
                "bind_addr": bind_addr
            }))
        } else {
            Err("STUN server is not running".to_string())
        }
    }

    /// Handle `stun.status` method - Get server status
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "stun.status",
    ///   "params": {},
    ///   "id": 3
    /// }
    /// ```
    ///
    /// # Response Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "result": {
    ///     "running": true,
    ///     "bind_addr": "0.0.0.0:3478",
    ///     "uptime_seconds": 3600
    ///   },
    ///   "id": 3
    /// }
    /// ```
    pub async fn handle_status(&self, _params: Value) -> Result<Value, String> {
        let instance = self.server_handle.read().await;

        if let Some(instance) = instance.as_ref() {
            let uptime = instance.start_time.elapsed().as_secs();

            Ok(json!({
                "running": true,
                "bind_addr": instance.bind_addr.to_string(),
                "uptime_seconds": uptime
            }))
        } else {
            Ok(json!({
                "running": false,
                "comment": "STUN server is not running (use stun.serve to start)"
            }))
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  STUN CLIENT METHODS — NAT traversal from the client side
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    /// Default STUN servers for public address discovery (sovereignty-vetted)
    const DEFAULT_STUN_SERVERS: &'static [&'static str] =
        &["stun.nextcloud.com:3478", "stun.cloudflare.com:3478", "stun.l.google.com:19302"];

    /// Handle `stun.get_public_address` method - Discover public IP/port via STUN
    ///
    /// Races multiple STUN servers concurrently for fastest response.
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "stun.get_public_address",
    ///   "params": {
    ///     "servers": ["stun.nextcloud.com:3478", "stun.cloudflare.com:3478"]
    ///   },
    ///   "id": 1
    /// }
    /// ```
    ///
    /// # Response Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "result": {
    ///     "public_address": "162.226.225.148",
    ///     "public_port": 54321,
    ///     "full_address": "162.226.225.148:54321",
    ///     "nat_type": "unknown",
    ///     "servers_tried": 3,
    ///     "method": "stun_racing"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_get_public_address(&self, params: Value) -> Result<Value, String> {
        info!("🌐 STUN: Discovering public address via racing");

        // Parse optional custom servers
        let servers: Vec<String> = if let Some(servers_val) = params.get("servers") {
            serde_json::from_value(servers_val.clone())
                .map_err(|e| format!("Invalid 'servers' parameter: {e}"))?
        } else {
            Self::DEFAULT_STUN_SERVERS.iter().map(std::string::ToString::to_string).collect()
        };

        if servers.is_empty() {
            return Err("No STUN servers provided".to_string());
        }

        let client = StunClient::new();
        let server_refs: Vec<&str> = servers.iter().map(std::string::String::as_str).collect();
        let servers_count = server_refs.len();

        let public_addr = client
            .discover_public_address_racing(&server_refs)
            .await
            .map_err(|e| format!("STUN discovery failed: {e}"))?;

        info!("✅ STUN discovered public address: {}", public_addr);

        Ok(json!({
            "public_address": public_addr.ip().to_string(),
            "public_port": public_addr.port(),
            "full_address": public_addr.to_string(),
            "nat_type": "unknown",
            "servers_tried": servers_count,
            "method": "stun_racing"
        }))
    }

    /// Handle `stun.bind` method - Bind local port and discover NAT mapping
    ///
    /// Binds a local UDP port and uses STUN to discover the external mapping.
    /// Useful for hole-punching preparation.
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "stun.bind",
    ///   "params": {
    ///     "local_port": 0,
    ///     "stun_server": "stun.nextcloud.com:3478"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    ///
    /// # Response Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "result": {
    ///     "local_address": "0.0.0.0:54321",
    ///     "public_address": "162.226.225.148:54321",
    ///     "nat_type": "unknown",
    ///     "stun_server": "stun.nextcloud.com:3478"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_bind(&self, params: Value) -> Result<Value, String> {
        let stun_server =
            params.get("stun_server").and_then(|v| v.as_str()).unwrap_or("stun.nextcloud.com:3478");

        info!("🌐 STUN: Binding and discovering NAT mapping via {}", stun_server);

        let client = StunClient::new();

        let endpoint = client
            .discover_public_endpoint(stun_server)
            .await
            .map_err(|e| format!("STUN bind failed: {e}"))?;

        info!("✅ STUN bind result: {} (NAT type: {:?})", endpoint.address, endpoint.nat_type);

        Ok(json!({
            "local_address": "0.0.0.0:0",
            "public_address": endpoint.address.to_string(),
            "public_ip": endpoint.address.ip().to_string(),
            "public_port": endpoint.address.port(),
            "nat_type": format!("{:?}", endpoint.nat_type).to_lowercase(),
            "stun_server": stun_server
        }))
    }
}

impl Default for StunHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_creation() {
        let handler = StunHandler::new();

        // Should not be running initially
        let status = handler.handle_status(json!({})).await.unwrap();
        assert_eq!(status["running"], false);
    }

    #[tokio::test]
    async fn test_status_when_not_running() {
        let handler = StunHandler::new();
        let result = handler.handle_status(json!({})).await.unwrap();

        assert_eq!(result["running"], false);
        assert!(result["comment"].as_str().unwrap().contains("not running"));
    }

    #[tokio::test]
    async fn test_stop_when_not_running() {
        let handler = StunHandler::new();
        let result = handler.handle_stop(json!({})).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not running"));
    }

    #[tokio::test]
    async fn test_serve_with_default_address() {
        let handler = StunHandler::new();

        // Start server with default params
        let result = handler.handle_serve(json!({})).await.unwrap();

        assert_eq!(result["status"], "started");
        assert!(result["bind_addr"].as_str().unwrap().contains("3478"));

        // Cleanup
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_serve_with_custom_address() {
        let handler = StunHandler::new();

        // Use random port to avoid conflicts
        let result = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await
            .unwrap();

        assert_eq!(result["status"], "started");

        // Cleanup
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_serve_twice_fails() {
        let handler = StunHandler::new();

        // Start server
        let _ = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await
            .unwrap();

        // Try to start again - should fail
        let result = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already running"));

        // Cleanup
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_status_when_running() {
        let handler = StunHandler::new();

        // Start server
        let _ = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await
            .unwrap();

        // Check status
        let status = handler.handle_status(json!({})).await.unwrap();

        assert_eq!(status["running"], true);
        assert!(status["bind_addr"].is_string());
        assert!(status["uptime_seconds"].is_number());

        // Cleanup
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_stop_after_start() {
        let handler = StunHandler::new();

        // Start server
        let _ = handler
            .handle_serve(json!({
                "bind_addr": "127.0.0.1:0"
            }))
            .await
            .unwrap();

        // Stop server
        let result = handler.handle_stop(json!({})).await.unwrap();

        assert_eq!(result["status"], "stopped");
        assert!(result["uptime_seconds"].is_number());
        assert!(result["bind_addr"].is_string());

        // Should not be running anymore
        let status = handler.handle_status(json!({})).await.unwrap();
        assert_eq!(status["running"], false);
    }

    #[tokio::test]
    async fn test_invalid_bind_address() {
        let handler = StunHandler::new();

        let result = handler
            .handle_serve(json!({
                "bind_addr": "invalid_address"
            }))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid bind address"));
    }
}
