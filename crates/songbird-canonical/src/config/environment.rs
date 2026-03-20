// SPDX-License-Identifier: AGPL-3.0-only
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
            level: "info".to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
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
            bind_address: "0.0.0.0".to_string(),
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
            auth_method: "bearer".to_string(),
            enable_authz: true,
            enable_audit: true,
            audit_log_path: None,
            enable_cors: true,
            cors_origins: vec!["*".to_string()],
        }
    }
}
