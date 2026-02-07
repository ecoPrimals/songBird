//! Mesh Network JSON-RPC Handler
//!
//! Exposes the BeaconMesh functionality via JSON-RPC for distributed
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

use serde_json::{json, Value};
use songbird_onion_relay::mesh::{BeaconMesh, EndpointType, RelayEndpoint};
use std::sync::Arc;
use std::time::Instant;
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
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
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
        let mesh_guard = self.mesh.read().await;
        let mesh = mesh_guard
            .as_ref()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let reachable = mesh.get_reachable_nodes().await;
        let node_id = self.node_id.read().await.clone();
        let uptime = self.start_time.elapsed().as_secs();

        // Count path types
        let mut direct_count = 0;
        let mut relay_count = 0;
        let mut onion_count = 0;
        let mut local_count = 0;

        for peer_id in &reachable {
            if let Some(path) = mesh.get_best_path(peer_id).await {
                match path.endpoint_type {
                    EndpointType::Local { .. } => local_count += 1,
                    EndpointType::Direct { .. } => direct_count += 1,
                    EndpointType::FamilyRelay { .. } => relay_count += 1,
                    EndpointType::TorOnion { .. } => onion_count += 1,
                }
            }
        }

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
        let mesh_guard = self.mesh.read().await;
        let mesh = mesh_guard
            .as_ref()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let target_node_id = params
            .get("target_node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing target_node_id parameter")?;

        debug!("🔍 Finding path to {}", target_node_id);

        // First try to get the best known path
        if let Some(path) = mesh.get_best_path(target_node_id).await {
            return Ok(self.path_to_json(&path, true));
        }

        // Try to find a relay
        if let Some(path) = mesh.find_relay_for(target_node_id).await {
            return Ok(self.path_to_json(&path, true));
        }

        // Not found
        Ok(json!({
            "found": false,
            "target_node_id": target_node_id,
            "reason": "peer_not_discovered"
        }))
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
        let mesh_guard = self.mesh.read().await;
        let mesh = mesh_guard
            .as_ref()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let as_relay = params
            .get("as_relay")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !as_relay {
            return Ok(json!({
                "announced": false,
                "reason": "as_relay must be true"
            }));
        }

        let msg = mesh.announce_as_relay().await;
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
        let mesh_guard = self.mesh.read().await;
        let mesh = mesh_guard
            .as_ref()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let include_offline = params
            .get("include_offline")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

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

                let latency_ms = path.latency.map(|d| d.as_millis() as u64);

                peers.push(json!({
                    "node_id": node_id,
                    "path_type": path_type,
                    "address": address,
                    "last_seen_ms": path.last_seen.elapsed().as_millis() as u64,
                    "is_relay": is_relay,
                    "latency_ms": latency_ms,
                    "reachable": path.reachable
                }));
            }
        }

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
        let mesh_guard = self.mesh.read().await;
        let mesh = mesh_guard
            .as_ref()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        // Run the mesh health check to update reachability
        mesh.health_check().await;

        // If no target_node_ids specified, check all reachable nodes
        let target_ids: Vec<String> = if let Some(arr) = params.get("target_node_ids").and_then(|v| v.as_array()) {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
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
                    "latency_ms": path.latency.map(|d| d.as_millis() as u64),
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

        Ok(json!({
            "results": results,
            "all_healthy": all_healthy
        }))
    }

    // --- Helper methods ---

    fn path_to_json(&self, path: &RelayEndpoint, found: bool) -> Value {
        let (path_type, address) = self.endpoint_to_strings(&path.endpoint_type);
        
        json!({
            "found": found,
            "target_node_id": path.node_id,
            "path_type": path_type,
            "address": address,
            "estimated_latency_ms": path.latency.map(|d| d.as_millis() as u64),
            "reachable": path.reachable
        })
    }

    fn endpoint_to_strings(&self, endpoint: &EndpointType) -> (String, Option<String>) {
        match endpoint {
            EndpointType::Local { addr } => ("local".to_string(), Some(addr.to_string())),
            EndpointType::Direct { addr } => ("direct".to_string(), Some(addr.to_string())),
            EndpointType::FamilyRelay { relay_node_id } => {
                ("family_relay".to_string(), Some(relay_node_id.clone()))
            }
            EndpointType::TorOnion { onion_addr } => ("onion".to_string(), Some(onion_addr.clone())),
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
    use std::time::Duration;

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
