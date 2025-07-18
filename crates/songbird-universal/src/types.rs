//! Universal types for ecosystem integration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// uuid re-exported by other modules

/// Universal primal types - extensible for future primals
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    #[serde(rename = "toadstool")]
    ToadStool,
    #[serde(rename = "songbird")]
    Songbird,
    #[serde(rename = "beardog")]
    BearDog,
    #[serde(rename = "nestgate")]
    NestGate,
    #[serde(rename = "squirrel")]
    Squirrel,
    #[serde(rename = "biomeos")]
    BiomeOS,
    /// Future primals can be added here
    #[serde(untagged)]
    Unknown(String),
}

impl PrimalType {
    pub fn as_str(&self) -> &str {
        match self {
            PrimalType::ToadStool => "toadstool",
            PrimalType::Songbird => "songbird",
            PrimalType::BearDog => "beardog",
            PrimalType::NestGate => "nestgate",
            PrimalType::Squirrel => "squirrel",
            PrimalType::BiomeOS => "biomeos",
            PrimalType::Unknown(name) => name,
        }
    }

    pub fn from_string(s: &str) -> Self {
        s.parse()
            .unwrap_or_else(|_| PrimalType::Unknown(s.to_string()))
    }
}

impl std::str::FromStr for PrimalType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "toadstool" => PrimalType::ToadStool,
            "songbird" => PrimalType::Songbird,
            "beardog" => PrimalType::BearDog,
            "nestgate" => PrimalType::NestGate,
            "squirrel" => PrimalType::Squirrel,
            "biomeos" => PrimalType::BiomeOS,
            other => PrimalType::Unknown(other.to_string()),
        })
    }
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Universal service identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIdentification {
    pub name: String,
    pub version: String,
    pub description: String,
    pub primal_type: PrimalType,
    pub instance_id: String,
}

/// Universal service endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub url: String,
    pub protocol: String,
    pub health_check_path: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Universal resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub disk_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<u64>,
    pub gpu_count: Option<u32>,
    pub custom_resources: HashMap<String, serde_json::Value>,
}

/// Universal security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub auth_required: bool,
    pub auth_methods: Vec<String>,
    pub encryption_required: bool,
    pub security_level: SecurityLevel,
    pub custom_security: HashMap<String, serde_json::Value>,
}

/// Security levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Public,
    Internal,
    Restricted,
    Confidential,
}

/// Universal health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub custom_checks: HashMap<String, serde_json::Value>,
}

/// Universal health status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl PartialOrd for HealthStatus {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HealthStatus {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_priority = match self {
            HealthStatus::Healthy => 3,
            HealthStatus::Degraded => 2,
            HealthStatus::Unhealthy => 1,
            HealthStatus::Unknown => 0,
        };
        let other_priority = match other {
            HealthStatus::Healthy => 3,
            HealthStatus::Degraded => 2,
            HealthStatus::Unhealthy => 1,
            HealthStatus::Unknown => 0,
        };
        self_priority.cmp(&other_priority)
    }
}

/// Universal service health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub response_time: std::time::Duration,
    pub error: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
}

/// Universal service metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub performance_score: f64,
    pub average_latency_ms: f64,
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub uptime_percentage: f64,
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self {
            performance_score: 0.0,
            average_latency_ms: 0.0,
            requests_per_second: 0.0,
            error_rate: 0.0,
            uptime_percentage: 100.0,
            custom_metrics: HashMap::new(),
        }
    }
}

/// Universal service status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Registered,
    Active,
    Inactive,
    Deregistered,
    Failed,
}

/// Universal retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Universal circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout_ms: u64,
    pub test_request_count: u32,
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_ms: 60000,
            test_request_count: 3,
            success_threshold: 2,
        }
    }
}

/// Universal load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub strategy: String,
    pub health_check_enabled: bool,
    pub sticky_sessions: bool,
    pub custom_config: HashMap<String, serde_json::Value>,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: "capability_based".to_string(),
            health_check_enabled: true,
            sticky_sessions: false,
            custom_config: HashMap::new(),
        }
    }
}

/// Universal authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Token,
    Jwt,
    Oauth2,
    BearDog,
    Custom(String),
}

/// Universal feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub development_mode: bool,
    pub debug_logging: bool,
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub experimental_features: Vec<String>,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            development_mode: false,
            debug_logging: false,
            metrics_enabled: true,
            tracing_enabled: true,
            experimental_features: Vec::new(),
        }
    }
}
