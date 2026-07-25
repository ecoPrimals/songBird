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
    /// 1. Validates routing field (must be `"local"` or `"any"`)
    /// 2. Resolves the capability to a local provider (via registry)
    /// 3. If local: connects to the provider's UDS socket and forwards the operation
    /// 4. If not local and routing is `"any"`: attempts remote dispatch via mesh peer
    pub(super) async fn handle_capability_call(&self, params: Value) -> Result<Value, String> {
        let call: CapabilityCallParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        // Routing validation: only explicit values accepted (pen finding: capability-escalation)
        if call.routing != "local" && call.routing != "any" {
            return Err(format!("Invalid routing '{}': must be 'local' or 'any'", call.routing));
        }

        // Capability name validation: prevent injection via malformed capability names
        if call.capability.is_empty()
            || call.capability.len() > 128
            || call.capability.chars().any(char::is_control)
        {
            return Err(String::from("Invalid capability name"));
        }

        // Operation name validation
        if call.operation.is_empty()
            || call.operation.len() > 256
            || call.operation.chars().any(char::is_control)
        {
            return Err(String::from("Invalid operation name"));
        }

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
                match self
                    .forward_to_local_provider_with_retry(path, &call.operation, &call.params)
                    .await
                {
                    Ok(result) => {
                        let response = CapabilityCallResult {
                            provider: primal_id,
                            gate: String::from("local"),
                            result,
                        };
                        return serde_json::to_value(response)
                            .map_err(|e| format!("Serialization error: {e}"));
                    }
                    Err(e) => {
                        debug!(
                            provider = %primal_id,
                            path = %path,
                            error = %e,
                            "Local provider dispatch failed after retries"
                        );
                        return Err(e);
                    }
                }
            }

            return Err(format!(
                "Provider '{}' registered for '{}' but has no connectable socket",
                primal_id, call.capability
            ));
        }
        drop(registry);

        // Fallback: drawbridge proxy route or remote dispatch
        if let Some(route) = self.capability_router.route(&call.capability) {
            return self.forward_via_drawbridge_route(&call, route).await;
        }

        if call.routing == "local" {
            return Err(format!(
                "No local provider for capability '{}' (routing=local, remote dispatch disabled)",
                call.capability
            ));
        }

        self.forward_to_remote_gate(&call).await
    }

    /// Forward a capability call through a drawbridge proxy route (HTTP backend).
    async fn forward_via_drawbridge_route(
        &self,
        call: &CapabilityCallParams,
        route: crate::service::ProxyRoute,
    ) -> Result<Value, String> {
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
        serde_json::to_value(response).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Forward to a local provider with exponential backoff retry.
    ///
    /// Handles transient provider restarts (e.g. bearDog cycling) by retrying
    /// up to 2 additional times with increasing delay (100ms, 300ms).
    async fn forward_to_local_provider_with_retry(
        &self,
        socket_path: &str,
        operation: &str,
        params: &Value,
    ) -> Result<Value, String> {
        const RETRY_DELAYS_MS: &[u64] = &[100, 300];

        match self.forward_to_local_provider(socket_path, operation, params).await {
            Ok(result) => Ok(result),
            Err(e) if e.contains("Cannot connect") || e.contains("unreachable") => {
                debug!(
                    socket_path,
                    error = %e,
                    "Provider connect failed — entering retry loop"
                );
                let mut last_err = e;
                for delay_ms in RETRY_DELAYS_MS {
                    tokio::time::sleep(std::time::Duration::from_millis(*delay_ms)).await;
                    match self.forward_to_local_provider(socket_path, operation, params).await {
                        Ok(result) => return Ok(result),
                        Err(retry_err) => last_err = retry_err,
                    }
                }
                Err(last_err)
            }
            Err(e) => Err(e),
        }
    }

    /// Forward an operation to a local provider via its IPC socket.
    /// On Unix: connects to a Unix domain socket at `socket_path`.
    /// On Windows: connects to TCP localhost, parsing the port from `socket_path`.
    ///
    /// Uses the IPC connection pool to avoid per-request connect/disconnect overhead.
    /// Failed connections are retried once with a fresh connection.
    pub(super) async fn forward_to_local_provider(
        &self,
        socket_path: &str,
        operation: &str,
        params: &Value,
    ) -> Result<Value, String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": operation,
            "params": params,
            "id": 1
        });

        let mut request_bytes = serde_json::to_vec(&request)
            .map_err(|e| format!("Failed to serialize request: {e}"))?;
        request_bytes.push(b'\n');

        let response_line = self
            .ipc_pool
            .execute_jsonrpc(socket_path, &request_bytes, DEFAULT_SOCKET_IO_TIMEOUT)
            .await?;

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

    /// Handle `capability.health` — dispatch-path health probe.
    ///
    /// For each registered provider with a local socket, attempts a lightweight
    /// connection probe and reports reachability. Designed for cellMembrane
    /// service monitoring and automated failover decisions.
    ///
    /// # Errors
    ///
    /// Returns an error only if internal state is unreadable.
    pub(super) async fn handle_capability_health(&self, _params: Value) -> Result<Value, String> {
        let registry = self.registry.read().await;
        let names = registry.list_services().await;
        let mut providers: Vec<Value> = Vec::new();

        for name in &names {
            let Some(entry) = registry.get_service(name).await else {
                continue;
            };
            let Some(socket_path) = entry.native_endpoint.socket_path() else {
                continue;
            };

            let probe_start = std::time::Instant::now();
            let reachable = tokio::time::timeout(
                std::time::Duration::from_millis(500),
                self.ipc_pool.acquire(&socket_path),
            )
            .await;

            let (status, latency_ms) = match reachable {
                Ok(Ok(stream)) => {
                    let ms = u64::try_from(probe_start.elapsed().as_millis()).unwrap_or(u64::MAX);
                    self.ipc_pool.release(&socket_path, stream).await;
                    ("reachable", Some(ms))
                }
                Ok(Err(e)) => {
                    debug!(name, socket_path, error = %e, "capability.health: unreachable");
                    ("unreachable", None)
                }
                Err(_) => {
                    debug!(name, socket_path, "capability.health: timeout");
                    ("timeout", None)
                }
            };

            providers.push(serde_json::json!({
                "primal_id": name,
                "socket": socket_path,
                "status": status,
                "latency_ms": latency_ms,
            }));
        }

        let all_healthy = providers.iter().all(|p| p["status"] == "reachable");
        Ok(serde_json::json!({
            "healthy": all_healthy,
            "provider_count": providers.len(),
            "providers": providers,
        }))
    }
}
