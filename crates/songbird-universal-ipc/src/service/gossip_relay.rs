// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

//! `gossip.relay` + `gossip.inject` — `MeshRelay` transport for swarmVine gossip.
//!
//! Enables cross-gate gossip propagation through songBird's `:7700` federation
//! mesh when swarmVine's direct TCP 7800 path is unreachable between gates.
//!
//! Flow:
//! 1. Local swarmVine → songBird `gossip.relay {target_gate, topic, payload}`
//! 2. songBird resolves best path to target gate via `MeshHandler::mesh()`
//! 3. songBird POSTs `gossip.inject` to remote songBird on `:7700`
//! 4. Remote songBird injects into its local swarmVine via `gossip.forward` UDS

use super::IpcServiceHandler;
use serde_json::{Value, json};
use songbird_onion_relay::mesh::EndpointType;
use std::time::Duration;
use tracing::{debug, warn};

impl IpcServiceHandler {
    /// Handle `gossip.relay` — relay a gossip payload to a target gate's swarmVine.
    ///
    /// Params:
    /// - `target_gate` (string, optional): node ID of the destination gate.
    ///   If absent, empty, or `"local"`, injects into local swarmVine only.
    /// - `topic` (string): gossip topic (e.g. `"tower"`, `"capability"`)
    /// - `key` (string, optional): dedup key for the gossip entry
    /// - `payload` (object): the gossip payload to propagate
    ///
    /// Returns `{relayed_to, status}` on success.
    pub(super) async fn handle_gossip_relay(&self, params: Value) -> Result<Value, String> {
        let target_gate = params
            .get("target_gate")
            .and_then(Value::as_str)
            .unwrap_or("local");

        let topic = params
            .get("topic")
            .and_then(Value::as_str)
            .ok_or("Missing required field: topic")?;

        let payload = params
            .get("payload")
            .cloned()
            .unwrap_or(Value::Null);

        let key = params
            .get("key")
            .and_then(Value::as_str)
            .unwrap_or("");

        if target_gate.is_empty() || target_gate == "local" {
            self.inject_gossip_locally(topic, key, &payload).await?;
            return Ok(json!({
                "relayed_to": "local",
                "status": "injected"
            }));
        }

        // Resolve path to target gate via mesh
        let mesh_guard = self.mesh_handler.mesh().await;
        let mesh = mesh_guard
            .as_ref()
            .ok_or("Mesh not initialized — cannot relay gossip")?;

        let path = mesh
            .get_best_path(target_gate)
            .await
            .ok_or_else(|| format!("No path to gate '{target_gate}'"))?;

        let addr = match path.endpoint_type {
            EndpointType::Direct { addr }
            | EndpointType::Local { addr }
            | EndpointType::Overlay { addr, .. } => addr,
            EndpointType::FamilyRelay { .. } | EndpointType::TorOnion { .. } => {
                return Err(format!(
                    "No direct/LAN/overlay path to '{target_gate}' — relay/onion not supported for gossip"
                ));
            }
        };

        let url = songbird_types::constants::jsonrpc_endpoint_url(&addr);

        let our_node_id = self.mesh_handler.node_id_async().await;
        let request = json!({
            "jsonrpc": "2.0",
            "method": "gossip.inject",
            "params": {
                "topic": topic,
                "key": key,
                "payload": payload,
                "origin_gate": our_node_id,
            },
            "id": 1
        });

        debug!(
            target: "songbird::gossip_relay",
            target_gate,
            topic,
            url = %url,
            "relaying gossip via mesh"
        );

        post_gossip_relay(&url, &request).await?;

        Ok(json!({
            "relayed_to": target_gate,
            "via": url,
            "status": "relayed"
        }))
    }

    /// Handle `gossip.inject` — inject a gossip payload into local swarmVine.
    ///
    /// Called by remote songBird peers (via `:7700` federation) or directly by
    /// local primals that want to inject gossip without specifying a target gate.
    pub(super) async fn handle_gossip_inject(&self, params: Value) -> Result<Value, String> {
        let topic = params
            .get("topic")
            .and_then(Value::as_str)
            .ok_or("Missing required field: topic")?;

        let key = params
            .get("key")
            .and_then(Value::as_str)
            .unwrap_or("");

        let payload = params
            .get("payload")
            .cloned()
            .unwrap_or(Value::Null);

        let origin = params
            .get("origin_gate")
            .and_then(Value::as_str)
            .unwrap_or("unknown");

        debug!(
            target: "songbird::gossip_relay",
            topic,
            origin,
            "injecting gossip into local swarmVine"
        );

        self.inject_gossip_locally(topic, key, &payload).await?;

        Ok(json!({
            "status": "injected",
            "topic": topic,
            "origin_gate": origin,
        }))
    }

    /// Handle `gossip.spread` — broadcast gossip to ALL reachable mesh peers.
    ///
    /// Epidemic fan-out: injects locally AND relays to every reachable federation
    /// peer. Prevents relay loops via `origin_gate` — peers that originated the
    /// gossip or already received it (tracked by `seen_gates`) are skipped.
    ///
    /// Params:
    /// - `topic` (string): gossip topic
    /// - `key` (string, optional): dedup key
    /// - `payload` (object): gossip payload
    /// - `origin_gate` (string, optional): originator gate (loop prevention)
    /// - `seen_gates` (array of strings, optional): gates that already have this entry
    pub(super) async fn handle_gossip_spread(&self, params: Value) -> Result<Value, String> {
        let topic = params
            .get("topic")
            .and_then(Value::as_str)
            .ok_or("Missing required field: topic")?;

        let key = params
            .get("key")
            .and_then(Value::as_str)
            .unwrap_or("");

        let payload = params
            .get("payload")
            .cloned()
            .unwrap_or(Value::Null);

        let origin = params
            .get("origin_gate")
            .and_then(Value::as_str)
            .unwrap_or("");

        let seen_gates: Vec<String> = params
            .get("seen_gates")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        self.inject_gossip_locally(topic, key, &payload).await?;

        let mesh_guard = self.mesh_handler.mesh().await;
        let Some(mesh) = mesh_guard.as_ref() else {
            return Ok(json!({
                "status": "local_only",
                "reason": "mesh not initialized",
                "spread_to": 0
            }));
        };

        let our_node_id = self.mesh_handler.node_id_async().await;
        let reachable = mesh.get_reachable_nodes().await;
        drop(mesh_guard);

        let inject_request = json!({
            "topic": topic,
            "key": key,
            "payload": payload,
            "origin_gate": our_node_id,
        });

        let (spread_count, failures) =
            self.fan_out_gossip(&reachable, origin, &seen_gates, &our_node_id, &inject_request)
                .await;

        let mut result = json!({
            "status": "spread",
            "topic": topic,
            "spread_to": spread_count,
            "local_injected": true,
        });

        if !failures.is_empty() {
            result["unreachable_peers"] = json!(failures);
        }

        Ok(result)
    }

    /// Fan-out a gossip payload to reachable peers, skipping origin and seen gates.
    async fn fan_out_gossip(
        &self,
        reachable: &[String],
        origin: &str,
        seen_gates: &[String],
        our_node_id: &str,
        inject_params: &Value,
    ) -> (u32, Vec<String>) {
        use songbird_onion_relay::mesh::EndpointType;

        let mut spread_count: u32 = 0;
        let mut failures: Vec<String> = Vec::new();

        let mut seen: std::collections::HashSet<&str> =
            seen_gates.iter().map(String::as_str).collect();
        if !origin.is_empty() {
            seen.insert(origin);
        }
        seen.insert(our_node_id);

        for peer_id in reachable {
            if seen.contains(peer_id.as_str()) {
                continue;
            }

            let mesh_guard = self.mesh_handler.mesh().await;
            let Some(mesh) = mesh_guard.as_ref() else {
                break;
            };
            let path = mesh.get_best_path(peer_id).await;
            drop(mesh_guard);

            let Some(path) = path else {
                continue;
            };

            let (EndpointType::Direct { addr }
            | EndpointType::Local { addr }
            | EndpointType::Overlay { addr, .. }) = path.endpoint_type
            else {
                continue;
            };

            let url = songbird_types::constants::jsonrpc_endpoint_url(&addr);

            let request = json!({
                "jsonrpc": "2.0",
                "method": "gossip.inject",
                "params": inject_params,
                "id": 1
            });

            match post_gossip_relay(&url, &request).await {
                Ok(()) => spread_count += 1,
                Err(e) => {
                    debug!(
                        target: "songbird::gossip_relay",
                        peer = %peer_id,
                        error = %e,
                        "spread failed to peer"
                    );
                    failures.push(peer_id.clone());
                }
            }
        }

        (spread_count, failures)
    }

    /// Inject a gossip payload into the local swarmVine via UDS.
    ///
    /// Uses swarmVine's expected `0xEC 0x01` preamble followed by JSON-RPC.
    async fn inject_gossip_locally(
        &self,
        topic: &str,
        key: &str,
        payload: &Value,
    ) -> Result<(), String> {
        #[cfg(unix)]
        {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            use tokio::net::UnixStream;

            let socket_path = super::swarmvine_gossip::discover_swarmvine_socket();
            let Some(socket_path) = socket_path else {
                warn!(
                    target: "songbird::gossip_relay",
                    topic,
                    "swarmVine not reachable — gossip injection skipped"
                );
                return Ok(());
            };

            let rpc = json!({
                "jsonrpc": "2.0",
                "method": "gossip.inject",
                "params": {
                    "topic": topic,
                    "key": key,
                    "payload": payload,
                },
                "id": 1
            });

            let inject_timeout = Duration::from_secs(3);
            match tokio::time::timeout(inject_timeout, UnixStream::connect(&socket_path)).await {
                Ok(Ok(stream)) => {
                    let (reader, mut writer) = stream.into_split();
                    // swarmVine expects local injection preamble: 0xEC 0x01
                    if writer.write_all(&[0xEC, 0x01]).await.is_err() {
                        return Ok(());
                    }
                    let msg = format!("{rpc}\n");
                    if writer.write_all(msg.as_bytes()).await.is_ok() {
                        let mut response = String::new();
                        let mut buf_reader = BufReader::new(reader);
                        let _ = tokio::time::timeout(
                            Duration::from_secs(2),
                            buf_reader.read_line(&mut response),
                        )
                        .await;
                        debug!(
                            target: "songbird::gossip_relay",
                            topic,
                            "gossip injected into swarmVine"
                        );
                    }
                }
                Ok(Err(e)) => {
                    warn!(
                        target: "songbird::gossip_relay",
                        topic,
                        error = %e,
                        "failed to connect to swarmVine socket"
                    );
                }
                Err(_) => {
                    warn!(
                        target: "songbird::gossip_relay",
                        topic,
                        "timeout connecting to swarmVine socket"
                    );
                }
            }
        }

        #[cfg(not(unix))]
        {
            let _ = (topic, key, payload);
            debug!(
                target: "songbird::gossip_relay",
                "gossip injection not available on this platform (UDS-only)"
            );
        }

        Ok(())
    }
}

/// POST a JSON-RPC request to a remote songBird peer for gossip relay.
async fn post_gossip_relay(url: &str, body: &Value) -> Result<(), String> {
    use http_body_util::Full;
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
        .map_err(|_| format!("Timeout relaying gossip to {url}"))?
        .map_err(|e| format!("HTTP error relaying to {url}: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Remote peer {url} returned HTTP {status}"));
    }

    Ok(())
}
