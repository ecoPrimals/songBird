// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for canonical API configuration types
#![allow(clippy::expect_used, reason = "test assertions and harness ergonomics")]

use songbird_types::config::api::*;
use std::time::Duration;

#[test]
fn test_canonical_api_config_default() {
    let config = CanonicalApiConfig::default();
    // Verify nested defaults are applied
    assert!(config.mesh.enable_mesh);
    assert!(config.service_registration.auto_registration);
}

#[test]
fn test_canonical_session_config_default() {
    let config = CanonicalSessionConfig::default();
    assert_eq!(config.max_concurrent_sessions, 1000);
    assert_eq!(config.session_timeout, Duration::from_secs(300));
    assert_eq!(config.keep_alive_interval, Duration::from_secs(30));
    assert_eq!(config.buffer_size, 8192);
    assert!(config.enable_persistence);
    assert_eq!(config.cleanup_interval, Duration::from_secs(60));
}

#[test]
fn test_canonical_connection_config_default() {
    let config = CanonicalConnectionConfig::default();
    assert_eq!(config.max_connections_per_client, 10);
    assert_eq!(config.connection_timeout, Duration::from_secs(30));
    assert_eq!(config.read_timeout, Duration::from_secs(60));
    assert_eq!(config.write_timeout, Duration::from_secs(60));
    assert!(config.enable_pooling);
    assert_eq!(config.pool_size, 100);
    assert_eq!(config.pool_timeout, Duration::from_secs(30));
}

#[test]
fn test_canonical_mesh_config_default() {
    let config = CanonicalMeshConfig::default();
    assert!(config.enable_mesh);
    assert_eq!(config.discovery_interval, Duration::from_secs(30));
    assert_eq!(config.max_nodes, 100);
}

#[test]
fn test_canonical_health_monitoring_config_default() {
    let config = CanonicalHealthMonitoringConfig::default();
    assert_eq!(config.check_interval, Duration::from_secs(30));
    assert_eq!(config.check_timeout, Duration::from_secs(10));
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.recovery_threshold, 2);
    assert!(config.enable_detailed_metrics);
}

#[test]
fn test_canonical_performance_analysis_config_default() {
    let config = CanonicalPerformanceAnalysisConfig::default();
    assert!(config.enabled);
    assert_eq!(config.metrics_interval, Duration::from_secs(10));
    assert_eq!(config.analysis_window, Duration::from_secs(300));
    assert!((config.cpu_threshold - 80.0).abs() < f64::EPSILON);
    assert!((config.memory_threshold - 85.0).abs() < f64::EPSILON);
    assert_eq!(config.latency_threshold, Duration::from_millis(100));
}

#[test]
fn test_canonical_service_registration_config_default() {
    let config = CanonicalServiceRegistrationConfig::default();
    assert!(config.auto_registration);
    assert_eq!(config.registration_timeout, Duration::from_secs(30));
}

#[test]
fn test_canonical_circuit_breaker_config_default() {
    let config = CanonicalCircuitBreakerConfig::default();
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.recovery_timeout, Duration::from_secs(30));
    assert!(config.enabled);
}

#[test]
fn test_canonical_monitoring_config_default() {
    let config = CanonicalMonitoringConfig::default();
    assert!(config.enabled);
    assert_eq!(config.metrics_interval, Duration::from_secs(60));
    assert_eq!(config.log_level, "info");
    assert!(config.enable_tracing);
}

#[test]
fn test_session_config_clone() {
    let config = CanonicalSessionConfig::default();
    let cloned = config.clone();
    assert_eq!(config.max_concurrent_sessions, cloned.max_concurrent_sessions);
    assert_eq!(config.session_timeout, cloned.session_timeout);
}

#[test]
fn test_connection_config_clone() {
    let config = CanonicalConnectionConfig::default();
    let cloned = config.clone();
    assert_eq!(config.pool_size, cloned.pool_size);
    assert_eq!(config.connection_timeout, cloned.connection_timeout);
}

#[test]
fn test_mesh_config_clone() {
    let config = CanonicalMeshConfig::default();
    let cloned = config.clone();
    assert_eq!(config.max_nodes, cloned.max_nodes);
    assert_eq!(config.enable_mesh, cloned.enable_mesh);
}

#[test]
fn test_circuit_breaker_config_clone() {
    let config = CanonicalCircuitBreakerConfig::default();
    let cloned = config.clone();
    assert_eq!(config.failure_threshold, cloned.failure_threshold);
    assert_eq!(config.enabled, cloned.enabled);
}

#[test]
fn test_monitoring_config_clone() {
    let config = CanonicalMonitoringConfig::default();
    let cloned = config.clone();
    assert_eq!(config.log_level, cloned.log_level);
    assert_eq!(config.enable_tracing, cloned.enable_tracing);
}

#[test]
fn test_api_config_serialization() {
    let config = CanonicalApiConfig::default();
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("session"));
    assert!(json.contains("connection"));
    assert!(json.contains("mesh"));
}

#[test]
fn test_session_config_deserialization() {
    let json = r#"{"max_concurrent_sessions":500,"session_timeout":{"secs":120,"nanos":0},"keep_alive_interval":{"secs":15,"nanos":0},"buffer_size":4096,"enable_persistence":false,"cleanup_interval":{"secs":30,"nanos":0}}"#;
    let config: CanonicalSessionConfig =
        serde_json::from_str(json).expect("Deserialization should succeed");
    assert_eq!(config.max_concurrent_sessions, 500);
    assert_eq!(config.buffer_size, 4096);
    assert!(!config.enable_persistence);
}
