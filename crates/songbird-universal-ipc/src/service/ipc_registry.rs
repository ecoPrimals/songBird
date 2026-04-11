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

impl IpcServiceHandler {
    /// Handle `ipc.register` method
    pub(super) async fn handle_register(&self, params: Value) -> Result<Value, String> {
        let params: RegisterParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        tracing::info!("Registering primal: {} at {}", params.primal_id, params.endpoint);

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

        // Register in registry (`register` takes `&self` and uses its own inner lock)
        let virtual_endpoint = self
            .registry
            .read()
            .await
            .register(&params.primal_id, native_endpoint, params.capabilities)
            .await
            .map_err(|e| format!("Registration failed: {e}"))?;

        let result = RegisterResult {
            virtual_endpoint: virtual_endpoint.path,
            registered_at: chrono::Utc::now().to_rfc3339(),
        };

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `ipc.resolve` method
    pub(super) async fn handle_resolve(&self, params: Value) -> Result<Value, String> {
        let params: ResolveParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        debug!("Resolving primal: {}", params.primal_id);

        // Get service entry from registry
        let entry = self
            .registry
            .read()
            .await
            .get_service(&params.primal_id)
            .await
            .ok_or_else(|| format!("Primal not found: {}", params.primal_id))?;

        let result = ResolveResult {
            virtual_endpoint: entry.virtual_endpoint.path,
            native_endpoint: entry.native_endpoint.display(),
            capabilities: entry.capabilities,
        };

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
