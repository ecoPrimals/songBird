//! Universal Types for Capability-Based Primal Integration
//!
//! This module provides comprehensive type definitions for the universal
//! capability adapter system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Universal primal type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PrimalType {
    /// Primary category of the primal (e.g., "ai", "storage", "compute")
    pub category: String,
    /// Optional subcategory for finer classification
    pub subcategory: Option<String>,
    /// Version of the primal type specification
    pub version: String,
}

impl PrimalType {
    /// Creates a new `PrimalType` with the given category
    #[must_use]
    pub fn new(category: &str) -> Self {
        Self {
            category: category.to_string(),
            subcategory: None,
            version: "1.0".to_string(),
        }
    }

    /// Create from string (for backward compatibility)
    #[must_use]
    pub fn from_string(category: &str) -> Self {
        Self::new(category)
    }

    /// Returns the category as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.category
    }
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.category)
    }
}

/// Security level classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    /// No security requirements
    None,
    /// Basic security (authentication only)
    Basic,
    /// Standard security (authentication + encryption)
    #[default]
    Standard,
    /// High security (standard + authorization)
    High,
    /// Maximum security (all features + audit logging)
    Maximum,
}

/// Primal capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCapability {
    /// Type of capability (e.g., "inference", "storage", "compute")
    pub capability_type: String,
    /// Version of the capability specification
    pub version: String,
    /// Capability-specific configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Quality of service metrics for this capability
    pub qos_metrics: QosMetrics,
}

/// Quality of Service metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QosMetrics {
    /// Average latency in milliseconds
    pub latency_ms: Option<f64>,
    /// Throughput in operations per second
    pub throughput_ops_sec: Option<f64>,
    /// Availability percentage (0.0 to 1.0)
    pub availability: Option<f64>,
    /// Reliability score (0.0 to 1.0)
    pub reliability: Option<f64>,
}

/// Discovered capability with deployment information
///
/// This type represents a capability that has been discovered from a primal service,
/// including its deployment details (endpoint, provider, health status).
///
/// **Note**: For capability definitions and specifications, use
/// `crate::capabilities::Capability` instead. This type is specifically for
/// representing capabilities that have been discovered and are ready for use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredCapability {
    /// Name of the capability
    pub name: String,
    /// Version of the capability
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Provider identifier
    pub provider: String,
    /// Network endpoint for accessing the capability
    pub endpoint: String,
    /// Quality of service metrics
    pub qos_metrics: QosMetrics,
    /// Current health status
    pub health_status: HealthStatus,
}

/// Type alias for backward compatibility during migration
///
/// **Deprecated**: Use `DiscoveredCapability` instead to distinguish from
/// `capabilities::Capability` (the capability definition type).
#[deprecated(
    since = "0.1.0",
    note = "Use DiscoveredCapability for discovered capabilities with deployment info, or capabilities::Capability for capability definitions"
)]
pub type Capability = DiscoveredCapability;

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is operational but with reduced performance
    Degraded,
    /// Service is not operational
    Unhealthy,
    /// Health status is unknown or not yet determined
    #[default]
    Unknown,
}

/// Discovery filters for primal search
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryFilters {
    /// Filter by specific capability types
    pub capability_types: Vec<String>,
    /// Filter by security level requirements
    pub security_levels: Vec<SecurityLevel>,
    /// Filter by geographic regions
    pub geographic_regions: Vec<String>,
    /// Filter by performance requirements
    pub performance_requirements: Option<QosMetrics>,
}

/// Service information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Name of the service
    pub name: String,
    /// Type of primal this service represents
    pub primal_type: PrimalType,
    /// Network endpoint for accessing the service
    pub endpoint: String,
    /// List of capabilities provided by this service
    pub capabilities: Vec<DiscoveredCapability>,
    /// Current health status of the service
    pub health: HealthStatus,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Service event for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    /// Name of the service that generated the event
    pub service_name: String,
    /// Type of event (e.g., "started", "stopped", "error")
    pub event_type: String,
    /// Timestamp when the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional event details
    pub details: HashMap<String, serde_json::Value>,
}

/// Registered service in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService {
    /// Unique identifier for this service registration
    pub id: String,
    /// Service information and capabilities
    pub service_info: ServiceInfo,
    /// Timestamp when the service was registered
    pub registration_time: chrono::DateTime<chrono::Utc>,
    /// Timestamp of the last heartbeat received
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether security features are enabled
    pub enabled: bool,
    /// Security level to enforce
    pub level: SecurityLevel,
    /// Whether authentication is required for all operations
    pub authentication_required: bool,
    /// Whether TLS encryption is enabled
    pub tls_enabled: bool,
    /// Path to TLS certificate file
    pub certificate_path: Option<String>,
}

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Optional user identifier
    pub user_id: Option<String>,
    /// Session identifier
    pub session_id: String,
    /// List of granted permissions
    pub permissions: Vec<String>,
    /// Security level of this context
    pub security_level: SecurityLevel,
}

/// Universal event for system-wide communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalEvent {
    /// Unique identifier for this event
    pub event_id: String,
    /// Type of event
    pub event_type: String,
    /// Source system or service that generated the event
    pub source: String,
    /// Timestamp when the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event payload data
    pub payload: serde_json::Value,
}

/// Universal request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    /// Unique identifier for this request
    pub request_id: String,
    /// Source system or service making the request
    pub source: String,
    /// Target system or service for the request
    pub target: String,
    /// Action to be performed
    pub action: String,
    /// Request parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Optional security context for authorization
    pub security_context: Option<SecurityContext>,
}

/// Universal response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    /// Request identifier this response corresponds to
    pub request_id: String,
    /// Status of the response
    pub status: ResponseStatus,
    /// Optional response data
    pub data: Option<serde_json::Value>,
    /// Optional error message
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Response status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ResponseStatus {
    /// Request completed successfully
    #[default]
    Success,
    /// Request partially completed
    Partial,
    /// Request failed
    Failed,
    /// Request timed out
    Timeout,
    /// Requested resource not found
    NotFound,
}

/// Protocol characteristics for communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCharacteristics {
    /// Name of the protocol (e.g., "HTTP", "gRPC", "WebSocket")
    pub protocol_name: String,
    /// Protocol version
    pub version: String,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Whether the protocol supports streaming
    pub supports_streaming: bool,
    /// List of supported security features
    pub security_features: Vec<String>,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing strategy to use
    pub strategy: LoadBalancingStrategy,
    /// Interval between health checks
    pub health_check_interval: Duration,
    /// Maximum number of retries before marking service unhealthy
    pub max_retries: u32,
    /// Timeout for individual requests
    pub timeout: Duration,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::RoundRobin,
            health_check_interval: Duration::from_secs(30),
            max_retries: 3,
            timeout: Duration::from_secs(10),
        }
    }
}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    #[default]
    RoundRobin,
    /// Weighted round-robin based on capacity
    WeightedRoundRobin,
    /// Route to service with fewest active connections
    LeastConnections,
    /// Route based on health scores
    HealthBased,
    /// Random selection
    Random,
}

/// Service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Current health status
    pub status: HealthStatus,
    /// Timestamp of last health check
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Average response time in milliseconds
    pub response_time_ms: Option<f64>,
    /// Error rate as a percentage (0.0 to 1.0)
    pub error_rate: Option<f64>,
    /// Additional health check details
    pub details: HashMap<String, serde_json::Value>,
}

/// Error types for various subsystems
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// Network communication error
    #[error("Network error: {0}")]
    Network(String),
    /// Data parsing error
    #[error("Parse error: {0}")]
    Parse(String),
    /// Operation timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Load balancing error types
#[derive(Debug, thiserror::Error)]
pub enum LoadBalancingError {
    /// No healthy service instances available
    #[error("No healthy instances available")]
    NoHealthyInstances,
    /// Invalid load balancer configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    /// Service is unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Registry error types
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// Service not found in registry
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    /// Service registration failed
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
    /// Duplicate service registration attempted
    #[error("Duplicate service: {0}")]
    DuplicateService(String),
}

/// Protocol error types
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    /// Protocol mismatch between client and server
    #[error("Protocol mismatch: {0}")]
    ProtocolMismatch(String),
    /// Incompatible protocol versions
    #[error("Version incompatible: {0}")]
    VersionIncompatible(String),
    /// Message format error
    #[error("Message format error: {0}")]
    MessageFormat(String),
}

/// Service error types
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// Service is unavailable
    #[error("Service unavailable: {0}")]
    Unavailable(String),
    /// Service is overloaded
    #[error("Service overloaded: {0}")]
    Overloaded(String),
    /// Invalid request to service
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

/// Security error types
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    /// Authorization denied
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),
    /// Security policy violation
    #[error("Security violation: {0}")]
    SecurityViolation(String),
}

/// Metrics error types
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// Metric collection failed
    #[error("Metric collection failed: {0}")]
    CollectionFailed(String),
    /// Invalid metric data or configuration
    #[error("Invalid metric: {0}")]
    InvalidMetric(String),
    /// Metric storage error
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Event processing error types
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    /// Event processing failed
    #[error("Event processing failed: {0}")]
    ProcessingFailed(String),
    /// Invalid event format
    #[error("Invalid event format: {0}")]
    InvalidFormat(String),
    /// Event delivery failed
    #[error("Event delivery failed: {0}")]
    DeliveryFailed(String),
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Invalid configuration
    #[error("Configuration invalid: {0}")]
    Invalid(String),
    /// Configuration not found
    #[error("Configuration not found: {0}")]
    NotFound(String),
    /// Configuration parsing error
    #[error("Configuration parse error: {0}")]
    ParseError(String),
}

/// Capability requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    /// Type of capability required
    pub capability_type: String,
    /// Minimum version required
    pub minimum_version: String,
    /// Optional `QoS` requirements
    pub required_qos: Option<QosMetrics>,
    /// Required security level
    pub security_level: SecurityLevel,
}

/// Service capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability {
    /// Type of capability provided
    pub capability_type: String,
    /// Capability version
    pub version: String,
    /// Endpoint path for this capability
    pub endpoint_path: String,
    /// List of supported operations
    pub supported_operations: Vec<String>,
    /// `QoS` guarantees for this capability
    pub qos_guarantees: QosMetrics,
}

impl Default for PrimalType {
    fn default() -> Self {
        Self::new("universal")
    }
}

/// Retry configuration for resilient operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Number of successes needed to close the circuit
    pub success_threshold: u32,
    /// Timeout duration for circuit breaker
    pub timeout: Duration,
    /// Maximum calls allowed in half-open state
    pub half_open_max_calls: u32,
}

/// Service identification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIdentification {
    /// Unique service identifier
    pub service_id: String,
    /// Human-readable service name
    pub service_name: String,
    /// Service version
    pub version: String,
    /// Unique instance identifier
    pub instance_id: String,
}

/// Service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Full URL of the service endpoint
    pub url: String,
    /// Protocol used (http, https, grpc, etc.)
    pub protocol: String,
    /// Port number
    pub port: u16,
    /// Optional path component
    pub path: Option<String>,
    /// Whether TLS/SSL is enabled
    pub tls_enabled: bool,
}

impl Default for ServiceEndpoint {
    fn default() -> Self {
        let default_host =
            std::env::var("DEFAULT_SERVICE_HOST").unwrap_or_else(|_| "localhost".to_string());
        let default_port = std::env::var("DEFAULT_SERVICE_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);

        Self {
            url: format!("http://{default_host}:{default_port}"),
            protocol: "http".to_string(),
            port: default_port,
            path: None,
            tls_enabled: false,
        }
    }
}

/// Resource specification for services
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceSpec {
    /// Number of CPU cores required
    pub cpu_cores: Option<u32>,
    /// Memory required in megabytes
    pub memory_mb: Option<u64>,
    /// Disk space required in gigabytes
    pub disk_gb: Option<u64>,
    /// Network bandwidth required in Mbps
    pub network_bandwidth_mbps: Option<u32>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Whether health checks are enabled
    pub enabled: bool,
    /// Health check endpoint URL
    pub endpoint: String,
    /// Interval between health checks
    pub interval: Duration,
    /// Timeout for each health check
    pub timeout: Duration,
    /// Number of successful checks before marking healthy
    pub healthy_threshold: u32,
    /// Number of failed checks before marking unhealthy
    pub unhealthy_threshold: u32,
}

/// Feature flags for capability control
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlags {
    /// Map of feature names to their enabled status
    pub features: HashMap<String, bool>,
    /// Map of features to their rollout percentage (0.0-100.0)
    pub rollout_percentage: HashMap<String, f64>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 1,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/health".to_string(),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            healthy_threshold: 2,
            unhealthy_threshold: 3,
        }
    }
}
