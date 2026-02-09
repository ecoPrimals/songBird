//! Pure Rust Tor Protocol JSON-RPC Handler
//!
//! Provides JSON-RPC methods for the pure Rust Tor protocol, enabling
//! full Tor network integration without external dependencies.
//!
//! ## Methods
//!
//! - `tor.status` - Get Tor circuit status
//! - `tor.connect` - Connect to .onion address via Tor network
//! - `tor.service.start` - Start hosting .onion service
//! - `tor.service.stop` - Stop .onion service
//! - `tor.consensus.fetch` - Fetch network consensus
//! - `tor.circuit.build` - Build a new circuit
//! - `tor.circuit.close` - Close a circuit
//!
//! ## TRUE PRIMAL Architecture
//!
//! All crypto operations are delegated to BearDog via `BeardogCryptoClient`.
//! Zero embedded crypto in Songbird.

use serde_json::{json, Value};
use songbird_tor_protocol::crypto::BeardogCryptoClient;
use songbird_tor_protocol::directory::Consensus;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Tor handler for JSON-RPC integration
///
/// Manages pure Rust Tor protocol operations.
///
/// ## Design Principles
///
/// - **TRUE PRIMAL**: All crypto via BearDog delegation  
/// - **Pure Rust**: No external Tor daemon required
/// - **Safe**: All operations use safe Rust
/// - **Async**: Modern async/await patterns
#[derive(Clone)]
pub struct TorHandler {
    /// Connection state
    state: Arc<RwLock<TorState>>,
}

/// Tor connection state
#[derive(Debug, Clone, Default)]
struct TorState {
    /// Whether initialized
    initialized: bool,
    /// Active circuit count
    circuit_count: u32,
    /// Consensus fetched
    consensus_valid: bool,
    /// Relay count from last consensus
    relay_count: usize,
    /// Service running
    service_running: bool,
    /// Service onion address
    service_address: Option<String>,
    /// BearDog socket path
    beardog_socket: Option<String>,
}

impl TorHandler {
    /// Create a new Tor handler
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(TorState::default())),
        }
    }

    /// Set BearDog socket path
    pub async fn set_beardog_socket(&self, socket_path: String) {
        let mut state = self.state.write().await;
        state.beardog_socket = Some(socket_path);
    }

    /// Get BearDog socket path from environment
    fn get_beardog_socket() -> Option<String> {
        std::env::var("BEARDOG_SOCKET")
            .or_else(|_| std::env::var("BEARDOG_CRYPTO_SOCKET"))
            .ok()
    }

    /// Handle `tor.status` - Get Tor connection status
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "tor.status",
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
    ///     "initialized": true,
    ///     "circuit_count": 3,
    ///     "consensus_valid": true,
    ///     "service_running": false,
    ///     "beardog_available": true
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_status(&self, _params: Value) -> Result<Value, String> {
        let state = self.state.read().await;
        let beardog_available = Self::get_beardog_socket().is_some();

        Ok(json!({
            "initialized": state.initialized,
            "circuit_count": state.circuit_count,
            "consensus_valid": state.consensus_valid,
            "relay_count": state.relay_count,
            "service_running": state.service_running,
            "service_address": state.service_address,
            "beardog_available": beardog_available,
            "comment": "Pure Rust Tor protocol (TRUE PRIMAL architecture)"
        }))
    }

    /// Handle `tor.connect` - Connect to .onion via Tor network
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "tor.connect",
    ///   "params": {
    ///     "address": "xyz123abc456def789.onion",
    ///     "port": 80
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_connect(&self, params: Value) -> Result<Value, String> {
        let address = params
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'address' parameter")?;

        let port = params
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(80) as u16;

        info!(
            address = address,
            port = port,
            "Connecting to .onion via pure Rust Tor"
        );

        // Check if BearDog is available
        let beardog_socket = Self::get_beardog_socket()
            .ok_or("BearDog not available. Set BEARDOG_SOCKET environment variable.")?;

        debug!(beardog = %beardog_socket, "Using BearDog for crypto operations");

        // TODO: Implement actual Tor connection using songbird-tor-protocol
        // This requires:
        // 1. Fetch consensus (if not cached)
        // 2. Build circuit to rendezvous point
        // 3. Connect to hidden service
        // 4. Stream data

        // For now, return status indicating work in progress
        Ok(json!({
            "connected": false,
            "target_address": format!("{}:{}", address, port),
            "status": "pending",
            "comment": "Pure Rust Tor connect in progress - awaiting circuit building implementation",
            "beardog_socket": beardog_socket
        }))
    }

    /// Handle `tor.service.start` - Start hosting .onion service
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "tor.service.start",
    ///   "params": {
    ///     "port": 80,
    ///     "private_key_id": "my_service_key"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_service_start(&self, params: Value) -> Result<Value, String> {
        // Check if already running
        {
            let state = self.state.read().await;
            if state.service_running {
                return Err("Tor service already running (use tor.service.stop first)".to_string());
            }
        }

        let port = params
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(80) as u16;

        let _key_id = params
            .get("private_key_id")
            .and_then(|v| v.as_str());

        info!(port = port, "Starting Tor hidden service via pure Rust");

        // Check if BearDog is available
        let beardog_socket = Self::get_beardog_socket()
            .ok_or("BearDog not available. Set BEARDOG_SOCKET environment variable.")?;

        // TODO: Implement actual service hosting using songbird-tor-protocol
        // This requires:
        // 1. Generate/load Ed25519 keypair via BearDog
        // 2. Derive .onion address from public key
        // 3. Build circuits to introduction points
        // 4. Publish service descriptor to HSDir
        // 5. Handle incoming connections

        // Update state
        {
            let mut state = self.state.write().await;
            state.service_running = true;
            state.service_address = Some("placeholder.onion".to_string());
        }

        Ok(json!({
            "started": true,
            "port": port,
            "status": "pending",
            "comment": "Pure Rust Tor service start in progress - awaiting intro point implementation",
            "beardog_socket": beardog_socket
        }))
    }

    /// Handle `tor.service.stop` - Stop .onion service
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "tor.service.stop",
    ///   "params": {},
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_service_stop(&self, _params: Value) -> Result<Value, String> {
        let was_running = {
            let mut state = self.state.write().await;
            let was_running = state.service_running;
            state.service_running = false;
            state.service_address = None;
            was_running
        };

        if was_running {
            info!("Tor hidden service stopped");
            Ok(json!({
                "stopped": true,
                "comment": "Tor service stopped"
            }))
        } else {
            Ok(json!({
                "stopped": false,
                "comment": "Tor service was not running"
            }))
        }
    }

    /// Handle `tor.consensus.fetch` - Fetch network consensus
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "tor.consensus.fetch",
    ///   "params": {
    ///     "force": false
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_consensus_fetch(&self, params: Value) -> Result<Value, String> {
        let force = params
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check if we already have valid consensus
        {
            let state = self.state.read().await;
            if state.consensus_valid && !force {
                return Ok(json!({
                    "fetched": false,
                    "cached": true,
                    "relay_count": state.relay_count,
                    "comment": "Using cached consensus (use force=true to refresh)"
                }));
            }
        }

        info!("Fetching Tor network consensus via pure Rust");

        // Create BearDog crypto client
        let beardog = match BeardogCryptoClient::from_env() {
            Ok(client) => client,
            Err(e) => {
                error!(error = %e, "Failed to create BearDog client");
                return Err(format!("BearDog unavailable: {}", e));
            }
        };

        // Fetch consensus using songbird-tor-protocol
        match Consensus::fetch(&beardog).await {
            Ok(consensus) => {
                let relay_count = consensus.relays.len();
                let is_valid = consensus.is_valid();
                let is_fresh = consensus.is_fresh();

                info!(
                    relay_count = relay_count,
                    is_valid = is_valid,
                    is_fresh = is_fresh,
                    "Consensus fetched successfully"
                );

                // Update state
                {
                    let mut state = self.state.write().await;
                    state.consensus_valid = is_valid;
                    state.relay_count = relay_count;
                    state.initialized = true;
                }

                Ok(json!({
                    "fetched": true,
                    "cached": false,
                    "relay_count": relay_count,
                    "is_valid": is_valid,
                    "is_fresh": is_fresh,
                    "comment": "Consensus fetched from Tor directory authority"
                }))
            }
            Err(e) => {
                error!(error = %e, "Failed to fetch consensus");
                Err(format!("Consensus fetch failed: {}", e))
            }
        }
    }

    /// Handle `tor.circuit.build` - Build a new circuit
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "tor.circuit.build",
    ///   "params": {
    ///     "purpose": "general"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_circuit_build(&self, params: Value) -> Result<Value, String> {
        let purpose = params
            .get("purpose")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        info!(purpose = purpose, "Building Tor circuit");

        // Check if BearDog is available
        let beardog_socket = Self::get_beardog_socket()
            .ok_or("BearDog not available. Set BEARDOG_SOCKET environment variable.")?;

        // TODO: Implement actual circuit building using songbird-tor-protocol
        // This requires:
        // 1. Valid consensus
        // 2. Select path (guard, middle, exit)
        // 3. CREATE2 to guard (ntor handshake via BearDog)
        // 4. EXTEND2 to middle
        // 5. EXTEND2 to exit

        // Update state
        {
            let mut state = self.state.write().await;
            state.circuit_count += 1;
        }

        let circuit_id = {
            let state = self.state.read().await;
            state.circuit_count
        };

        Ok(json!({
            "circuit_id": circuit_id,
            "purpose": purpose,
            "status": "pending",
            "comment": "Circuit building in progress - awaiting ntor handshake via BearDog",
            "beardog_socket": beardog_socket
        }))
    }

    /// Handle `tor.circuit.close` - Close a circuit
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "tor.circuit.close",
    ///   "params": {
    ///     "circuit_id": 1
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_circuit_close(&self, params: Value) -> Result<Value, String> {
        let circuit_id = params
            .get("circuit_id")
            .and_then(|v| v.as_u64())
            .ok_or("Missing 'circuit_id' parameter")? as u32;

        info!(circuit_id = circuit_id, "Closing Tor circuit");

        // Update state
        {
            let mut state = self.state.write().await;
            if state.circuit_count > 0 {
                state.circuit_count -= 1;
            }
        }

        Ok(json!({
            "circuit_id": circuit_id,
            "closed": true,
            "comment": "Circuit closed"
        }))
    }
}

impl Default for TorHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_tor_handler_new() {
        let handler = TorHandler::new();
        let status = handler.handle_status(json!({})).await.unwrap();
        assert_eq!(status["initialized"], false);
        assert_eq!(status["circuit_count"], 0);
    }

    #[tokio::test]
    async fn test_tor_status() {
        let handler = TorHandler::new();
        let result = handler.handle_status(json!({})).await;
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status["initialized"], false);
        assert_eq!(status["service_running"], false);
    }

    #[tokio::test]
    async fn test_tor_service_stop_not_running() {
        let handler = TorHandler::new();
        let result = handler.handle_service_stop(json!({})).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["stopped"], false);
    }

    #[tokio::test]
    async fn test_tor_connect_missing_address() {
        let handler = TorHandler::new();
        let result = handler.handle_connect(json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("address"));
    }

    #[tokio::test]
    async fn test_tor_circuit_close() {
        let handler = TorHandler::new();
        let result = handler
            .handle_circuit_close(json!({"circuit_id": 1}))
            .await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["closed"], true);
    }
}
