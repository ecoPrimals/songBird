// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-based routing and dispatch for the IPC service.
//!
//! Extracted from `ipc_registry.rs` for SRP: contains `handle_capability_resolve`,
//! `handle_capability_call`, and `forward_to_local_provider`.

use super::{
    CapabilityCallParams, CapabilityCallResult, CapabilityResolveParams, CapabilityResolveResult,
    IpcServiceHandler, TransportEndpoint,
};
use serde_json::Value;
use songbird_types::defaults::timeouts::DEFAULT_SOCKET_IO_TIMEOUT;
use tracing::debug;

use super::ipc_registry::transport_endpoint_from_native;

/// Parse `"host:port"` or `"host"` into components for `TransportEndpoint::Tcp`.
fn parse_host_port(addr: &str) -> (String, u16) {
    if let Some((h, p)) = addr.rsplit_once(':') {
        (h.to_string(), p.parse().unwrap_or(80))
    } else {
        (addr.to_string(), 80)
    }
}

impl IpcServiceHandler {
    /// Handle `capability.resolve` — single-step routing by capability.
    ///
    /// Returns the best provider endpoint for the requested capability (most
    /// recently seen wins). Falls back to mesh peers when no local provider
    /// exists (topology-aware cross-gate routing).
    pub(super) async fn handle_capability_resolve(&self, params: Value) -> Result<Value, String> {
        let params: CapabilityResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!("Resolving best provider for capability: {}", params.capability);

        let registry = self.registry.read().await;
        if let Some((name, entry)) = registry.resolve_by_capability(&params.capability).await {
            let endpoint = transport_endpoint_from_native(&entry.native_endpoint);
            let result = CapabilityResolveResult {
                primal_id: name,
                socket: entry.native_endpoint.socket_path(),
                virtual_endpoint: entry.virtual_endpoint.path,
                native_endpoint: entry.native_endpoint.display(),
                endpoint,
                capabilities: entry.capabilities,
                signature: entry.signature,
                signed_payload: entry.signed_payload,
            };
            return serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"));
        }
        drop(registry);

        // Drawbridge weak bond: resolve capabilities served via proxy router
        if let Some(route) = self.capability_router.route(&params.capability) {
            debug!(
                capability = %params.capability,
                backend = %route.base_url,
                "capability.resolve: resolved via drawbridge proxy route"
            );
            let (host, port) = parse_host_port(&route.base_url);
            let result = CapabilityResolveResult {
                primal_id: String::from("drawbridge"),
                socket: None,
                virtual_endpoint: String::new(),
                native_endpoint: format!("http://{}", route.base_url),
                endpoint: TransportEndpoint::Tcp {
                    host,
                    port,
                },
                capabilities: vec![params.capability.clone()],
                signature: None,
                signed_payload: None,
            };
            return serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"));
        }

        if let Some((peer_id, peer_caps)) =
            self.mesh_handler.find_peer_with_capability(&params.capability).await
        {
            debug!(
                capability = %params.capability,
                peer = %peer_id,
                "capability.resolve: resolved via mesh peer (cross-gate routing)"
            );
            let native_endpoint = format!("mesh://{peer_id}");
            let primal_id = format!("remote:{peer_id}");
            let result = CapabilityResolveResult {
                primal_id,
                socket: None,
                virtual_endpoint: String::new(),
                native_endpoint,
                endpoint: TransportEndpoint::MeshRelay {
                    peer_id,
                    capability: params.capability.clone(),
                },
                capabilities: peer_caps,
                signature: None,
                signed_payload: None,
            };
            return serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"));
        }

        Err(format!("No provider found for capability: {}", params.capability))
    }

    /// Handle `capability.call` — cross-gate capability dispatch.
    ///
    /// 1. Resolves the capability to a local provider (via registry)
    /// 2. If local: connects to the provider's UDS socket and forwards the operation
    /// 3. If not local and routing is `"any"`: attempts remote dispatch via mesh peer
    pub(super) async fn handle_capability_call(&self, params: Value) -> Result<Value, String> {
        let call: CapabilityCallParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!(
            capability = %call.capability,
            operation = %call.operation,
            routing = %call.routing,
            "capability.call dispatch"
        );

        let registry = self.registry.read().await;
        if let Some((primal_id, entry)) = registry.resolve_by_capability(&call.capability).await {
            let socket_path = entry.native_endpoint.socket_path();
            drop(registry);

            if let Some(ref path) = socket_path {
                let result =
                    self.forward_to_local_provider(path, &call.operation, &call.params).await?;

                let response = CapabilityCallResult {
                    provider: primal_id,
                    gate: String::from("local"),
                    result,
                };
                return serde_json::to_value(response)
                    .map_err(|e| format!("Serialization error: {e}"));
            }

            return Err(format!(
                "Provider '{}' registered for '{}' but has no connectable socket",
                primal_id, call.capability
            ));
        }
        drop(registry);

        // Fallback: check if the capability is served by the proxy router (drawbridge weak bond).
        // This covers capabilities registered via SONGBIRD_PROXY_ROUTES or SONGBIRD_DRAWBRIDGE_ROUTES
        // that have no dedicated UDS provider.
        if let Some(route) = self.capability_router.route(&call.capability) {
            debug!(
                capability = %call.capability,
                backend = %route.base_url,
                "capability.call → drawbridge proxy route"
            );
            let path = call.params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let method = call.params.get("method").and_then(|v| v.as_str()).unwrap_or("POST");
            let body = call.params.get("body").and_then(|v| v.as_str()).map(String::from);
            let headers: std::collections::HashMap<String, String> = call
                .params
                .get("headers")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            let url = if path.is_empty() {
                route.base_url.clone()
            } else {
                format!("{}/{}", route.base_url.trim_end_matches('/'), path.trim_start_matches('/'))
            };

            let mut merged_headers = route.default_headers.clone();
            merged_headers.extend(headers);

            if let Some(api_key) = &route.api_key_env
                && let Ok(key) = songbird_process_env::var(api_key)
            {
                merged_headers.insert(String::from("Authorization"), format!("Bearer {key}"));
            }

            let request_params = crate::handlers::http_handler::HttpRequestParams {
                url,
                method: method.to_string(),
                headers: merged_headers,
                body,
                timeout_ms: route.timeout_ms,
            };

            let result = self
                .http_handler
                .handle_request(request_params)
                .await
                .map_err(|e| format!("Drawbridge proxy failed: {e}"))?;

            let response = CapabilityCallResult {
                provider: String::from("drawbridge"),
                gate: String::from("local"),
                result: serde_json::to_value(result)
                    .map_err(|e| format!("Serialization error: {e}"))?,
            };
            return serde_json::to_value(response).map_err(|e| format!("Serialization error: {e}"));
        }

        if call.routing == "local" {
            return Err(format!(
                "No local provider for capability '{}' (routing=local, remote dispatch disabled)",
                call.capability
            ));
        }

        self.forward_to_remote_gate(&call).await
    }

    /// Forward an operation to a local provider via its IPC socket.
    /// On Unix: connects to a Unix domain socket at `socket_path`.
    /// On Windows: connects to TCP localhost, parsing the port from `socket_path`.
    pub(super) async fn forward_to_local_provider(
        &self,
        socket_path: &str,
        operation: &str,
        params: &Value,
    ) -> Result<Value, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        #[cfg(unix)]
        let stream = tokio::time::timeout(
            DEFAULT_SOCKET_IO_TIMEOUT,
            tokio::net::UnixStream::connect(socket_path),
        )
        .await
        .map_err(|_| format!("Timeout connecting to provider at {socket_path}"))?
        .map_err(|e| format!("Cannot connect to provider at {socket_path}: {e}"))?;

        #[cfg(windows)]
        let stream = {
            let port: u16 = std::fs::read_to_string(socket_path)
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(songbird_types::defaults::ports::DEFAULT_HTTP_PORT);
            let addr = format!("127.0.0.1:{port}");
            tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, tokio::net::TcpStream::connect(&addr))
                .await
                .map_err(|_| format!("Timeout connecting to provider at {addr}"))?
                .map_err(|e| format!("Cannot connect to provider at {addr}: {e}"))?
        };

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": operation,
            "params": params,
            "id": 1
        });

        let mut request_bytes = serde_json::to_vec(&request)
            .map_err(|e| format!("Failed to serialize request: {e}"))?;
        request_bytes.push(b'\n');

        let (reader, mut writer) = stream.into_split();

        tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, writer.write_all(&request_bytes))
            .await
            .map_err(|_| format!("Timeout writing to provider at {socket_path}"))?
            .map_err(|e| format!("Write error to provider: {e}"))?;

        let mut buf_reader = BufReader::new(reader);
        let mut response_line = String::new();
        tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, buf_reader.read_line(&mut response_line))
            .await
            .map_err(|_| format!("Timeout reading from provider at {socket_path}"))?
            .map_err(|e| format!("Read error from provider: {e}"))?;

        let response: Value = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Invalid JSON response from provider: {e}"))?;

        if let Some(error) = response.get("error") {
            return Err(format!(
                "Provider error: {}",
                error.get("message").and_then(Value::as_str).unwrap_or("unknown")
            ));
        }

        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }
}
