// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Relay server JSON-RPC handler for biomeOS integration
//!
//! **Pure Rust | Zero Unsafe | Modern Async**
//!
//! Provides lifecycle management for the Lineage Relay Server
//! through IPC methods exposed to the biomeOS orchestrator.
//!
//! ## JSON-RPC Methods
//!
//! | Method | Purpose |
//! |--------|---------|
//! | `relay.serve` | Start relay server |
//! | `relay.allocate` | Allocate relay session (pre-provision cross-subnet path) |
//! | `relay.status` | Get server statistics |
//! | `relay.stop` | Stop relay server |
//!
//! ## Example Request
//!
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "relay.serve",
//!   "params": {
//!     "bind_addr": "0.0.0.0:3479"
//!   },
//!   "id": 1
//! }
//! ```

use crate::relay::RelayAuthority;
use crate::relay_protocol::AllocationRequest;
use crate::relay_server::RelayServer;
use crate::types::NodeId;
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// Relay handler state
pub struct RelayHandler {
    /// Current relay server instance
    server: Arc<RwLock<Option<Arc<RelayServer>>>>,

    /// Server task handle
    task: Arc<RwLock<Option<JoinHandle<()>>>>,

    /// Lineage authority (`security provider` integration)
    authority: Arc<RelayAuthority>,
}

impl RelayHandler {
    /// Create new relay handler
    #[must_use]
    pub fn new(authority: Arc<RelayAuthority>) -> Self {
        Self {
            server: Arc::new(RwLock::new(None)),
            task: Arc::new(RwLock::new(None)),
            authority,
        }
    }

    /// Handle `relay.serve` method - Start relay server
    ///
    /// # Parameters
    ///
    /// ```json
    /// {
    ///   "bind_addr": "0.0.0.0:3479"  // Optional; default port from `SONGBIRD_RELAY_PORT` (3479)
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// ```json
    /// {
    ///   "status": "running",
    ///   "bind_addr": "0.0.0.0:3479"
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if server is already running or bind fails.
    pub async fn handle_serve(&self, params: Value) -> std::result::Result<Value, String> {
        // Check if already running
        {
            let server_guard = self.server.read().await;
            if server_guard.is_some() {
                return Err("Relay server already running".to_string());
            }
        }

        // Parse parameters
        let default_port = songbird_process_env::var("SONGBIRD_RELAY_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(songbird_types::defaults::ports::DEFAULT_RELAY_PORT);

        let bind_addr: SocketAddr = match params.get("bind_addr").and_then(|v| v.as_str()) {
            Some(s) => s.parse().map_err(|e| format!("Invalid bind address '{s}': {e}"))?,
            None => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), default_port),
        };

        info!("🚀 Starting relay server on {}", bind_addr);

        // Create relay server
        let server = RelayServer::new(bind_addr, self.authority.clone())
            .await
            .map_err(|e| format!("Failed to create relay server: {e}"))?;

        let actual_addr = server.bind_addr();
        let server = Arc::new(server);

        // Spawn server task
        let server_clone = server.clone();
        let task = tokio::spawn(async move {
            if let Err(e) = server_clone.run().await {
                error!("❌ Relay server error: {}", e);
            }
        });

        // Store server and task (separate scopes so write locks are not held together)
        {
            let mut server_guard = self.server.write().await;
            *server_guard = Some(server);
        }
        {
            let mut task_guard = self.task.write().await;
            *task_guard = Some(task);
        }

        info!("✅ Relay server started on {}", actual_addr);

        Ok(json!({
            "status": "running",
            "bind_addr": actual_addr.to_string()
        }))
    }

    /// Handle `relay.stop` method - Stop relay server
    ///
    /// # Returns
    ///
    /// ```json
    /// {
    ///   "status": "stopped"
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if server is not running.
    pub async fn handle_stop(&self, _params: Value) -> std::result::Result<Value, String> {
        info!("🛑 Stopping relay server");

        let server = {
            let mut server_guard = self.server.write().await;
            server_guard.take().ok_or_else(|| "Relay server not running".to_string())?
        };
        let task = {
            let mut task_guard = self.task.write().await;
            task_guard.take()
        };

        // Shutdown server
        server.shutdown().await.map_err(|e| format!("Failed to shutdown relay server: {e}"))?;

        // Abort task
        if let Some(task) = task {
            task.abort();
        }

        info!("✅ Relay server stopped");

        Ok(json!({
            "status": "stopped"
        }))
    }

    /// Handle `relay.status` method - Get server statistics
    ///
    /// # Returns
    ///
    /// ```json
    /// {
    ///   "running": true,
    ///   "sessions_active": 12,
    ///   "sessions_total": 345,
    ///   "bytes_forwarded": 1234567890,
    ///   "packets_forwarded": 98765,
    ///   "uptime_seconds": 3600
    /// }
    /// ```
    pub async fn handle_status(&self, _params: Value) -> std::result::Result<Value, String> {
        let server_guard = self.server.read().await;

        match &*server_guard {
            Some(server) => {
                let stats = server.stats().await;

                Ok(json!({
                    "running": true,
                    "bind_addr": server.bind_addr().to_string(),
                    "sessions_active": stats.sessions_active,
                    "sessions_total": stats.sessions_total,
                    "bytes_forwarded": stats.bytes_forwarded,
                    "packets_forwarded": stats.packets_forwarded,
                    "authorization_failures": stats.authorization_failures,
                    "uptime_seconds": stats.uptime_seconds()
                }))
            }
            None => Ok(json!({
                "running": false
            })),
        }
    }

    /// Handle `relay.allocate` method - Test relay allocation
    ///
    /// Useful for testing, but normally clients discover relays via `BirdSong`.
    ///
    /// # Parameters
    ///
    /// ```json
    /// {
    ///   "relay_node": "tower",
    ///   "requester": "pixel",
    ///   "target_addr": "192.168.1.100:5000",
    ///   "lineage_proof": "base64_encoded_proof",
    ///   "ttl_seconds": 300
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// ```json
    /// {
    ///   "success": true,
    ///   "session_id": "550e8400-e29b-41d4-a716-446655440000",
    ///   "relay_addr": "198.51.100.1:3479",
    ///   "ttl_seconds": 300
    /// }
    /// ```
    pub async fn handle_allocate(&self, params: Value) -> std::result::Result<Value, String> {
        // Check if server is running
        {
            let server_guard = self.server.read().await;
            if server_guard.is_none() {
                return Err("Relay server not running".to_string());
            }
        }

        // Parse parameters
        let relay_node: NodeId = params
            .get("relay_node")
            .and_then(|v| v.as_str())
            .map(std::convert::Into::into)
            .ok_or_else(|| "Missing 'relay_node'".to_string())?;

        let requester: NodeId = params
            .get("requester")
            .and_then(|v| v.as_str())
            .map(std::convert::Into::into)
            .ok_or_else(|| "Missing 'requester'".to_string())?;

        let target_addr: SocketAddr = params
            .get("target_addr")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| "Invalid 'target_addr'".to_string())?;

        let lineage_proof: Vec<u8> = params
            .get("lineage_proof")
            .and_then(|v| v.as_str())
            .and_then(|s| {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.decode(s).ok()
            })
            .unwrap_or_default();

        let ttl_seconds: u32 = params
            .get("ttl_seconds")
            .and_then(serde_json::Value::as_u64)
            .map_or(300, |n| u32::try_from(n).unwrap_or(u32::MAX));

        let request =
            AllocationRequest::new(relay_node, requester, target_addr, lineage_proof, ttl_seconds);

        // Authorize via lineage authority
        let auth_result =
            self.authority.authorize_relay(&request.relay_node, &request.requester).await;

        match auth_result {
            Ok(auth) if auth.authorized => {
                let session_id = uuid::Uuid::new_v4();
                let bind_addr = {
                    let server_guard = self.server.read().await;
                    server_guard
                        .as_ref()
                        .map_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0), |s| {
                            s.bind_addr()
                        })
                };

                info!(
                    "✅ relay.allocate: session {} for {} → {}",
                    session_id, request.requester.0, request.target_addr
                );

                Ok(json!({
                    "success": true,
                    "session_id": session_id.to_string(),
                    "relay_addr": bind_addr.to_string(),
                    "ttl_seconds": request.ttl_seconds,
                    "masking_level": format!("{:?}", auth.masking_level)
                }))
            }
            Ok(_) => Err(format!("Relay authorization denied for {}", request.requester.0)),
            Err(e) => Err(format!("Authorization check failed: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::relay::RelayAuthority;
    #[tokio::test]
    async fn test_relay_handler_serve() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);

        let params = json!({
            "bind_addr": "127.0.0.1:0"
        });

        let result = handler.handle_serve(params).await.unwrap();

        assert_eq!(result["status"], "running");
        assert!(result["bind_addr"].as_str().unwrap().starts_with("127.0.0.1:"));

        // Cleanup
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_relay_handler_serve_already_running() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);

        let params = json!({"bind_addr": "127.0.0.1:0"});

        // Start first time
        handler.handle_serve(params.clone()).await.unwrap();

        // Try to start again (should fail)
        let result = handler.handle_serve(params).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already running"));

        // Cleanup
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_relay_handler_status_not_running() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);

        let result = handler.handle_status(json!({})).await.unwrap();

        assert_eq!(result["running"], false);
    }

    #[tokio::test]
    async fn test_relay_handler_status_running() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);

        // Start server
        let params = json!({"bind_addr": "127.0.0.1:0"});
        handler.handle_serve(params).await.unwrap();

        // Check status
        let result = handler.handle_status(json!({})).await.unwrap();

        assert_eq!(result["running"], true);
        assert_eq!(result["sessions_active"], 0);
        assert_eq!(result["sessions_total"], 0);

        // Cleanup
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn test_relay_handler_stop() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);

        // Start server
        let params = json!({"bind_addr": "127.0.0.1:0"});
        handler.handle_serve(params).await.unwrap();

        // Stop server
        let result = handler.handle_stop(json!({})).await.unwrap();

        assert_eq!(result["status"], "stopped");

        // Verify not running
        let status = handler.handle_status(json!({})).await.unwrap();
        assert_eq!(status["running"], false);
    }

    #[tokio::test]
    async fn test_relay_handler_stop_not_running() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);

        let result = handler.handle_stop(json!({})).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not running"));
    }

    #[tokio::test]
    async fn test_relay_handler_allocate() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);

        handler.handle_serve(json!({"bind_addr": "127.0.0.1:0"})).await.unwrap();

        let params = json!({
            "relay_node": "tower",
            "requester": "pixel",
            "target_addr": "192.168.1.100:5000",
            "ttl_seconds": 300
        });

        let result = handler.handle_allocate(params).await.unwrap();

        assert_eq!(result["success"], true);
        assert!(result["session_id"].as_str().is_some());
        assert!(result["relay_addr"].as_str().is_some());
        assert_eq!(result["ttl_seconds"], 300);

        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn handle_allocate_errors_without_relay_node() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);
        handler.handle_serve(json!({"bind_addr": "127.0.0.1:0"})).await.unwrap();
        let err = handler
            .handle_allocate(json!({
                "requester": "pixel",
                "target_addr": "192.168.1.1:5000"
            }))
            .await
            .expect_err("missing relay_node");
        assert!(err.contains("relay_node"), "{err}");
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn handle_allocate_errors_without_requester() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);
        handler.handle_serve(json!({"bind_addr": "127.0.0.1:0"})).await.unwrap();
        let err = handler
            .handle_allocate(json!({
                "relay_node": "tower",
                "target_addr": "192.168.1.1:5000"
            }))
            .await
            .expect_err("missing requester");
        assert!(err.contains("requester"), "{err}");
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn handle_allocate_errors_on_invalid_target_addr() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);
        handler.handle_serve(json!({"bind_addr": "127.0.0.1:0"})).await.unwrap();
        let err = handler
            .handle_allocate(json!({
                "relay_node": "tower",
                "requester": "pixel",
                "target_addr": "not-a-socket-addr"
            }))
            .await
            .expect_err("bad addr");
        assert!(err.contains("target_addr"), "{err}");
        let _ = handler.handle_stop(json!({})).await;
    }

    #[tokio::test]
    async fn handle_allocate_errors_when_server_not_running() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);
        let err = handler
            .handle_allocate(json!({
                "relay_node": "tower",
                "requester": "pixel",
                "target_addr": "192.168.1.1:5000"
            }))
            .await
            .expect_err("server down");
        assert!(err.contains("not running"), "{err}");
    }

    #[tokio::test]
    async fn handle_allocate_decodes_base64_lineage_proof() {
        let authority = Arc::new(RelayAuthority::StubAllow);
        let handler = RelayHandler::new(authority);
        handler.handle_serve(json!({"bind_addr": "127.0.0.1:0"})).await.unwrap();
        use base64::{Engine as _, engine::general_purpose};
        let proof = general_purpose::STANDARD.encode([1, 2, 3, 4]);
        let result = handler
            .handle_allocate(json!({
                "relay_node": "tower",
                "requester": "pixel",
                "target_addr": "10.0.0.1:9000",
                "lineage_proof": proof,
                "ttl_seconds": 60
            }))
            .await
            .unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["ttl_seconds"], 60);
        let _ = handler.handle_stop(json!({})).await;
    }
}
