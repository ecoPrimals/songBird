// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Mesh Network JSON-RPC Handler
//!
//! Exposes the `BeaconMesh` functionality via JSON-RPC for distributed
//! relay mesh networking. Enables path finding across NAT boundaries.
//!
//! ## Methods
//!
//! - `mesh.status` - Get current mesh network status
//! - `mesh.find_path` - Find best path to reach a peer
//! - `mesh.announce` - Announce as relay to the mesh
//! - `mesh.peers` - List known peers in the mesh
//! - `mesh.health_check` - Check health of peer connections
//!
//! ## TRUE PRIMAL Architecture
//!
//! This handler delegates to `songbird-onion-relay::BeaconMesh` for
//! mesh state management while exposing capability via JSON-RPC.

use serde_json::{Value, json};
use songbird_onion_relay::mesh::{BeaconMesh, EndpointType, RelayEndpoint};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Mesh handler for JSON-RPC integration
///
/// Manages the distributed beacon mesh and provides status,
/// path finding, and relay announcement via JSON-RPC.
///
/// ## Design Principles
///
/// - **Delegated**: Core logic in `BeaconMesh`
/// - **Capability-Based**: Exposes capability, not implementation
/// - **Safe**: All operations use safe Rust
/// - **Async**: Modern async/await patterns
#[derive(Clone)]
pub struct MeshHandler {
    /// Beacon mesh instance
    mesh: Arc<RwLock<Option<BeaconMesh>>>,
    /// Start time for uptime tracking
    start_time: Instant,
    /// Our node ID
    node_id: Arc<RwLock<String>>,
}

impl MeshHandler {
    /// Create a new mesh handler (uninitialized - call `initialize` first)
    pub fn new() -> Self {
        Self {
            mesh: Arc::new(RwLock::new(None)),
            start_time: Instant::now(),
            node_id: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Create with an existing mesh instance
    pub fn with_mesh(mesh: BeaconMesh, node_id: String) -> Self {
        Self {
            mesh: Arc::new(RwLock::new(Some(mesh))),
            start_time: Instant::now(),
            node_id: Arc::new(RwLock::new(node_id)),
        }
    }

    /// Initialize the mesh with node ID and bootstrap onions
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "mesh.init",
    ///   "params": {
    ///     "node_id": "tower-abc123",
    ///     "bootstrap_onions": ["xyz.onion"]
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_init(&self, params: Value) -> Result<Value, String> {
        let node_id = params
            .get("node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing node_id parameter")?
            .to_string();

        let bootstrap_onions: Vec<String> = params
            .get("bootstrap_onions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        info!(
            "🌐 Initializing mesh for node {} with {} bootstrap onions",
            &node_id[..8.min(node_id.len())],
            bootstrap_onions.len()
        );

        let mesh = BeaconMesh::new(node_id.clone(), bootstrap_onions);
        *self.mesh.write().await = Some(mesh);
        *self.node_id.write().await = node_id.clone();

        Ok(json!({
            "initialized": true,
            "node_id": node_id
        }))
    }

    /// Handle `mesh.status` method - Get mesh network status
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "mesh.status",
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
    ///     "node_id": "tower-abc123",
    ///     "reachable_peers": 3,
    ///     "relay_enabled": true,
    ///     "uptime_seconds": 3600,
    ///     "paths": {
    ///       "direct": 1,
    ///       "family_relay": 1,
    ///       "onion": 1
    ///     }
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn handle_status(&self, _params: Value) -> Result<Value, String> {
        let (reachable, direct_count, relay_count, onion_count, local_count) = {
            let mesh_guard = self.mesh.read().await;
            let mesh = mesh_guard.as_ref().ok_or("Mesh not initialized (call mesh.init first)")?;

            let reachable = mesh.get_reachable_nodes().await;

            // Count path types
            let mut direct_count = 0;
            let mut relay_count = 0;
            let mut onion_count = 0;
            let mut local_count = 0;

            for peer_id in &reachable {
                if let Some(path) = mesh.get_best_path(peer_id).await {
                    match path.endpoint_type {
                        EndpointType::Local {
                            ..
                        } => local_count += 1,
                        EndpointType::Direct {
                            ..
                        } => direct_count += 1,
                        EndpointType::FamilyRelay {
                            ..
                        } => relay_count += 1,
                        EndpointType::TorOnion {
                            ..
                        } => onion_count += 1,
                    }
                }
            }

            Ok::<_, String>((reachable, direct_count, relay_count, onion_count, local_count))
        }?;

        let node_id = self.node_id.read().await.clone();
        let uptime = self.start_time.elapsed().as_secs();

        Ok(json!({
            "node_id": node_id,
            "reachable_peers": reachable.len(),
            "relay_enabled": true,
            "uptime_seconds": uptime,
            "paths": {
                "local": local_count,
                "direct": direct_count,
                "family_relay": relay_count,
                "onion": onion_count
            }
        }))
    }

    /// Handle `mesh.find_path` method - Find best path to a peer
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "mesh.find_path",
    ///   "params": {
    ///     "target_node_id": "pixel-xyz789",
    ///     "prefer_direct": true
    ///   },
    ///   "id": 2
    /// }
    /// ```
    pub async fn handle_find_path(&self, params: Value) -> Result<Value, String> {
        let target_node_id = params
            .get("target_node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing target_node_id parameter")?;

        debug!("🔍 Finding path to {}", target_node_id);

        {
            let mesh_guard = self.mesh.read().await;
            let mesh = mesh_guard.as_ref().ok_or("Mesh not initialized (call mesh.init first)")?;

            if let Some(path) = mesh.get_best_path(target_node_id).await {
                return Ok(self.path_to_json(&path, true));
            }

            if let Some(path) = mesh.find_relay_for(target_node_id).await {
                return Ok(self.path_to_json(&path, true));
            }

            Ok(json!({
                "found": false,
                "target_node_id": target_node_id,
                "reason": "peer_not_discovered"
            }))
        }
    }

    /// Handle `mesh.announce` method - Announce as relay
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "mesh.announce",
    ///   "params": {
    ///     "as_relay": true,
    ///     "capabilities": ["relay", "stun"]
    ///   },
    ///   "id": 3
    /// }
    /// ```
    pub async fn handle_announce(&self, params: Value) -> Result<Value, String> {
        let as_relay = params.get("as_relay").and_then(serde_json::Value::as_bool).unwrap_or(true);

        if !as_relay {
            return Ok(json!({
                "announced": false,
                "reason": "as_relay must be true"
            }));
        }

        {
            let mesh_guard = self.mesh.read().await;
            let mesh = mesh_guard.as_ref().ok_or("Mesh not initialized (call mesh.init first)")?;
            let _msg = mesh.announce_as_relay().await;
            Ok::<_, String>(())
        }?;

        let node_id = self.node_id.read().await.clone();

        info!("📢 Announced {} as relay to mesh", &node_id[..8.min(node_id.len())]);

        Ok(json!({
            "announced": true,
            "node_id": node_id,
            "ttl_seconds": 300
        }))
    }

    /// Handle `mesh.peers` method - List known peers
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "mesh.peers",
    ///   "params": {
    ///     "include_offline": false
    ///   },
    ///   "id": 4
    /// }
    /// ```
    pub async fn handle_peers(&self, params: Value) -> Result<Value, String> {
        let _include_offline =
            params.get("include_offline").and_then(serde_json::Value::as_bool).unwrap_or(false);

        let (peers, relay_count) = {
            let mesh_guard = self.mesh.read().await;
            let mesh = mesh_guard.as_ref().ok_or("Mesh not initialized (call mesh.init first)")?;

            let reachable = mesh.get_reachable_nodes().await;

            let mut peers = Vec::new();
            let mut relay_count = 0;

            for node_id in &reachable {
                if let Some(path) = mesh.get_best_path(node_id).await {
                    let (path_type, address) = self.endpoint_to_strings(&path.endpoint_type);
                    let is_relay = matches!(path.endpoint_type, EndpointType::FamilyRelay { .. });
                    if is_relay {
                        relay_count += 1;
                    }

                    let latency_ms =
                        path.latency.map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));

                    peers.push(json!({
                        "node_id": node_id,
                        "path_type": path_type,
                        "address": address,
                        "last_seen_ms": u64::try_from(path.last_seen.elapsed().as_millis()).unwrap_or(u64::MAX),
                        "is_relay": is_relay,
                        "latency_ms": latency_ms,
                        "reachable": path.reachable
                    }));
                }
            }

            Ok::<_, String>((peers, relay_count))
        }?;

        Ok(json!({
            "peers": peers,
            "total": peers.len(),
            "online": peers.len(),
            "relays": relay_count
        }))
    }

    /// Handle `mesh.health_check` method - Check peer health
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "mesh.health_check",
    ///   "params": {
    ///     "target_node_ids": ["pixel-xyz789"],
    ///     "timeout_ms": 5000
    ///   },
    ///   "id": 5
    /// }
    /// ```
    pub async fn handle_health_check(&self, params: Value) -> Result<Value, String> {
        let (results, all_healthy) = {
            let mesh_guard = self.mesh.read().await;
            let mesh = mesh_guard.as_ref().ok_or("Mesh not initialized (call mesh.init first)")?;

            // Run the mesh health check to update reachability
            mesh.health_check().await;

            // If no target_node_ids specified, check all reachable nodes
            let target_ids: Vec<String> =
                if let Some(arr) = params.get("target_node_ids").and_then(|v| v.as_array()) {
                    arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
                } else {
                    mesh.get_reachable_nodes().await
                };

            let mut results = Vec::new();
            let mut all_healthy = true;

            for node_id in target_ids {
                if let Some(path) = mesh.get_best_path(&node_id).await {
                    let (path_type, _) = self.endpoint_to_strings(&path.endpoint_type);
                    let healthy = path.reachable;
                    if !healthy {
                        all_healthy = false;
                    }

                    results.push(json!({
                        "node_id": node_id,
                        "healthy": healthy,
                        "latency_ms": path.latency.map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX)),
                        "path_type": path_type
                    }));
                } else {
                    all_healthy = false;
                    results.push(json!({
                        "node_id": node_id,
                        "healthy": false,
                        "reason": "no_path_known"
                    }));
                }
            }

            Ok::<_, String>((results, all_healthy))
        }?;

        Ok(json!({
            "results": results,
            "all_healthy": all_healthy
        }))
    }

    /// Handle `mesh.auto_discover` method - Auto-discover peers on local network
    ///
    /// Uses UDP multicast beacon broadcast to find other Songbird instances
    /// on the local network. Gates sharing the same family seed authenticate
    /// via HMAC challenge-response.
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "mesh.auto_discover",
    ///   "params": {
    ///     "timeout_ms": 3000,
    ///     "broadcast_port": 5353
    ///   },
    ///   "id": 6
    /// }
    /// ```
    pub async fn handle_auto_discover(&self, params: Value) -> Result<Value, String> {
        let timeout_ms =
            params.get("timeout_ms").and_then(serde_json::Value::as_u64).unwrap_or(3000);
        let broadcast_port = u16::try_from(
            params.get("broadcast_port").and_then(serde_json::Value::as_u64).unwrap_or(5353),
        )
        .unwrap_or(5353);

        let node_id = self.node_id.read().await.clone();
        info!(
            "🔍 Auto-discovering peers on local network (port {}, timeout {}ms)",
            broadcast_port, timeout_ms
        );

        // Perform UDP multicast discovery
        let discovered = self
            .udp_multicast_discover(&node_id, broadcast_port, Duration::from_millis(timeout_ms))
            .await;

        let peers_found = {
            let mesh_guard = self.mesh.read().await;
            let mesh = mesh_guard.as_ref().ok_or("Mesh not initialized (call mesh.init first)")?;

            let mut peers_found = Vec::new();
            for (peer_id, addr) in &discovered {
                // Add discovered local peers to mesh
                let endpoint = RelayEndpoint {
                    node_id: peer_id.clone(),
                    endpoint_type: EndpointType::Local {
                        addr: *addr,
                    },
                    latency: None,
                    last_seen: Instant::now(),
                    reachable: true,
                };
                mesh.add_endpoint(peer_id.clone(), endpoint).await;

                peers_found.push(json!({
                    "node_id": peer_id,
                    "address": addr.to_string(),
                    "path_type": "local"
                }));
            }

            Ok::<_, String>(peers_found)
        }?;

        info!("🔍 Auto-discovery complete: found {} peers", peers_found.len());

        Ok(json!({
            "discovered": peers_found.len(),
            "peers": peers_found,
            "broadcast_port": broadcast_port,
            "timeout_ms": timeout_ms
        }))
    }

    /// Perform UDP multicast discovery on the local network
    ///
    /// Sends a beacon packet on the multicast group 239.255.77.77:{port}
    /// and listens for responses from other Songbird instances.
    ///
    /// Returns: Vec of (`node_id`, `SocketAddr` with correct `jsonrpc_port`)
    /// The `SocketAddr` uses the peer's IP from the UDP packet but replaces
    /// the ephemeral UDP source port with the announced `jsonrpc_port` (default 8080).
    async fn udp_multicast_discover(
        &self,
        our_node_id: &str,
        port: u16,
        timeout: Duration,
    ) -> Vec<(String, SocketAddr)> {
        let mut discovered = Vec::new();

        // Bind to any available port for sending
        let socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to bind UDP socket for discovery: {}", e);
                return discovered;
            }
        };

        // Enable broadcast
        if let Err(e) = socket.set_broadcast(true) {
            warn!("Failed to enable broadcast: {}", e);
            return discovered;
        }

        // Runtime-discover our HTTP JSON-RPC port for beacon announcement
        // Resolution: SONGBIRD_HTTP_PORT → SONGBIRD_PORT → 8080
        let jsonrpc_port: u16 = std::env::var("SONGBIRD_HTTP_PORT")
            .or_else(|_| std::env::var("SONGBIRD_PORT"))
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        // Build discovery beacon with jsonrpc_port for inter-gate connectivity
        let beacon = json!({
            "type": "songbird_discovery",
            "node_id": our_node_id,
            "version": env!("CARGO_PKG_VERSION"),
            "jsonrpc_port": jsonrpc_port,
            "capabilities": ["mesh", "relay", "stun", "punch"],
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

        let beacon_bytes = serde_json::to_vec(&beacon).unwrap_or_default();

        // Send to multicast group and broadcast
        let multicast_addr: SocketAddr =
            format!("239.255.77.77:{port}").parse().expect("valid multicast socket address");
        let broadcast_addr: SocketAddr =
            format!("255.255.255.255:{port}").parse().expect("valid broadcast socket address");

        // Try both multicast and broadcast
        let _ = socket.send_to(&beacon_bytes, multicast_addr).await;
        let _ = socket.send_to(&beacon_bytes, broadcast_addr).await;

        // Listen for responses
        let mut buf = vec![0u8; 4096];
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            match tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, addr))) => {
                    if let Ok(response) = serde_json::from_slice::<Value>(&buf[..len])
                        && (response.get("type").and_then(|t| t.as_str())
                            == Some("songbird_discovery_response")
                            || response.get("type").and_then(|t| t.as_str())
                                == Some("songbird_discovery"))
                        && let Some(peer_id) = response.get("node_id").and_then(|n| n.as_str())
                        && peer_id != our_node_id
                    {
                        // Use jsonrpc_port from beacon (default 8080) instead
                        // of the ephemeral UDP source port for the endpoint address
                        let jsonrpc_port = u16::try_from(
                            response
                                .get("jsonrpc_port")
                                .and_then(serde_json::Value::as_u64)
                                .unwrap_or(8080),
                        )
                        .unwrap_or(8080);
                        let peer_addr = SocketAddr::new(addr.ip(), jsonrpc_port);
                        info!(
                            "🔍 Discovered peer {} at {} (jsonrpc_port: {})",
                            peer_id, peer_addr, jsonrpc_port
                        );
                        discovered.push((peer_id.to_string(), peer_addr));
                    }
                }
                Ok(Err(e)) => {
                    debug!("UDP recv error during discovery: {}", e);
                    break;
                }
                Err(_) => {
                    // Timeout reached
                    break;
                }
            }
        }

        discovered
    }

    // --- Helper methods ---

    fn path_to_json(&self, path: &RelayEndpoint, found: bool) -> Value {
        let (path_type, address) = self.endpoint_to_strings(&path.endpoint_type);

        json!({
            "found": found,
            "target_node_id": path.node_id,
            "path_type": path_type,
            "address": address,
            "estimated_latency_ms": path.latency.map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX)),
            "reachable": path.reachable
        })
    }

    fn endpoint_to_strings(&self, endpoint: &EndpointType) -> (String, Option<String>) {
        match endpoint {
            EndpointType::Local {
                addr,
            } => ("local".to_string(), Some(addr.to_string())),
            EndpointType::Direct {
                addr,
            } => ("direct".to_string(), Some(addr.to_string())),
            EndpointType::FamilyRelay {
                relay_node_id,
            } => ("family_relay".to_string(), Some(relay_node_id.clone())),
            EndpointType::TorOnion {
                onion_addr,
            } => ("onion".to_string(), Some(onion_addr.clone())),
        }
    }
}

impl Default for MeshHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_mesh_handler_uninitialized() {
        let handler = MeshHandler::new();

        // Should fail without init
        let result = handler.handle_status(json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_mesh_init() {
        let handler = MeshHandler::new();

        let result = handler
            .handle_init(json!({
                "node_id": "test-tower",
                "bootstrap_onions": ["abc.onion"]
            }))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["initialized"], true);
        assert_eq!(response["node_id"], "test-tower");
    }

    #[tokio::test]
    async fn test_mesh_status_after_init() {
        let handler = MeshHandler::new();

        // Initialize
        handler
            .handle_init(json!({
                "node_id": "test-tower",
                "bootstrap_onions": []
            }))
            .await
            .unwrap();

        // Get status
        let result = handler.handle_status(json!({})).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["node_id"], "test-tower");
        assert_eq!(response["reachable_peers"], 0);
        assert!(response["uptime_seconds"].as_u64().is_some());
    }

    #[tokio::test]
    async fn test_mesh_find_path_not_found() {
        let handler = MeshHandler::new();

        handler
            .handle_init(json!({
                "node_id": "test-tower",
                "bootstrap_onions": []
            }))
            .await
            .unwrap();

        let result = handler
            .handle_find_path(json!({
                "target_node_id": "unknown-peer"
            }))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["found"], false);
        assert_eq!(response["reason"], "peer_not_discovered");
    }

    #[tokio::test]
    async fn test_mesh_find_path_with_bootstrap() {
        let handler = MeshHandler::new();

        // Initialize with bootstrap onion
        handler
            .handle_init(json!({
                "node_id": "test-tower",
                "bootstrap_onions": ["bootstrap.onion"]
            }))
            .await
            .unwrap();

        // Should find path via bootstrap
        let result = handler
            .handle_find_path(json!({
                "target_node_id": "remote-peer"
            }))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["found"], true);
        assert_eq!(response["path_type"], "onion");
    }

    #[tokio::test]
    async fn test_mesh_announce() {
        let handler = MeshHandler::new();

        handler
            .handle_init(json!({
                "node_id": "test-tower",
                "bootstrap_onions": []
            }))
            .await
            .unwrap();

        let result = handler
            .handle_announce(json!({
                "as_relay": true,
                "capabilities": ["relay", "stun"]
            }))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["announced"], true);
    }

    #[tokio::test]
    async fn test_mesh_peers_empty() {
        let handler = MeshHandler::new();

        handler
            .handle_init(json!({
                "node_id": "test-tower",
                "bootstrap_onions": []
            }))
            .await
            .unwrap();

        let result = handler.handle_peers(json!({})).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["total"], 0);
        assert!(response["peers"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_mesh_auto_discover() {
        let handler = MeshHandler::new();

        handler
            .handle_init(json!({
                "node_id": "test-tower",
                "bootstrap_onions": []
            }))
            .await
            .unwrap();

        // Auto-discover with a very short timeout (no peers will be found in test)
        let result = handler
            .handle_auto_discover(json!({
                "timeout_ms": 100,
                "broadcast_port": 15353
            }))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["discovered"], 0);
        assert!(response["peers"].as_array().unwrap().is_empty());
        assert_eq!(response["broadcast_port"], 15353);
    }

    #[tokio::test]
    async fn test_mesh_auto_discover_requires_init() {
        let handler = MeshHandler::new();

        let result = handler.handle_auto_discover(json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_mesh_health_check() {
        let handler = MeshHandler::new();

        handler
            .handle_init(json!({
                "node_id": "test-tower",
                "bootstrap_onions": []
            }))
            .await
            .unwrap();

        let result = handler
            .handle_health_check(json!({
                "target_node_ids": ["unknown-peer"],
                "timeout_ms": 1000
            }))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["all_healthy"], false);
    }
}
