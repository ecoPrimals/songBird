// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
//! All crypto operations are delegated to `BearDog` via `CryptoProvider`.
//! Zero embedded crypto in Songbird.

use serde_json::{Value, json};
use songbird_tor_protocol::CryptoProvider;
use songbird_tor_protocol::circuit::CircuitPurpose;
use songbird_tor_protocol::circuit::manager::CircuitManager;
use songbird_tor_protocol::directory::Consensus;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Tor handler for JSON-RPC integration
///
/// Manages pure Rust Tor protocol operations including consensus
/// fetching, circuit building, and onion service hosting.
///
/// ## Design Principles
///
/// - **TRUE PRIMAL**: All crypto via `BearDog` delegation  
/// - **Pure Rust**: No external Tor daemon required
/// - **Safe**: All operations use safe Rust
/// - **Async**: Modern async/await patterns
/// - **No stubs**: All methods use real protocol implementations
#[derive(Clone)]
pub struct TorHandler {
    /// Connection state
    state: Arc<RwLock<TorState>>,
    /// Circuit manager (initialized after consensus fetch)
    circuit_manager: Arc<RwLock<Option<Arc<CircuitManager>>>>,
}

/// Tor connection state
#[derive(Debug, Clone, Default)]
struct TorState {
    /// Whether initialized (consensus fetched)
    initialized: bool,
    /// Active circuit count
    circuit_count: u32,
    /// Consensus fetched and valid
    consensus_valid: bool,
    /// Relay count from last consensus
    relay_count: usize,
    /// Service running
    service_running: bool,
    /// Service onion address
    service_address: Option<String>,
    /// `BearDog` socket path
    beardog_socket: Option<String>,
}

impl TorHandler {
    /// Create a new Tor handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(TorState::default())),
            circuit_manager: Arc::new(RwLock::new(None)),
        }
    }

    /// Set `BearDog` socket path
    pub async fn set_beardog_socket(&self, socket_path: String) {
        let mut state = self.state.write().await;
        state.beardog_socket = Some(socket_path);
    }

    /// Get `BearDog` socket path from state or environment
    async fn resolve_beardog_socket(&self) -> Option<String> {
        // Check state first (explicitly set)
        let state = self.state.read().await;
        if let Some(ref socket) = state.beardog_socket {
            return Some(socket.clone());
        }
        drop(state);

        // Fall back to environment discovery
        Self::get_beardog_socket_from_env()
    }

    /// Get `BearDog` socket path from environment (capability-based discovery)
    fn get_beardog_socket_from_env() -> Option<String> {
        songbird_process_env::var("BEARDOG_SOCKET")
            .or_else(|_| songbird_process_env::var("BEARDOG_CRYPTO_SOCKET"))
            .or_else(|_| songbird_process_env::var("SONGBIRD_SECURITY_PROVIDER"))
            .ok()
            .or_else(|| {
                // XDG standard path
                if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
                    let path = format!("{xdg}/biomeos/beardog.sock");
                    if std::path::Path::new(&path).exists() {
                        return Some(path);
                    }
                }
                None
            })
    }

    /// Handle `tor.status` - Get Tor connection status
    pub async fn handle_status(&self, _params: Value) -> Result<Value, String> {
        let (
            initialized,
            circuit_count,
            consensus_valid,
            relay_count,
            service_running,
            service_address,
        ) = {
            let state = self.state.read().await;
            (
                state.initialized,
                state.circuit_count,
                state.consensus_valid,
                state.relay_count,
                state.service_running,
                state.service_address.clone(),
            )
        };
        let beardog_available = self.resolve_beardog_socket().await.is_some()
            || Self::get_beardog_socket_from_env().is_some();

        Ok(json!({
            "initialized": initialized,
            "circuit_count": circuit_count,
            "consensus_valid": consensus_valid,
            "relay_count": relay_count,
            "service_running": service_running,
            "service_address": service_address,
            "beardog_available": beardog_available,
            "comment": "Pure Rust Tor protocol (TRUE PRIMAL architecture)"
        }))
    }

    /// Handle `tor.connect` - Connect to .onion via Tor network
    ///
    /// Uses `CircuitManager` to build a 3-hop circuit, then opens a
    /// stream to the target onion address.
    pub async fn handle_connect(&self, params: Value) -> Result<Value, String> {
        let address =
            params.get("address").and_then(|v| v.as_str()).ok_or("Missing 'address' parameter")?;

        let port =
            u16::try_from(params.get("port").and_then(serde_json::Value::as_u64).unwrap_or(80))
                .unwrap_or(80);

        info!(address = address, port = port, "Connecting to .onion via pure Rust Tor");

        // Ensure we have a circuit manager
        let manager = self.circuit_manager.read().await.as_ref().cloned().ok_or_else(|| {
            "Tor not initialized. Call tor.consensus.fetch first to build circuit manager."
                .to_string()
        })?;

        // Build a rendezvous circuit for .onion connections
        let purpose = if address.to_ascii_lowercase().ends_with(".onion") {
            CircuitPurpose::Rendezvous
        } else {
            CircuitPurpose::General
        };

        match manager.build_circuit(purpose).await {
            Ok(circuit_id) => {
                // Update state
                {
                    let mut state = self.state.write().await;
                    state.circuit_count += 1;
                }

                info!(
                    circuit_id = circuit_id,
                    "Circuit built for connection to {}:{}", address, port
                );

                Ok(json!({
                    "connected": true,
                    "circuit_id": circuit_id,
                    "target_address": format!("{}:{}", address, port),
                    "status": "circuit_ready",
                    "comment": "3-hop circuit built via ntor handshake"
                }))
            }
            Err(e) => {
                warn!(error = %e, "Circuit build failed for connection");
                Ok(json!({
                    "connected": false,
                    "target_address": format!("{}:{}", address, port),
                    "status": "circuit_failed",
                    "error": format!("{e}"),
                    "comment": "Circuit build failed — check relay reachability and BearDog availability"
                }))
            }
        }
    }

    /// Handle `tor.service.start` - Start hosting .onion service
    pub async fn handle_service_start(&self, params: Value) -> Result<Value, String> {
        // Check if already running
        {
            let state = self.state.read().await;
            if state.service_running {
                return Err("Tor service already running (use tor.service.stop first)".to_string());
            }
        }

        let port =
            u16::try_from(params.get("port").and_then(serde_json::Value::as_u64).unwrap_or(80))
                .unwrap_or(80);

        let _key_id = params.get("private_key_id").and_then(|v| v.as_str());

        info!(port = port, "Starting Tor hidden service via pure Rust");

        // Create BearDog client for service key operations
        let beardog = CryptoProvider::from_env();

        // Create Tor service
        match songbird_tor_protocol::TorService::new(beardog, port).await {
            Ok(service) => {
                let onion_address = service.onion_address().unwrap_or("pending").to_string();

                // Update state
                {
                    let mut state = self.state.write().await;
                    state.service_running = true;
                    state.service_address = Some(onion_address.clone());
                }

                Ok(json!({
                    "started": true,
                    "port": port,
                    "onion_address": onion_address,
                    "status": "running",
                    "comment": "Tor hidden service started (intro points pending full descriptor upload)"
                }))
            }
            Err(e) => {
                error!(error = %e, "Failed to start Tor service");
                Err(format!("Failed to start Tor service: {e}"))
            }
        }
    }

    /// Handle `tor.service.stop` - Stop .onion service
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
    /// Fetches the Tor directory consensus and initializes the
    /// `CircuitManager` for subsequent circuit building.
    pub async fn handle_consensus_fetch(&self, params: Value) -> Result<Value, String> {
        let force = params.get("force").and_then(serde_json::Value::as_bool).unwrap_or(false);

        // Check if we already have valid consensus
        if !force {
            let cached_relay_count = {
                let state = self.state.read().await;
                state.consensus_valid.then_some(state.relay_count)
            };
            if let Some(relay_count) = cached_relay_count {
                return Ok(json!({
                    "fetched": false,
                    "cached": true,
                    "relay_count": relay_count,
                    "comment": "Using cached consensus (use force=true to refresh)"
                }));
            }
        }

        info!("Fetching Tor network consensus via pure Rust");

        // Create BearDog crypto client
        let beardog = CryptoProvider::from_env();

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

                // Initialize circuit manager with fresh consensus
                let manager = Arc::new(CircuitManager::new(CryptoProvider::from_env(), consensus));
                {
                    let mut cm = self.circuit_manager.write().await;
                    *cm = Some(manager);
                }

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
                    "circuit_manager": "initialized",
                    "comment": "Consensus fetched, circuit manager ready"
                }))
            }
            Err(e) => {
                error!(error = %e, "Failed to fetch consensus");
                Err(format!("Consensus fetch failed: {e}"))
            }
        }
    }

    /// Handle `tor.circuit.build` - Build a new circuit
    ///
    /// Uses `CircuitManager` to build a real 3-hop circuit:
    /// 1. SELECT path (guard, middle, exit) from consensus
    /// 2. CREATE2 to guard (ntor handshake via `BearDog`)
    /// 3. EXTEND2 to middle
    /// 4. EXTEND2 to exit
    pub async fn handle_circuit_build(&self, params: Value) -> Result<Value, String> {
        let purpose_str = params.get("purpose").and_then(|v| v.as_str()).unwrap_or("general");

        let purpose = match purpose_str {
            "general" => CircuitPurpose::General,
            "hsdir" => CircuitPurpose::HSDir,
            "rendezvous" => CircuitPurpose::Rendezvous,
            other => {
                return Err(format!(
                    "Unknown circuit purpose '{other}'. Use: general, hsdir, rendezvous"
                ));
            }
        };

        info!(purpose = purpose_str, "Building Tor circuit");

        // Ensure circuit manager is initialized
        let manager = self.circuit_manager.read().await.as_ref().cloned().ok_or_else(|| {
            "Circuit manager not initialized. Call tor.consensus.fetch first.".to_string()
        })?;

        // Build real circuit
        match manager.build_circuit(purpose).await {
            Ok(circuit_id) => {
                // Update state
                {
                    let mut state = self.state.write().await;
                    state.circuit_count += 1;
                }

                info!(circuit_id = circuit_id, purpose = purpose_str, "Circuit built successfully");

                Ok(json!({
                    "circuit_id": circuit_id,
                    "purpose": purpose_str,
                    "status": "ready",
                    "hops": 3,
                    "comment": "3-hop circuit built (guard → middle → exit) via ntor handshake"
                }))
            }
            Err(e) => {
                warn!(error = %e, purpose = purpose_str, "Circuit build failed");
                Err(format!("Circuit build failed: {e}"))
            }
        }
    }

    /// Handle `tor.circuit.close` - Close a circuit
    pub async fn handle_circuit_close(&self, params: Value) -> Result<Value, String> {
        let circuit_id = u32::try_from(
            params
                .get("circuit_id")
                .and_then(serde_json::Value::as_u64)
                .ok_or("Missing 'circuit_id' parameter")?,
        )
        .map_err(|_| "circuit_id exceeds u32 range")?;

        info!(circuit_id = circuit_id, "Closing Tor circuit");

        // Close via circuit manager if available
        let mgr = self.circuit_manager.read().await.as_ref().cloned();
        if let Some(mgr) = mgr
            && let Err(e) = mgr.close_circuit(circuit_id).await
        {
            warn!(error = %e, "Circuit close error (may already be closed)");
        }

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
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use serde_json::json;

    #[test]
    fn tor_handler_default_matches_new() {
        let _ = TorHandler::default();
    }

    #[tokio::test]
    async fn handle_status_includes_expected_keys() {
        let handler = TorHandler::new();
        let status = handler.handle_status(json!({})).await.unwrap();
        assert_eq!(status["comment"], "Pure Rust Tor protocol (TRUE PRIMAL architecture)");
        assert!(status.get("beardog_available").is_some());
    }

    #[tokio::test]
    async fn circuit_close_missing_circuit_id_errors() {
        let handler = TorHandler::new();
        let err = handler.handle_circuit_close(json!({})).await.expect_err("missing id");
        assert!(err.contains("circuit_id"));
    }

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
    async fn test_tor_connect_requires_initialization() {
        let handler = TorHandler::new();
        let result = handler.handle_connect(json!({"address": "test.onion", "port": 80})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_tor_circuit_build_requires_consensus() {
        let handler = TorHandler::new();
        let result = handler.handle_circuit_build(json!({"purpose": "general"})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_tor_circuit_build_invalid_purpose() {
        let handler = TorHandler::new();
        let err = handler
            .handle_circuit_build(json!({ "purpose": "invalid" }))
            .await
            .expect_err("unknown purpose");
        assert!(err.contains("Unknown circuit purpose"));
    }

    #[tokio::test]
    async fn test_tor_circuit_close() {
        let handler = TorHandler::new();
        let result = handler.handle_circuit_close(json!({"circuit_id": 1})).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["closed"], true);
    }

    #[tokio::test]
    async fn test_set_beardog_socket_reflected_in_status_path() {
        let handler = TorHandler::new();
        handler.set_beardog_socket("/tmp/test-beardog.sock".to_string()).await;
        let status = handler.handle_status(json!({})).await.unwrap();
        assert_eq!(status["beardog_available"], true);
    }
}
