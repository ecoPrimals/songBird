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
//! within [`CAPABILITY_TTL`]) are evicted on each health cycle.

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
    /// expired (older than [`CAPABILITY_TTL`]).
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
    /// Searches the `peer_capabilities` map for a non-expired entry containing
    /// the capability string. Returns `Some((node_id, capabilities))` for the
    /// first match, or `None` if no peer provides it.
    pub async fn find_peer_with_capability(
        &self,
        capability: &str,
    ) -> Option<(String, Vec<String>)> {
        let guard = self.peer_capabilities.read().await;
        for (node_id, entry) in guard.iter() {
            if entry.last_seen.elapsed() < CAPABILITY_TTL
                && entry.capabilities.iter().any(|c| c == capability)
            {
                return Some((node_id.clone(), entry.capabilities.clone()));
            }
        }
        None
    }

    /// Handle `mesh.capabilities_announce` — receive remote peer capabilities.
    ///
    /// Called by remote gates when their primals register capabilities.
    /// Stores the announced capabilities so `discovery.peers` can return them.
    pub async fn handle_capabilities_announce(&self, params: Value) -> Result<Value, String> {
        let node_id =
            params.get("node_id").and_then(Value::as_str).ok_or("Missing node_id")?.to_string();

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

        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();

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

    /// Retry failed capability announcements (called from health cycle).
    ///
    /// Uses exponential backoff: an entry with N attempts is only retried if
    /// `2^N` health cycles have elapsed since enqueueing. Entries exceeding
    /// [`MAX_ANNOUNCE_RETRIES`] or older than 10 minutes are dropped.
    ///
    /// Also evicts stale entries from `peer_capabilities` that haven't been
    /// refreshed within [`CAPABILITY_TTL`].
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
