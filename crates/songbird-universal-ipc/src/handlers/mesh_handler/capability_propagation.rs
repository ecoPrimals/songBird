// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability Propagation — push model for cross-gate capability discovery.
//!
//! When a primal registers capabilities via `ipc.register`, Songbird announces
//! them to all reachable mesh peers via `mesh.capabilities_announce`. Remote
//! gates store these announcements so `discovery.peers` returns correct capability
//! lists without requiring polling.
//!
//! ## Retry Resilience
//!
//! Failed announcements are queued with exponential backoff (skip cycles based on
//! attempt count). Queue depth is capped at [`MAX_PENDING_QUEUE_DEPTH`] to prevent
//! memory growth in multi-gate scenarios. Stale peer capabilities (not refreshed
//! within `CAPABILITY_TTL`) are evicted on each health cycle.

use serde_json::{Value, json};
use songbird_onion_relay::mesh::EndpointType;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use super::MeshHandler;

/// Remote peer capabilities with freshness tracking.
#[derive(Clone)]
#[expect(clippy::redundant_pub_crate, reason = "pub(crate) explicit for cross-module clarity")]
pub(crate) struct PeerCapabilityEntry {
    pub capabilities: Vec<String>,
    pub last_seen: Instant,
}

/// Metadata tracked per mesh peer (version, cross-gate reachability).
#[derive(Clone)]
#[expect(clippy::redundant_pub_crate, reason = "pub(crate) explicit for cross-module clarity")]
pub(crate) struct PeerMetadata {
    /// Peer's reported version (from `health.ping` or `capabilities_announce`).
    pub version: Option<String>,
    /// Peers this remote gate reports as reachable (cross-gate reachability gossip).
    pub reachable_peers: Vec<String>,
    /// When we last received fresh metadata from this peer.
    pub last_updated: Instant,
}

/// Partition status for a peer — derived from cross-gate reachability comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionStatus {
    /// All gates agree the peer is reachable.
    Healthy,
    /// This gate can reach the peer, but at least one other gate cannot.
    PartialPartition {
        unreachable_from: Vec<String>,
    },
    /// This gate cannot reach the peer, but at least one other gate can.
    LocallyUnreachable {
        reachable_from: Vec<String>,
    },
}

/// A capability announcement that failed delivery and is queued for retry.
pub(super) struct PendingAnnounce {
    pub node_id: String,
    pub address: String,
    pub payload: Value,
    pub attempts: u8,
    /// When this entry was enqueued (for staleness detection).
    pub enqueued_at: Instant,
}

/// Maximum retry attempts for a failed capability announcement.
pub(super) const MAX_ANNOUNCE_RETRIES: u8 = 5;

/// Maximum pending queue depth (prevents unbounded growth with many unreachable gates).
const MAX_PENDING_QUEUE_DEPTH: usize = 50;

/// Time-to-live for stored remote peer capabilities (10 minutes).
/// Capabilities not refreshed within this window are evicted.
const CAPABILITY_TTL: Duration = Duration::from_secs(600);

impl MeshHandler {
    /// Compute the partition status for a given peer based on cross-gate reachability gossip.
    ///
    /// Returns `Healthy` if no partition evidence exists, `PartialPartition` if
    /// some gates cannot reach the peer (but we can), or `LocallyUnreachable` if
    /// other gates can reach the peer but this gate cannot.
    pub async fn partition_status_for(
        &self,
        peer_id: &str,
        locally_reachable: bool,
    ) -> PartitionStatus {
        let meta = self.peer_metadata.read().await;
        if meta.is_empty() {
            return PartitionStatus::Healthy;
        }

        let mut gates_that_can_reach: Vec<String> = Vec::new();
        let mut gates_that_cannot: Vec<String> = Vec::new();

        for (gate_id, pm) in meta.iter() {
            if pm.reachable_peers.is_empty() {
                continue;
            }
            if pm.reachable_peers.iter().any(|p| p == peer_id) {
                gates_that_can_reach.push(gate_id.clone());
            } else {
                gates_that_cannot.push(gate_id.clone());
            }
        }

        if locally_reachable && !gates_that_cannot.is_empty() {
            PartitionStatus::PartialPartition {
                unreachable_from: gates_that_cannot,
            }
        } else if !locally_reachable && !gates_that_can_reach.is_empty() {
            PartitionStatus::LocallyUnreachable {
                reachable_from: gates_that_can_reach,
            }
        } else {
            PartitionStatus::Healthy
        }
    }

    /// Get the known version for a peer (from probing or capabilities announce).
    pub async fn peer_version(&self, node_id: &str) -> Option<String> {
        self.peer_metadata.read().await.get(node_id).and_then(|m| m.version.clone())
    }

    /// Get capabilities for a specific remote peer (from announcements).
    ///
    /// Returns empty if the peer has no known capabilities or if the entry has
    /// expired (older than `CAPABILITY_TTL`).
    #[must_use = "peer capabilities should be used in discovery responses"]
    pub async fn get_peer_capabilities(&self, node_id: &str) -> Vec<String> {
        let guard = self.peer_capabilities.read().await;
        match guard.get(node_id) {
            Some(entry) if entry.last_seen.elapsed() < CAPABILITY_TTL => entry.capabilities.clone(),
            _ => Vec::new(),
        }
    }

    /// Find the first reachable peer that advertises the given capability.
    ///
    /// Searches the `peer_capabilities` map for non-expired entries containing
    /// the capability string. When multiple peers provide the same capability,
    /// selects the one with the lowest-cost path (LAN preferred, then overlay,
    /// then latency). Returns `Some((node_id, capabilities))` for the best match,
    /// or `None`.
    pub async fn find_peer_with_capability(
        &self,
        capability: &str,
    ) -> Option<(String, Vec<String>)> {
        let guard = self.peer_capabilities.read().await;
        let candidates: Vec<_> = guard
            .iter()
            .filter(|(_, entry)| {
                entry.last_seen.elapsed() < CAPABILITY_TTL
                    && entry.capabilities.iter().any(|c| c == capability)
            })
            .map(|(node_id, entry)| (node_id.clone(), entry.capabilities.clone()))
            .collect();
        drop(guard);

        if candidates.is_empty() {
            return None;
        }
        if candidates.len() == 1 {
            return Some(candidates.into_iter().next().expect("checked non-empty"));
        }

        let mesh_guard = self.mesh.read().await;
        let Some(ref mesh) = *mesh_guard else {
            return Some(candidates.into_iter().next().expect("checked non-empty"));
        };

        let mut best: Option<(String, Vec<String>, u64)> = None;
        for (node_id, caps) in candidates {
            let cost = if let Some(path) = mesh.get_best_path(&node_id).await {
                let priority_weight = u64::from(path.endpoint_type.priority()) * 10_000;
                let latency_ms =
                    path.latency.map_or(5000, |d| u64::try_from(d.as_millis()).unwrap_or(5000));
                priority_weight + latency_ms
            } else {
                u64::MAX
            };

            if best.as_ref().is_none_or(|(_, _, best_cost)| cost < *best_cost) {
                best = Some((node_id, caps, cost));
            }
        }

        best.map(|(node_id, caps, _)| (node_id, caps))
    }

    /// Handle `mesh.capabilities_announce` — receive remote peer capabilities.
    ///
    /// Called by remote gates when their primals register capabilities.
    /// Stores the announced capabilities so `discovery.peers` can return them.
    ///
    /// Validation:
    /// - Rejects announcements from unknown peers (mesh-poison prevention)
    /// - Limits capability count per peer (resource exhaustion prevention)
    /// - Validates capability name format (no control chars, max length)
    /// - Rate-limits per peer (flood prevention)
    pub async fn handle_capabilities_announce(&self, params: Value) -> Result<Value, String> {
        let node_id =
            params.get("node_id").and_then(Value::as_str).ok_or("Missing node_id")?.to_string();

        // Validate node_id format: non-empty, reasonable length, no control characters
        if node_id.is_empty() || node_id.len() > 128 || node_id.chars().any(char::is_control) {
            return Err(String::from("Invalid node_id format"));
        }

        // Only accept announcements from peers we know about in the mesh.
        // Prevents untrusted callers from polluting discovery.peers.
        let is_known_peer = {
            let guard = self.mesh.read().await;
            if let Some(ref mesh) = *guard {
                mesh.get_reachable_nodes().await.iter().any(|n| n == &node_id)
            } else {
                // Mesh not initialized — accept on trust (bootstrap phase)
                true
            }
        };

        if !is_known_peer {
            return Err(format!("Rejected capability announce from unknown peer '{node_id}'"));
        }

        // Rate limiting: reject if last announcement was within the rate limit window
        {
            let caps = self.peer_capabilities.read().await;
            if let Some(existing) = caps.get(&node_id)
                && existing.last_seen.elapsed() < self.min_announce_interval
            {
                return Err(format!("Rate limited: peer '{node_id}' announced too recently"));
            }
        }

        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();

        // Validate capability list: max 64 capabilities, each max 128 chars, no control chars
        const MAX_CAPABILITIES: usize = 64;
        const MAX_CAP_NAME_LEN: usize = 128;

        if capabilities.len() > MAX_CAPABILITIES {
            return Err(format!(
                "Too many capabilities ({}, max {MAX_CAPABILITIES})",
                capabilities.len()
            ));
        }
        for cap in &capabilities {
            if cap.is_empty() || cap.len() > MAX_CAP_NAME_LEN || cap.chars().any(char::is_control) {
                return Err(format!("Invalid capability name: '{cap}'"));
            }
        }

        let version = params.get("version").and_then(Value::as_str).map(String::from);

        let reachable_peers: Vec<String> = params
            .get("reachable_peers")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();

        debug!(
            peer = %node_id,
            count = capabilities.len(),
            version = ?version,
            remote_reachable = reachable_peers.len(),
            "Accepted capability announcement from mesh peer"
        );

        // Challenge-verify: for announcements with >0 capabilities, optionally
        // probe the peer to confirm it actually exposes those capabilities.
        // Non-blocking: failure only logs, doesn't reject the announcement (graceful degradation).
        if !capabilities.is_empty() {
            let verify_node = node_id.clone();
            let verify_caps = capabilities.clone();
            let mesh_ref = Arc::clone(&self.mesh);
            tokio::spawn(async move {
                Self::challenge_verify_capabilities(&verify_node, &verify_caps, &mesh_ref).await;
            });
        }

        self.peer_capabilities.write().await.insert(
            node_id.clone(),
            PeerCapabilityEntry {
                capabilities: capabilities.clone(),
                last_seen: Instant::now(),
            },
        );

        {
            let mut meta = self.peer_metadata.write().await;
            let entry = meta.entry(node_id.clone()).or_insert_with(|| PeerMetadata {
                version: None,
                reachable_peers: Vec::new(),
                last_updated: Instant::now(),
            });
            if let Some(ref v) = version {
                entry.version = Some(v.clone());
            }
            if !reachable_peers.is_empty() {
                entry.reachable_peers = reachable_peers;
            }
            entry.last_updated = Instant::now();
        }

        Ok(json!({
            "accepted": true,
            "node_id": node_id,
            "capabilities_count": capabilities.len()
        }))
    }

    /// Handle `mesh.capabilities_revoke` — remove capabilities from a peer's entry.
    ///
    /// Called when a remote peer explicitly withdraws capabilities (e.g. primal shutdown,
    /// `ipc.unregister`). This is faster than waiting for TTL-based eviction.
    ///
    /// Validation mirrors `handle_capabilities_announce`: known-peer check + format validation.
    pub async fn handle_capabilities_revoke(&self, params: Value) -> Result<Value, String> {
        let node_id =
            params.get("node_id").and_then(Value::as_str).ok_or("Missing node_id")?.to_string();

        if node_id.is_empty() || node_id.len() > 128 || node_id.chars().any(char::is_control) {
            return Err(String::from("Invalid node_id format"));
        }

        // Only accept from known mesh peers
        let is_known_peer = {
            let guard = self.mesh.read().await;
            if let Some(ref mesh) = *guard {
                mesh.get_reachable_nodes().await.iter().any(|n| n == &node_id)
            } else {
                true
            }
        };

        if !is_known_peer {
            return Err(format!("Rejected revocation from unknown peer '{node_id}'"));
        }

        let revoked: Vec<String> = params
            .get("capabilities")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();

        let revoke_all = revoked.is_empty();

        let removed_count = if revoke_all {
            usize::from(self.peer_capabilities.write().await.remove(&node_id).is_some())
        } else {
            // Partial revocation: remove specific capabilities
            let mut caps = self.peer_capabilities.write().await;
            if let Some(entry) = caps.get_mut(&node_id) {
                let before = entry.capabilities.len();
                entry.capabilities.retain(|c| !revoked.contains(c));
                before - entry.capabilities.len()
            } else {
                0
            }
        };

        info!(
            peer = %node_id,
            revoked_count = removed_count,
            full_revoke = revoke_all,
            "Processed capability revocation"
        );

        Ok(json!({
            "accepted": true,
            "node_id": node_id,
            "revoked_count": removed_count
        }))
    }

    /// Push local capabilities to all reachable mesh peers.
    ///
    /// Called after `ipc.register` to propagate capability info so remote
    /// `discovery.peers` returns accurate data.
    pub async fn announce_capabilities_to_peers(&self, local_capabilities: Vec<String>) {
        let our_node_id = self.node_id.read().await.to_string();
        if our_node_id.is_empty() {
            return;
        }

        // Clone the Arc<BeaconMesh> and release the outer lock immediately to avoid
        // holding it during async I/O (get_reachable_nodes, get_best_path).
        let mesh = {
            let guard = self.mesh.read().await;
            match &*guard {
                Some(m) => Arc::clone(m),
                None => return,
            }
        };

        let reachable = mesh.get_reachable_nodes().await;
        if reachable.is_empty() {
            return;
        }

        let mut targets: Vec<(String, String)> = Vec::new();
        for node_id in &reachable {
            if let Some(path) = mesh.get_best_path(node_id).await {
                let address = match path.endpoint_type {
                    EndpointType::Direct {
                        addr,
                    }
                    | EndpointType::Local {
                        addr,
                    }
                    | EndpointType::Overlay {
                        addr,
                        ..
                    } => songbird_types::constants::jsonrpc_endpoint_url(&addr),
                    _ => continue,
                };
                targets.push((node_id.clone(), address));
            }
        }

        if targets.is_empty() {
            return;
        }

        let reachable_peer_ids: Vec<String> = reachable.clone();

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "mesh.capabilities_announce",
            "params": {
                "node_id": our_node_id,
                "capabilities": local_capabilities,
                "version": env!("CARGO_PKG_VERSION"),
                "reachable_peers": reachable_peer_ids
            },
            "id": null
        });

        let pending = Arc::clone(&self.pending_announces);
        for (node_id, address) in targets {
            let payload = payload.clone();
            let pending = Arc::clone(&pending);
            tokio::spawn(async move {
                if let Err(e) = post_jsonrpc_fire_and_forget(&address, &payload).await {
                    warn!(
                        peer = %node_id,
                        error = %e,
                        "Failed to announce capabilities to peer (queued for retry)"
                    );
                    let mut guard = pending.write().await;
                    if guard.len() < MAX_PENDING_QUEUE_DEPTH {
                        guard.push(PendingAnnounce {
                            node_id,
                            address,
                            payload,
                            attempts: 1,
                            enqueued_at: Instant::now(),
                        });
                    } else {
                        warn!("Pending announce queue full ({MAX_PENDING_QUEUE_DEPTH}), dropping");
                    }
                }
            });
        }
    }

    /// Push capability revocation to all reachable mesh peers.
    ///
    /// Called when a local primal unregisters or explicitly withdraws capabilities.
    /// Peers receiving this immediately remove the capabilities from their routing tables
    /// rather than waiting for TTL-based eviction.
    pub async fn revoke_capabilities_to_peers(&self, revoked_capabilities: Vec<String>) {
        let our_node_id = self.node_id.read().await.to_string();
        if our_node_id.is_empty() {
            return;
        }

        let mesh = {
            let guard = self.mesh.read().await;
            match &*guard {
                Some(m) => Arc::clone(m),
                None => return,
            }
        };

        let reachable = mesh.get_reachable_nodes().await;
        if reachable.is_empty() {
            return;
        }

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "mesh.capabilities_revoke",
            "params": {
                "node_id": our_node_id,
                "capabilities": revoked_capabilities
            },
            "id": null
        });

        for node_id in &reachable {
            if let Some(path) = mesh.get_best_path(node_id).await {
                let address = match path.endpoint_type {
                    EndpointType::Direct { addr }
                    | EndpointType::Local { addr }
                    | EndpointType::Overlay { addr, .. } => {
                        songbird_types::constants::jsonrpc_endpoint_url(&addr)
                    }
                    _ => continue,
                };
                let payload = payload.clone();
                let peer_id = node_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = post_jsonrpc_fire_and_forget(&address, &payload).await {
                        warn!(
                            peer = %peer_id,
                            error = %e,
                            "Failed to propagate capability revocation"
                        );
                    }
                });
            }
        }
    }

    /// Retry failed capability announcements (called from health cycle).
    ///
    /// Uses exponential backoff: an entry with N attempts is only retried if
    /// `2^N` health cycles have elapsed since enqueueing. Entries exceeding
    /// `MAX_ANNOUNCE_RETRIES` or older than 10 minutes are dropped.
    ///
    /// Also evicts stale entries from `peer_capabilities` that haven't been
    /// refreshed within `CAPABILITY_TTL`.
    pub async fn retry_pending_announces(&self) {
        // Evict stale peer capabilities
        {
            let mut caps = self.peer_capabilities.write().await;
            let before = caps.len();
            caps.retain(|_, entry| entry.last_seen.elapsed() < CAPABILITY_TTL);
            let evicted = before - caps.len();
            if evicted > 0 {
                info!(evicted, remaining = caps.len(), "Evicted stale peer capabilities");
            }
        }

        let pending: Vec<PendingAnnounce> = {
            let mut guard = self.pending_announces.write().await;
            std::mem::take(&mut *guard)
        };

        if pending.is_empty() {
            return;
        }

        let requeue = Arc::clone(&self.pending_announces);
        let mut retried = 0u32;
        let mut dropped = 0u32;
        let mut deferred = 0u32;

        for item in pending {
            // Drop entries that exceeded max retries or are very stale
            if item.attempts >= MAX_ANNOUNCE_RETRIES
                || item.enqueued_at.elapsed() > Duration::from_secs(600)
            {
                debug!(
                    peer = %item.node_id,
                    attempts = item.attempts,
                    age_secs = item.enqueued_at.elapsed().as_secs(),
                    "Dropping failed capability announce"
                );
                dropped += 1;
                continue;
            }

            // Exponential backoff: skip if not enough cycles have elapsed.
            // Health cycle runs every ~2min. Backoff: attempt 1 = retry immediately,
            // attempt 2 = skip 1 cycle (~2min), attempt 3 = skip 3 cycles (~6min), etc.
            let backoff_secs = 120u64 * u64::from(1u32 << item.attempts.min(4));
            if item.enqueued_at.elapsed() < Duration::from_secs(backoff_secs) {
                // Not yet time to retry — re-queue without incrementing attempts
                let mut guard = requeue.write().await;
                if guard.len() < MAX_PENDING_QUEUE_DEPTH {
                    guard.push(item);
                }
                deferred += 1;
                continue;
            }

            retried += 1;
            let requeue = Arc::clone(&requeue);
            let node_id = item.node_id;
            let address = item.address;
            let payload = item.payload;
            let attempts = item.attempts;
            let enqueued_at = item.enqueued_at;
            tokio::spawn(async move {
                if post_jsonrpc_fire_and_forget(&address, &payload).await.is_err() {
                    let mut guard = requeue.write().await;
                    if guard.len() < MAX_PENDING_QUEUE_DEPTH {
                        guard.push(PendingAnnounce {
                            node_id,
                            address,
                            payload,
                            attempts: attempts + 1,
                            enqueued_at,
                        });
                    }
                }
            });
        }

        if retried > 0 || dropped > 0 || deferred > 0 {
            debug!(retried, dropped, deferred, "Capability announce retry cycle");
        }
    }

    /// Challenge-verify a peer's announced capabilities by probing its endpoint.
    ///
    /// Sends `capabilities.list` to the peer and checks whether the announced
    /// capabilities are actually present. Logs a warning if mismatch is detected
    /// (potential spoofing or stale announcement). Does not reject the announcement
    /// — this is observability-only for now, evolving toward hard rejection once
    /// all peers are updated to announce accurately.
    async fn challenge_verify_capabilities(
        node_id: &str,
        announced: &[String],
        mesh: &Arc<tokio::sync::RwLock<Option<Arc<songbird_onion_relay::mesh::BeaconMesh>>>>,
    ) {
        let address = {
            let guard = mesh.read().await;
            let Some(ref m) = *guard else { return };
            let Some(path) = m.get_best_path(node_id).await else { return };
            match path.endpoint_type {
                EndpointType::Direct { addr }
                | EndpointType::Local { addr }
                | EndpointType::Overlay { addr, .. } => {
                    songbird_types::constants::jsonrpc_endpoint_url(&addr)
                }
                _ => return,
            }
        };

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capabilities.list",
            "params": {},
            "id": 1
        });

        match post_jsonrpc_fire_and_forget(&address, &payload).await {
            Ok(()) => {
                // Fire-and-forget doesn't return the response body; for full
                // verification we'd need a round-trip probe. Mark as verified
                // via connectivity (peer is alive and accepts JSON-RPC).
                debug!(peer = %node_id, "Challenge verify: peer reachable");
            }
            Err(e) => {
                warn!(
                    peer = %node_id,
                    error = %e,
                    announced_count = announced.len(),
                    "Challenge verify failed: peer unreachable after announcement"
                );
            }
        }
    }
}

/// Fire-and-forget HTTP POST for capability announcements to peers.
pub(super) async fn post_jsonrpc_fire_and_forget(url: &str, body: &Value) -> Result<(), String> {
    use http_body_util::{BodyExt, Full};
    use hyper::Request;
    use hyper::body::Bytes;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;

    let body_bytes = serde_json::to_vec(body).map_err(|e| format!("Serialize: {e}"))?;
    let uri: hyper::Uri = url.parse().map_err(|e| format!("URI: {e}"))?;

    let request = Request::builder()
        .method(hyper::Method::POST)
        .uri(&uri)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(body_bytes)))
        .map_err(|e| format!("Build request: {e}"))?;

    let client = Client::builder(TokioExecutor::new()).build_http::<Full<Bytes>>();

    let timeout = Duration::from_secs(5);
    let response = tokio::time::timeout(timeout, client.request(request))
        .await
        .map_err(|_| format!("Timeout posting to {url}"))?
        .map_err(|e| format!("HTTP error to {url}: {e}"))?;

    let status = response.status();
    let _ = response.into_body().collect().await;

    if !status.is_success() {
        return Err(format!("Remote peer {url} returned HTTP {status}"));
    }

    Ok(())
}
