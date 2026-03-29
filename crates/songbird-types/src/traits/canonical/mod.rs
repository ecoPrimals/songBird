// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🔧 Canonical Provider Traits - Single Source of Truth
//!
//! **UNIFIED CANONICAL TRAIT SYSTEM** ✅
//!
//! This module provides the single, authoritative trait hierarchy for all Songbird providers.
//! It consolidates and replaces all fragmented trait definitions across the ecosystem.
//!
//! ## Consolidation Summary
//! - **Provider**: Base trait for all providers (replaces 3+ definitions)
//! - **`ServiceProvider`**: Service-oriented operations (unified from discovery traits)
//! - **`PrimalProvider`**: Primal-specific operations (moved from songbird-universal-primals)
//! - **`DiscoveryProvider`**: Service discovery (consolidated from 4+ definitions)
//! - **`CapabilityProvider`**: Capability-based systems (unified interface)
//! - **`SecurityProvider`**: Security operations (consolidated)
//! - **`OrchestrationProvider`**: Service orchestration (unified)
//! - **`ObservabilityProvider`**: Metrics & monitoring (consolidated)
//!
//! ## Dyn-Compatibility Note (November 2025)
//! These traits use `#[async_trait]` to maintain dyn-compatibility. They are extensively
//! used with trait objects (`Box<dyn Provider>`, `Arc<dyn Provider>`) throughout the codebase
//! for plugin systems, registries, and dynamic dispatch. While native async traits offer
//! better performance, they cannot be used with trait objects in current Rust.
//!
//! **Trade-off**: We prioritize dyn-compatibility over the 15-40% perf gains from native async,
//! as the provider system's architecture fundamentally requires dynamic dispatch.

use async_trait::async_trait;
use futures_util::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

mod canonical_types;

// Re-export core error types
pub use crate::errors::{SongbirdError, SongbirdResult};
pub use canonical_types::*;

// ============================================================================
// CANONICAL BASE PROVIDER TRAIT
// ============================================================================

/// **CANONICAL**: Base provider trait - foundation for all providers
///
/// This trait provides the common interface that all providers must implement.
/// It replaces multiple scattered provider definitions across crates.
///
/// **Dyn-Compatible**: Uses `#[async_trait]` to support trait objects (`Box<dyn Provider>`).
#[async_trait]
pub trait Provider: Send + Sync + 'static {
    /// Unique provider identifier
    fn id(&self) -> &str;

    /// Human-readable provider name
    fn name(&self) -> &str;

    /// Provider version
    fn version(&self) -> &str;

    /// Provider type (e.g., "service", "primal", "discovery")
    fn provider_type(&self) -> ProviderType;

    /// Initialize the provider with configuration
    async fn initialize(&mut self, config: ProviderConfig) -> SongbirdResult<()>;

    /// Shutdown the provider gracefully
    async fn shutdown(&mut self) -> SongbirdResult<()>;

    /// Perform health check
    async fn health_check(&self) -> SongbirdResult<HealthStatus>;

    /// Get provider metadata
    fn metadata(&self) -> ProviderMetadata;

    /// Get provider capabilities
    async fn capabilities(&self) -> SongbirdResult<Vec<Capability>>;
}

// ============================================================================
// CANONICAL SERVICE PROVIDER TRAIT
// ============================================================================

/// **CANONICAL**: Service provider trait - for service-oriented providers
///
/// Consolidates service-related functionality from multiple trait definitions.
/// Replaces `ServiceProvider` from `unified_providers` and discovery traits.
#[async_trait]
pub trait ServiceProvider: Provider {
    /// Get service type identifier
    fn service_type(&self) -> ServiceType;

    /// Handle service request
    async fn handle_request(&self, request: ServiceRequest) -> SongbirdResult<ServiceResponse>;

    /// Get service metrics
    async fn metrics(&self) -> SongbirdResult<ServiceMetrics>;

    /// Register service with discovery system
    async fn register_service(&self, info: ServiceInfo) -> SongbirdResult<()>;

    /// Unregister service from discovery system
    async fn unregister_service(&self, service_id: &str) -> SongbirdResult<()>;

    /// Update service health status
    async fn update_health(&self, health: HealthStatus) -> SongbirdResult<()>;
}

// ============================================================================
// CANONICAL PRIMAL PROVIDER TRAIT
// ============================================================================

/// **CANONICAL**: Primal provider trait - for primal-specific operations
///
/// Consolidates primal functionality from songbird-universal-primals.
/// This is the single source of truth for primal provider interfaces.
#[async_trait]
pub trait PrimalProvider: Provider {
    /// Get primal type (security, storage, compute, ai, network, custom)
    fn primal_type(&self) -> PrimalType;

    /// Execute primal capability
    async fn execute_capability(
        &self,
        capability: &str,
        context: PrimalContext,
        params: HashMap<String, serde_json::Value>,
    ) -> SongbirdResult<PrimalResponse>;

    /// Get primal dependencies
    async fn dependencies(&self) -> SongbirdResult<Vec<PrimalDependency>>;

    /// Check if primal can integrate with another primal
    async fn can_integrate_with(
        &self,
        other_type: &str,
        other_capabilities: &[String],
    ) -> SongbirdResult<bool>;

    /// Integrate with another primal at runtime
    ///
    /// Note: Uses generic parameter for native async trait compatibility.
    /// Implementations can accept any type implementing `PrimalProvider`.
    async fn integrate_with<P: PrimalProvider>(
        &mut self,
        other_primal: Arc<P>,
    ) -> SongbirdResult<IntegrationResult>;

    /// Get primal configuration schema
    fn config_schema(&self) -> serde_json::Value;

    /// Apply configuration dynamically
    async fn apply_config(&mut self, config: serde_json::Value) -> SongbirdResult<()>;
}

// ============================================================================
// CANONICAL DISCOVERY PROVIDER TRAIT
// ============================================================================

/// **CANONICAL**: Discovery provider trait - for service discovery
///
/// Consolidates all discovery functionality from multiple definitions.
/// Replaces `CanonicalServiceDiscovery`, `ServiceDiscovery`, and `DiscoveryProvider`.
#[async_trait]
pub trait DiscoveryProvider: Provider {
    /// Discover services matching criteria
    async fn discover_services(
        &self,
        criteria: DiscoveryCriteria,
    ) -> SongbirdResult<Vec<ServiceInfo>>;

    /// Discover primals by capability
    async fn discover_primals(&self, capability: &str) -> SongbirdResult<Vec<PrimalInfo>>;

    /// Register service with discovery system
    async fn register(&self, service: ServiceInfo) -> SongbirdResult<()>;

    /// Unregister service from discovery system
    async fn unregister(&self, service_id: &str) -> SongbirdResult<()>;

    /// Watch for service changes
    async fn watch_services(
        &self,
        query: DiscoveryQuery,
    ) -> SongbirdResult<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>>;

    /// Update service metadata
    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> SongbirdResult<()>;

    /// Check if service is registered
    async fn is_registered(&self, service_id: &str) -> SongbirdResult<bool>;

    /// List all registered services
    async fn list_all(&self) -> SongbirdResult<Vec<ServiceInfo>>;

    /// Get discovery backend type
    fn backend_type(&self) -> &'static str;
}

// ============================================================================
// CANONICAL CAPABILITY PROVIDER TRAIT
// ============================================================================

/// **CANONICAL**: Capability provider trait - for capability-based systems
///
/// Unified interface for capability-based operations.
#[async_trait]
pub trait CapabilityProvider: Provider {
    /// Get available capabilities
    async fn get_capabilities(&self) -> SongbirdResult<Vec<Capability>>;

    /// Check if capability is supported
    async fn supports_capability(&self, capability: &str) -> SongbirdResult<bool>;

    /// Execute capability with parameters
    async fn execute_capability(
        &self,
        capability: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> SongbirdResult<serde_json::Value>;

    /// Get capability metadata
    async fn capability_metadata(&self, capability: &str) -> SongbirdResult<CapabilityMetadata>;
}

// ============================================================================
// CANONICAL SECURITY PROVIDER TRAIT
// ============================================================================

/// **CANONICAL**: Security provider trait - for security operations
#[async_trait]
pub trait SecurityProvider: Provider {
    /// Authenticate user with credentials
    async fn authenticate(&self, credentials: Credentials) -> SongbirdResult<AuthToken>;

    /// Authorize request with token
    async fn authorize(
        &self,
        token: &AuthToken,
        resource: &str,
        action: &str,
    ) -> SongbirdResult<bool>;

    /// Encrypt data
    async fn encrypt(&self, data: &[u8]) -> SongbirdResult<Vec<u8>>;

    /// Decrypt data
    async fn decrypt(&self, data: &[u8]) -> SongbirdResult<Vec<u8>>;

    /// Generate secure token
    async fn generate_token(&self, claims: TokenClaims) -> SongbirdResult<AuthToken>;

    /// Validate token
    async fn validate_token(&self, token: &AuthToken) -> SongbirdResult<TokenValidation>;
}

// ============================================================================
// CANONICAL ORCHESTRATION PROVIDER TRAIT
// ============================================================================

/// **CANONICAL**: Orchestration provider trait - for service orchestration
#[async_trait]
pub trait OrchestrationProvider: Provider {
    /// Deploy service
    async fn deploy(&self, deployment: DeploymentSpec) -> SongbirdResult<DeploymentResult>;

    /// Scale service
    async fn scale(&self, service_id: &str, replicas: u32) -> SongbirdResult<()>;

    /// Update service
    async fn update(&self, service_id: &str, spec: DeploymentSpec) -> SongbirdResult<()>;

    /// Delete service
    async fn delete(&self, service_id: &str) -> SongbirdResult<()>;

    /// Get deployment status
    async fn get_status(&self, service_id: &str) -> SongbirdResult<DeploymentStatus>;

    /// List all deployments
    async fn list_deployments(&self) -> SongbirdResult<Vec<DeploymentInfo>>;

    /// Get deployment logs
    async fn get_logs(&self, service_id: &str, lines: Option<u32>) -> SongbirdResult<Vec<String>>;
}

// ============================================================================
// CANONICAL OBSERVABILITY PROVIDER TRAIT
// ============================================================================

/// **CANONICAL**: Observability provider trait - for metrics & monitoring
#[async_trait]
pub trait ObservabilityProvider: Provider {
    /// Record metric value
    async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> SongbirdResult<()>;

    /// Increment counter
    async fn increment_counter(
        &self,
        name: &str,
        labels: HashMap<String, String>,
    ) -> SongbirdResult<()>;

    /// Record histogram value
    async fn record_histogram(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> SongbirdResult<()>;

    /// Start trace span
    async fn start_span(
        &self,
        name: &str,
        parent: Option<SpanContext>,
    ) -> SongbirdResult<SpanContext>;

    /// End trace span
    async fn end_span(&self, span: SpanContext) -> SongbirdResult<()>;

    /// Query metrics
    async fn query_metrics(&self, query: MetricQuery) -> SongbirdResult<Vec<MetricResult>>;

    /// Get system health
    async fn system_health(&self) -> SongbirdResult<SystemHealth>;
}

#[cfg(test)]
#[path = "../canonical_tests.rs"]
mod tests;
