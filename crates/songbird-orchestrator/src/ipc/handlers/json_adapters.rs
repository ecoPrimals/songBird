// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Pure JSON adapter methods delegating to domain handler modules.

use super::IpcHandlers;

impl IpcHandlers {
    /// Service Registry: `register_service` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn register_service_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::service_registry::register_service_json(self, params).await
    }

    /// `capability.resolve` — single-step routing by capability (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if no provider is registered for the capability.
    pub async fn capability_resolve_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::service_registry::capability_resolve_json(self, params).await
    }

    /// Service Registry: `discover_by_capability` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_by_capability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::service_registry::discover_by_capability_json(self, params).await
    }

    /// Service Registry: `get_service_health` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_service_health_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::service_registry::get_service_health_json(self, params).await
    }

    /// Service Registry: `health_check` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn health_check_json(
        &self,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::service_registry::health_check_json(self).await
    }

    /// P2P Discovery: `discover_by_family` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_by_family_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::p2p_discovery::discover_by_family_json(self, params).await
    }

    /// P2P Discovery: `create_genetic_tunnel` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn create_genetic_tunnel_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::p2p_discovery::create_genetic_tunnel_json(self, params).await
    }

    /// P2P Discovery: `announce_capabilities` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn announce_capabilities_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::p2p_discovery::announce_capabilities_json(self, params).await
    }

    /// Graph Intelligence: `validate_graph` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn validate_graph_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::graph_intelligence::validate_graph_json(self, params).await
    }

    /// Graph Intelligence: `check_availability` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn check_availability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::graph_intelligence::check_availability_json(self, params).await
    }

    /// Graph Intelligence: `suggest_alternatives` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn suggest_alternatives_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::graph_intelligence::suggest_alternatives_json(self, params).await
    }

    /// Graph Intelligence: `validate_coordination_pattern` (pure JSON adapter)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn validate_coordination_pattern_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        super::graph_intelligence::validate_coordination_pattern_json(self, params).await
    }
}
