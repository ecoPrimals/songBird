// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::{IpcServiceHandler, ResolveParams, ResolveResult};
use super::transport::transport_endpoint_from_native;
use serde_json::Value;
use tracing::debug;

impl IpcServiceHandler {
    /// Handle `ipc.resolve` method — resolves by `capability` or `primal_id`/`name`.
    ///
    /// **Capability-first** is the standard pattern: callers should resolve by what
    /// a primal *does* (e.g. `"security"`), not what it *is* (e.g. `"beardog"`).
    ///
    /// Precedence: `capability` > `primal_id`/`name`.
    ///
    /// Graceful fallback: if `capability` lookup fails, the same string is tried
    /// as a primal name. This accommodates callers who conflate capability tokens
    /// with primal names (e.g. `resolve({"capability": "beardog"})` will succeed
    /// if `BearDog` registered with `primal_id` = `"beardog"`).
    ///
    /// `ipc.resolve_by_name` is a normalization alias that routes here.
    pub(in crate::service) async fn handle_resolve(&self, params: Value) -> Result<Value, String> {
        let params: ResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        let registry = self.registry.read().await;

        let (resolved_name, entry) = if let Some(ref capability) = params.capability {
            debug!("Resolving by capability: {capability}");
            if let Some(found) = registry.resolve_by_capability(capability).await {
                found
            } else if let Some(entry) = registry.get_service(capability).await {
                tracing::warn!(
                    capability = %capability,
                    "Name-based fallback in ipc.resolve is deprecated; resolve by capability, not primal name"
                );
                debug!(
                    "Capability '{capability}' not found, but matched as primal name (capability-first fallback)"
                );
                (capability.clone(), entry)
            } else {
                drop(registry);
                // Mesh fallback: check if a remote peer advertises this capability
                if let Some((peer_id, peer_caps)) =
                    self.mesh_handler.find_peer_with_capability(capability).await
                {
                    debug!(
                        capability = %capability,
                        peer = %peer_id,
                        "Resolved capability via mesh peer (topology-aware routing)"
                    );
                    let native_endpoint = format!("mesh://{peer_id}");
                    let result = ResolveResult {
                        socket: None,
                        virtual_endpoint: String::new(),
                        native_endpoint,
                        endpoint: super::super::TransportEndpoint::MeshRelay {
                            peer_id,
                            capability: capability.clone(),
                        },
                        capabilities: peer_caps,
                        relay: true,
                        relay_socket: None,
                        signature: None,
                        signed_payload: None,
                    };
                    return serde_json::to_value(result)
                        .map_err(|e| format!("Serialization error: {e}"));
                }
                return Err(format!("No provider found for capability: {capability}"));
            }
        } else if let Some(ref primal_id) = params.primal_id {
            debug!("Resolving by primal_id: {primal_id}");
            let entry = registry
                .get_service(primal_id)
                .await
                .ok_or_else(|| format!("Primal not found: {primal_id}"))?;
            (primal_id.clone(), entry)
        } else {
            return Err(
                "ipc.resolve requires either `primal_id`/`name` or `capability` parameter".into()
            );
        };

        let native_transport = transport_endpoint_from_native(&entry.native_endpoint);
        let native_socket = entry.native_endpoint.socket_path();
        let native_display = entry.native_endpoint.display();
        let resolved_capability = params.capability.clone().unwrap_or_default();
        let capabilities = entry.capabilities;
        let signature = entry.signature;
        let signed_payload = entry.signed_payload;
        let virtual_path = entry.virtual_endpoint.path;
        drop(registry);

        // Determine relay availability and whether to use it
        let relay_path = self.virtual_relay.get_relay_path(&resolved_name).await;
        let use_relay = params.prefer_virtual && !params.native && relay_path.is_some();

        let socket = if use_relay {
            relay_path.as_ref().map(|p| p.display().to_string())
        } else {
            native_socket
        };

        // Build transport-qualified endpoint (Phase 2)
        let endpoint = if use_relay {
            super::super::TransportEndpoint::MeshRelay {
                peer_id: resolved_name.clone(),
                capability: resolved_capability,
            }
        } else {
            native_transport
        };

        let result = ResolveResult {
            socket,
            virtual_endpoint: virtual_path,
            native_endpoint: native_display,
            endpoint,
            capabilities,
            relay: use_relay,
            relay_socket: relay_path.map(|p| p.display().to_string()),
            signature,
            signed_payload,
        };

        debug!("Resolved to: {resolved_name} (relay={})", result.relay);

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }
}
