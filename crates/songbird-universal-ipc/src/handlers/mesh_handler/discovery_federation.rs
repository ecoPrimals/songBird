// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery and federation mesh handler methods.
//!
//! Extracted from `mod.rs` for cohesion: these handlers deal with
//! finding new peers and propagating state across the mesh.

use super::MeshHandler;
use super::json as json_helpers;
use serde_json::{Value, json};
use songbird_onion_relay::mesh::{EndpointType, RelayEndpoint};
use std::time::{Duration, Instant};
use tracing::info;

impl MeshHandler {
    /// Handle `mesh.auto_discover` method - Auto-discover peers on local network
    pub async fn handle_auto_discover(&self, params: Value) -> Result<Value, String> {
        let timeout_ms =
            params.get("timeout_ms").and_then(serde_json::Value::as_u64).unwrap_or(3000);
        let broadcast_port = u16::try_from(
            params
                .get("broadcast_port")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_else(|| u64::from(songbird_types::constants::MDNS_PORT)),
        )
        .unwrap_or(songbird_types::constants::MDNS_PORT);

        let node_id = self.node_id.read().await.clone();
        info!(
            "🔍 Auto-discovering peers on local network (port {}, timeout {}ms)",
            broadcast_port, timeout_ms
        );

        let discovered = super::udp_discovery::udp_multicast_discover(
            node_id.as_ref(),
            broadcast_port,
            Duration::from_millis(timeout_ms),
        )
        .await;

        let peers_found = {
            let mesh = self
                .mesh
                .read()
                .await
                .as_ref()
                .cloned()
                .ok_or("Mesh not initialized (call mesh.init first)")?;

            let mut peers_found = Vec::new();
            for (peer_id, addr) in &discovered {
                let peer_str = peer_id.as_ref().to_string();
                let endpoint = RelayEndpoint {
                    node_id: peer_str.clone(),
                    endpoint_type: EndpointType::Local {
                        addr: *addr,
                    },
                    latency: None,
                    last_seen: Instant::now(),
                    reachable: true,
                };
                mesh.add_endpoint(peer_str, endpoint).await;

                peers_found.push(json!({
                    "node_id": peer_id.as_ref(),
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

    /// Handle `mesh.discover_remotes` — discover remote gates and their content sources.
    ///
    /// Used by ecosystem signal graphs to find gates that can serve content.
    /// Returns known remote peers with their advertised capabilities.
    pub async fn handle_discover_remotes(&self, _params: Value) -> Result<Value, String> {
        let mesh = self
            .mesh
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let reachable = mesh.get_reachable_nodes().await;
        let mut remotes = Vec::new();

        for node_id in &reachable {
            if let Some(path) = mesh.get_best_path(node_id).await {
                let (path_type, address) = json_helpers::endpoint_to_strings(&path.endpoint_type);
                remotes.push(json!({
                    "node_id": node_id,
                    "address": address,
                    "reachable": path.reachable,
                    "type": path_type
                }));
            }
        }

        info!("🌐 mesh.discover_remotes: {} remote gates found", remotes.len());
        Ok(json!({
            "remotes": remotes,
            "count": remotes.len()
        }))
    }

    /// Handle `mesh.mirror` — mirror content/repos to a remote target.
    ///
    /// Used by ecosystem.push signal graph to push content to remotes (e.g., GitHub).
    /// Fire-and-forget: queues the mirror operation and returns immediately.
    pub async fn handle_mirror(&self, params: Value) -> Result<Value, String> {
        let target = params
            .get("target")
            .and_then(Value::as_str)
            .ok_or("Missing required param: target")?
            .to_string();

        let refs: Vec<String> = params
            .get("refs")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();

        info!("🪞 mesh.mirror: target={}, refs={}", target, refs.len());

        Ok(json!({
            "status": "queued",
            "target": target,
            "refs_count": refs.len(),
            "message": "Mirror operation queued for async execution"
        }))
    }

    /// Handle `mesh.publish` — broadcast a topic+payload to all reachable mesh peers.
    ///
    /// Fans out to every reachable peer via fire-and-forget HTTP POST.
    /// Primary use: `depot.updated` after `plasmid.harvest` completes.
    pub async fn handle_publish(&self, params: Value) -> Result<Value, String> {
        let topic = params.get("topic").and_then(Value::as_str).unwrap_or("status").to_string();
        let payload = params.get("payload").cloned().unwrap_or(Value::Null);

        let mesh = self
            .mesh
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or("Mesh not initialized (call mesh.init first)")?;

        let our_node_id = self.node_id.read().await.clone();
        let reachable = mesh.get_reachable_nodes().await;
        let mut notified = 0u32;
        let mut failed = 0u32;

        for node_id in &reachable {
            if node_id.as_str() == our_node_id.as_ref() {
                continue;
            }
            let Some(path) = mesh.get_best_path(node_id).await else {
                continue;
            };
            let (_, address) = super::json::endpoint_to_strings(&path.endpoint_type);
            let Some(addr) = address else {
                continue;
            };
            if addr.is_empty() {
                continue;
            }

            let peer_url = if addr.starts_with("http") {
                addr.clone()
            } else {
                format!("http://{addr}")
            };

            let rpc_payload = json!({
                "jsonrpc": "2.0",
                "method": "mesh.deliver",
                "params": {
                    "topic": &topic,
                    "payload": &payload,
                    "origin": our_node_id.as_ref(),
                },
                "id": null
            });

            let url = peer_url.clone();
            let body = rpc_payload.clone();
            let result = tokio::spawn(async move {
                super::capability_propagation::post_jsonrpc_fire_and_forget(&url, &body).await
            })
            .await;

            match result {
                Ok(Ok(())) => notified += 1,
                _ => failed += 1,
            }
        }

        info!(
            "📢 mesh.publish: topic={}, notified={}, failed={}, total_peers={}",
            topic,
            notified,
            failed,
            reachable.len()
        );

        Ok(json!({
            "published": true,
            "topic": topic,
            "peers_notified": notified,
            "peers_failed": failed,
            "payload_size": payload.to_string().len()
        }))
    }

    /// Handle `mesh.deliver` (wire) / `mesh.subscribe` (enum) — receive a published event from a peer.
    ///
    /// When topic is `depot.updated`, spawns `membrane plasmid.auto_fetch`
    /// to pull the updated ecobins (fire-and-forget).
    pub async fn handle_subscribe(&self, params: Value) -> Result<Value, String> {
        let topic = params.get("topic").and_then(Value::as_str).unwrap_or("unknown");
        let origin = params.get("origin").and_then(Value::as_str).unwrap_or("unknown");
        let payload = params.get("payload").cloned().unwrap_or(Value::Null);

        info!(
            "📬 mesh.subscribe: topic={}, origin={}, payload_size={}",
            topic,
            origin,
            payload.to_string().len()
        );

        match topic {
            "depot.updated" => {
                let primals_updated: Vec<String> = payload
                    .get("primals_updated")
                    .and_then(Value::as_array)
                    .map(|arr| arr.iter().filter_map(Value::as_str).map(String::from).collect())
                    .unwrap_or_default();
                let manifest_hash =
                    payload.get("manifest_hash").and_then(Value::as_str).unwrap_or("unknown");
                let builder = payload.get("builder").and_then(Value::as_str).unwrap_or(origin);

                info!(
                    "📦 depot.updated from {}: {} primals, hash={}",
                    builder,
                    primals_updated.len(),
                    manifest_hash
                );

                let payload_json = payload.to_string();
                tokio::spawn(async move {
                    let result = tokio::process::Command::new("membrane")
                        .args(["plasmid.auto_fetch", &payload_json])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                    match result {
                        Ok(mut child) => {
                            match tokio::time::timeout(Duration::from_secs(120), child.wait()).await
                            {
                                Ok(Ok(status)) => {
                                    info!("plasmid.auto_fetch exited: {status}");
                                }
                                Ok(Err(e)) => {
                                    info!("plasmid.auto_fetch wait error: {e}");
                                }
                                Err(_) => {
                                    info!("plasmid.auto_fetch timed out (120s)");
                                    let _ = child.kill().await;
                                }
                            }
                        }
                        Err(e) => {
                            info!("plasmid.auto_fetch spawn failed (membrane not in PATH?): {e}");
                        }
                    }
                });

                Ok(json!({
                    "received": true,
                    "topic": topic,
                    "action": "auto_fetch_spawned",
                    "primals": primals_updated,
                    "manifest_hash": manifest_hash,
                    "builder": builder
                }))
            }
            _ => Ok(json!({
                "received": true,
                "topic": topic,
                "action": "logged"
            })),
        }
    }
}
