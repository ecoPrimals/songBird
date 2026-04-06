// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for Environment Configuration
//!
//! Comprehensive test coverage for environment and deployment configuration.

use super::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

// ============================================================================
// EnvironmentConfig Tests
// ============================================================================

#[test]
fn test_environment_config_default() -> SongbirdResult<()> {
    let config = EnvironmentConfig::default();

    assert_eq!(config.ports.discovery_port, 8081);
    assert_eq!(config.logging.level, "info");
    assert!(config.observability.enable_metrics);
    assert_eq!(config.network.bind_address, "0.0.0.0");
    Ok(())
}

#[test]
fn test_environment_config_serialization() -> SongbirdResult<()> {
    let config = EnvironmentConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
    let deserialized: EnvironmentConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;

    assert_eq!(config.ports.discovery_port, deserialized.ports.discovery_port);
    Ok(())
}

#[test]
fn test_environment_config_clone() {
    let config = EnvironmentConfig::default();
    let cloned = config.clone();

    assert_eq!(config.ports.discovery_port, cloned.ports.discovery_port);
}

// ============================================================================
// Environment Tests
// ============================================================================

#[test]
fn test_environment_variants() -> SongbirdResult<()> {
    let dev = Environment::Development;
    let staging = Environment::Staging;
    let production = Environment::Production;
    let testing = Environment::Testing;
    let custom = Environment::Custom("preview".to_string());

    // Verify variants exist
    let _ = dev;
    let _ = staging;
    let _ = production;
    let _ = testing;
    let _ = custom;
    Ok(())
}

#[test]
fn test_environment_serialization() -> SongbirdResult<()> {
    let env = Environment::Production;
    let json = serde_json::to_string(&env)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;

    assert!(json.contains("Production"));
    Ok(())
}

// ============================================================================
// PortConfig Tests
// ============================================================================

#[test]
fn test_port_config_default() {
    let config = PortConfig::default();

    // Ports now come from environment or defaults
    assert_eq!(config.discovery_port, songbird_config::defaults::ports::discovery_port());
    assert_eq!(config.federation_port, songbird_config::defaults::ports::federation_port());
    assert_eq!(config.health_port, songbird_config::defaults::ports::health_port());
    assert_eq!(config.dynamic_port_range, (9000, 9999));
}

#[test]
fn test_port_config_custom_ports() {
    let mut config = PortConfig::default();
    config.discovery_port = 9091;
    config.federation_port = 9092;
    config.health_port = 9095;

    assert_eq!(config.discovery_port, 9091);
    assert_eq!(config.federation_port, 9092);
    assert_eq!(config.health_port, 9095);
}

#[test]
fn test_port_config_dynamic_range() {
    let mut config = PortConfig::default();
    config.dynamic_port_range = (10000, 19999);

    assert_eq!(config.dynamic_port_range.0, 10000);
    assert_eq!(config.dynamic_port_range.1, 19999);
}

// ============================================================================
// LoggingConfig Tests
// ============================================================================

#[test]
fn test_logging_config_default() {
    let config = LoggingConfig::default();

    assert_eq!(config.level, "info");
    assert_eq!(config.format, "json");
    assert_eq!(config.output, "stdout");
    assert!(config.file_rotation);
    assert_eq!(config.max_file_size_mb, 100);
    assert_eq!(config.max_files, 10);
    assert!(config.structured);
}

#[test]
fn test_logging_config_log_levels() {
    let mut config = LoggingConfig::default();

    config.level = "trace".to_string();
    assert_eq!(config.level, "trace");

    config.level = "debug".to_string();
    assert_eq!(config.level, "debug");

    config.level = "warn".to_string();
    assert_eq!(config.level, "warn");

    config.level = "error".to_string();
    assert_eq!(config.level, "error");
}

#[test]
fn test_logging_config_formats() {
    let mut config = LoggingConfig::default();

    config.format = "plain".to_string();
    assert_eq!(config.format, "plain");

    config.format = "structured".to_string();
    assert_eq!(config.format, "structured");
}

#[test]
fn test_logging_config_file_rotation() {
    let mut config = LoggingConfig::default();
    config.file_rotation = false;

    assert!(!config.file_rotation);
}

#[test]
fn test_logging_config_file_limits() {
    let mut config = LoggingConfig::default();
    config.max_file_size_mb = 500;
    config.max_files = 20;

    assert_eq!(config.max_file_size_mb, 500);
    assert_eq!(config.max_files, 20);
}

// ============================================================================
// ObservabilityConfig Tests
// ============================================================================

#[test]
fn test_observability_config_default() {
    let config = ObservabilityConfig::default();

    assert!(config.enable_metrics);
    assert_eq!(config.metrics_interval, 60);
    assert!(config.enable_tracing);
    assert_eq!(config.trace_sampling_rate, 0.1);
    assert!(config.enable_health_checks);
    assert_eq!(config.health_check_interval, 30);
    assert!(config.custom_tags.is_empty());
}

#[test]
fn test_observability_config_metrics_disabled() {
    let mut config = ObservabilityConfig::default();
    config.enable_metrics = false;

    assert!(!config.enable_metrics);
}

#[test]
fn test_observability_config_custom_intervals() {
    let mut config = ObservabilityConfig::default();
    config.metrics_interval = 120;
    config.health_check_interval = 60;

    assert_eq!(config.metrics_interval, 120);
    assert_eq!(config.health_check_interval, 60);
}

#[test]
fn test_observability_config_sampling_rate() {
    let mut config = ObservabilityConfig::default();
    config.trace_sampling_rate = 1.0; // 100% sampling

    assert_eq!(config.trace_sampling_rate, 1.0);
}

#[test]
fn test_observability_config_custom_tags() {
    let mut config = ObservabilityConfig::default();
    config.custom_tags.insert("environment".to_string(), "production".to_string());
    config.custom_tags.insert("region".to_string(), "us-west-2".to_string());

    assert_eq!(config.custom_tags.len(), 2);
    assert_eq!(config.custom_tags.get("environment"), Some(&"production".to_string()));
}

// ============================================================================
// NetworkConfig Tests
// ============================================================================

#[test]
fn test_network_config_default() {
    let config = NetworkConfig::default();

    assert_eq!(config.bind_address, "0.0.0.0");
    assert!(!config.enable_tls);
    assert!(config.tls_cert_path.is_none());
    assert!(config.tls_key_path.is_none());
    assert_eq!(config.connection_timeout, 30);
    assert_eq!(config.read_timeout, 30);
    assert_eq!(config.write_timeout, 30);
    assert_eq!(config.max_connections, 1000);
}

#[test]
fn test_network_config_with_tls() {
    let mut config = NetworkConfig::default();
    config.enable_tls = true;
    config.tls_cert_path = Some("/etc/certs/cert.pem".to_string());
    config.tls_key_path = Some("/etc/certs/key.pem".to_string());

    assert!(config.enable_tls);
    assert!(config.tls_cert_path.is_some());
    assert!(config.tls_key_path.is_some());
}

#[test]
fn test_network_config_timeouts() {
    let mut config = NetworkConfig::default();
    config.connection_timeout = 60;
    config.read_timeout = 45;
    config.write_timeout = 45;

    assert_eq!(config.connection_timeout, 60);
    assert_eq!(config.read_timeout, 45);
    assert_eq!(config.write_timeout, 45);
}

#[test]
fn test_network_config_max_connections() {
    let mut config = NetworkConfig::default();
    config.max_connections = 5000;

    assert_eq!(config.max_connections, 5000);
}

// ============================================================================
// EnvironmentSecurityConfig Tests
// ============================================================================

#[test]
fn test_environment_security_config_default() {
    let config = EnvironmentSecurityConfig::default();

    assert!(config.enable_auth);
    assert_eq!(config.auth_method, "bearer");
    assert!(config.enable_authz);
    assert!(config.enable_audit);
    assert!(config.audit_log_path.is_none());
    assert!(config.enable_cors);
    assert_eq!(config.cors_origins, vec!["*".to_string()]);
}

#[test]
fn test_environment_security_config_auth_disabled() -> SongbirdResult<()> {
    let mut config = EnvironmentSecurityConfig::default();
    config.enable_auth = false;

    assert!(!config.enable_auth);
    Ok(())
}

#[test]
fn test_environment_security_config_auth_methods() -> SongbirdResult<()> {
    let mut config = EnvironmentSecurityConfig::default();

    config.auth_method = "jwt".to_string();
    assert_eq!(config.auth_method, "jwt");

    config.auth_method = "oauth2".to_string();
    assert_eq!(config.auth_method, "oauth2");
    Ok(())
}

#[test]
fn test_environment_security_config_audit_path() -> SongbirdResult<()> {
    let mut config = EnvironmentSecurityConfig::default();
    config.audit_log_path = Some("/var/log/audit.log".to_string());

    assert!(config.audit_log_path.is_some());
    assert_eq!(config.audit_log_path.unwrap(), "/var/log/audit.log");
    Ok(())
}

#[test]
fn test_environment_security_config_cors_origins() {
    let mut config = EnvironmentSecurityConfig::default();
    config.cors_origins =
        vec!["https://app.example.com".to_string(), "https://api.example.com".to_string()];

    assert_eq!(config.cors_origins.len(), 2);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_environment_config_production_preset() {
    let config = EnvironmentConfig {
        environment: Environment::Production,
        ports: PortConfig {
            discovery_port: 8081,
            federation_port: 8082,
            health_port: 8085,
            dynamic_port_range: (9000, 9999),
        },
        logging: LoggingConfig {
            level: "warn".to_string(),
            format: "json".to_string(),
            output: "file".to_string(),
            file_rotation: true,
            max_file_size_mb: 1000,
            max_files: 30,
            structured: true,
        },
        observability: ObservabilityConfig {
            enable_metrics: true,
            metrics_interval: 30,
            enable_tracing: true,
            trace_sampling_rate: 0.05, // 5% sampling in production
            enable_health_checks: true,
            health_check_interval: 15,
            custom_tags: HashMap::new(),
        },
        network: NetworkConfig {
            bind_address: "0.0.0.0".to_string(),
            enable_tls: true,
            tls_cert_path: Some("/etc/certs/cert.pem".to_string()),
            tls_key_path: Some("/etc/certs/key.pem".to_string()),
            connection_timeout: 60,
            read_timeout: 60,
            write_timeout: 60,
            max_connections: 10000,
        },
        security: EnvironmentSecurityConfig {
            enable_auth: true,
            auth_method: "jwt".to_string(),
            enable_authz: true,
            enable_audit: true,
            audit_log_path: Some("/var/log/audit.log".to_string()),
            enable_cors: true,
            cors_origins: vec!["https://app.example.com".to_string()],
        },
    };

    // Verify production settings
    assert_eq!(config.logging.level, "warn");
    assert!(config.network.enable_tls);
    assert_eq!(config.observability.trace_sampling_rate, 0.05);
    assert_eq!(config.network.max_connections, 10000);
}

#[test]
fn test_environment_config_development_preset() {
    let mut config = EnvironmentConfig::default();
    config.environment = Environment::Development;
    config.logging.level = "debug".to_string();
    config.observability.trace_sampling_rate = 1.0; // 100% in development
    config.network.enable_tls = false;
    config.security.enable_auth = false;

    assert_eq!(config.logging.level, "debug");
    assert_eq!(config.observability.trace_sampling_rate, 1.0);
    assert!(!config.network.enable_tls);
    assert!(!config.security.enable_auth);
}
