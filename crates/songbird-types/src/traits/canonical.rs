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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;

// Re-export core error types
pub use crate::errors::{SongbirdError, SongbirdResult};

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

// ============================================================================
// CANONICAL TYPE DEFINITIONS
// ============================================================================

/// Provider type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderType {
    Service,
    Primal,
    Discovery,
    Capability,
    Security,
    Orchestration,
    Observability,
    Custom(String),
}

/// Primal type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PrimalType {
    Security,
    Storage,
    Compute,
    AI,
    Network,
    Custom(String),
}

/// Service type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceType {
    WebService,
    Database,
    MessageQueue,
    Cache,
    FileStorage,
    Authentication,
    Custom(String),
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub settings: HashMap<String, serde_json::Value>,
    pub enabled_features: Vec<String>,
    pub environment: String,
}

/// Provider metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub description: String,
    pub tags: Vec<String>,
    pub documentation_url: Option<String>,
    pub support_contact: Option<String>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub version: String,
    pub description: String,
    pub parameters: HashMap<String, ParameterSpec>,
}

/// Parameter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub parameter_type: String,
    pub required: bool,
    pub description: String,
    pub default_value: Option<serde_json::Value>,
}

/// Service request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
    pub timestamp: SystemTime,
}

/// Service response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub id: String,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
    pub timestamp: SystemTime,
}

/// Service metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub request_count: u64,
    pub error_count: u64,
    pub average_response_time_ms: f64,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub service_type: ServiceType,
    pub version: String,
    pub endpoints: Vec<Endpoint>,
    pub health: HealthStatus,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
    pub capabilities: Vec<String>,
    pub last_updated: SystemTime,
}

/// Network endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub path: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Primal context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalContext {
    pub user_id: String,
    pub device_id: String,
    pub environment: String,
    pub security_level: String,
    pub metadata: HashMap<String, String>,
}

/// Primal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub execution_time_ms: u64,
}

/// Primal dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDependency {
    pub service_name: String,
    pub required_version: String,
    pub optional: bool,
    pub capabilities: Vec<String>,
}

/// Integration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResult {
    pub success: bool,
    pub shared_capabilities: Vec<String>,
    pub communication_channels: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    pub id: String,
    pub name: String,
    pub primal_type: PrimalType,
    pub version: String,
    pub capabilities: Vec<String>,
    pub endpoints: Vec<Endpoint>,
    pub health: HealthStatus,
    pub metadata: HashMap<String, String>,
}

/// Discovery criteria
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryCriteria {
    pub name: Option<String>,
    pub service_type: Option<ServiceType>,
    pub version: Option<String>,
    pub tags: Vec<String>,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub health_status: Option<HealthStatus>,
    pub limit: Option<usize>,
}

/// Discovery query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryQuery {
    pub criteria: DiscoveryCriteria,
    pub watch_changes: bool,
    pub include_metadata: bool,
}

/// Service event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    Registered(ServiceInfo),
    Updated(ServiceInfo),
    Unregistered {
        service_id: String,
    },
    HealthChanged {
        service_id: String,
        health: HealthStatus,
    },
}

/// Capability metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub parameters: HashMap<String, ParameterSpec>,
    pub examples: Vec<serde_json::Value>,
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub credential_type: String,
    pub data: HashMap<String, serde_json::Value>,
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub token_type: String,
    pub expires_at: Option<SystemTime>,
    pub metadata: HashMap<String, String>,
}

/// Token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub subject: String,
    pub audience: Vec<String>,
    pub expires_at: Option<SystemTime>,
    pub custom_claims: HashMap<String, serde_json::Value>,
}

/// Token validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidation {
    pub valid: bool,
    pub claims: Option<TokenClaims>,
    pub error: Option<String>,
}

/// Deployment specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSpec {
    pub name: String,
    pub image: String,
    pub replicas: u32,
    pub resources: ResourceRequirements,
    pub environment: HashMap<String, String>,
    pub ports: Vec<PortSpec>,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub cpu_request: Option<String>,
    pub memory_request: Option<String>,
}

/// Port specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    pub name: String,
    pub port: u16,
    pub target_port: u16,
    pub protocol: String,
}

/// Deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub deployment_id: String,
    pub status: DeploymentStatus,
    pub endpoints: Vec<Endpoint>,
    pub message: String,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Running,
    Failed,
    Terminated,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub id: String,
    pub name: String,
    pub status: DeploymentStatus,
    pub replicas: u32,
    pub ready_replicas: u32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Span context for tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub baggage: HashMap<String, String>,
}

/// Metric query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricQuery {
    pub metric_name: String,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub labels: HashMap<String, String>,
    pub aggregation: Option<String>,
}

/// Metric result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    pub metric_name: String,
    pub timestamp: SystemTime,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// System health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub components: HashMap<String, HealthStatus>,
    pub metrics: HashMap<String, f64>,
    pub last_check: SystemTime,
}
