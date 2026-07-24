// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery JSON-RPC Handler
//!
//! Exposes peer discovery and content distribution functionality via JSON-RPC
//! for Dark Forest rendezvous protocol and the seeder/leecher pattern.
//!
//! ## Methods
//! - `discovery.peers` — List discovered peers (supports `family_only`, `capability_filter`)
//! - `discovery.announce` — Announce presence or content availability to the mesh
//! - `discovery.content_peers` — Find seeders for a specific content topic
//! - `discovery.get_peer` — Get specific peer by ID
//!
//! ## Content Distribution
//! `discovery.announce` with a `topic` param stores content announcements in an
//! in-memory registry with TTL-based expiration. Leechers query available content
//! via `discovery.content_peers` to find seeders for specific topics. Topics use
//! the `content:<namespace>` convention (e.g., `content:ludospring:assets`) per
//! `content_distribution_federation.toml`. Manifest hashes use BLAKE3 addressing
//! from the storage provider's `ContentManifest`.
//!
//! ## Security Note
//! Peer information includes network addresses. Only expose to trusted consumers.

mod content;
mod slot;
pub mod types;

#[cfg(test)]
mod tests;

pub use content::ContentAnnouncement;
pub use slot::PeerRegistrySlot;
pub use types::{DiscoveredPeerInfo, DiscoveryGetPeerParams, DiscoveryPeersResult, PeerRegistry};

use crate::error::{IpcError, IpcResult};
use crate::handlers::mesh_handler::MeshHandler;
use content::ContentAnnouncementStore;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Discovery handler for peer discovery and content distribution operations.
///
/// Maintains both a peer registry (injected from orchestrator) and an in-memory
/// content announcement store for the seeder/leecher pattern defined by
/// `content_distribution_federation.toml`.
///
/// When a `MeshHandler` is attached, `discovery.peers` also returns peers known
/// to the beacon mesh (those added via `mesh.init` `bootstrap_peers`).
pub struct DiscoveryHandler {
    peer_registry: Option<PeerRegistrySlot>,
    mesh_handler: Option<Arc<MeshHandler>>,
    content_announcements: Arc<RwLock<ContentAnnouncementStore>>,
}

impl DiscoveryHandler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            peer_registry: None,
            mesh_handler: None,
            content_announcements: Arc::new(RwLock::new(ContentAnnouncementStore::new())),
        }
    }

    /// Attach the orchestrator discovery bridge (production).
    #[must_use]
    pub fn with_bridge(
        bridge: Arc<crate::handlers::discovery_bridge::DiscoveryListenerBridge>,
    ) -> Self {
        Self {
            peer_registry: Some(PeerRegistrySlot::Bridge(bridge)),
            mesh_handler: None,
            content_announcements: Arc::new(RwLock::new(ContentAnnouncementStore::new())),
        }
    }

    /// Inject a custom registry slot (e.g. `PeerRegistrySlot::Mock` in unit tests).
    #[must_use]
    pub fn with_peer_registry(slot: PeerRegistrySlot) -> Self {
        Self {
            peer_registry: Some(slot),
            mesh_handler: None,
            content_announcements: Arc::new(RwLock::new(ContentAnnouncementStore::new())),
        }
    }

    /// Attach a mesh handler so `discovery.peers` includes mesh-known peers.
    pub fn set_mesh_handler(&mut self, mesh: Arc<MeshHandler>) {
        self.mesh_handler = Some(mesh);
    }

    /// Handle `discovery.peers` JSON-RPC method.
    pub async fn handle_list_peers(&self, params: Value) -> IpcResult<DiscoveryPeersResult> {
        self.handle_list_peers_with(params, || {
            songbird_process_env::var("FAMILY_ID")
                .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
        })
        .await
    }

    /// Testable variant with injectable `family_id` resolver.
    async fn handle_list_peers_with<F>(
        &self,
        params: Value,
        resolve_family: F,
    ) -> IpcResult<DiscoveryPeersResult>
    where
        F: FnOnce() -> Result<String, std::env::VarError>,
    {
        let family_only = params.get("family_only").and_then(Value::as_bool).unwrap_or(false);

        let capability_filter: Vec<String> = match params.get("capability_filter") {
            Some(Value::Array(arr)) => {
                arr.iter().filter_map(Value::as_str).map(String::from).collect()
            }
            Some(Value::String(s)) => vec![s.clone()],
            _ => Vec::new(),
        };

        debug!(
            "Discovery: list_peers (family_only={family_only}, cap_filter={:?})",
            capability_filter,
        );

        let mut peers = if let Some(ref registry) = self.peer_registry {
            registry.get_all_peers().await?
        } else {
            Vec::new()
        };

        if let Some(ref mesh) = self.mesh_handler {
            let mesh_peers = self.collect_mesh_peers(mesh).await;
            for mp in mesh_peers {
                if !peers.iter().any(|p| p.node_id == mp.node_id) {
                    peers.push(mp);
                }
            }
        }

        if family_only {
            let own_family = resolve_family().unwrap_or_default();
            if own_family.is_empty() {
                debug!(
                    "Discovery: family_only requested but no FAMILY_ID set — returning all peers"
                );
            } else {
                debug!("Discovery: family_only filter active (family={own_family})");
                peers.retain(|peer| peer.family_id == own_family);
            }
        }

        if !capability_filter.is_empty() {
            peers.retain(|peer| {
                capability_filter
                    .iter()
                    .all(|required| peer.capabilities.iter().any(|c| c == required))
            });
        }

        let total_count = peers.len();
        info!("Discovery: found {total_count} peers");

        Ok(DiscoveryPeersResult {
            peers,
            total_count,
        })
    }

    /// Collect peers from the beacon mesh as [`DiscoveredPeerInfo`] entries.
    ///
    /// Capabilities are populated from `mesh.capabilities_announce` data received
    /// from remote peers (push/gossip propagation model).
    async fn collect_mesh_peers(&self, mesh: &MeshHandler) -> Vec<DiscoveredPeerInfo> {
        use songbird_onion_relay::EndpointType;

        let guard = mesh.mesh().await;
        let Some(ref beacon_mesh) = *guard else {
            return Vec::new();
        };

        let reachable = beacon_mesh.get_reachable_nodes().await;
        let mut result = Vec::new();

        for node_id in &reachable {
            let (address, latency_ms) = if let Some(path) = beacon_mesh.get_best_path(node_id).await
            {
                let addr = match &path.endpoint_type {
                    EndpointType::Direct {
                        addr,
                    }
                    | EndpointType::Local {
                        addr,
                    }
                    | EndpointType::Overlay {
                        addr,
                        ..
                    } => addr.to_string(),
                    EndpointType::FamilyRelay {
                        relay_node_id,
                    } => relay_node_id.clone(),
                    EndpointType::TorOnion {
                        onion_addr,
                    } => onion_addr.clone(),
                };
                let ms = path.latency.map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));
                (addr, ms)
            } else {
                (String::new(), None)
            };

            let tcp_port = address.parse::<std::net::SocketAddr>().ok().map(|a| a.port());
            let capabilities = mesh.get_peer_capabilities(node_id).await;

            result.push(DiscoveredPeerInfo {
                node_id: node_id.clone(),
                family_id: String::new(),
                address,
                tcp_port,
                capabilities,
                last_seen: "mesh".to_string(),
                quality: Some(1.0),
                node_name: None,
                protocols: vec!["tcp".to_string()],
                latency_ms,
            });
        }

        result
    }

    /// Handle `discovery.announce` JSON-RPC method.
    pub async fn handle_announce(&self, params: Value) -> IpcResult<Value> {
        let family_id = params.get("family_id").and_then(Value::as_str).unwrap_or("unknown");
        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();
        let topic = params.get("topic").and_then(Value::as_str);
        let manifest_hash = params.get("manifest_hash").and_then(Value::as_str);
        let seeder_count = params.get("seeder_count").and_then(Value::as_u64);
        let bond_types: Vec<String> = params
            .get("bond_types_accepted")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();
        let node_id = params.get("node_id").and_then(Value::as_str).unwrap_or("unknown");

        if let Some(topic) = topic {
            let announcement = ContentAnnouncement {
                topic: topic.to_string(),
                manifest_hash: manifest_hash.map(String::from),
                family_id: family_id.to_string(),
                node_id: node_id.to_string(),
                seeder_count: seeder_count.unwrap_or(1),
                bond_types_accepted: bond_types.clone(),
                announced_at: Instant::now(),
            };

            let mut store = self.content_announcements.write().await;
            store.gc();
            store.insert(announcement);

            info!(
                "Discovery: content announce stored (topic={}, manifest={}, seeders={}, node={})",
                topic,
                manifest_hash.unwrap_or("none"),
                seeder_count.unwrap_or(1),
                node_id,
            );

            Ok(serde_json::json!({
                "announced": true,
                "mode": "topic",
                "topic": topic,
                "manifest_hash": manifest_hash,
                "seeder_count": seeder_count.unwrap_or(1),
                "bond_types_accepted": bond_types,
                "family_id": family_id,
                "node_id": node_id,
            }))
        } else {
            info!(
                "Discovery: presence announce (family={}, capabilities={})",
                family_id,
                capabilities.len()
            );
            Ok(serde_json::json!({
                "announced": true,
                "mode": "presence",
                "family_id": family_id,
                "capabilities": capabilities,
            }))
        }
    }

    /// Handle `discovery.content_peers` JSON-RPC method.
    pub async fn handle_content_peers(&self, params: Value) -> IpcResult<Value> {
        self.handle_content_peers_with(params, || {
            songbird_process_env::var("FAMILY_ID")
                .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
        })
        .await
    }

    /// Testable variant with injectable `family_id` resolver.
    async fn handle_content_peers_with<F>(
        &self,
        params: Value,
        resolve_family: F,
    ) -> IpcResult<Value>
    where
        F: FnOnce() -> Result<String, std::env::VarError>,
    {
        let topic = params
            .get("topic")
            .and_then(Value::as_str)
            .ok_or_else(|| IpcError::InvalidParams("missing required 'topic' param".into()))?;
        let family_only = params.get("family_only").and_then(Value::as_bool).unwrap_or(false);
        let manifest_filter = params.get("manifest_hash").and_then(Value::as_str);

        let store = self.content_announcements.read().await;
        let mut announcements: Vec<&ContentAnnouncement> = store.query(topic);

        if let Some(hash) = manifest_filter {
            announcements.retain(|a| a.manifest_hash.as_deref() == Some(hash));
        }

        if family_only {
            let own_family = resolve_family().unwrap_or_default();
            if own_family.is_empty() {
                warn!("Discovery: family_only requested but no FAMILY_ID set");
            } else {
                announcements.retain(|a| a.family_id == own_family);
            }
        }

        let results: Vec<Value> = announcements
            .iter()
            .map(|a| {
                serde_json::json!({
                    "node_id": a.node_id,
                    "family_id": a.family_id,
                    "topic": a.topic,
                    "manifest_hash": a.manifest_hash,
                    "seeder_count": a.seeder_count,
                    "bond_types_accepted": a.bond_types_accepted,
                })
            })
            .collect();

        let total = results.len();
        info!("Discovery: content_peers query (topic={topic}) found {total} seeders");

        Ok(serde_json::json!({
            "seeders": results,
            "total_count": total,
            "topic": topic,
        }))
    }

    /// Handle `discovery.get_peer` JSON-RPC method.
    pub async fn handle_get_peer(&self, params: Value) -> IpcResult<Option<DiscoveredPeerInfo>> {
        let params: DiscoveryGetPeerParams = serde_json::from_value(params)
            .map_err(|e| IpcError::InvalidParams(format!("Failed to parse params: {e}")))?;

        debug!("Discovery: get_peer (peer_id: {})", params.peer_id);

        if let Some(ref registry) = self.peer_registry {
            let peer = registry.get_peer(&params.peer_id).await?;
            Ok(peer)
        } else {
            Ok(None)
        }
    }

    /// Handle `discovery.topology` — mesh gate topology for composition consumers.
    ///
    /// Returns the current mesh topology: gates, overlay addresses, connectivity.
    /// Used by esotericWebb to render the mesh graph in real-time.
    pub async fn handle_topology(&self, _params: Value) -> IpcResult<Value> {
        let mut gates = Vec::new();

        if let Some(ref mesh) = self.mesh_handler {
            let guard = mesh.mesh().await;
            if let Some(ref beacon_mesh) = *guard {
                let reachable = beacon_mesh.get_reachable_nodes().await;
                for node_id in &reachable {
                    let endpoint = beacon_mesh
                        .get_best_path(node_id)
                        .await
                        .map(|p| format!("{:?}", p.endpoint_type));
                    gates.push(serde_json::json!({
                        "node_id": node_id,
                        "reachable": true,
                        "endpoint": endpoint,
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "gates": gates,
            "gate_count": gates.len(),
            "self_node_id": songbird_process_env::var("SONGBIRD_NODE_ID").unwrap_or_default(),
        }))
    }

    /// Handle `discovery.health` — node health status.
    ///
    /// Returns liveness and readiness information. Designed for health checks
    /// from composition consumers (esotericWebb, footPrint).
    pub async fn handle_health(&self, _params: Value) -> IpcResult<Value> {
        let mesh_active = if let Some(ref mesh) = self.mesh_handler {
            mesh.mesh().await.is_some()
        } else {
            false
        };

        let peer_count = if let Some(ref registry) = self.peer_registry {
            registry.get_all_peers().await.map(|p| p.len()).unwrap_or(0)
        } else {
            0
        };

        Ok(serde_json::json!({
            "alive": true,
            "mesh_active": mesh_active,
            "peer_count": peer_count,
            "uptime_secs": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }))
    }

    /// Handle `discovery.query` — generic capability/service discovery.
    ///
    /// Accepts optional `capability` filter and returns services providing it.
    pub async fn handle_query(&self, params: Value) -> IpcResult<Value> {
        let capability_filter = params.get("capability").and_then(Value::as_str);

        let peers = if let Some(ref registry) = self.peer_registry {
            registry.get_all_peers().await.unwrap_or_default()
        } else {
            Vec::new()
        };

        let results: Vec<&DiscoveredPeerInfo> = capability_filter.map_or_else(
            || peers.iter().collect(),
            |cap| peers.iter().filter(|p| p.capabilities.iter().any(|c| c == cap)).collect(),
        );

        let services: Vec<Value> = results
            .iter()
            .map(|p| {
                serde_json::json!({
                    "node_id": p.node_id,
                    "address": p.address,
                    "capabilities": p.capabilities,
                    "protocols": p.protocols,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "services": services,
            "total_count": services.len(),
            "filter": capability_filter,
        }))
    }

    /// Handle `discovery.bonds` — external API bonds configured on this node.
    ///
    /// Returns the drawbridge external allowlist (science APIs, GIS services, etc.)
    /// configured via `SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST`.
    pub async fn handle_bonds(&self, _params: Value) -> IpcResult<Value> {
        let allowlist_raw =
            songbird_process_env::var("SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST").unwrap_or_default();

        let bonds: Vec<Value> = allowlist_raw
            .split(',')
            .filter_map(|entry| {
                let entry = entry.trim();
                let (name, url) = entry.split_once('=')?;
                Some(serde_json::json!({
                    "service": name.trim(),
                    "base_url": url.trim(),
                }))
            })
            .collect();

        Ok(serde_json::json!({
            "bonds": bonds,
            "bond_count": bonds.len(),
        }))
    }
}

impl Default for DiscoveryHandler {
    fn default() -> Self {
        Self::new()
    }
}
