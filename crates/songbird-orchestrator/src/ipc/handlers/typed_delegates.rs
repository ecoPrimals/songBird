// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Typed RPC delegate methods for service registry, P2P discovery, and graph intelligence.

use super::IpcHandlers;

impl IpcHandlers {
    // ========================================================================
    // Service Registry APIs (v3.20.0) - Delegated to service_registry module
    // ========================================================================

    /// Handle `register_service` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn register_service(
        &self,
        params: serde_json::Value,
    ) -> Result<
        super::super::types::RegisterServiceResponse,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        super::service_registry::register_service(self, params).await
    }

    /// Handle `discover_by_capability` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_by_capability(
        &self,
        params: serde_json::Value,
    ) -> Result<
        super::super::types::DiscoverByCapabilityResponse,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        super::service_registry::discover_by_capability(self, params).await
    }

    /// Handle `get_service_health` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_service_health(
        &self,
        params: serde_json::Value,
    ) -> Result<
        super::super::types::GetServiceHealthResponse,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        super::service_registry::get_service_health(self, params).await
    }

    /// Handle `health_check` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn health_check(
        &self,
        params: serde_json::Value,
    ) -> Result<super::super::types::HealthCheckResponse, crate::ipc::pure_rust_server::JsonRpcError>
    {
        super::service_registry::health_check(self, params).await
    }

    // ========================================================================
    // P2P Discovery APIs (v3.19.1) - Delegated to p2p_discovery module
    // ========================================================================

    /// Handle `discover_by_family` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn discover_by_family(
        &self,
        params: serde_json::Value,
    ) -> Result<
        super::super::types::DiscoverByFamilyResponse,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        super::p2p_discovery::discover_by_family(self, params).await
    }

    /// Handle `create_genetic_tunnel` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn create_genetic_tunnel(
        &self,
        params: serde_json::Value,
    ) -> Result<
        super::super::types::CreateGeneticTunnelResponse,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        super::p2p_discovery::create_genetic_tunnel(self, params).await
    }

    /// Handle `announce_capabilities` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn announce_capabilities(
        &self,
        params: serde_json::Value,
    ) -> Result<
        super::super::types::AnnounceCapabilitiesResponse,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        super::p2p_discovery::announce_capabilities(self, params).await
    }

    // ========================================================================
    // Graph Intelligence APIs (v3.21.0) - Delegated to graph_intelligence module
    // ========================================================================

    /// Handle `graph.validate` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn validate_graph(
        &self,
        params: serde_json::Value,
    ) -> Result<crate::graph::ValidationResult, crate::ipc::pure_rust_server::JsonRpcError> {
        super::graph_intelligence::validate_graph(self, params).await
    }

    /// Handle `graph.check_availability` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn check_availability(
        &self,
        params: serde_json::Value,
    ) -> Result<crate::graph::AvailabilityReport, crate::ipc::pure_rust_server::JsonRpcError> {
        super::graph_intelligence::check_availability(self, params).await
    }

    /// Handle `graph.suggest_alternatives` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn suggest_alternatives(
        &self,
        params: serde_json::Value,
    ) -> Result<crate::graph::AlternativeSuggestions, crate::ipc::pure_rust_server::JsonRpcError>
    {
        super::graph_intelligence::suggest_alternatives(self, params).await
    }

    /// Handle `coordination.validate_pattern` RPC call (Pure Rust, v3.34.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn validate_coordination_pattern(
        &self,
        params: serde_json::Value,
    ) -> Result<
        crate::graph::CoordinationValidationResult,
        crate::ipc::pure_rust_server::JsonRpcError,
    > {
        super::graph_intelligence::validate_coordination_pattern(self, params).await
    }
}
