// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability Propagation — push model for cross-gate capability discovery.
//!
//! When a primal registers capabilities via `ipc.register`, Songbird announces
//! them to all reachable mesh peers via `mesh.capabilities_announce`. Remote
//! gates store these announcements so `discovery.peers` returns correct capability
//! lists without requiring polling.
//!
//! Includes retry logic: failed announcements are queued and retried on the
//! periodic health cycle (max [`MAX_ANNOUNCE_RETRIES`] attempts).

use serde_json::{Value, json};
use songbird_onion_relay::mesh::EndpointType;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

use super::MeshHandler;

/// A capability announcement that failed delivery and is queued for retry.
pub(super) struct PendingAnnounce {
    pub node_id: String,
    pub address: String,
    pub payload: Value,
    pub attempts: u8,
}

/// Maximum retry attempts for a failed capability announcement.
pub(super) const MAX_ANNOUNCE_RETRIES: u8 = 3;

impl MeshHandler {
    /// Get capabilities for a specific remote peer (from announcements).
    #[must_use = "peer capabilities should be used in discovery responses"]
    pub async fn get_peer_capabilities(&self, node_id: &str) -> Vec<String> {
        self.peer_capabilities.read().await.get(node_id).cloned().unwrap_or_default()
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

        debug!(
            peer = %node_id,
            count = capabilities.len(),
            "Accepted capability announcement from mesh peer"
        );

        self.peer_capabilities.write().await.insert(node_id.clone(), capabilities.clone());

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
                    } => {
                        format!("http://{}:{}/jsonrpc", addr.ip(), addr.port())
                    }
                    _ => continue,
                };
                targets.push((node_id.clone(), address));
            }
        }

        if targets.is_empty() {
            return;
        }

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "mesh.capabilities_announce",
            "params": {
                "node_id": our_node_id,
                "capabilities": local_capabilities
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
                    pending.write().await.push(PendingAnnounce {
                        node_id,
                        address,
                        payload,
                        attempts: 1,
                    });
                }
            });
        }
    }

    /// Retry failed capability announcements (called from health cycle).
    ///
    /// Drains the pending queue, retries each, and re-queues those that still fail
    /// (up to [`MAX_ANNOUNCE_RETRIES`] attempts). Peers that exceed the retry limit
    /// are dropped — they will receive a fresh announcement on the next `ipc.register`.
    pub async fn retry_pending_announces(&self) {
        let pending: Vec<PendingAnnounce> = {
            let mut guard = self.pending_announces.write().await;
            std::mem::take(&mut *guard)
        };

        if pending.is_empty() {
            return;
        }

        let requeue = Arc::clone(&self.pending_announces);
        for item in pending {
            if item.attempts >= MAX_ANNOUNCE_RETRIES {
                debug!(
                    peer = %item.node_id,
                    attempts = item.attempts,
                    "Dropping failed capability announce (max retries exceeded)"
                );
                continue;
            }

            let requeue = Arc::clone(&requeue);
            let node_id = item.node_id;
            let address = item.address;
            let payload = item.payload;
            let attempts = item.attempts;
            tokio::spawn(async move {
                if post_jsonrpc_fire_and_forget(&address, &payload).await.is_err() {
                    requeue.write().await.push(PendingAnnounce {
                        node_id,
                        address,
                        payload,
                        attempts: attempts + 1,
                    });
                }
            });
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
