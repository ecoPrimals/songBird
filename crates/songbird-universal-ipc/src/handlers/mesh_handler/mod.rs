// SPDX-License-Identifier: AGPL-3.0-or-later
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
//! - `mesh.topology` - Get full mesh network topology graph
//! - `mesh.health_check` - Check health of peer connections
//!
//! ## TRUE PRIMAL Architecture
//!
//! This handler delegates to `songbird-onion-relay::BeaconMesh` for
//! mesh state management while exposing capability via JSON-RPC.

pub(crate) mod capability_propagation;
mod discovery_federation;
mod enrollment;
mod health_probing;
mod json;
pub mod persistence;
mod prune_stale;
pub(crate) mod topology_graph;
mod udp_discovery;

#[cfg(test)]
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests;

use serde_json::{Value, json};
use songbird_onion_relay::mesh::{BeaconMesh, EndpointType};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

pub use capability_propagation::PartitionStatus;
use capability_propagation::PendingAnnounce;
pub(crate) use capability_propagation::{PeerCapabilityEntry, PeerMetadata};

/// Probe result including RTT and optional peer version.
pub struct ProbeResult {
    pub(super) rtt: Duration,
    pub(super) version: Option<String>,
}

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
    pub(crate) mesh: Arc<RwLock<Option<Arc<BeaconMesh>>>>,
    /// Start time for uptime tracking
    start_time: Instant,
    /// Our node ID
    pub(super) node_id: Arc<RwLock<Arc<str>>>,
    /// Remote peer capabilities received via `mesh.capabilities_announce`.
    /// Key: `node_id`, Value: capabilities + last-seen timestamp.
    pub(crate) peer_capabilities: Arc<RwLock<HashMap<String, PeerCapabilityEntry>>>,
    /// Peers that failed to receive announcements (retried on next health cycle).
    pending_announces: Arc<RwLock<Vec<PendingAnnounce>>>,
    /// Per-peer metadata: version, cross-gate reachability reports (for partition detection).
    pub(crate) peer_metadata: Arc<RwLock<HashMap<String, PeerMetadata>>>,
    /// Minimum interval between capability announcements from the same peer (flood prevention).
    pub(crate) min_announce_interval: Duration,
}

impl MeshHandler {
    /// Create a new mesh handler (uninitialized - call `initialize` first)
    #[must_use]
    pub fn new() -> Self {
        let min_announce_interval = songbird_process_env::var("SONGBIRD_ANNOUNCE_RATE_LIMIT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map_or(Duration::from_secs(2), Duration::from_millis);

        Self {
            mesh: Arc::new(RwLock::new(None::<Arc<BeaconMesh>>)),
            start_time: Instant::now(),
            node_id: Arc::new(RwLock::new(Arc::from(""))),
            peer_capabilities: Arc::new(RwLock::new(HashMap::new())),
            pending_announces: Arc::new(RwLock::new(Vec::new())),
            peer_metadata: Arc::new(RwLock::new(HashMap::new())),
            min_announce_interval,
        }
    }

    /// Access the mesh state (for cross-gate dispatch capability resolution).
    pub async fn mesh(&self) -> tokio::sync::RwLockReadGuard<'_, Option<Arc<BeaconMesh>>> {
        self.mesh.read().await
    }

    /// Create with an existing mesh instance
    pub fn with_mesh(mesh: BeaconMesh, node_id: impl Into<Arc<str>>) -> Self {
        Self {
            mesh: Arc::new(RwLock::new(Some(Arc::new(mesh)))),
            start_time: Instant::now(),
            node_id: Arc::new(RwLock::new(node_id.into())),
            peer_capabilities: Arc::new(RwLock::new(HashMap::new())),
            pending_announces: Arc::new(RwLock::new(Vec::new())),
            peer_metadata: Arc::new(RwLock::new(HashMap::new())),
            min_announce_interval: Duration::from_secs(2),
        }
    }

    /// Initialize the mesh with node ID, bootstrap onions, and/or bootstrap peers.
    ///
    /// `bootstrap_peers` enables cross-gate discovery by adding TCP-reachable peers
    /// to the mesh at init time (connection attempts are spawned asynchronously).
    ///
    /// `lan_peers` (optional) registers same-subnet LAN peers as `EndpointType::Local`
    /// (priority 0 — always preferred over overlay/direct). Use physical LAN addresses
    /// here so `mesh.find_path` returns the sub-millisecond local path instead of
    /// routing through `WireGuard` overlay at 100ms+ penalty.
    ///
    /// `overlay_peers` (optional) registers WireGuard/VPN endpoints for peers
    /// (priority 1 — used when no LAN path exists).
    ///
    /// # Request Example
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "mesh.init",
    ///   "params": {
    ///     "node_id": "tower-abc123",
    ///     "bootstrap_onions": ["xyz.onion"],
    ///     "bootstrap_peers": [
    ///       { "node_id": "west-gate", "address": "192.168.1.50:7700" }
    ///     ],
    ///     "lan_peers": [
    ///       { "node_id": "west-gate", "address": "192.168.4.50:7700" }
    ///     ],
    ///     "overlay_peers": [
    ///       { "node_id": "west-gate", "address": "10.13.37.2:7700" }
    ///     ],
    ///     "overlay_name": "wireguard"
    ///   },
    ///   "id": 1
    /// }
    /// ```
    #[expect(
        clippy::too_many_lines,
        reason = "mesh.init handles bootstrap + overlay + health + persistence"
    )]
    pub async fn handle_init(&self, params: Value) -> Result<Value, String> {
        use songbird_onion_relay::{EndpointType, RelayEndpoint};

        let node_id: Arc<str> = params
            .get("node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing node_id parameter")?
            .into();

        let bootstrap_onions: Vec<String> = params
            .get("bootstrap_onions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let bootstrap_peers = parse_peer_list(&params, &["bootstrap_peers", "peers"], false);

        info!(
            "🌐 Initializing mesh for node {} with {} bootstrap onions, {} bootstrap peers",
            &node_id.as_ref()[..8.min(node_id.len())],
            bootstrap_onions.len(),
            bootstrap_peers.len()
        );

        let mesh = BeaconMesh::new(node_id.as_ref().to_string(), bootstrap_onions);
        let mesh = Arc::new(mesh);

        for (peer_id, addr) in &bootstrap_peers {
            let endpoint = RelayEndpoint {
                node_id: peer_id.clone(),
                endpoint_type: EndpointType::Direct {
                    addr: *addr,
                },
                latency: None,
                last_seen: std::time::Instant::now(),
                reachable: true,
            };
            mesh.add_endpoint(peer_id.clone(), endpoint).await;
        }

        // Register LAN endpoints (same-subnet — priority 0, always preferred)
        let lan_peers = parse_peer_list(&params, &["lan_peers"], true);

        for (peer_id, addr) in &lan_peers {
            let endpoint = RelayEndpoint {
                node_id: peer_id.clone(),
                endpoint_type: EndpointType::Local {
                    addr: *addr,
                },
                latency: None,
                last_seen: std::time::Instant::now(),
                reachable: true,
            };
            mesh.add_endpoint(peer_id.clone(), endpoint).await;
        }

        // Register overlay endpoints (WireGuard/VPN — priority 1, fallback when no LAN)
        let overlay_name =
            params.get("overlay_name").and_then(Value::as_str).unwrap_or("wireguard");
        let overlay_peers = parse_peer_list(&params, &["overlay_peers"], true);

        for (peer_id, addr) in &overlay_peers {
            let endpoint = RelayEndpoint {
                node_id: peer_id.clone(),
                endpoint_type: EndpointType::Overlay {
                    addr: *addr,
                    overlay_name: String::from(overlay_name),
                },
                latency: None,
                last_seen: std::time::Instant::now(),
                reachable: true,
            };
            mesh.add_endpoint(peer_id.clone(), endpoint).await;
        }

        // Load persisted peers — restore LAN endpoints from prior enrollment
        if let Some((_, persisted)) = persistence::load_persisted_peers_full() {
            for peer in &persisted {
                if let Some(lan) = peer.lan_addr {
                    let already_has_local = lan_peers.iter().any(|(id, _)| id == &peer.node_id);
                    if !already_has_local {
                        let endpoint = RelayEndpoint {
                            node_id: peer.node_id.clone(),
                            endpoint_type: EndpointType::Local {
                                addr: lan,
                            },
                            latency: None,
                            last_seen: std::time::Instant::now(),
                            reachable: true,
                        };
                        mesh.add_endpoint(peer.node_id.clone(), endpoint).await;
                    }
                }
            }
        }

        let mesh_ref = Arc::clone(&mesh);
        *self.mesh.write().await = Some(mesh);
        *self.node_id.write().await = node_id.clone();

        let peers_added = bootstrap_peers.len();
        let lan_peers_added = lan_peers.len();
        let overlay_for_health: Vec<(String, std::net::SocketAddr, String)> = overlay_peers
            .iter()
            .map(|(id, addr)| (id.clone(), *addr, String::from(overlay_name)))
            .collect();

        if !bootstrap_peers.is_empty() || !overlay_for_health.is_empty() {
            Self::spawn_peer_health_loop(
                mesh_ref,
                bootstrap_peers.clone(),
                overlay_for_health,
                Arc::clone(&self.peer_metadata),
            );

            if !bootstrap_peers.is_empty() {
                let node_id_owned = node_id.as_ref().to_string();
                tokio::task::spawn_blocking(move || {
                    persistence::save_peers(&node_id_owned, &bootstrap_peers);
                });
            }
        }

        Ok(json!({
            "initialized": true,
            "node_id": node_id.as_ref(),
            "bootstrap_peers_added": peers_added,
            "lan_peers_added": lan_peers_added
        }))
    }

    /// Handle `mesh.status` method - Get mesh network status.
    ///
    /// Returns initialization state even when mesh is not yet initialized,
    /// allowing probers to distinguish "not running" from "running but empty".
    #[expect(clippy::too_many_lines, reason = "status aggregation across endpoint types")]
    pub async fn handle_status(&self, _params: Value) -> Result<Value, String> {
        let (reachable, direct_count, relay_count, onion_count, local_count, overlay_count) = {
            let Some(mesh) = self.mesh.read().await.as_ref().cloned() else {
                let node_id = self.node_id.read().await.clone();
                return Ok(json!({
                    "initialized": false,
                    "node_id": &*node_id,
                    "status": "awaiting_init",
                    "message": "Mesh not yet initialized — will auto-seed from SONGBIRD_PEERS, persisted state, or WireGuard peers",
                    "uptime_seconds": self.start_time.elapsed().as_secs()
                }));
            };

            let reachable = mesh.get_reachable_nodes().await;

            let mut direct_count = 0;
            let mut relay_count = 0;
            let mut onion_count = 0;
            let mut local_count = 0;
            let mut overlay_count = 0;

            for peer_id in &reachable {
                if let Some(path) = mesh.get_best_path(peer_id).await {
                    match path.endpoint_type {
                        EndpointType::Local {
                            ..
                        } => local_count += 1,
                        EndpointType::Overlay {
                            ..
                        } => overlay_count += 1,
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

            Ok::<_, String>((
                reachable,
                direct_count,
                relay_count,
                onion_count,
                local_count,
                overlay_count,
            ))
        }?;

        let node_id = self.node_id.read().await.clone();
        let uptime = self.start_time.elapsed().as_secs();

        let our_version = env!("CARGO_PKG_VERSION");
        let meta = self.peer_metadata.read().await;
        let mut version_skew: Vec<Value> = Vec::new();
        let mut partition_warnings: Vec<Value> = Vec::new();

        for peer_id in &reachable {
            if let Some(pm) = meta.get(peer_id)
                && let Some(ref v) = pm.version
                && v != our_version
            {
                version_skew.push(json!({
                    "peer": peer_id,
                    "version": v,
                    "local_version": our_version
                }));
            }
        }

        let locally_reachable: std::collections::HashSet<&String> = reachable.iter().collect();
        for (gate_id, pm) in meta.iter() {
            if pm.reachable_peers.is_empty() {
                continue;
            }
            for remote_peer in &pm.reachable_peers {
                if !locally_reachable.contains(remote_peer) && remote_peer != node_id.as_ref() {
                    partition_warnings.push(json!({
                        "peer": remote_peer,
                        "reachable_from": gate_id,
                        "unreachable_from": node_id.as_ref(),
                        "type": "local_partition"
                    }));
                }
            }
        }

        drop(meta);

        let mut response = json!({
            "initialized": true,
            "node_id": node_id.as_ref(),
            "reachable_peers": reachable.len(),
            "relay_enabled": true,
            "uptime_seconds": uptime,
            "version": our_version,
            "paths": {
                "local": local_count,
                "overlay": overlay_count,
                "direct": direct_count,
                "family_relay": relay_count,
                "onion": onion_count
            }
        });

        if !version_skew.is_empty() {
            response["version_skew"] = json!(version_skew);
        }
        if !partition_warnings.is_empty() {
            response["partition_warnings"] = json!(partition_warnings);
        }

        let stale_peers = detect_stale_gate_heads();
        if !stale_peers.is_empty() {
            response["stale_peers"] = json!(stale_peers);
        }

        Ok(response)
    }

    /// Handle `mesh.find_path` method - Find best path to a peer
    pub async fn handle_find_path(&self, params: Value) -> Result<Value, String> {
        let target_node_id = params
            .get("target_node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing target_node_id parameter")?;

        debug!("🔍 Finding path to {}", target_node_id);

        {
            let mesh = self
                .mesh
                .read()
                .await
                .as_ref()
                .cloned()
                .ok_or("Mesh not initialized (call mesh.init first)")?;

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
            let mesh = self
                .mesh
                .read()
                .await
                .as_ref()
                .cloned()
                .ok_or("Mesh not initialized (call mesh.init first)")?;
            let _msg = mesh.announce_as_relay().await;
            Ok::<_, String>(())
        }?;

        let node_id = self.node_id.read().await.clone();

        info!("📢 Announced {} as relay to mesh", &node_id.as_ref()[..8.min(node_id.len())]);

        Ok(json!({
            "announced": true,
            "node_id": node_id.as_ref(),
            "ttl_seconds": 300
        }))
    }

    /// Handle `mesh.peers` method - List known peers
    pub async fn handle_peers(&self, params: Value) -> Result<Value, String> {
        let _include_offline =
            params.get("include_offline").and_then(serde_json::Value::as_bool).unwrap_or(false);

        let (peers, relay_count) = {
            let mesh = self
                .mesh
                .read()
                .await
                .as_ref()
                .cloned()
                .ok_or("Mesh not initialized (call mesh.init first)")?;

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

                    peers.push((node_id.clone(), json!({
                        "node_id": node_id,
                        "path_type": path_type,
                        "priority": path.endpoint_type.priority(),
                        "address": address,
                        "last_seen_ms": u64::try_from(path.last_seen.elapsed().as_millis()).unwrap_or(u64::MAX),
                        "is_relay": is_relay,
                        "latency_ms": latency_ms,
                        "reachable": path.reachable
                    })));
                }
            }

            Ok::<_, String>((peers, relay_count))
        }?;

        let meta = self.peer_metadata.read().await;
        let our_version = env!("CARGO_PKG_VERSION");
        let enriched_peers: Vec<Value> = peers
            .into_iter()
            .map(|(node_id, mut peer_json)| {
                if let Some(pm) = meta.get(&node_id)
                    && let Some(ref v) = pm.version
                {
                    peer_json["version"] = json!(v);
                    if v != our_version {
                        peer_json["version_mismatch"] = json!(true);
                    }
                }
                peer_json
            })
            .collect();
        drop(meta);

        Ok(json!({
            "peers": enriched_peers,
            "total": enriched_peers.len(),
            "online": enriched_peers.len(),
            "relays": relay_count,
            "local_version": our_version
        }))
    }

    /// Handle `mesh.topology` method — Return full mesh graph topology
    ///
    /// Returns nodes with their connections and path types, giving a
    /// graph-level view of the mesh for monitoring and visualization.
    ///
    /// When `include_gossip` is true (default), merges peer-reported reachability
    /// into the graph, showing inferred edges from remote peer gossip.
    pub async fn handle_topology(&self, params: Value) -> Result<Value, String> {
        let mesh = self
            .mesh
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let node_id = self.node_id.read().await.clone();
        let reachable = mesh.get_reachable_nodes().await;

        let include_gossip = params.get("include_gossip").and_then(Value::as_bool).unwrap_or(true);

        if include_gossip {
            let meta = self.peer_metadata.read().await;
            let mut local_latencies = std::collections::HashMap::new();
            for peer_id in &reachable {
                if let Some(path) = mesh.get_best_path(peer_id).await
                    && let Some(lat) = path.latency
                {
                    local_latencies.insert(
                        peer_id.clone(),
                        u64::try_from(lat.as_millis()).unwrap_or(u64::MAX),
                    );
                }
            }

            let graph = topology_graph::build_topology(
                node_id.as_ref(),
                &reachable,
                &meta,
                &local_latencies,
            );

            let mut response = graph.to_json(node_id.as_ref(), self.start_time.elapsed().as_secs());
            let partitioned = graph.partitioned_nodes();
            if !partitioned.is_empty() {
                response["partitioned_nodes"] =
                    json!(partitioned.iter().map(|s| json!(s)).collect::<Vec<_>>());
            }
            return Ok(response);
        }

        // Legacy star-from-self view (include_gossip: false)
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let self_id = node_id.clone();
        nodes.push(json!({
            "id": self_id.as_ref(),
            "role": "self",
            "reachable": true
        }));

        for peer_id in &reachable {
            let paths = mesh.get_all_paths(peer_id).await;
            let best = mesh.get_best_path(peer_id).await;

            let is_relay =
                paths.iter().any(|p| matches!(p.endpoint_type, EndpointType::FamilyRelay { .. }));

            nodes.push(json!({
                "id": peer_id,
                "role": if is_relay { "relay" } else { "peer" },
                "reachable": best.as_ref().is_some_and(|b| b.reachable),
                "path_count": paths.len()
            }));

            for path in &paths {
                let (path_type, address) = json::endpoint_to_strings(&path.endpoint_type);
                edges.push(json!({
                    "from": self_id.as_ref(),
                    "to": peer_id,
                    "path_type": path_type,
                    "priority": path.endpoint_type.priority(),
                    "address": address,
                    "latency_ms": path.latency.map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX)),
                    "reachable": path.reachable,
                    "is_best": best.as_ref().is_some_and(|b| b.endpoint_type == path.endpoint_type)
                }));
            }
        }

        Ok(json!({
            "nodes": nodes,
            "edges": edges,
            "node_count": nodes.len(),
            "edge_count": edges.len(),
            "self_node_id": node_id.as_ref(),
            "uptime_seconds": self.start_time.elapsed().as_secs()
        }))
    }
}

impl Default for MeshHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
use songbird_onion_relay::mesh::RelayEndpoint;

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

    pub async fn test_init_with_peers(
        &self,
        node_id: &str,
        peers: &[(String, std::net::SocketAddr, bool)],
    ) {
        let mesh = Arc::new(BeaconMesh::new(String::from(node_id), vec![]));
        for (peer_id, addr, reachable) in peers {
            mesh.add_endpoint(
                peer_id.clone(),
                RelayEndpoint {
                    node_id: peer_id.clone(),
                    endpoint_type: EndpointType::Direct {
                        addr: *addr,
                    },
                    latency: None,
                    last_seen: Instant::now(),
                    reachable: *reachable,
                },
            )
            .await;
        }
        *self.mesh.write().await = Some(mesh);
        *self.node_id.write().await = Arc::from(node_id);
    }
}

/// Parse a peer list from a JSON array field.
///
/// Supports two formats:
/// - Object: `{"node_id": "...", "address": "host:port"}`
/// - String: `"node_id@host:port"` or `"host:port"` (auto-named)
///
/// If `require_node_id` is true, bare `"host:port"` strings without `@` are skipped.
fn parse_peer_list(
    params: &Value,
    keys: &[&str],
    require_node_id: bool,
) -> Vec<(String, std::net::SocketAddr)> {
    let arr = keys.iter().find_map(|k| params.get(*k).and_then(Value::as_array));
    let Some(arr) = arr else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|entry| {
            if let Some(obj_id) = entry.get("node_id").and_then(Value::as_str) {
                let addr_str = entry.get("address")?.as_str()?;
                let addr: std::net::SocketAddr = addr_str.parse().ok()?;
                return Some((obj_id.to_string(), addr));
            }
            let s = entry.as_str()?;
            if let Some((nid, addr_part)) = s.split_once('@') {
                let addr: std::net::SocketAddr = addr_part.parse().ok()?;
                Some((nid.to_string(), addr))
            } else if require_node_id {
                None
            } else {
                let addr: std::net::SocketAddr = s.parse().ok()?;
                Some((format!("peer-{}", addr.ip()), addr))
            }
        })
        .collect()
}

/// Scan `wateringHole/heads/*.toml` for gate head files older than 24 hours.
///
/// Returns a list of `{ gate, age_hours, file }` entries for stale peers.
/// Used by `mesh.status` to enrich the response with convergence health.
fn detect_stale_gate_heads() -> Vec<serde_json::Value> {
    const STALE_THRESHOLD_SECS: u64 = 24 * 3600;

    let workspace =
        std::env::var("ECOPRIMALS_ROOT").unwrap_or_else(|_| String::from("/opt/ecoPrimals"));
    let heads_dir =
        std::path::PathBuf::from(&workspace).join("infra").join("wateringHole").join("heads");

    let Ok(entries) = std::fs::read_dir(&heads_dir) else {
        return Vec::new();
    };

    let now = std::time::SystemTime::now();
    let mut stale = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let gate = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();

        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let age = now.duration_since(modified).unwrap_or_default();
        if age.as_secs() > STALE_THRESHOLD_SECS {
            let age_hours = age.as_secs() / 3600;
            stale.push(serde_json::json!({
                "gate": gate,
                "age_hours": age_hours,
                "file": path.display().to_string(),
            }));
        }
    }

    stale
}
