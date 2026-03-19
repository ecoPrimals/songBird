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
        let server = StunServer::new(bind_addr);

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

    /// Handle `stun.probe_port_pattern` method - Detect NAT port allocation pattern
    ///
    /// Sends N STUN probes to detect whether the NAT allocates ports
    /// sequentially (predictable) or randomly. Sequential patterns enable
    /// coordinated hole punching for symmetric NAT traversal.
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "stun.probe_port_pattern",
    ///   "params": {
    ///     "stun_server": "192.168.1.144:3478",
    ///     "probes": 5
    ///   },
    ///   "id": 1
    /// }
    /// ```
    ///
    /// # Response Example (Sequential)
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "result": {
    ///     "pattern": "sequential",
    ///     "step": 1,
    ///     "last_port": 41204,
    ///     "predicted_next": 41205,
    ///     "confidence": 0.85,
    ///     "supports_coordinated_punch": true
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_probe_port_pattern(&self, params: Value) -> Result<Value, String> {
        let stun_server =
            params.get("stun_server").and_then(|v| v.as_str()).unwrap_or("stun.nextcloud.com:3478");

        let probes = params
            .get("probes")
            .and_then(serde_json::Value::as_u64)
            .map_or(5, |n| usize::try_from(n).unwrap_or(5));

        info!("🔍 STUN: Probing port pattern ({} probes to {})", probes, stun_server);

        let client = StunClient::new();

        let pattern = client
            .probe_port_pattern(stun_server, probes)
            .await
            .map_err(|e| format!("Port pattern probing failed: {e}"))?;

        let response = match &pattern {
            songbird_stun::PortPattern::Sequential {
                step,
                last_port,
                predicted_next,
                confidence,
            } => {
                info!("✅ Sequential pattern: step={}, predicted={}", step, predicted_next);
                json!({
                    "pattern": "sequential",
                    "step": step,
                    "last_port": last_port,
                    "predicted_next": predicted_next,
                    "confidence": confidence,
                    "supports_coordinated_punch": pattern.supports_coordinated_punch()
                })
            }
            songbird_stun::PortPattern::Random {
                observed,
            } => {
                info!("⚠️ Random pattern: {} ports observed", observed.len());
                json!({
                    "pattern": "random",
                    "observed_ports": observed,
                    "supports_coordinated_punch": false
                })
            }
            songbird_stun::PortPattern::Unknown => {
                warn!("⚠️ Could not determine port pattern");
                json!({
                    "pattern": "unknown",
                    "supports_coordinated_punch": false
                })
            }
        };

        Ok(response)
    }

    /// Handle `stun.detect_nat_type` method - Detect NAT type via multiple probes
    ///
    /// Uses two STUN servers to compare port mappings and determine NAT type.
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "stun.detect_nat_type",
    ///   "params": {
    ///     "servers": ["stun.nextcloud.com:3478", "stun.cloudflare.com:3478"]
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_detect_nat_type(&self, params: Value) -> Result<Value, String> {
        let servers: Vec<String> = if let Some(servers_val) = params.get("servers") {
            serde_json::from_value(servers_val.clone())
                .map_err(|e| format!("Invalid 'servers' parameter: {e}"))?
        } else {
            Self::DEFAULT_STUN_SERVERS
                .iter()
                .take(2)
                .map(std::string::ToString::to_string)
                .collect()
        };

        if servers.len() < 2 {
            return Err("Need at least 2 STUN servers for NAT type detection".to_string());
        }

        info!("🔍 STUN: Detecting NAT type via {} servers", servers.len());

        let client = StunClient::new();

        // Query two different STUN servers from the same socket
        let addr1 = client
            .discover_public_address(&servers[0])
            .await
            .map_err(|e| format!("STUN server 1 failed: {e}"))?;

        let addr2 = client
            .discover_public_address(&servers[1])
            .await
            .map_err(|e| format!("STUN server 2 failed: {e}"))?;

        let (nat_type, description) = if addr1.ip() != addr2.ip() {
            // Different IPs means something very unusual (multi-homed NAT)
            ("unknown", "Different public IPs detected — unusual topology")
        } else if addr1.port() == addr2.port() {
            // Same port for different destinations = cone NAT
            ("cone", "Same port for different destinations — likely cone NAT (good for punching)")
        } else {
            // Different ports = symmetric NAT
            (
                "symmetric",
                "Different ports for different destinations — symmetric NAT (needs relay-assisted punch)",
            )
        };

        info!("✅ NAT type detected: {} — {}", nat_type, description);

        Ok(json!({
            "nat_type": nat_type,
            "description": description,
            "probe_results": {
                "server_1": { "server": &servers[0], "public_addr": addr1.to_string() },
                "server_2": { "server": &servers[1], "public_addr": addr2.to_string() }
            },
            "recommendation": if nat_type == "symmetric" {
                "Use relay-assisted coordinated punch (punch.coordinate)"
            } else {
                "Direct hole punch should work (punch.request)"
            }
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
