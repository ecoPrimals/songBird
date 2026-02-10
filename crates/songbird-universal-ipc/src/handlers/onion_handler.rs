//! Sovereign Onion Service JSON-RPC Handler
//!
//! Provides JSON-RPC methods for the sovereign onion service, enabling
//! NAT-traversing P2P connections via cryptographic .onion addresses.
//!
//! ## Methods
//!
//! - `onion.start` - Start onion service (generate .onion address)
//! - `onion.stop` - Stop onion service
//! - `onion.status` - Get service status and .onion address
//! - `onion.connect` - Connect to a remote .onion address
//!
//! ## TRUE PRIMAL Architecture
//!
//! All crypto operations are delegated to `BearDog` via `BeardogCryptoClient`.
//! Zero embedded crypto in Songbird.

use serde_json::{json, Value};
use songbird_sovereign_onion::{BeardogCryptoClient, OnionConnector, OnionService};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Onion handler for JSON-RPC integration
///
/// Manages the sovereign onion service lifecycle and provides
/// connection capabilities via JSON-RPC.
///
/// ## Design Principles
///
/// - **TRUE PRIMAL**: All crypto via `BearDog` delegation
/// - **Self-Sovereign**: No external onion routers needed
/// - **Safe**: All operations use safe Rust
/// - **Async**: Modern async/await patterns
#[derive(Clone)]
pub struct OnionHandler {
    /// Running onion service (if started)
    service: Arc<RwLock<Option<Arc<OnionService>>>>,
    /// Service start time
    start_time: Arc<RwLock<Option<std::time::Instant>>>,
    /// `BearDog` socket path (for creating clients on demand)
    beardog_socket: Arc<RwLock<Option<String>>>,
}

impl OnionHandler {
    /// Create a new onion handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            service: Arc::new(RwLock::new(None)),
            start_time: Arc::new(RwLock::new(None)),
            beardog_socket: Arc::new(RwLock::new(None)),
        }
    }

    /// Set `BearDog` socket path (optional - will auto-detect from env)
    pub async fn set_beardog_socket(&self, socket_path: String) {
        *self.beardog_socket.write().await = Some(socket_path);
    }

    /// Get or create a `BearDog` client
    fn get_beardog_client(&self) -> Result<BeardogCryptoClient, String> {
        // Try from env first (standard approach)
        BeardogCryptoClient::from_env()
            .map_err(|e| format!("BearDog client not available: {e}. Set BEARDOG_SOCKET or ensure BearDog is running."))
    }

    /// Handle `onion.start` - Start the sovereign onion service
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "onion.start",
    ///   "params": {
    ///     "port": 3492
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
    ///     "started": true,
    ///     "onion_address": "xyz123abc456def789.onion",
    ///     "port": 3492
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_start(&self, params: Value) -> Result<Value, String> {
        // Check if already running
        {
            let service = self.service.read().await;
            if service.is_some() {
                return Err("Onion service already running (use onion.stop first)".to_string());
            }
        }

        // Get BearDog client (creates fresh from env)
        let beardog = self.get_beardog_client()?;

        // Parse port from params
        let port =
            u16::try_from(params.get("port").and_then(serde_json::Value::as_u64).unwrap_or(3492))
                .unwrap_or(3492);

        info!(port = port, "Starting sovereign onion service via BearDog");

        // Create onion service
        let service = OnionService::new_via_beardog(port, beardog)
            .await
            .map_err(|e| format!("Failed to create onion service: {e}"))?;

        let onion_address = service.onion_address().to_string();

        // Store service
        *self.service.write().await = Some(Arc::new(service));
        *self.start_time.write().await = Some(std::time::Instant::now());

        info!(
            onion_address = %onion_address,
            port = port,
            "Sovereign onion service started (TRUE PRIMAL)"
        );

        Ok(json!({
            "started": true,
            "onion_address": onion_address,
            "port": port,
            "comment": "Service ready for incoming connections"
        }))
    }

    /// Handle `onion.stop` - Stop the onion service
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "onion.stop",
    ///   "params": {},
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_stop(&self, _params: Value) -> Result<Value, String> {
        let was_running = {
            let mut service = self.service.write().await;
            let was_running = service.is_some();
            *service = None;
            was_running
        };

        if was_running {
            *self.start_time.write().await = None;
            info!("Sovereign onion service stopped");
            Ok(json!({
                "stopped": true,
                "comment": "Onion service stopped"
            }))
        } else {
            Ok(json!({
                "stopped": false,
                "comment": "Onion service was not running"
            }))
        }
    }

    /// Handle `onion.status` - Get onion service status
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "onion.status",
    ///   "params": {},
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
    ///     "running": true,
    ///     "onion_address": "xyz123abc456def789.onion",
    ///     "port": 3492,
    ///     "uptime_seconds": 3600,
    ///     "beardog_connected": true
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_status(&self, _params: Value) -> Result<Value, String> {
        let service = self.service.read().await;
        let start_time = self.start_time.read().await;
        let beardog_available = self.get_beardog_client().is_ok();

        if let Some(svc) = service.as_ref() {
            let uptime = start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);

            Ok(json!({
                "running": true,
                "onion_address": svc.onion_address(),
                "port": svc.port(),
                "uptime_seconds": uptime,
                "beardog_available": beardog_available
            }))
        } else {
            Ok(json!({
                "running": false,
                "onion_address": null,
                "beardog_available": beardog_available
            }))
        }
    }

    /// Handle `onion.connect` - Connect to a remote onion address
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "onion.connect",
    ///   "params": {
    ///     "address": "xyz123abc456def789.onion",
    ///     "port": 3492
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
    ///     "connected": true,
    ///     "target_address": "xyz123abc456def789.onion:3492",
    ///     "session_id": "abc123"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_connect(&self, params: Value) -> Result<Value, String> {
        // Get BearDog client (creates fresh from env)
        let beardog = self.get_beardog_client()?;

        let address =
            params.get("address").and_then(|v| v.as_str()).ok_or("Missing 'address' parameter")?;

        let port =
            u16::try_from(params.get("port").and_then(serde_json::Value::as_u64).unwrap_or(3492))
                .unwrap_or(3492);

        info!(address = address, port = port, "Connecting to onion address via BearDog");

        // Create connector with BearDog
        let connector = OnionConnector::new_via_beardog(beardog);

        // Attempt connection
        match connector.connect(address, port).await {
            Ok(_connection) => {
                // For now, we just verify connection works
                // Future: Store connection for send/recv operations
                info!(address = address, port = port, "Successfully connected to onion service");

                Ok(json!({
                    "connected": true,
                    "target_address": format!("{}:{}", address, port),
                    "comment": "Connection established via BearDog crypto"
                }))
            }
            Err(e) => {
                warn!(
                    address = address,
                    error = %e,
                    "Failed to connect to onion address"
                );

                Ok(json!({
                    "connected": false,
                    "target_address": format!("{}:{}", address, port),
                    "error": e.to_string()
                }))
            }
        }
    }

    /// Handle `onion.address` - Get just the .onion address (convenience)
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "onion.address",
    ///   "params": {},
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_address(&self, _params: Value) -> Result<Value, String> {
        let service = self.service.read().await;

        if let Some(svc) = service.as_ref() {
            Ok(json!({
                "address": svc.onion_address(),
                "port": svc.port(),
                "full": format!("{}:{}", svc.onion_address(), svc.port())
            }))
        } else {
            Err("Onion service not running".to_string())
        }
    }
}

impl Default for OnionHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_onion_handler_new() {
        let handler = OnionHandler::new();
        // Should start with no service
        let status = handler.handle_status(json!({})).await.unwrap();
        assert_eq!(status["running"], false);
    }

    #[tokio::test]
    async fn test_onion_status_not_running() {
        let handler = OnionHandler::new();
        let result = handler.handle_status(json!({})).await;
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status["running"], false);
        // beardog_available depends on env, just check field exists
        assert!(status.get("beardog_available").is_some());
    }

    #[tokio::test]
    async fn test_onion_stop_not_running() {
        let handler = OnionHandler::new();
        let result = handler.handle_stop(json!({})).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["stopped"], false);
    }

    #[tokio::test]
    async fn test_onion_start_without_beardog() {
        // This test verifies the handler attempts to find BearDog
        // In CI/test env without BearDog, it should fail gracefully
        let handler = OnionHandler::new();
        let result = handler.handle_start(json!({"port": 3492})).await;
        // Either works (BearDog available) or fails with a relevant error
        // Valid errors include: socket not found, RPC method not found, connection refused
        if let Err(e) = result {
            let e_lower = e.to_lowercase();
            assert!(
                e_lower.contains("beardog")
                    || e_lower.contains("socket")
                    || e_lower.contains("ipc")
                    || e_lower.contains("rpc")
                    || e_lower.contains("method not found")
                    || e_lower.contains("connection refused")
                    || e_lower.contains("failed to create"),
                "Error should be crypto/connection related: {}",
                e
            );
        }
    }

    #[tokio::test]
    async fn test_onion_connect_without_beardog() {
        // This test verifies the handler attempts to find BearDog
        let handler = OnionHandler::new();
        let result = handler
            .handle_connect(json!({
                "address": "test.onion",
                "port": 3492
            }))
            .await;
        // Either works or fails appropriately
        if let Err(e) = result {
            let e_lower = e.to_lowercase();
            assert!(
                e_lower.contains("beardog")
                    || e_lower.contains("socket")
                    || e_lower.contains("ipc")
                    || e_lower.contains("rpc")
                    || e_lower.contains("method not found")
                    || e_lower.contains("connection refused")
                    || e_lower.contains("failed to create"),
                "Error should be crypto/connection related: {}",
                e
            );
        }
    }

    #[tokio::test]
    async fn test_onion_address_not_running() {
        let handler = OnionHandler::new();
        let result = handler.handle_address(json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not running"));
    }
}
