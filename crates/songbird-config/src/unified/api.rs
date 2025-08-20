//! API Configuration Module
//!
//! Consolidates all API-related configuration structs into a unified hierarchy

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Unified API configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    /// Session configuration for real-time AI streaming
    #[serde(default)]
    pub session: SessionConfig,

    /// Connection configuration for API connections
    #[serde(default)]
    pub connection: ConnectionConfig,

    /// AI mesh configuration
    #[serde(default)]
    pub mesh: MeshConfig,

    /// Universal service registration configuration
    #[serde(default)]
    pub service_registration: ServiceRegistrationConfig,
}

/// Session configuration (consolidated from `SessionConfiguration`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum concurrent sessions
    pub max_concurrent_sessions: usize,

    /// Session timeout duration
    pub session_timeout: Duration,

    /// Keep-alive interval
    pub keep_alive_interval: Duration,

    /// Buffer size for streaming
    pub buffer_size: usize,

    /// Enable session persistence
    pub enable_persistence: bool,

    /// Session cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: 1000,
            session_timeout: Duration::from_secs(300),
            keep_alive_interval: Duration::from_secs(30),
            buffer_size: 8192,
            enable_persistence: true,
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

/// Connection configuration (consolidated from `ConnectionConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Maximum connections per client
    pub max_connections_per_client: usize,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Read timeout
    pub read_timeout: Duration,

    /// Write timeout
    pub write_timeout: Duration,

    /// Enable connection pooling
    pub enable_pooling: bool,

    /// Pool size
    pub pool_size: usize,

    /// Pool timeout
    pub pool_timeout: Duration,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            max_connections_per_client: 10,
            connection_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(60),
            write_timeout: Duration::from_secs(60),
            enable_pooling: true,
            pool_size: 100,
            pool_timeout: Duration::from_secs(30),
        }
    }
}

/// AI Mesh configuration (consolidated from `MeshConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    /// Health monitoring configuration
    pub health_monitoring: HealthMonitoringConfig,

    /// Performance analysis configuration
    pub performance_analysis: PerformanceAnalysisConfig,

    /// Enable mesh networking
    pub enable_mesh: bool,

    /// Mesh discovery interval
    pub discovery_interval: Duration,

    /// Maximum mesh nodes
    pub max_nodes: usize,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            health_monitoring: HealthMonitoringConfig::default(),
            performance_analysis: PerformanceAnalysisConfig::default(),
            enable_mesh: true,
            discovery_interval: Duration::from_secs(30),
            max_nodes: 100,
        }
    }
}

/// Health monitoring configuration (consolidated from `HealthMonitoringConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitoringConfig {
    /// Health check interval
    pub check_interval: Duration,

    /// Health check timeout
    pub check_timeout: Duration,

    /// Failure threshold before marking unhealthy
    pub failure_threshold: u32,

    /// Recovery threshold before marking healthy
    pub recovery_threshold: u32,

    /// Enable detailed health metrics
    pub enable_detailed_metrics: bool,
}

impl Default for HealthMonitoringConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_threshold: 2,
            enable_detailed_metrics: true,
        }
    }
}

/// Performance analysis configuration (consolidated from `PerformanceAnalysisConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisConfig {
    /// Enable performance monitoring
    pub enabled: bool,

    /// Metrics collection interval
    pub metrics_interval: Duration,

    /// Performance window size for analysis
    pub analysis_window: Duration,

    /// CPU threshold for alerts
    pub cpu_threshold: f64,

    /// Memory threshold for alerts
    pub memory_threshold: f64,

    /// Network latency threshold
    pub latency_threshold: Duration,
}

impl Default for PerformanceAnalysisConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_interval: Duration::from_secs(10),
            analysis_window: Duration::from_secs(300),
            cpu_threshold: 80.0,
            memory_threshold: 85.0,
            latency_threshold: Duration::from_millis(100),
        }
    }
}

/// Service registration configuration (consolidated from `ServiceRegistrationConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistrationConfig {
    /// Health check configuration
    pub health_check: HealthCheckConfiguration,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Monitoring configuration
    pub monitoring: MonitoringConfiguration,

    /// Auto-registration enabled
    pub auto_registration: bool,

    /// Registration timeout
    pub registration_timeout: Duration,
}

impl Default for ServiceRegistrationConfig {
    fn default() -> Self {
        Self {
            health_check: HealthCheckConfiguration::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            monitoring: MonitoringConfiguration::default(),
            auto_registration: true,
            registration_timeout: Duration::from_secs(30),
        }
    }
}

/// Health check configuration (consolidated from `HealthCheckConfiguration`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfiguration {
    /// Health check endpoint path
    pub endpoint: String,

    /// Check interval
    pub interval: Duration,

    /// Check timeout
    pub timeout: Duration,

    /// Enable health checks
    pub enabled: bool,

    /// Failure threshold
    pub failure_threshold: u32,
}

impl Default for HealthCheckConfiguration {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            enabled: true,
            failure_threshold: 3,
        }
    }
}

/// Circuit breaker configuration (consolidated from `CircuitBreakerConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,

    /// Timeout before attempting recovery
    pub timeout: Duration,

    /// Recovery timeout
    pub recovery_timeout: Duration,

    /// Enable circuit breaker
    pub enabled: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(30),
            enabled: true,
        }
    }
}

/// Monitoring configuration (consolidated from `MonitoringConfiguration`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfiguration {
    /// Enable monitoring
    pub enabled: bool,

    /// Metrics collection interval
    pub metrics_interval: Duration,

    /// Log level
    pub log_level: String,

    /// Enable tracing
    pub enable_tracing: bool,
}

impl Default for MonitoringConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_interval: Duration::from_secs(60),
            log_level: "info".to_string(),
            enable_tracing: true,
        }
    }
}
