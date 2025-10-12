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
    pub category: String,
    pub subcategory: Option<String>,
    pub version: String,
}

impl PrimalType {
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
    None,
    Basic,
    #[default]
    Standard,
    High,
    Maximum,
}

/// Primal capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCapability {
    pub capability_type: String,
    pub version: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub qos_metrics: QosMetrics,
}

/// Quality of Service metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QosMetrics {
    pub latency_ms: Option<f64>,
    pub throughput_ops_sec: Option<f64>,
    pub availability: Option<f64>,
    pub reliability: Option<f64>,
}

/// Universal capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub version: String,
    pub description: String,
    pub provider: String,
    pub endpoint: String,
    pub qos_metrics: QosMetrics,
    pub health_status: HealthStatus,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    #[default]
    Unknown,
}

/// Discovery filters for primal search
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryFilters {
    pub capability_types: Vec<String>,
    pub security_levels: Vec<SecurityLevel>,
    pub geographic_regions: Vec<String>,
    pub performance_requirements: Option<QosMetrics>,
}

/// Service information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub primal_type: PrimalType,
    pub endpoint: String,
    pub capabilities: Vec<Capability>,
    pub health: HealthStatus,
    pub metadata: HashMap<String, String>,
}

/// Service event for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    pub service_name: String,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

/// Registered service in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService {
    pub id: String,
    pub service_info: ServiceInfo,
    pub registration_time: chrono::DateTime<chrono::Utc>,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub level: SecurityLevel,
    pub authentication_required: bool,
    pub tls_enabled: bool,
    pub certificate_path: Option<String>,
}

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub user_id: Option<String>,
    pub session_id: String,
    pub permissions: Vec<String>,
    pub security_level: SecurityLevel,
}

/// Universal event for system-wide communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalEvent {
    pub event_id: String,
    pub event_type: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: serde_json::Value,
}

/// Universal request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    pub request_id: String,
    pub source: String,
    pub target: String,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub security_context: Option<SecurityContext>,
}

/// Universal response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    pub request_id: String,
    pub status: ResponseStatus,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Response status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ResponseStatus {
    #[default]
    Success,
    Partial,
    Failed,
    Timeout,
    NotFound,
}

/// Protocol characteristics for communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCharacteristics {
    pub protocol_name: String,
    pub version: String,
    pub max_message_size: usize,
    pub supports_streaming: bool,
    pub security_features: Vec<String>,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub strategy: LoadBalancingStrategy,
    pub health_check_interval: Duration,
    pub max_retries: u32,
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
    #[default]
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    HealthBased,
    Random,
}

/// Service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: Option<f64>,
    pub error_rate: Option<f64>,
    pub details: HashMap<String, serde_json::Value>,
}

/// Error types for various subsystems
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

#[derive(Debug, thiserror::Error)]
pub enum LoadBalancingError {
    #[error("No healthy instances available")]
    NoHealthyInstances,
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
    #[error("Duplicate service: {0}")]
    DuplicateService(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Protocol mismatch: {0}")]
    ProtocolMismatch(String),
    #[error("Version incompatible: {0}")]
    VersionIncompatible(String),
    #[error("Message format error: {0}")]
    MessageFormat(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Service unavailable: {0}")]
    Unavailable(String),
    #[error("Service overloaded: {0}")]
    Overloaded(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
}

#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Metric collection failed: {0}")]
    CollectionFailed(String),
    #[error("Invalid metric: {0}")]
    InvalidMetric(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Invalid event format: {0}")]
    InvalidFormat(String),
    #[error("Event delivery failed: {0}")]
    DeliveryFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration invalid: {0}")]
    Invalid(String),
    #[error("Configuration not found: {0}")]
    NotFound(String),
    #[error("Configuration parse error: {0}")]
    ParseError(String),
}

/// Capability requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    pub capability_type: String,
    pub minimum_version: String,
    pub required_qos: Option<QosMetrics>,
    pub security_level: SecurityLevel,
}

/// Service capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability {
    pub capability_type: String,
    pub version: String,
    pub endpoint_path: String,
    pub supported_operations: Vec<String>,
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
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub half_open_max_calls: u32,
}

/// Service identification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIdentification {
    pub service_id: String,
    pub service_name: String,
    pub version: String,
    pub instance_id: String,
}

/// Service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub url: String,
    pub protocol: String,
    pub port: u16,
    pub path: Option<String>,
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
            url: format!("http://{}:{}", default_host, default_port),
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
    pub cpu_cores: Option<u32>,
    pub memory_mb: Option<u64>,
    pub disk_gb: Option<u64>,
    pub network_bandwidth_mbps: Option<u32>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub interval: Duration,
    pub timeout: Duration,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
}

/// Feature flags for capability control
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlags {
    pub features: HashMap<String, bool>,
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
