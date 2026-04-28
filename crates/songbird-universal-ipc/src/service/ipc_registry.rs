// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::{
    CapabilityResolveParams, CapabilityResolveResult, CompositionPrimalInfo, CompositionState,
    DiscoverParams, DiscoverResult, IpcServiceHandler, ListResult, ProviderInfo, RegisterParams,
    RegisterResult, ResolveParams, ResolveResult, ServiceInfo, ValidateConsumedResult,
};
use crate::endpoint::NativeEndpoint;
use crate::introspection::CONSUMED_CAPABILITIES;
use serde_json::Value;
use tracing::debug;

/// Build a deterministic canonical JSON payload for signing.
///
/// Field ordering is alphabetical by short key to ensure identical bytes
/// regardless of capability insertion order.
fn build_canonical_payload(
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
    pub(super) async fn handle_register(&self, params: Value) -> Result<Value, String> {
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

        // Sign via BearDog if crypto provider is available
        let (signature, signed_payload) = self.sign_payload(&canonical).await;

        // Register in registry (`register` takes `&self` and uses its own inner lock)
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

        let result = RegisterResult {
            virtual_endpoint: virtual_endpoint.path,
            registered_at,
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
            Ok(result) => {
                let sig = result.get("signature").and_then(|v| v.as_str()).map(String::from);
                if let Some(ref s) = sig {
                    debug!("Signed registration payload ({} bytes)", s.len());
                    (Some(s.clone()), Some(canonical_payload.to_string()))
                } else {
                    tracing::warn!("crypto.sign.ed25519 returned no signature field");
                    (None, None)
                }
            }
            Err(e) => {
                tracing::warn!("Failed to sign registration (standalone fallback): {e}");
                (None, None)
            }
        }
    }

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
    pub(super) async fn handle_resolve(&self, params: Value) -> Result<Value, String> {
        let params: ResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        let registry = self.registry.read().await;

        let (resolved_name, entry) = if let Some(ref capability) = params.capability {
            debug!("Resolving by capability: {capability}");
            if let Some(found) = registry.resolve_by_capability(capability).await {
                found
            } else if let Some(entry) = registry.get_service(capability).await {
                debug!(
                    "Capability '{capability}' not found, but matched as primal name (capability-first fallback)"
                );
                (capability.clone(), entry)
            } else {
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

        let result = ResolveResult {
            virtual_endpoint: entry.virtual_endpoint.path,
            native_endpoint: entry.native_endpoint.display(),
            capabilities: entry.capabilities,
            signature: entry.signature,
            signed_payload: entry.signed_payload,
        };

        drop(registry);
        debug!("Resolved to: {resolved_name}");

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.discover` method
    pub(super) async fn handle_discover(&self, params: Value) -> Result<Value, String> {
        let params: DiscoverParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!("Discovering capability: {}", params.capability);

        // Discover from registry (returns virtual paths)
        let registry = self.registry.read().await;
        let virtual_paths = registry.find_by_capability(&params.capability).await;

        // Get full service entries for each path
        let mut provider_infos = Vec::new();
        for virtual_path in virtual_paths {
            // Extract service name from virtual path
            if let Some(name) = virtual_path.strip_prefix("/primal/")
                && let Some(entry) = registry.get_service(name).await
            {
                provider_infos.push(ProviderInfo {
                    primal_id: name.to_string(),
                    virtual_endpoint: entry.virtual_endpoint.path,
                    native_endpoint: entry.native_endpoint.display(),
                    capabilities: entry.capabilities,
                    signature: entry.signature,
                    signed_payload: entry.signed_payload,
                });
            }
        }

        let result = DiscoverResult {
            providers: provider_infos,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.list` method
    pub(super) async fn handle_list(&self, _params: Value) -> Result<Value, String> {
        debug!("Listing all services");

        let registry = self.registry.read().await;
        let service_names = registry.list_services().await;

        let mut service_infos = Vec::new();
        for name in service_names {
            if let Some(entry) = registry.get_service(&name).await {
                service_infos.push(ServiceInfo {
                    primal_id: name,
                    virtual_endpoint: entry.virtual_endpoint.path,
                    capabilities: entry.capabilities,
                });
            }
        }

        let result = ListResult {
            services: service_infos,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `capability.resolve` — single-step routing by capability.
    ///
    /// Returns the best provider endpoint for the requested capability (most
    /// recently seen wins). This is the IPC equivalent of DNS resolution:
    /// springs call `capability.resolve("crypto.sign")` and get back a single
    /// socket/endpoint instead of iterating a list.
    pub(super) async fn handle_capability_resolve(&self, params: Value) -> Result<Value, String> {
        let params: CapabilityResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!("Resolving best provider for capability: {}", params.capability);

        let registry = self.registry.read().await;
        let (name, entry) = registry
            .resolve_by_capability(&params.capability)
            .await
            .ok_or_else(|| format!("No provider found for capability: {}", params.capability))?;

        let result = CapabilityResolveResult {
            primal_id: name,
            virtual_endpoint: entry.virtual_endpoint.path,
            native_endpoint: entry.native_endpoint.display(),
            capabilities: entry.capabilities,
            signature: entry.signature,
            signed_payload: entry.signed_payload,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `lifecycle.composition` — returns current composition state for dashboards.
    pub(super) async fn handle_lifecycle_composition(
        &self,
        _params: Value,
    ) -> Result<Value, String> {
        debug!("Returning composition state");

        let registry = self.registry.read().await;
        let service_names = registry.list_services().await;

        let mut primals = Vec::new();
        let mut total_capabilities = 0usize;

        for name in service_names {
            if let Some(entry) = registry.get_service(&name).await {
                total_capabilities += entry.capabilities.len();
                primals.push(CompositionPrimalInfo {
                    primal_id: name,
                    capabilities: entry.capabilities,
                    virtual_endpoint: entry.virtual_endpoint.path,
                    status: "up",
                });
            }
        }

        let result = CompositionState {
            primals,
            total_capabilities,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `lifecycle.validate_consumed` — checks that all consumed capabilities
    /// are satisfiable by currently registered providers in the composition.
    pub(super) async fn handle_validate_consumed(&self, _params: Value) -> Result<Value, String> {
        debug!("Validating consumed capabilities");

        let registry = self.registry.read().await;
        let mut satisfied = Vec::new();
        let mut unsatisfied = Vec::new();

        for &cap in CONSUMED_CAPABILITIES {
            let providers = registry.find_by_capability(cap).await;
            if providers.is_empty() {
                unsatisfied.push(cap.to_string());
            } else {
                satisfied.push(cap.to_string());
            }
        }

        let result = ValidateConsumedResult {
            valid: unsatisfied.is_empty(),
            satisfied,
            unsatisfied,
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn canonical_payload_is_deterministic_regardless_of_cap_order() {
        let a = build_canonical_payload(
            "nestgate",
            &["storage".into(), "crypto".into(), "auth".into()],
            "/tmp/nestgate.sock",
            "2026-04-28T12:00:00Z",
        );
        let b = build_canonical_payload(
            "nestgate",
            &["auth".into(), "crypto".into(), "storage".into()],
            "/tmp/nestgate.sock",
            "2026-04-28T12:00:00Z",
        );
        assert_eq!(a, b, "canonical payload must be order-independent");
    }

    #[test]
    fn canonical_payload_contains_all_fields() {
        let payload = build_canonical_payload(
            "beardog",
            &["crypto".into(), "security".into()],
            "/run/user/1000/biomeos/beardog.sock",
            "2026-04-28T14:30:00Z",
        );
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["p"], "beardog");
        assert_eq!(parsed["e"], "/run/user/1000/biomeos/beardog.sock");
        assert_eq!(parsed["t"], "2026-04-28T14:30:00Z");
        let caps = parsed["c"].as_array().unwrap();
        assert_eq!(caps[0], "crypto");
        assert_eq!(caps[1], "security");
    }

    #[test]
    fn canonical_payload_empty_capabilities() {
        let payload =
            build_canonical_payload("minimal", &[], "tcp://127.0.0.1:9000", "2026-01-01T00:00:00Z");
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert!(parsed["c"].as_array().unwrap().is_empty());
    }
}
