// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Cross-gate capability dispatch via mesh peers.
//!
//! Handles `capability.call` routing beyond the local registry:
//! 1. Direct TCP to a peer's Songbird JSON-RPC endpoint
//! 2. TURN relay fallback for NAT'd peers (CGNAT, double-NAT residential)
//!
//! This module owns the network I/O for remote dispatch — the registry
//! handler delegates here when local resolution fails and `routing != "local"`.

use super::{CapabilityCallParams, CapabilityCallResult, IpcServiceHandler};
use serde_json::Value;
use songbird_types::defaults::timeouts::DEFAULT_SOCKET_IO_TIMEOUT;
use tracing::{debug, info, warn};

impl IpcServiceHandler {
    /// Dispatch a capability call to a remote gate via mesh.
    ///
    /// Resolution strategy:
    /// 1. Check cached `peer_capabilities` for a known holder (path-optimal selection)
    /// 2. If known, target that peer directly via best path
    /// 3. If unknown, fall back to probing reachable peers
    /// 4. TURN relay as final fallback for NAT'd peers
    pub(super) async fn forward_to_remote_gate(
        &self,
        call: &CapabilityCallParams,
    ) -> Result<Value, String> {
        use songbird_types::defaults::ports::DEFAULT_HTTP_PORT;

        let mesh_guard = self.mesh_handler.mesh().await;
        let mesh =
            mesh_guard.as_ref().ok_or("Mesh not initialized — cannot discover remote gates")?;

        // Fast path: resolve from cached capability announcements (path-optimal)
        if let Some((holder_id, _)) =
            self.mesh_handler.find_peer_with_capability(&call.capability).await
            && let Some(path) = mesh.get_best_path(&holder_id).await
        {
            let peer_sock = path.endpoint_type.socket_addr().unwrap_or_else(|| {
                let ip = path
                    .endpoint_type
                    .address()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
                std::net::SocketAddr::new(ip, DEFAULT_HTTP_PORT)
            });

            let tcp_endpoint = songbird_types::constants::jsonrpc_endpoint_url(&peer_sock);

            match self.forward_to_remote_tcp(&tcp_endpoint, call).await {
                Ok(result) => {
                    let response = CapabilityCallResult {
                        provider: format!("remote:{holder_id}"),
                        gate: holder_id,
                        result,
                    };
                    return serde_json::to_value(response)
                        .map_err(|e| format!("Serialization error: {e}"));
                }
                Err(e) => {
                    debug!(
                        peer = %holder_id,
                        error = %e,
                        "Direct dispatch to cached capability holder failed, falling through"
                    );
                }
            }
        }

        // Slow path: discover by probing reachable peers
        self.discover_and_dispatch(mesh, call).await
    }

    /// Slow-path discovery: iterate reachable peers, probe capabilities, dispatch.
    async fn discover_and_dispatch(
        &self,
        mesh: &std::sync::Arc<songbird_onion_relay::mesh::BeaconMesh>,
        call: &CapabilityCallParams,
    ) -> Result<Value, String> {
        use songbird_types::defaults::ports::DEFAULT_HTTP_PORT;

        let reachable = mesh.get_reachable_nodes().await;
        if reachable.is_empty() {
            return Err(format!(
                "No local provider for '{}' and no reachable mesh peers for remote dispatch",
                call.capability
            ));
        }

        let mut last_tcp_error = String::new();
        let mut peer_addrs: Vec<(String, std::net::SocketAddr)> = Vec::new();

        for node_id in &reachable {
            let Some(path) = mesh.get_best_path(node_id).await else {
                continue;
            };
            let peer_sock = path.endpoint_type.socket_addr().unwrap_or_else(|| {
                let ip = path
                    .endpoint_type
                    .address()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
                std::net::SocketAddr::new(ip, DEFAULT_HTTP_PORT)
            });
            peer_addrs.push((node_id.clone(), peer_sock));

            let tcp_endpoint = songbird_types::constants::jsonrpc_endpoint_url(&peer_sock);

            if self.peer_has_capability(&tcp_endpoint, &call.capability).await == Ok(false) {
                debug!(peer = %node_id, "Peer lacks capability '{}', skipping", call.capability);
                continue;
            }

            match self.forward_to_remote_tcp(&tcp_endpoint, call).await {
                Ok(result) => {
                    let response = CapabilityCallResult {
                        provider: format!("remote:{node_id}"),
                        gate: node_id.clone(),
                        result,
                    };
                    return serde_json::to_value(response)
                        .map_err(|e| format!("Serialization error: {e}"));
                }
                Err(e) => {
                    debug!(
                        peer = %node_id,
                        error = %e,
                        "Direct TCP dispatch failed, trying next peer"
                    );
                    last_tcp_error = e;
                }
            }
        }

        // TURN relay fallback for NAT'd peers
        for (node_id, peer_addr) in &peer_addrs {
            match self.forward_to_remote_via_turn(*peer_addr, call).await {
                Ok(result) => {
                    let response = CapabilityCallResult {
                        provider: format!("remote:{node_id}"),
                        gate: node_id.clone(),
                        result,
                    };
                    return serde_json::to_value(response)
                        .map_err(|e| format!("Serialization error: {e}"));
                }
                Err(e) => {
                    debug!(
                        peer = %node_id,
                        error = %e,
                        "TURN relay dispatch also failed"
                    );
                }
            }
        }

        Err(format!(
            "No local or remote provider found for capability '{}' \
             (tried {} mesh peers via TCP and TURN relay; last error: {last_tcp_error})",
            call.capability,
            reachable.len()
        ))
    }

    /// Forward a capability call over a TURN relay (RFC 5766).
    ///
    /// Used when direct TCP fails (CGNAT, double-NAT). Allocates a TURN
    /// session to the peer's address, sends the JSON-RPC request as bytes
    /// through the relay, and reads the response.
    ///
    /// Requires `SONGBIRD_TURN_SERVER`, `SONGBIRD_TURN_USERNAME`, and
    /// `SONGBIRD_TURN_KEY` environment variables.
    pub(super) async fn forward_to_remote_via_turn(
        &self,
        peer_addr: std::net::SocketAddr,
        call: &CapabilityCallParams,
    ) -> Result<Value, String> {
        use songbird_turn_client::{TurnSession, TurnSessionConfig};

        let config = TurnSessionConfig::from_env(peer_addr)
            .map_err(|e| format!("TURN not configured: {e}"))?;

        let session = TurnSession::connect(config)
            .await
            .map_err(|e| format!("TURN allocation failed: {e}"))?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.call",
            "params": {
                "capability": call.capability,
                "operation": call.operation,
                "params": call.params,
                "routing": "local"
            },
            "id": 1
        });

        let mut request_bytes = serde_json::to_vec(&request)
            .map_err(|e| format!("Failed to serialize TURN request: {e}"))?;
        request_bytes.push(b'\n');

        session.send(&request_bytes).await.map_err(|e| format!("TURN send failed: {e}"))?;

        let mut buf = vec![0u8; 65536];
        let n = session.recv(&mut buf).await.map_err(|e| format!("TURN recv failed: {e}"))?;

        let response_str = std::str::from_utf8(&buf[..n])
            .map_err(|e| format!("Invalid UTF-8 from TURN relay: {e}"))?;

        let response: Value = serde_json::from_str(response_str.trim())
            .map_err(|e| format!("Invalid JSON from TURN relay: {e}"))?;

        let _ = session.close().await;

        if let Some(error) = response.get("error") {
            return Err(format!(
                "Remote gate error (via TURN): {}",
                error.get("message").and_then(Value::as_str).unwrap_or("unknown")
            ));
        }

        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }

    /// Probe whether a remote peer advertises a given capability.
    ///
    /// Sends `capabilities.list` via HTTP POST and checks `provided_capabilities`.
    /// Returns `Ok(true)` if the peer has it, `Ok(false)` if it doesn't, or `Err`
    /// if the probe itself failed (treat as "unknown — try anyway").
    pub(super) async fn peer_has_capability(
        &self,
        http_endpoint: &str,
        capability: &str,
    ) -> Result<bool, String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capabilities.list",
            "params": {},
            "id": 1
        });

        let resp = self
            .http_post_jsonrpc(http_endpoint, &request)
            .await
            .map_err(|e| format!("probe: {e}"))?;

        let caps = resp
            .get("result")
            .and_then(|r| r.get("provided_capabilities"))
            .and_then(Value::as_array);

        caps.map_or_else(
            || {
                let flat = resp
                    .get("result")
                    .and_then(|r| r.get("capabilities"))
                    .and_then(Value::as_array);
                flat.map_or_else(
                    || Err(String::from("no provided_capabilities in response")),
                    |arr| Ok(arr.iter().any(|c| c.as_str() == Some(capability))),
                )
            },
            |arr| {
                Ok(arr.iter().any(|c| {
                    c.as_str() == Some(capability)
                        || c.get("type").and_then(Value::as_str) == Some(capability)
                }))
            },
        )
    }

    /// Send a `capability.call` to a remote Songbird instance via HTTP POST.
    ///
    /// The remote peer runs an axum HTTP server on its mesh port. We POST
    /// JSON-RPC to `/jsonrpc` with `Content-Type: application/json`.
    pub(super) async fn forward_to_remote_tcp(
        &self,
        endpoint: &str,
        call: &CapabilityCallParams,
    ) -> Result<Value, String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.call",
            "params": {
                "capability": call.capability,
                "operation": call.operation,
                "params": call.params,
                "routing": "local"
            },
            "id": 1
        });

        let response = self
            .http_post_jsonrpc(endpoint, &request)
            .await
            .map_err(|e| format!("Remote gate HTTP error: {e}"))?;

        if let Some(error) = response.get("error") {
            return Err(format!(
                "Remote gate error: {}",
                error.get("message").and_then(Value::as_str).unwrap_or("unknown")
            ));
        }

        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }

    /// HTTP POST a JSON-RPC request to a remote peer's `/jsonrpc` endpoint.
    ///
    /// Uses hyper for HTTP/1.1 transport — the remote peer is an axum server,
    /// not a raw NDJSON stream.
    async fn http_post_jsonrpc(&self, url: &str, request: &Value) -> Result<Value, String> {
        use http_body_util::{BodyExt, Full};
        use hyper::Request;
        use hyper::body::Bytes;
        use hyper_util::client::legacy::Client;
        use hyper_util::rt::TokioExecutor;

        let body_bytes =
            serde_json::to_vec(request).map_err(|e| format!("Serialization error: {e}"))?;

        let uri: hyper::Uri =
            url.parse().map_err(|e| format!("Invalid endpoint URI '{url}': {e}"))?;

        let http_request = Request::builder()
            .method(hyper::Method::POST)
            .uri(&uri)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(body_bytes)))
            .map_err(|e| format!("Failed to build HTTP request: {e}"))?;

        let client = Client::builder(TokioExecutor::new()).build_http::<Full<Bytes>>();

        let response =
            tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, client.request(http_request))
                .await
                .map_err(|_| format!("Timeout posting to remote gate at {url}"))?
                .map_err(|e| format!("HTTP request to {url} failed: {e}"))?;

        let status = response.status();
        let body = response
            .into_body()
            .collect()
            .await
            .map_err(|e| format!("Failed to read response body from {url}: {e}"))?
            .to_bytes();

        if !status.is_success() {
            let snippet = String::from_utf8_lossy(&body[..body.len().min(200)]);
            return Err(format!("Remote gate returned HTTP {status} from {url}: {snippet}"));
        }

        serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid JSON from remote gate at {url}: {e}"))
    }

    /// Handle `relay.forward` — the transport graduation entry point.
    ///
    /// cellMembrane's `call_via_relay` sends `{peer_id, capability, payload}` to
    /// the local songBird socket. This handler resolves the peer via the mesh,
    /// then forwards the raw JSON-RPC payload to the target gate's songBird
    /// instance using the existing remote dispatch infrastructure (TCP direct,
    /// TURN relay fallback).
    ///
    /// This closes the integration gap between `TransportEndpoint::MeshRelay`
    /// resolution in cellMembrane and songBird's cross-gate transport.
    pub async fn handle_relay_forward(&self, params: Value) -> Result<Value, String> {
        let peer_id = params
            .get("peer_id")
            .and_then(Value::as_str)
            .ok_or("relay.forward: missing 'peer_id' parameter")?;

        let capability = params
            .get("capability")
            .and_then(Value::as_str)
            .ok_or("relay.forward: missing 'capability' parameter")?;

        let payload_str = params
            .get("payload")
            .and_then(Value::as_str)
            .ok_or("relay.forward: missing 'payload' parameter (raw JSON-RPC string)")?;

        info!(
            peer = %peer_id,
            capability = %capability,
            payload_len = payload_str.len(),
            "relay.forward: routing envelope to remote gate"
        );

        let call = CapabilityCallParams {
            capability: capability.to_string(),
            operation: String::from("forward"),
            params: serde_json::from_str(payload_str)
                .unwrap_or_else(|_| serde_json::json!({ "_raw_payload": payload_str })),
            routing: "any".to_string(),
        };

        match self.forward_to_remote_gate(&call).await {
            Ok(result) => {
                info!(peer = %peer_id, "relay.forward: delivery successful");
                Ok(result)
            }
            Err(e) => {
                warn!(peer = %peer_id, error = %e, "relay.forward: delivery failed");
                Err(format!("relay.forward to {peer_id}: {e}"))
            }
        }
    }
}
