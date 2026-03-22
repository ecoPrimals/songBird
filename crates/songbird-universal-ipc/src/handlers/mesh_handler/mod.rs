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

mod json;
mod udp_discovery;

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests;

use serde_json::{Value, json};
use songbird_onion_relay::mesh::{BeaconMesh, EndpointType, RelayEndpoint};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

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
    pub async fn handle_status(&self, _params: Value) -> Result<Value, String> {
        let (reachable, direct_count, relay_count, onion_count, local_count) = {
            let mesh_guard = self.mesh.read().await;
            let mesh = mesh_guard.as_ref().ok_or("Mesh not initialized (call mesh.init first)")?;

            let reachable = mesh.get_reachable_nodes().await;

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
                return Ok(json::path_to_json(&path, true));
            }

            if let Some(path) = mesh.find_relay_for(target_node_id).await {
                return Ok(json::path_to_json(&path, true));
            }

            Ok(json!({
                "found": false,
                "target_node_id": target_node_id,
                "reason": "peer_not_discovered"
            }))
        }
    }

    /// Handle `mesh.announce` method - Announce as relay
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
                    let (path_type, address) = json::endpoint_to_strings(&path.endpoint_type);
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
    pub async fn handle_health_check(&self, params: Value) -> Result<Value, String> {
        let (results, all_healthy) = {
            let mesh_guard = self.mesh.read().await;
            let mesh = mesh_guard.as_ref().ok_or("Mesh not initialized (call mesh.init first)")?;

            mesh.health_check().await;

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
                    let (path_type, _) = json::endpoint_to_strings(&path.endpoint_type);
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

        let discovered = udp_discovery::udp_multicast_discover(
            &node_id,
            broadcast_port,
            Duration::from_millis(timeout_ms),
        )
        .await;

        let peers_found = {
            let mesh_guard = self.mesh.read().await;
            let mesh = mesh_guard.as_ref().ok_or("Mesh not initialized (call mesh.init first)")?;

            let mut peers_found = Vec::new();
            for (peer_id, addr) in &discovered {
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
}

impl Default for MeshHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl MeshHandler {
    pub fn endpoint_strings_for_test(
        &self,
        endpoint: &EndpointType,
    ) -> (&'static str, Option<String>) {
        let _ = self;
        json::endpoint_to_strings(endpoint)
    }

    pub fn path_json_for_test(&self, path: &RelayEndpoint, found: bool) -> Value {
        let _ = self;
        json::path_to_json(path, found)
    }
}
