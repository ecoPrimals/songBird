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
use tracing::debug;

impl IpcServiceHandler {
    /// Dispatch a capability call to a remote gate via mesh.
    ///
    /// Called when local resolution fails and `routing` permits remote dispatch.
    /// Tries direct TCP first, then TURN relay for each reachable peer.
    pub(super) async fn forward_to_remote_gate(
        &self,
        call: &CapabilityCallParams,
    ) -> Result<Value, String> {
        use songbird_types::defaults::ports::DEFAULT_HTTP_PORT;

        let mesh_guard = self.mesh_handler.mesh().await;
        let mesh =
            mesh_guard.as_ref().ok_or("Mesh not initialized — cannot discover remote gates")?;

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
            // Use the peer's advertised socket address (includes port); fall back to
            // DEFAULT_HTTP_PORT only if the endpoint type lacks port info.
            let peer_sock = path.endpoint_type.socket_addr().unwrap_or_else(|| {
                let ip = path
                    .endpoint_type
                    .address()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
                std::net::SocketAddr::new(ip, DEFAULT_HTTP_PORT)
            });
            peer_addrs.push((node_id.clone(), peer_sock));

            let tcp_endpoint = format!("http://{}:{}/jsonrpc", peer_sock.ip(), peer_sock.port());

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
        if !peer_addrs.is_empty() {
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
    /// Sends `capabilities.list` and checks `provided_capabilities`. Returns
    /// `Ok(true)` if the peer has it, `Ok(false)` if it doesn't, or `Err` if
    /// the probe itself failed (treat as "unknown — try anyway").
    pub(super) async fn peer_has_capability(
        &self,
        tcp_endpoint: &str,
        capability: &str,
    ) -> Result<bool, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpStream;

        let addr = tcp_endpoint.trim_start_matches("http://").trim_end_matches("/jsonrpc");
        let probe_timeout = std::time::Duration::from_secs(3);

        let stream = tokio::time::timeout(probe_timeout, TcpStream::connect(addr))
            .await
            .map_err(|_| "probe timeout")?
            .map_err(|e| format!("probe connect: {e}"))?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capabilities.list",
            "params": {},
            "id": 1
        });

        let mut bytes = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
        bytes.push(b'\n');

        let (reader, mut writer) = stream.into_split();
        tokio::time::timeout(probe_timeout, writer.write_all(&bytes))
            .await
            .map_err(|_| "probe write timeout")?
            .map_err(|e| format!("probe write: {e}"))?;

        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        tokio::time::timeout(probe_timeout, buf_reader.read_line(&mut line))
            .await
            .map_err(|_| "probe read timeout")?
            .map_err(|e| format!("probe read: {e}"))?;

        let resp: Value =
            serde_json::from_str(line.trim()).map_err(|e| format!("probe parse: {e}"))?;

        let caps = resp
            .get("result")
            .and_then(|r| r.get("provided_capabilities"))
            .and_then(Value::as_array);

        if let Some(arr) = caps {
            Ok(arr.iter().any(|c| {
                c.as_str() == Some(capability)
                    || c.get("type").and_then(Value::as_str) == Some(capability)
            }))
        } else {
            let flat =
                resp.get("result").and_then(|r| r.get("capabilities")).and_then(Value::as_array);
            if let Some(arr) = flat {
                Ok(arr.iter().any(|c| c.as_str() == Some(capability)))
            } else {
                Err(String::from("no provided_capabilities in response"))
            }
        }
    }

    /// Send a `capability.call` to a remote Songbird instance via TCP JSON-RPC.
    pub(super) async fn forward_to_remote_tcp(
        &self,
        endpoint: &str,
        call: &CapabilityCallParams,
    ) -> Result<Value, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::TcpStream;

        let addr = endpoint.trim_start_matches("http://").trim_end_matches("/jsonrpc");

        let stream = tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, TcpStream::connect(addr))
            .await
            .map_err(|_| format!("Timeout connecting to remote gate at {addr}"))?
            .map_err(|e| format!("Cannot connect to remote gate at {addr}: {e}"))?;

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
            .map_err(|e| format!("Failed to serialize remote request: {e}"))?;
        request_bytes.push(b'\n');

        let (reader, mut writer) = stream.into_split();

        tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, writer.write_all(&request_bytes))
            .await
            .map_err(|_| format!("Timeout writing to remote gate at {addr}"))?
            .map_err(|e| format!("Write error to remote gate: {e}"))?;

        let mut buf_reader = BufReader::new(reader);
        let mut response_line = String::new();
        tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, buf_reader.read_line(&mut response_line))
            .await
            .map_err(|_| format!("Timeout reading from remote gate at {addr}"))?
            .map_err(|e| format!("Read error from remote gate: {e}"))?;

        let response: Value = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Invalid JSON from remote gate: {e}"))?;

        if let Some(error) = response.get("error") {
            return Err(format!(
                "Remote gate error: {}",
                error.get("message").and_then(Value::as_str).unwrap_or("unknown")
            ));
        }

        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }
}
