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

        if call.routing == "local" {
            return Err(format!(
                "No local provider for capability '{}' (routing=local, remote dispatch disabled)",
                call.capability
            ));
        }

        self.forward_to_remote_gate(&call).await
    }

    /// Forward an operation to a local provider via its UDS socket.
    pub(super) async fn forward_to_local_provider(
        &self,
        socket_path: &str,
        operation: &str,
        params: &Value,
    ) -> Result<Value, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let stream =
            tokio::time::timeout(DEFAULT_SOCKET_IO_TIMEOUT, UnixStream::connect(socket_path))
                .await
                .map_err(|_| format!("Timeout connecting to provider at {socket_path}"))?
                .map_err(|e| format!("Cannot connect to provider at {socket_path}: {e}"))?;

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
