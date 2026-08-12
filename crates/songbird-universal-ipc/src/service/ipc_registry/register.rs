// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::swarmvine_gossip::inject_to_swarmvine;
use super::super::{IpcServiceHandler, RegisterParams, RegisterResult, TransportEndpoint};
use crate::endpoint::NativeEndpoint;
use serde_json::Value;
use songbird_types::defaults::timeouts::DEFAULT_SOCKET_IO_TIMEOUT;
use std::sync::Arc;
use tracing::debug;

/// Build a deterministic canonical JSON payload for signing.
///
/// Field ordering is alphabetical by short key to ensure identical bytes
/// regardless of capability insertion order.
pub(super) fn build_canonical_payload(
    primal_id: &str,
    capabilities: &[String],
    endpoint: &str,
    registered_at: &str,
) -> String {
    let mut sorted_caps = capabilities.to_vec();
    sorted_caps.sort();
    serde_json::json!({
        "c": sorted_caps,
        "e": endpoint,
        "p": primal_id,
        "t": registered_at,
    })
    .to_string()
}

impl IpcServiceHandler {
    /// Handle `ipc.register` method
    pub(in crate::service) async fn handle_register(&self, params: Value) -> Result<Value, String> {
        let params: RegisterParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        tracing::info!("Registering primal: {} at {}", params.primal_id, params.endpoint);

        let registered_at = chrono::Utc::now().to_rfc3339();

        // Build canonical payload before endpoint parsing moves the string
        let canonical = build_canonical_payload(
            &params.primal_id,
            &params.capabilities,
            &params.endpoint,
            &registered_at,
        );

        // Parse native endpoint
        let native_endpoint = if params.endpoint.starts_with('/') {
            NativeEndpoint::UnixSocket(params.endpoint.into())
        } else if let Some(stripped) = params.endpoint.strip_prefix("unix://") {
            NativeEndpoint::UnixSocket(stripped.into())
        } else if let Some(stripped) = params.endpoint.strip_prefix("tcp://") {
            let port = Self::parse_tcp_port(stripped)?;
            NativeEndpoint::TcpLocal(port)
        } else if let Some(port) = Self::parse_local_tcp_endpoint(&params.endpoint) {
            NativeEndpoint::TcpLocal(port)
        } else {
            return Err(format!(
                "Invalid endpoint format: '{}'. Expected unix socket path, unix://<path>, tcp://<host>:<port>, or <localhost>:<port>",
                params.endpoint
            ));
        };

        // Verify registrant identity: probe the endpoint with identity.get and confirm
        // the primal_id matches. Gracefully degrades if endpoint is unreachable (trust-on-first-use
        // for primals still starting). Hard-rejects identity mismatch (spoofed names).
        self.verify_registrant_identity(&native_endpoint, &params.primal_id).await?;

        // Sign via BearDog if crypto provider is available
        let (signature, signed_payload) = self.sign_payload(&canonical).await;

        // Build structured transport before register() consumes native_endpoint
        let transport = match &native_endpoint {
            NativeEndpoint::UnixSocket(path) => Some(TransportEndpoint::Uds {
                path: path.to_string_lossy().to_string(),
            }),
            NativeEndpoint::AbstractSocket(name) => Some(TransportEndpoint::Uds {
                path: format!("@{name}"),
            }),
            NativeEndpoint::TcpLocal(port) => Some(TransportEndpoint::Tcp {
                host: songbird_types::constants::LOCALHOST.to_string(),
                port: *port,
            }),
            NativeEndpoint::NamedPipe(name) => Some(TransportEndpoint::NamedPipe {
                name: name.clone(),
            }),
            _ => None,
        };

        // Register in registry (`register` takes `&self` and uses its own inner lock)
        let native_socket = native_endpoint.socket_path();
        let has_capabilities = !params.capabilities.is_empty();
        let virtual_endpoint = self
            .registry
            .read()
            .await
            .register(
                &params.primal_id,
                native_endpoint,
                params.capabilities,
                signature.clone(),
                signed_payload.clone(),
            )
            .await
            .map_err(|e| format!("Registration failed: {e}"))?;

        // Phase 1 (shadow mode): spawn virtual relay listener alongside native endpoint
        if let Some(ref socket_path) = native_socket
            && let Err(e) = self.virtual_relay.start_relay(&params.primal_id, socket_path).await
        {
            tracing::warn!(
                primal = %params.primal_id,
                error = %e,
                "Virtual relay start failed (non-blocking)"
            );
        }

        // Propagate capabilities to mesh peers (push model for cross-gate discovery)
        if has_capabilities {
            let all_capabilities = self.collect_all_local_capabilities().await;
            let mesh = Arc::clone(&self.mesh_handler);
            let primal_id_for_gossip = params.primal_id.clone();
            let caps_for_gossip = all_capabilities.clone();
            tokio::spawn(async move {
                mesh.announce_capabilities_to_peers(all_capabilities).await;
            });

            // Phase 3 seam: inject into local swarmVine for epidemic gossip propagation.
            // Resolve gate identity from environment (same as swarmVine itself does).
            let gate_id = songbird_process_env::var("GATE_ID")
                .or_else(|_| songbird_process_env::var("HOSTNAME"))
                .unwrap_or_default();
            if !gate_id.is_empty() {
                tokio::spawn(async move {
                    inject_to_swarmvine(&gate_id, &primal_id_for_gossip, &caps_for_gossip).await;
                });
            }
        }

        let result = RegisterResult {
            virtual_endpoint: virtual_endpoint.path,
            registered_at,
            transport,
            signature,
            signed_payload,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Sign a canonical payload via the crypto provider (`BearDog` Ed25519 delegation).
    ///
    /// Returns `(Some(signature), Some(payload))` on success, `(None, None)` if
    /// no crypto provider is configured or signing fails (standalone mode).
    async fn sign_payload(&self, canonical_payload: &str) -> (Option<String>, Option<String>) {
        use base64::Engine as _;

        let Some(ref provider) = self.crypto_provider else {
            return (None, None);
        };

        let data_b64 =
            base64::engine::general_purpose::STANDARD.encode(canonical_payload.as_bytes());

        match provider.call("crypto.sign.ed25519", serde_json::json!({ "data": data_b64 })).await {
            Ok(result) => result.get("signature").and_then(|v| v.as_str()).map_or_else(
                || {
                    tracing::warn!("crypto.sign.ed25519 returned no signature field");
                    (None, None)
                },
                |s| {
                    debug!("Signed registration payload ({} bytes)", s.len());
                    (Some(s.to_string()), Some(canonical_payload.to_string()))
                },
            ),
            Err(e) => {
                tracing::warn!("Failed to sign registration (standalone fallback): {e}");
                (None, None)
            }
        }
    }

    /// Collect all capabilities from all locally registered primals.
    ///
    /// Used to build the aggregate capability set announced to mesh peers.
    async fn collect_all_local_capabilities(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        let metadata = registry.get_all_metadata().await;
        let mut all_caps: Vec<String> = metadata.into_iter().flat_map(|m| m.capabilities).collect();
        all_caps.sort();
        all_caps.dedup();
        all_caps
    }

    /// Verify a registering primal's identity by probing its endpoint with `identity.get`.
    ///
    /// Returns `Ok(())` if the probe confirms the primal's identity matches `expected_name`,
    /// or if the endpoint is unreachable (graceful degradation for primals still starting).
    /// Returns `Err(reason)` only if the primal responds but claims a DIFFERENT identity.
    async fn verify_registrant_identity(
        &self,
        endpoint: &NativeEndpoint,
        expected_name: &str,
    ) -> Result<(), String> {
        let Some(response) = self.probe_identity(endpoint).await else {
            tracing::warn!(
                "ipc.register identity probe: no response from {expected_name} \
                 (allowing trust-on-first-use)"
            );
            return Ok(());
        };

        if response.get("error").is_some() {
            debug!("ipc.register identity probe: identity.get returned error (non-fatal)");
            return Ok(());
        }

        let Some(claimed_name) =
            response.get("result").and_then(|r| r.get("primal")).and_then(|p| p.as_str())
        else {
            return Ok(());
        };

        if !claimed_name.eq_ignore_ascii_case(expected_name) {
            return Err(format!(
                "Identity mismatch: registering as '{expected_name}' but endpoint \
                 claims to be '{claimed_name}'"
            ));
        }
        debug!("ipc.register identity verified: {expected_name}");
        Ok(())
    }

    /// Send `identity.get` to an endpoint and return the parsed JSON response.
    ///
    /// Returns `None` on connection failure, timeout, or unparseable response
    /// (all treated as trust-on-first-use by the caller).
    async fn probe_identity(&self, endpoint: &NativeEndpoint) -> Option<Value> {
        let req_bytes =
            b"{\"jsonrpc\":\"2.0\",\"method\":\"identity.get\",\"params\":{},\"id\":1}\n";

        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                #[cfg(unix)]
                {
                    let stream = tokio::time::timeout(
                        DEFAULT_SOCKET_IO_TIMEOUT,
                        tokio::net::UnixStream::connect(path),
                    )
                    .await
                    .ok()?
                    .ok()?;

                    Self::send_and_read(stream, req_bytes).await
                }
                #[cfg(not(unix))]
                {
                    let _ = path;
                    None
                }
            }
            NativeEndpoint::TcpLocal(port) => {
                let addr = format!("{}:{port}", songbird_types::constants::LOCALHOST);
                let stream = tokio::time::timeout(
                    DEFAULT_SOCKET_IO_TIMEOUT,
                    tokio::net::TcpStream::connect(&addr),
                )
                .await
                .ok()?
                .ok()?;

                Self::send_and_read(stream, req_bytes).await
            }
            _ => None,
        }
    }

    /// Write a request and read a single NDJSON response line from a stream.
    async fn send_and_read<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(
        stream: S,
        request: &[u8],
    ) -> Option<Value> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);
        writer.write_all(request).await.ok()?;
        writer.flush().await.ok()?;

        let mut line = String::new();
        let n = tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, reader.read_line(&mut line))
            .await
            .ok()?
            .ok()?;
        if n == 0 {
            return None;
        }
        serde_json::from_str(line.trim()).ok()
    }
}
