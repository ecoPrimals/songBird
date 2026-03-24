// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical provider data types shared by the [`canonical`](crate::traits::canonical) traits module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

// ============================================================================
// CANONICAL TYPE DEFINITIONS
// ============================================================================

/// Classify a provider implementation for routing and policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderType {
    /// Exposes HTTP or RPC-style services.
    Service,
    /// Runs a Songbird primal workload.
    Primal,
    /// Participates in service discovery.
    Discovery,
    /// Advertises or resolves capabilities.
    Capability,
    /// Handles authn/authz and secrets.
    Security,
    /// Coordinates deployments and lifecycle.
    Orchestration,
    /// Emits metrics, logs, and traces.
    Observability,
    /// Extension point for custom provider kinds.
    Custom(String),
}

/// Identify which primal domain an instance belongs to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PrimalType {
    /// Security and policy primal.
    Security,
    /// Storage primal.
    Storage,
    /// Compute primal.
    Compute,
    /// AI/ML primal.
    AI,
    /// Network primal.
    Network,
    /// Custom primal type label.
    Custom(String),
}

/// Describe the role of a discovered or registered service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceType {
    /// Web or HTTP API tier.
    WebService,
    /// Database or datastore.
    Database,
    /// Message broker or queue.
    MessageQueue,
    /// In-memory or distributed cache.
    Cache,
    /// Object or file storage.
    FileStorage,
    /// Identity and authentication.
    Authentication,
    /// Custom service classification.
    Custom(String),
}

/// Pass startup settings and feature toggles into a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Opaque JSON settings keyed by provider-specific names.
    pub settings: HashMap<String, serde_json::Value>,
    /// Feature flags enabled for this provider instance.
    pub enabled_features: Vec<String>,
    /// Deployment environment label (for example `production`).
    pub environment: String,
}

/// Describe a provider for dashboards and support tooling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Short human-readable summary.
    pub description: String,
    /// Searchable labels for discovery UIs.
    pub tags: Vec<String>,
    /// Link to external documentation.
    pub documentation_url: Option<String>,
    /// Escalation contact for operators.
    pub support_contact: Option<String>,
    /// Creation time of this metadata record.
    pub created_at: SystemTime,
    /// Last update time of this metadata record.
    pub updated_at: SystemTime,
}

/// Summarize readiness for load balancers and orchestrators.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Fully operational within SLO.
    Healthy,
    /// Partially degraded but still serving traffic.
    Degraded,
    /// Failing checks; should not receive new work.
    Unhealthy,
    /// Health could not be determined.
    Unknown,
}

/// Advertise a callable capability with typed parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Stable capability name.
    pub name: String,
    /// Semantic version of the capability contract.
    pub version: String,
    /// What callers should expect when invoking this capability.
    pub description: String,
    /// Parameter schemas keyed by parameter name.
    pub parameters: HashMap<String, ParameterSpec>,
}

/// Describe one parameter accepted by a capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    /// Logical type name (JSON schema id or shorthand).
    pub parameter_type: String,
    /// Whether callers must supply this parameter.
    pub required: bool,
    /// Human-readable parameter description.
    pub description: String,
    /// Default used when the parameter is omitted.
    pub default_value: Option<serde_json::Value>,
}

/// Carry one inbound service invocation across provider boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// Correlation id for logs and tracing.
    pub id: String,
    /// HTTP-style method or RPC verb.
    pub method: String,
    /// Resource path or RPC route.
    pub path: String,
    /// Transport headers (including auth).
    pub headers: HashMap<String, String>,
    /// JSON payload body.
    pub body: serde_json::Value,
    /// When the request was accepted.
    pub timestamp: SystemTime,
}

/// Return status, headers, and body for a service invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    /// Matches the request correlation id.
    pub id: String,
    /// HTTP-style status code.
    pub status_code: u16,
    /// Outbound headers.
    pub headers: HashMap<String, String>,
    /// JSON response body.
    pub body: serde_json::Value,
    /// When the response was produced.
    pub timestamp: SystemTime,
}

/// Aggregate runtime stats for a single service instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Total handled requests since start.
    pub request_count: u64,
    /// Total failed requests since start.
    pub error_count: u64,
    /// Rolling average latency in milliseconds.
    pub average_response_time_ms: f64,
    /// Seconds the instance has been running.
    pub uptime_seconds: u64,
    /// Resident memory usage in megabytes.
    pub memory_usage_mb: f64,
    /// Recent CPU utilization percentage.
    pub cpu_usage_percent: f64,
}

/// Identify and locate a registered service for discovery clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Unique service id in the registry.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Classify the service role.
    pub service_type: ServiceType,
    /// Deployed version string.
    pub version: String,
    /// Reachable network endpoints.
    pub endpoints: Vec<Endpoint>,
    /// Latest health signal.
    pub health: HealthStatus,
    /// Arbitrary key/value metadata for UIs and routing.
    pub metadata: HashMap<String, String>,
    /// Optional grouping or cost tags.
    pub tags: Vec<String>,
    /// Capability names this instance implements.
    pub capabilities: Vec<String>,
    /// When this record was last refreshed.
    pub last_updated: SystemTime,
}

/// Describe how to dial a single network endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// Scheme or transport label (for example `https`).
    pub protocol: String,
    /// Hostname or IP.
    pub host: String,
    /// TCP or UDP port.
    pub port: u16,
    /// Optional HTTP path or RPC subpath.
    pub path: Option<String>,
    /// Extra routing hints (TLS SNI, region, etc.).
    pub metadata: HashMap<String, String>,
}

/// Pass caller and device context into primal execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalContext {
    /// Authenticated user id when available.
    pub user_id: String,
    /// Stable device identifier.
    pub device_id: String,
    /// Deployment environment label.
    pub environment: String,
    /// Coarse security tier for policy checks.
    pub security_level: String,
    /// Free-form context for auditing.
    pub metadata: HashMap<String, String>,
}

/// Return structured output from a primal capability run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Whether the capability succeeded.
    pub success: bool,
    /// Primary JSON payload.
    pub data: serde_json::Value,
    /// Secondary metadata for clients and logs.
    pub metadata: HashMap<String, String>,
    /// Wall-clock execution time in milliseconds.
    pub execution_time_ms: u64,
}

/// Declare another service or primal this primal relies on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Logical name of the dependency.
    pub service_name: String,
    /// Minimum compatible version range description.
    pub required_version: String,
    /// Whether startup may proceed if the dependency is absent.
    pub optional: bool,
    /// Capabilities required from the dependency.
    pub capabilities: Vec<String>,
}

/// Report outcome when two primals link at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResult {
    /// Whether integration completed successfully.
    pub success: bool,
    /// Capabilities now shared across the link.
    pub shared_capabilities: Vec<String>,
    /// Established channels (socket paths, topics, etc.).
    pub communication_channels: Vec<String>,
    /// Diagnostic metadata for operators.
    pub metadata: HashMap<String, String>,
}

/// Summarize a primal for discovery and federation views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Stable primal instance id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Domain classification.
    pub primal_type: PrimalType,
    /// Deployed version.
    pub version: String,
    /// Advertised capability names.
    pub capabilities: Vec<String>,
    /// Network endpoints for this primal.
    pub endpoints: Vec<Endpoint>,
    /// Latest health.
    pub health: HealthStatus,
    /// Arbitrary metadata for routing policy.
    pub metadata: HashMap<String, String>,
}

/// Filter services returned by discovery queries.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryCriteria {
    /// Match on service name substring or exact id.
    pub name: Option<String>,
    /// Restrict to a service class.
    pub service_type: Option<ServiceType>,
    /// Require a minimum semantic version.
    pub version: Option<String>,
    /// Match if any tag overlaps.
    pub tags: Vec<String>,
    /// Require all listed capability names.
    pub capabilities: Vec<String>,
    /// Structured filters for advanced matchers.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Restrict to instances at a given health level.
    pub health_status: Option<HealthStatus>,
    /// Cap the number of results.
    pub limit: Option<usize>,
}

/// Configure long-poll or watch-style discovery calls.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryQuery {
    /// Filters to apply.
    pub criteria: DiscoveryCriteria,
    /// Wait for subsequent changes instead of a one-shot list.
    pub watch_changes: bool,
    /// Include full metadata blobs in results.
    pub include_metadata: bool,
}

/// Notify subscribers when registry membership or health changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    /// A new service instance appeared.
    Registered(ServiceInfo),
    /// An existing instance changed materially.
    Updated(ServiceInfo),
    /// A service left the registry.
    Unregistered {
        /// Id of the removed service.
        service_id: String,
    },
    /// Health status changed for a known instance.
    HealthChanged {
        /// Affected service id.
        service_id: String,
        /// New health value.
        health: HealthStatus,
    },
}

/// Richer capability description for catalogs and codegen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMetadata {
    /// Capability name.
    pub name: String,
    /// Long-form description for tooling.
    pub description: String,
    /// Contract version.
    pub version: String,
    /// Parameter specs keyed by name.
    pub parameters: HashMap<String, ParameterSpec>,
    /// Example JSON payloads for documentation.
    pub examples: Vec<serde_json::Value>,
}

/// Hold secrets or API keys for authentication flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Credential kind (password, `api_key`, `oauth_refresh`, etc.).
    pub credential_type: String,
    /// Opaque credential fields; never log verbatim.
    pub data: HashMap<String, serde_json::Value>,
}

/// Represent a bearer or session token returned by an auth provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// Serialized token string.
    pub token: String,
    /// Token profile (Bearer, MAC, etc.).
    pub token_type: String,
    /// Absolute expiry when known.
    pub expires_at: Option<SystemTime>,
    /// Issuer metadata and scopes.
    pub metadata: HashMap<String, String>,
}

/// Standard JWT-style claims used by internal auth adapters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject identifier (user or service id).
    pub subject: String,
    /// Intended recipients or audiences.
    pub audience: Vec<String>,
    /// Absolute expiry when present.
    pub expires_at: Option<SystemTime>,
    /// Extension claims as JSON.
    pub custom_claims: HashMap<String, serde_json::Value>,
}

/// Report whether a token parsed and validated successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidation {
    /// True when signature and time bounds are valid.
    pub valid: bool,
    /// Parsed claims when validation succeeded.
    pub claims: Option<TokenClaims>,
    /// Failure reason when validation failed.
    pub error: Option<String>,
}

/// Describe a workload the orchestrator should run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSpec {
    /// Kubernetes-style deployment name.
    pub name: String,
    /// Container image reference.
    pub image: String,
    /// Desired replica count.
    pub replicas: u32,
    /// CPU and memory bounds.
    pub resources: ResourceRequirements,
    /// Environment variables for the workload.
    pub environment: HashMap<String, String>,
    /// Ports to expose.
    pub ports: Vec<PortSpec>,
}

/// Express CPU and memory requests and limits for schedulers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Hard CPU cap (Kubernetes quantity string).
    pub cpu_limit: Option<String>,
    /// Hard memory cap.
    pub memory_limit: Option<String>,
    /// Guaranteed CPU reservation.
    pub cpu_request: Option<String>,
    /// Guaranteed memory reservation.
    pub memory_request: Option<String>,
}

/// Map a named service port inside a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    /// Port name for service discovery.
    pub name: String,
    /// Port exposed on the Service resource.
    pub port: u16,
    /// Port the container listens on.
    pub target_port: u16,
    /// Transport protocol (TCP/UDP).
    pub protocol: String,
}

/// Return orchestration outcome after applying a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    /// Orchestrator-assigned deployment id.
    pub deployment_id: String,
    /// High-level rollout state.
    pub status: DeploymentStatus,
    /// Endpoints that became reachable.
    pub endpoints: Vec<Endpoint>,
    /// Human-readable status or error text.
    pub message: String,
}

/// Track rollout lifecycle for a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Accepted but not yet scheduled.
    Pending,
    /// At least one replica is ready.
    Running,
    /// Rollout failed; inspect message and events.
    Failed,
    /// Workload has been torn down.
    Terminated,
}

/// Snapshot deployment state for dashboards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    /// Deployment id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Current rollout status.
    pub status: DeploymentStatus,
    /// Desired replica count.
    pub replicas: u32,
    /// Replicas passing readiness checks.
    pub ready_replicas: u32,
    /// Creation timestamp.
    pub created_at: SystemTime,
    /// Last status transition.
    pub updated_at: SystemTime,
}

/// Correlate distributed traces across services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    /// Root trace id.
    pub trace_id: String,
    /// This span id.
    pub span_id: String,
    /// Parent span when not a root span.
    pub parent_span_id: Option<String>,
    /// Cross-service key/value baggage.
    pub baggage: HashMap<String, String>,
}

/// Select a time range and labels when querying metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricQuery {
    /// Metric series name.
    pub metric_name: String,
    /// Inclusive range start.
    pub start_time: SystemTime,
    /// Inclusive range end.
    pub end_time: SystemTime,
    /// Label filters (job, instance, etc.).
    pub labels: HashMap<String, String>,
    /// Optional aggregation function name.
    pub aggregation: Option<String>,
}

/// One sampled metric point returned to callers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    /// Metric series name.
    pub metric_name: String,
    /// Sample timestamp.
    pub timestamp: SystemTime,
    /// Observed value.
    pub value: f64,
    /// Labels attached to this sample.
    pub labels: HashMap<String, String>,
}

/// Roll up component health for status pages and alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Worst-case status across checks.
    pub overall_status: HealthStatus,
    /// Per-component health (database, queue, etc.).
    pub components: HashMap<String, HealthStatus>,
    /// Numeric gauges (latency, error rate) for dashboards.
    pub metrics: HashMap<String, f64>,
    /// When this snapshot was taken.
    pub last_check: SystemTime,
}
