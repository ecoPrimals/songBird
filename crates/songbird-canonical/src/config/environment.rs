// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Environment /// Configuration capability Configuration
//!
//! Configuration structures for environment and deployment settings)
//! including networking, logging, observability, and port management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Environment and deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment type (development, staging, production)
    /// Environment field
    pub environment: Environment,
    /// Port configuration
    pub ports: PortConfig,
    /// Logging configuration
    /// Whether logging is enabled
    pub logging: LoggingConfig,
    /// Observability configuration
    pub observability: ObservabilityConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Security configuration
    pub security: EnvironmentSecurityConfig,
}

/// Environment types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
    /// Testing environment
    Testing,
    /// Custom environment
    Custom(String),
}

/// Port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// Orchestrator service port
    pub discovery_port: u16,
    /// Federation service port
    /// Federation Port field
    pub federation_port: u16,
    /// Dashboard port
    pub health_port: u16,
    /// Dynamic port range for services
    pub dynamic_port_range: (u16, u16),
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    /// Level field
    pub level: String,
    /// Log format (json, plain, structured)
    /// Format field
    pub format: String,
    /// Log output destination
    pub output: String,
    /// Enable file rotation
    /// File Rotation field
    pub file_rotation: bool,
    /// Maximum log file size (MB)
    /// Max File Size Mb field
    pub max_file_size_mb: u32,
    /// Number of log files to retain
    pub max_files: u32,
    /// Enable structured logging
    pub structured: bool,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Enable metrics collection
    /// Enable Metrics field
    pub enable_metrics: bool,
    /// Metrics collection interval (seconds)
    /// Metrics Interval field
    pub metrics_interval: u64,
    /// Enable distributed tracing
    /// Enable Tracing field
    pub enable_tracing: bool,
    /// Trace sampling rate (0.0-1.0)
    /// Trace Sampling Rate field
    pub trace_sampling_rate: f64,
    /// Enable health checks
    /// Enable Health Checks field
    pub enable_health_checks: bool,
    /// Health check interval (seconds)
    /// Health Check Interval field
    pub health_check_interval: u64,
    /// Custom metrics tags
    pub custom_tags: HashMap<String, String>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address for services
    /// Bind Address field
    pub bind_address: String,
    /// Enable /// TLS
    // TLS
    /// Enable Tls field
    pub enable_tls: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS private key path
    pub tls_key_path: Option<String>,
    /// Connection timeout (seconds)
    /// Connection Timeout field
    pub connection_timeout: u64,
    /// Read timeout (seconds)
    /// Read Timeout field
    pub read_timeout: u64,
    /// Write timeout (seconds)
    /// Write Timeout field
    pub write_timeout: u64,
    /// Maximum concurrent connections
    /// Max Connections field
    pub max_connections: usize,
}

/// Environment security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "intentional pattern; clippy false positive for this API"
)]
pub struct EnvironmentSecurityConfig {
    /// Enable authentication
    /// Enable Auth field
    pub enable_auth: bool,
    /// Authentication method
    pub auth_method: String,
    /// Enable authorization
    /// Enable Authz field
    pub enable_authz: bool,
    /// Enable audit logging
    /// Enable Audit field
    pub enable_audit: bool,
    /// Audit log path
    pub audit_log_path: Option<String>,
    /// Enable /// CORS
    // CORS
    /// Enable Cors field
    pub enable_cors: bool,
    /// Allowed origins for /// CORS
    // CORS
    /// Cors Origins field
    pub cors_origins: Vec<String>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            ports: PortConfig::default(),
            logging: LoggingConfig::default(),
            observability: ObservabilityConfig::default(),
            network: NetworkConfig::default(),
            security: EnvironmentSecurityConfig::default(),
        }
    }
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            discovery_port: songbird_config::defaults::ports::discovery_port(),
            federation_port: songbird_config::defaults::ports::federation_port(),
            health_port: songbird_config::defaults::ports::health_port(),
            dynamic_port_range: (9000, 9999),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: String::from("info"),
            format: String::from("json"),
            output: String::from("stdout"),
            file_rotation: true,
            max_file_size_mb: 100,
            max_files: 10,
            structured: true,
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_interval: 60,
            enable_tracing: true,
            trace_sampling_rate: 0.1,
            enable_health_checks: true,
            health_check_interval: 30,
            custom_tags: HashMap::new(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: String::from("0.0.0.0"),
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            connection_timeout: 30,
            read_timeout: 30,
            write_timeout: 30,
            max_connections: 1000,
        }
    }
}

impl Default for EnvironmentSecurityConfig {
    fn default() -> Self {
        Self {
            enable_auth: true,
            auth_method: String::from("bearer"),
            enable_authz: true,
            enable_audit: true,
            audit_log_path: None,
            enable_cors: true,
            cors_origins: vec![String::from("*")],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    fn assert_json_roundtrip<T>(value: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let json = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }

    #[test]
    fn environment_config_default_non_trivial() {
        let c = EnvironmentConfig::default();
        assert!(matches!(c.environment, Environment::Development));
        assert_eq!(c.ports.dynamic_port_range, (9000, 9999));
        assert_eq!(c.logging.level, "info");
        assert!(c.observability.enable_metrics);
        assert_eq!(c.network.bind_address, "0.0.0.0");
        assert!(c.security.enable_cors);
    }

    #[test]
    fn environment_config_roundtrip() {
        let mut c = EnvironmentConfig::default();
        c.environment = Environment::Custom(String::from("edge"));
        c.observability.custom_tags.insert(String::from("k"), String::from("v"));
        assert_json_roundtrip(&c);
    }

    #[test]
    fn environment_variants_roundtrip() {
        for env in [
            Environment::Development,
            Environment::Staging,
            Environment::Production,
            Environment::Testing,
            Environment::Custom(String::from("lab")),
        ] {
            assert_json_roundtrip(&env);
        }
    }

    #[test]
    fn port_config_default_and_roundtrip() {
        let p = PortConfig::default();
        assert!(p.discovery_port > 0);
        assert_json_roundtrip(&p);
    }

    #[test]
    fn logging_config_default_and_roundtrip() {
        let l = LoggingConfig::default();
        assert_eq!(l.format, "json");
        assert_eq!(l.max_files, 10);
        assert_json_roundtrip(&l);
    }

    #[test]
    fn observability_config_default_and_roundtrip() {
        let o = ObservabilityConfig::default();
        assert!((o.trace_sampling_rate - 0.1).abs() < f64::EPSILON);
        assert_json_roundtrip(&o);
    }

    #[test]
    fn network_config_default_and_roundtrip() {
        let n = NetworkConfig::default();
        assert_eq!(n.max_connections, 1000);
        assert_json_roundtrip(&n);
    }

    #[test]
    fn environment_security_config_default_and_roundtrip() {
        let s = EnvironmentSecurityConfig::default();
        assert_eq!(s.auth_method, "bearer");
        assert_json_roundtrip(&s);
    }
}
