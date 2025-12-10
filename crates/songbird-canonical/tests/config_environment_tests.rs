//! Tests for Environment Configuration
//!
//! Comprehensive tests for environment and deployment configuration structures

use songbird_canonical::config::environment::{
    Environment, EnvironmentConfig, EnvironmentSecurityConfig, LoggingConfig, NetworkConfig,
    ObservabilityConfig, PortConfig,
};
use songbird_test_utils::test_bind_address;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

#[test]
fn test_environment_config_default() {
    let config = EnvironmentConfig::default();

    assert!(matches!(config.environment, Environment::Development));
    assert_eq!(config.ports.discovery_port, 8081);
    assert_eq!(config.logging.level, "info");
    assert!(config.observability.enable_metrics);
    assert_eq!(config.network.bind_address, "0.0.0.0");
    assert!(config.security.enable_auth);
}

#[test]
fn test_environment_variants() {
    let dev = Environment::Development;
    let staging = Environment::Staging;
    let prod = Environment::Production;
    let test = Environment::Testing;
    let custom = Environment::Custom("MyEnv".to_string());

    assert!(matches!(dev, Environment::Development));
    assert!(matches!(staging, Environment::Staging));
    assert!(matches!(prod, Environment::Production));
    assert!(matches!(test, Environment::Testing));
    assert!(matches!(custom, Environment::Custom(_)));
}

#[test]
fn test_environment_custom() {
    let env = Environment::Custom("Integration".to_string());

    if let Environment::Custom(name) = env {
        assert_eq!(name, "Integration");
    } else {
        panic!("Expected Custom variant");
    }
}

#[test]
fn test_port_config_defaults() {
    let config = PortConfig::default();

    assert_eq!(config.discovery_port, 8081);
    assert_eq!(config.federation_port, 8082);
    assert_eq!(config.health_port, 8002); // Fixed: health port default is 8002
    assert_eq!(config.dynamic_port_range, (9000, 9999));
}

#[test]
fn test_port_config_custom() {
    let config = PortConfig {
        discovery_port: 7000,
        federation_port: 7001,
        health_port: 7002,
        dynamic_port_range: (8000, 8999),
    };

    assert_eq!(config.discovery_port, 7000);
    assert_eq!(config.federation_port, 7001);
    assert_eq!(config.health_port, 7002);
    assert!(config.dynamic_port_range.0 < config.dynamic_port_range.1);
}

#[test]
fn test_logging_config_defaults() {
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
fn test_logging_config_plain_format() {
    let config = LoggingConfig {
        level: "debug".to_string(),
        format: "plain".to_string(),
        output: "file".to_string(),
        file_rotation: true,
        max_file_size_mb: 50,
        max_files: 5,
        structured: false,
    };

    assert_eq!(config.level, "debug");
    assert_eq!(config.format, "plain");
    assert!(!config.structured);
}

#[test]
fn test_logging_config_levels() {
    let levels = vec!["trace", "debug", "info", "warn", "error"];

    for level in levels {
        let config = LoggingConfig {
            level: level.to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
            file_rotation: false,
            max_file_size_mb: 100,
            max_files: 10,
            structured: true,
        };

        assert_eq!(config.level, level);
    }
}

#[test]
fn test_observability_config_defaults() {
    let config = ObservabilityConfig::default();

    assert!(config.enable_metrics);
    assert_eq!(config.metrics_interval, 60);
    assert!(config.enable_tracing);
    assert!((config.trace_sampling_rate - 0.1).abs() < 0.001);
    assert!(config.enable_health_checks);
    assert_eq!(config.health_check_interval, 30);
    assert!(config.custom_tags.is_empty());
}

#[test]
fn test_observability_config_custom() {
    let mut tags = HashMap::new();
    tags.insert("team".to_string(), "backend".to_string());
    tags.insert("env".to_string(), "prod".to_string());

    let config = ObservabilityConfig {
        enable_metrics: true,
        metrics_interval: 30,
        enable_tracing: true,
        trace_sampling_rate: 1.0,
        enable_health_checks: true,
        health_check_interval: 10,
        custom_tags: tags.clone(),
    };

    assert_eq!(config.metrics_interval, 30);
    assert!((config.trace_sampling_rate - 1.0).abs() < 0.001);
    assert_eq!(config.custom_tags.len(), 2);
    assert_eq!(config.custom_tags.get("team"), Some(&"backend".to_string()));
}

#[test]
fn test_observability_sampling_rates() {
    let no_sampling = ObservabilityConfig {
        enable_metrics: true,
        metrics_interval: 60,
        enable_tracing: true,
        trace_sampling_rate: 0.0,
        enable_health_checks: true,
        health_check_interval: 30,
        custom_tags: HashMap::new(),
    };

    let full_sampling = ObservabilityConfig {
        enable_metrics: true,
        metrics_interval: 60,
        enable_tracing: true,
        trace_sampling_rate: 1.0,
        enable_health_checks: true,
        health_check_interval: 30,
        custom_tags: HashMap::new(),
    };

    assert!((no_sampling.trace_sampling_rate - 0.0).abs() < 0.001);
    assert!((full_sampling.trace_sampling_rate - 1.0).abs() < 0.001);
}

#[test]
fn test_network_config_defaults() {
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
fn test_network_config_tls_enabled() {
    let config = NetworkConfig {
        bind_address: test_bind_address(),
        enable_tls: true,
        tls_cert_path: Some("/path/to/cert.pem".to_string()),
        tls_key_path: Some("/path/to/key.pem".to_string()),
        connection_timeout: 60,
        read_timeout: 60,
        write_timeout: 60,
        max_connections: 5000,
    };

    assert!(config.enable_tls);
    assert!(config.tls_cert_path.is_some());
    assert!(config.tls_key_path.is_some());
    assert_eq!(config.max_connections, 5000);
}

#[test]
fn test_network_config_timeouts() {
    let fast = NetworkConfig {
        bind_address: "0.0.0.0".to_string(),
        enable_tls: false,
        tls_cert_path: None,
        tls_key_path: None,
        connection_timeout: 5,
        read_timeout: 10,
        write_timeout: 10,
        max_connections: 100,
    };

    let slow = NetworkConfig {
        bind_address: "0.0.0.0".to_string(),
        enable_tls: false,
        tls_cert_path: None,
        tls_key_path: None,
        connection_timeout: 120,
        read_timeout: 300,
        write_timeout: 300,
        max_connections: 10000,
    };

    assert!(fast.connection_timeout < slow.connection_timeout);
    assert!(fast.max_connections < slow.max_connections);
}

#[test]
fn test_environment_security_config_defaults() {
    let config = EnvironmentSecurityConfig::default();

    assert!(config.enable_auth);
    assert_eq!(config.auth_method, "bearer");
    assert!(config.enable_authz);
    assert!(config.enable_audit);
    assert!(config.audit_log_path.is_none());
    assert!(config.enable_cors);
    assert_eq!(config.cors_origins, vec!["*"]);
}

#[test]
fn test_environment_security_config_strict() {
    let config = EnvironmentSecurityConfig {
        enable_auth: true,
        auth_method: "oauth2".to_string(),
        enable_authz: true,
        enable_audit: true,
        audit_log_path: Some("/var/log/audit.log".to_string()),
        enable_cors: true,
        cors_origins: vec![
            "https://app.example.com".to_string(),
            "https://api.example.com".to_string(),
        ],
    };

    assert_eq!(config.auth_method, "oauth2");
    assert!(config.audit_log_path.is_some());
    assert_eq!(config.cors_origins.len(), 2);
}

#[test]
fn test_environment_security_config_permissive() -> SongbirdResult<()> {
    let config = EnvironmentSecurityConfig {
        enable_auth: false,
        auth_method: "none".to_string(),
        enable_authz: false,
        enable_audit: false,
        audit_log_path: None,
        enable_cors: false,
        cors_origins: vec![],
    };

    assert!(!config.enable_auth);
    assert!(!config.enable_authz);
    assert!(!config.enable_audit);
    assert!(config.cors_origins.is_empty());
    Ok(())
}

#[test]
fn test_environment_config_serialization() -> SongbirdResult<()> {
    let config = EnvironmentConfig::default();

    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("environment"));
    assert!(json.contains("ports"));
    assert!(json.contains("logging"));
    assert!(json.contains("observability"));
    assert!(json.contains("network"));
    assert!(json.contains("security"));

    let deserialized: EnvironmentConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Should deserialize: {}", e),
            debug_info: None,
        })?;
    assert_eq!(config.ports.discovery_port, deserialized.ports.discovery_port);
    Ok(())
}

#[test]
fn test_environment_config_clone() -> SongbirdResult<()> {
    let config = EnvironmentConfig::default();
    let cloned = config.clone();

    assert_eq!(config.ports.discovery_port, cloned.ports.discovery_port);
    assert_eq!(config.logging.level, cloned.logging.level);
    assert_eq!(config.network.bind_address, cloned.network.bind_address);
    Ok(())
}

#[test]
fn test_environment_config_debug() -> SongbirdResult<()> {
    let config = EnvironmentConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("EnvironmentConfig"));
    assert!(debug_str.contains("environment"));
    Ok(())
}

#[test]
fn test_production_environment_config() {
    let config = EnvironmentConfig {
        environment: Environment::Production,
        ports: PortConfig {
            discovery_port: 443,
            federation_port: 8443,
            health_port: 8080,
            dynamic_port_range: (10000, 19999),
        },
        logging: LoggingConfig {
            level: "warn".to_string(),
            format: "json".to_string(),
            output: "file".to_string(),
            file_rotation: true,
            max_file_size_mb: 500,
            max_files: 50,
            structured: true,
        },
        observability: ObservabilityConfig {
            enable_metrics: true,
            metrics_interval: 30,
            enable_tracing: true,
            trace_sampling_rate: 0.01,
            enable_health_checks: true,
            health_check_interval: 10,
            custom_tags: HashMap::from([("env".to_string(), "production".to_string())]),
        },
        network: NetworkConfig {
            bind_address: "0.0.0.0".to_string(),
            enable_tls: true,
            tls_cert_path: Some("/etc/ssl/cert.pem".to_string()),
            tls_key_path: Some("/etc/ssl/key.pem".to_string()),
            connection_timeout: 30,
            read_timeout: 60,
            write_timeout: 60,
            max_connections: 10000,
        },
        security: EnvironmentSecurityConfig {
            enable_auth: true,
            auth_method: "oauth2".to_string(),
            enable_authz: true,
            enable_audit: true,
            audit_log_path: Some("/var/log/songbird/audit.log".to_string()),
            enable_cors: true,
            cors_origins: vec!["https://songbird.example.com".to_string()],
        },
    };

    assert!(matches!(config.environment, Environment::Production));
    assert!(config.network.enable_tls);
    assert_eq!(config.logging.level, "warn");
}

#[test]
fn test_development_environment_config() {
    let config = EnvironmentConfig {
        environment: Environment::Development,
        ports: PortConfig::default(),
        logging: LoggingConfig {
            level: "debug".to_string(),
            format: "plain".to_string(),
            output: "stdout".to_string(),
            file_rotation: false,
            max_file_size_mb: 10,
            max_files: 1,
            structured: false,
        },
        observability: ObservabilityConfig {
            enable_metrics: false,
            metrics_interval: 60,
            enable_tracing: false,
            trace_sampling_rate: 1.0,
            enable_health_checks: false,
            health_check_interval: 60,
            custom_tags: HashMap::new(),
        },
        network: NetworkConfig {
            bind_address: test_bind_address(),
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            connection_timeout: 300,
            read_timeout: 300,
            write_timeout: 300,
            max_connections: 10,
        },
        security: EnvironmentSecurityConfig {
            enable_auth: false,
            auth_method: "none".to_string(),
            enable_authz: false,
            enable_audit: false,
            audit_log_path: None,
            enable_cors: true,
            cors_origins: vec!["*".to_string()],
        },
    };

    assert!(matches!(config.environment, Environment::Development));
    assert!(!config.network.enable_tls);
    assert_eq!(config.logging.level, "debug");
}

#[test]
fn test_port_ranges_validity() {
    let config = PortConfig::default();

    assert!(config.dynamic_port_range.0 < config.dynamic_port_range.1);
    assert!(config.dynamic_port_range.0 >= 1024); // Non-privileged ports
}

#[test]
fn test_logging_file_rotation() {
    let with_rotation = LoggingConfig {
        level: "info".to_string(),
        format: "json".to_string(),
        output: "file".to_string(),
        file_rotation: true,
        max_file_size_mb: 100,
        max_files: 10,
        structured: true,
    };

    let without_rotation = LoggingConfig {
        level: "info".to_string(),
        format: "json".to_string(),
        output: "file".to_string(),
        file_rotation: false,
        max_file_size_mb: 0,
        max_files: 0,
        structured: true,
    };

    assert!(with_rotation.file_rotation);
    assert!(!without_rotation.file_rotation);
}

#[test]
fn test_observability_all_disabled() -> SongbirdResult<()> {
    let config = ObservabilityConfig {
        enable_metrics: false,
        metrics_interval: 0,
        enable_tracing: false,
        trace_sampling_rate: 0.0,
        enable_health_checks: false,
        health_check_interval: 0,
        custom_tags: HashMap::new(),
    };

    assert!(!config.enable_metrics);
    assert!(!config.enable_tracing);
    assert!(!config.enable_health_checks);
    Ok(())
}

#[test]
fn test_environment_serialization() -> SongbirdResult<()> {
    let environments = vec![
        Environment::Development,
        Environment::Staging,
        Environment::Production,
        Environment::Testing,
        Environment::Custom("MyEnv".to_string()),
    ];

    for env in environments {
        let json = serde_json::to_string(&env)
            .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
        let _deserialized: Environment =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Should deserialize: {}", e),
                debug_info: None,
            })?;
    }
    Ok(())
}
