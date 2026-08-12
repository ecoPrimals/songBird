// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::{
    DiscoverParams, DiscoverResult, IpcServiceHandler, ListResult, ProviderInfo, ServiceInfo,
};
use serde_json::Value;
use tracing::debug;

impl IpcServiceHandler {
    /// Handle `ipc.discover` method
    pub(in crate::service) async fn handle_discover(&self, params: Value) -> Result<Value, String> {
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
                    socket: entry.native_endpoint.socket_path(),
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
    pub(in crate::service) async fn handle_list(&self, _params: Value) -> Result<Value, String> {
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

    /// Handle `ipc.relay_stats` — return virtual relay performance metrics.
    ///
    /// ## Response
    /// ```json
    /// { "active_relays": 3, "total_requests": 1024, "avg_overhead_us": 342 }
    /// ```
    pub(in crate::service) async fn handle_relay_stats(
        &self,
        _params: Value,
    ) -> Result<Value, String> {
        let relays = self.virtual_relay.list_relays().await;
        let metrics = self.virtual_relay.metrics();

        Ok(serde_json::json!({
            "active_relays": relays.len(),
            "relays": relays.iter().map(|(name, path)| {
                serde_json::json!({"primal": name, "socket": path.display().to_string()})
            }).collect::<Vec<_>>(),
            "total_requests": metrics.requests.load(std::sync::atomic::Ordering::Relaxed),
            "avg_overhead_us": metrics.avg_overhead_us(),
            "total_overhead_us": metrics.overhead_us.load(std::sync::atomic::Ordering::Relaxed),
        }))
    }

    /// Handle `ipc.watch` — poll for registry changes since a given revision.
    ///
    /// Enables consuming primals (e.g. toadStool) to detect when new providers
    /// register capabilities they depend on. Returns events since `since_revision`,
    /// optionally filtered by capability names.
    ///
    /// ## Params
    /// ```json
    /// { "since_revision": 0, "capabilities": ["shader", "compile"] }
    /// ```
    ///
    /// ## Response
    /// ```json
    /// {
    ///   "revision": 5,
    ///   "events": [
    ///     { "revision": 3, "kind": "registered", "primal": "coralReef",
    ///       "capabilities": ["shader", "compile", "visualization"],
    ///       "endpoint": "unix:///run/user/1000/biomeos/coralreef-nucleus01.sock" }
    ///   ]
    /// }
    /// ```
    pub(in crate::service) async fn handle_watch(&self, params: Value) -> Result<Value, String> {
        #[derive(serde::Deserialize)]
        struct WatchParams {
            #[serde(default)]
            since_revision: u64,
            #[serde(default)]
            capabilities: Option<Vec<String>>,
        }

        let params: WatchParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        let registry = self.registry.read().await;
        let (current_rev, events) =
            registry.events_since(params.since_revision, params.capabilities.as_deref()).await;

        serde_json::to_value(serde_json::json!({
            "revision": current_rev,
            "events": events,
        }))
        .map_err(|e| format!("Serialization error: {e}"))
    }
}
