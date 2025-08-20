use songbird_config::unified::*;
/// Configuration types for health monitoring system
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Universal health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate UniversalHealthConfig to songbird_config::unified
pub struct UniversalHealthConfig {
    /// Health check interval
    pub check_interval: Duration,

    /// Request timeout for health checks
    pub request_timeout: Duration,

    /// Maximum number of retries for failed checks
    pub max_retries: u32,

    /// Alert thresholds configuration
    pub alert_thresholds: AlertThresholds,

    /// Whether to enable performance metrics collection
    pub enable_metrics: bool,

    /// Whether to enable historical data collection
    pub enable_history: bool,

    /// Maximum number of historical snapshots to keep
    pub max_history_snapshots: usize,

    /// Service-specific health check configurations
    pub service_configs: HashMap<String, HealthCheckConfig>,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Response time threshold for warnings (ms)
    pub response_time_warning_ms: u64,

    /// Response time threshold for critical alerts (ms)
    pub response_time_critical_ms: u64,

    /// Error rate threshold for warnings (0.0 to 1.0)
    pub error_rate_warning: f64,

    /// Error rate threshold for critical alerts (0.0 to 1.0)
    pub error_rate_critical: f64,
}

/// Health check configuration for a specific service
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate HealthCheckConfig to songbird_config::unified
pub struct HealthCheckConfig {
    /// Whether health checks are enabled for this service
    pub enabled: bool,

    /// Health check interval (overrides global setting)
    pub check_interval: Option<Duration>,

    /// Request timeout (overrides global setting)
    pub timeout: Option<Duration>,

    /// Maximum retries (overrides global setting)
    pub max_retries: Option<u32>,

    /// Health endpoints to check
    pub endpoints: Vec<crate::health::types::HealthEndpoint>,

    /// Expected response patterns
    pub expected_patterns: Vec<String>,

    /// Custom headers for health check requests
    pub custom_headers: HashMap<String, String>,

    /// Whether to follow redirects
    pub follow_redirects: bool,

    /// Custom alert thresholds for this service
    pub alert_thresholds: Option<AlertThresholds>,
}

impl Default for UniversalHealthConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            request_timeout: Duration::from_secs(10),
            max_retries: 3,
            alert_thresholds: AlertThresholds::default(),
            enable_metrics: true,
            enable_history: true,
            max_history_snapshots: 100,
            service_configs: HashMap::new(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            response_time_warning_ms: std::env::var("SONGBIRD_HEALTH_WARNING_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            response_time_critical_ms: std::env::var("SONGBIRD_HEALTH_CRITICAL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5000),
            error_rate_warning: std::env::var("SONGBIRD_ERROR_RATE_WARNING")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.05),  // 5%
            error_rate_critical: std::env::var("SONGBIRD_ERROR_RATE_CRITICAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.10), // 10%
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: None,
            timeout: None,
            max_retries: None,
            endpoints: vec![crate::health::types::HealthEndpoint {
                url: "/health".to_string(),
                method: "GET".to_string(),
                expected_status: 200,
                timeout_ms: std::env::var("SONGBIRD_HEALTH_ENDPOINT_TIMEOUT_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5000),
                headers: HashMap::new(),
                expected_body_pattern: None,
            }],
            expected_patterns: vec!["ok".to_string(), "healthy".to_string()],
            custom_headers: HashMap::new(),
            follow_redirects: true,
            alert_thresholds: None,
        }
    }
}
