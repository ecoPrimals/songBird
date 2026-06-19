// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical API /// Configuration capability Configuration
//!
//! This module provides the single, canonical API configuration that consolidates
//! all API-related configuration structs from across the codebase.

use super::health::HealthCheckConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Canonical API configuration - consolidates all API-related configs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalApiConfig {
    /// Session configuration for real-time AI streaming
    /// Session field
    pub session: CanonicalSessionConfig,
    /// Connection configuration for API connections
    /// Connection field
    pub connection: CanonicalConnectionConfig,
    /// AI mesh configuration
    /// Whether mesh networking is supported
    pub mesh: CanonicalMeshConfig,
    /// Service registration configuration
    /// Service Registration field
    pub service_registration: CanonicalServiceRegistrationConfig,
}

/// Canonical session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSessionConfig {
    /// Maximum concurrent sessions
    /// Max Concurrent Sessions field
    pub max_concurrent_sessions: usize,

    /// Session timeout duration
    /// Session Timeout field
    pub session_timeout: Duration,
    /// Keep-alive interval
    /// Keep Alive Interval field
    pub keep_alive_interval: Duration,
    /// Buffer size for streaming
    pub buffer_size: usize,

    /// Enable session persistence
    /// Enable Persistence field
    pub enable_persistence: bool,

    /// Session cleanup interval
    /// Cleanup Interval field
    pub cleanup_interval: Duration,
}

impl Default for CanonicalSessionConfig {
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

/// Canonical connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalConnectionConfig {
    /// Maximum connections per client
    /// Max Connections Per Client field
    pub max_connections_per_client: usize,

    /// Connection timeout
    /// Connection Timeout field
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

impl Default for CanonicalConnectionConfig {
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

/// Canonical AI mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMeshConfig {
    /// Health monitoring configuration
    /// Health Monitoring field
    pub health_monitoring: CanonicalHealthMonitoringConfig,
    /// Performance analysis configuration
    /// Performance Analysis field
    pub performance_analysis: CanonicalPerformanceAnalysisConfig,
    /// Enable mesh networking
    /// Enable Mesh field
    pub enable_mesh: bool,

    /// Mesh discovery interval
    /// Discovery Interval field
    pub discovery_interval: Duration,
    /// Maximum mesh nodes
    pub max_nodes: usize,
}

impl Default for CanonicalMeshConfig {
    fn default() -> Self {
        Self {
            health_monitoring: CanonicalHealthMonitoringConfig::default(),
            performance_analysis: CanonicalPerformanceAnalysisConfig::default(),
            enable_mesh: true,
            discovery_interval: Duration::from_secs(30),
            max_nodes: 100,
        }
    }
}

/// Canonical health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthMonitoringConfig {
    /// Health check interval
    /// Check Interval field
    pub check_interval: Duration,
    /// Health check timeout
    pub check_timeout: Duration,
    /// Failure threshold before marking unhealthy
    pub failure_threshold: u32,

    /// Recovery threshold before marking healthy
    pub recovery_threshold: u32,

    /// Enable detailed health metrics
    /// Enable Detailed Metrics field
    pub enable_detailed_metrics: bool,
}

impl Default for CanonicalHealthMonitoringConfig {
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

/// Canonical performance analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPerformanceAnalysisConfig {
    /// Enable performance monitoring
    /// Enabled field
    pub enabled: bool,

    /// Metrics collection interval
    /// Metrics Interval field
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

impl Default for CanonicalPerformanceAnalysisConfig {
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

/// Canonical service registration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalServiceRegistrationConfig {
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CanonicalCircuitBreakerConfig,
    /// Monitoring configuration
    /// Monitoring field
    pub monitoring: CanonicalMonitoringConfig,
    /// Auto-registration enabled
    /// Auto Registration field
    pub auto_registration: bool,

    /// Registration timeout
    /// Registration Timeout field
    pub registration_timeout: Duration,
}

impl Default for CanonicalServiceRegistrationConfig {
    fn default() -> Self {
        Self {
            health_check: HealthCheckConfig::default(),
            circuit_breaker: CanonicalCircuitBreakerConfig::default(),
            monitoring: CanonicalMonitoringConfig::default(),
            auto_registration: true,
            registration_timeout: Duration::from_secs(30),
        }
    }
}

/// Canonical circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalCircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,

    /// Timeout before attempting recovery
    pub timeout: Duration,
    /// Recovery timeout
    pub recovery_timeout: Duration,
    /// Enable circuit breaker
    /// Enabled field
    pub enabled: bool,
}

impl Default for CanonicalCircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(30),
            enabled: true,
        }
    }
}

/// Canonical monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMonitoringConfig {
    /// Enable monitoring
    /// Enabled field
    pub enabled: bool,

    /// Metrics collection interval
    /// Metrics Interval field
    pub metrics_interval: Duration,
    /// Log level
    pub log_level: String,
    /// Enable tracing
    /// Enable Tracing field
    pub enable_tracing: bool,
}

impl Default for CanonicalMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_interval: Duration::from_secs(60),
            log_level: String::from("info"),
            enable_tracing: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    fn assert_json_roundtrip<T>(v: &T)
    where
        T: Serialize + DeserializeOwned + std::fmt::Debug,
    {
        let json = serde_json::to_value(v).unwrap();
        let back: T = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(&back).unwrap(), json);
    }

    #[test]
    fn default_canonical_api_config() {
        let c = CanonicalApiConfig::default();
        assert_eq!(c.session.max_concurrent_sessions, 1000);
        assert_eq!(c.connection.pool_size, 100);
        assert!(c.mesh.enable_mesh);
        assert!(c.service_registration.auto_registration);
    }

    #[test]
    fn default_session_connection_mesh_service_registration() {
        assert_eq!(CanonicalSessionConfig::default().buffer_size, 8192);
        assert!(CanonicalConnectionConfig::default().enable_pooling);
        assert_eq!(CanonicalMeshConfig::default().max_nodes, 100);
        let sr = CanonicalServiceRegistrationConfig::default();
        assert_eq!(sr.registration_timeout, Duration::from_secs(30));
    }

    #[test]
    fn default_health_performance_circuit_monitoring() {
        let h = CanonicalHealthMonitoringConfig::default();
        assert_eq!(h.failure_threshold, 3);
        let p = CanonicalPerformanceAnalysisConfig::default();
        assert!(p.enabled);
        let cb = CanonicalCircuitBreakerConfig::default();
        assert!(cb.enabled);
        let m = CanonicalMonitoringConfig::default();
        assert_eq!(m.log_level, "info");
    }

    #[test]
    fn roundtrip_canonical_api_config() {
        assert_json_roundtrip(&CanonicalApiConfig::default());
    }

    #[test]
    fn roundtrip_session_connection_mesh() {
        assert_json_roundtrip(&CanonicalSessionConfig::default());
        assert_json_roundtrip(&CanonicalConnectionConfig::default());
        assert_json_roundtrip(&CanonicalMeshConfig::default());
    }

    #[test]
    fn roundtrip_health_performance_service_registration() {
        assert_json_roundtrip(&CanonicalHealthMonitoringConfig::default());
        assert_json_roundtrip(&CanonicalPerformanceAnalysisConfig::default());
        assert_json_roundtrip(&CanonicalServiceRegistrationConfig::default());
    }

    #[test]
    fn roundtrip_circuit_breaker_monitoring() {
        assert_json_roundtrip(&CanonicalCircuitBreakerConfig::default());
        assert_json_roundtrip(&CanonicalMonitoringConfig::default());
    }

    #[test]
    fn roundtrip_mesh_max_nodes_edge() {
        let mut m = CanonicalMeshConfig::default();
        m.max_nodes = 0;
        assert_json_roundtrip(&m);
    }
}
