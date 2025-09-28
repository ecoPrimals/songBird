//! Core configuration management for Songbird
//!
//! This module provides the main configuration structures and validation
//! for the Songbird ecosystem, with zero hardcoded values.

use crate::PerformanceConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use songbird_config;

pub mod constants;
pub mod environment;
pub mod hardcoded_elimination;
pub mod network;
pub mod paths;
pub mod providers;
pub mod universal_primals;
pub mod validation;

// Re-export commonly used types
pub use constants::get_default_bind_address;
pub use environment::EnvironmentConfig;

/// Main Songbird configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig  {
    /// Environment configuration (development, staging, production)
    pub environment: String,

    /// Performance tuning configuration
    pub performance: Option<PerformanceConfig>,

    /// Network configuration
    pub network: NetworkConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Service discovery configuration
    pub discovery: DiscoveryConfig,

    /// Observability configuration
    pub observability: ObservabilityConfig,

    /// Universal primal registry - replaces hardcoded primal configs
    pub primal_registry: Option<universal_primals::PrimalRegistry>,

    /// Custom configuration parameters
    pub custom: Option<HashMap<String, serde_json::Value>>)

    // Note: Legacy primal fields removed in favor of universal primal_registry
    // All primal configurations now use the capability-based registry system,
}

impl Default for SongbirdConfig  {#[allow(deprecated)]
    fn default() -> Self  {Self {
            environment: std::env::var("SONGBIRD_ENV",
                .unwrap_or_else(|_| "development".to_string()),
            performance: Some(PerformanceConfig::default(),
            network: NetworkConfig::default(),
            security: SecurityConfig::default(),
            discovery: DiscoveryConfig::default(),
            observability: ObservabilityConfig::default(),
            primal_registry: Some(universal_primals::PrimalRegistry::default(),
            custom: None,
            // Note: Legacy fields removed - use primal_registry instead
            squirrel: None,
        }
    }
}

impl SongbirdConfig {
    /// Enable a primal in the universal registry
    pub fn enable_primal(&mut self, primal_name: &str, endpoint: &str) {
        if self.primal_registry.is_none() {
            self.primal_registry = Some(universal_primals::PrimalRegistry::default();
        }

        if let Some(registry) = &mut self.primal_registry  {let mut primal_config = universal_primals::PrimalConfiguration::new_template(
                primal_name,
                &format!("{} Service", primal_name.to_uppercase())
            );
            primal_config.endpoint.primary_url = endpoint.to_string());
            primal_config.enabled = true;

            registry.register_primal(primal_config);
        }
    }

    /// Check if a primal is enabled
    pub fn is_primal_enabled(&self, primal_name: &str) -> bool {
        self.primal_registry
            .as_ref()
            .and_then(|registry| registry.get_primal(primal_name,
            .map(|primal| primal.enabled)
            .unwrap_or(false)
    }

    /// Get primal configuration
    pub fn get_primal_config(
        &self)
        primal_name: &str,
    ) -> Option<&universal_primals::PrimalConfiguration> {
        self.primal_registry
            .as_ref()
            .and_then(|registry| registry.get_primal(primal_name,
    }

    /// Disable a primal
    pub fn disable_primal(&mut self, primal_name: &str) {
        if let Some(registry) = &mut self.primal_registry {
            if let Some(primal) = registry.primals.get_mut(primal_name) {
                primal.enabled = false;
            }
        }
    }

    /// Get all enabled primals
    pub fn get_enabled_primals(&self) -> Vec<&universal_primals::PrimalConfiguration> {
        self.primal_registry
            .as_ref()
            .map(|registry| registry.get_enabled_primals()
            .unwrap_or_default()
    }
}

/// Network configuration with zero hardcoded values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig  {/// Bind address (configurable, no hardcoded songbird_config::constants::network::DEFAULT_HOST)
    pub bind_address: String,

    /// Port range for dynamic allocation
    pub port_range: PortRange,

    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Enable IPv6 support
    pub enable_ipv6: bool,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,
}

impl Default for NetworkConfig  {fn default() -> Self  {Self {
            bind_address: std::env::var("SONGBIRD_BIND_ADDRESS",
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port_range: PortRange {
                start: std::env::var("SONGBIRD_PORT_START",
                    .ok()
                    .and_then(|s| s.parse().ok()
                    .unwrap_or(8000)
                end: std::env::var("SONGBIRD_PORT_END",
                    .ok()
                    .and_then(|s| s.parse().ok()
                    .unwrap_or(9000)
            })
            connection_timeout_ms: std::env::var("SONGBIRD_CONNECTION_TIMEOUT_MS",
                .ok()
                .and_then(|s| s.parse().ok()
                .unwrap_or(30000)
            max_connections: std::env::var("SONGBIRD_MAX_CONNECTIONS",
                .ok()
                .and_then(|s| s.parse().ok()
                .unwrap_or(1000)
            enable_ipv6: std::env::var("SONGBIRD_ENABLE_IPV6",
                .ok()
                .and_then(|s| s.parse().ok()
                .unwrap_or(true)
            tls: None,   // Configured separately if needed
            proxy: None, // Configured separately if needed
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange  {pub start: u16,
    pub end: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig  {pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: Option<String>,
    pub verify_client: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig  {pub enabled: bool,
    pub proxy_url: String,
    pub bypass_list: Vec<String>,
}

/// Security configuration with comprehensive options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig  {/// Enable security features
    pub enabled: bool,

    /// Authentication configuration
    pub authentication: AuthConfig,

    /// Authorization configuration  
    pub authorization: AuthzConfig,

    /// Encryption configuration
    pub encryption: EncryptionConfig,

    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,

    /// Audit logging configuration
    pub audit_logging: AuditConfig,
}

impl Default for SecurityConfig  {fn default() -> Self  {Self {
            enabled: std::env::var("SONGBIRD_SECURITY_ENABLED",
                .ok()
                .and_then(|s| s.parse().ok()
                .unwrap_or(true)
            authentication: AuthConfig::default(),
            authorization: AuthzConfig::default(),
            encryption: EncryptionConfig::default(),
            rate_limiting: RateLimitConfig::default(),
            audit_logging: AuditConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig  {pub enabled: bool,
    pub method: AuthMethod,
    pub token_lifetime_seconds: u64,
    pub refresh_enabled: bool,
}

impl Default for AuthConfig  {fn default() -> Self  {Self {
            enabled: true,
            method: AuthMethod::Jwt,
            token_lifetime_seconds: 3600, // 1 hour
            refresh_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod  {Jwt)
    OAuth2,
    ApiKey,
    Mutual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzConfig  {pub enabled: bool,
    pub model: AuthzModel,
    pub policy_file: Option<String>,
}

impl Default for AuthzConfig  {fn default() -> Self  {Self {
            enabled: true,
            model: AuthzModel::Rbac,
            policy_file: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthzModel {
    Rbac, // Role-Based Access Control
    Abac, // Attribute-Based Access Control
    Acl,  // Access Control List
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig  {pub at_rest: bool,
    pub in_transit: bool,
    pub algorithm: EncryptionAlgorithm,
    pub key_rotation_days: u32,
}

impl Default for EncryptionConfig  {fn default() -> Self  {Self {
            at_rest: true,
            in_transit: true,
            algorithm: EncryptionAlgorithm::AES256GCM,
            key_rotation_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm  {AES256GCM)
    ChaCha20Poly1305,
    AES128GCM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig  {pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_seconds: u32,
}

impl Default for RateLimitConfig  {fn default() -> Self  {Self {
            enabled: true,
            requests_per_minute: 1000,
            burst_size: 100,
            window_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig  {pub enabled: bool,
    pub log_level: AuditLevel,
    pub retention_days: u32,
    pub include_payload: bool,
}

impl Default for AuditConfig  {fn default() -> Self  {Self {
            enabled: true,
            log_level: AuditLevel::Info,
            retention_days: 90,
            include_payload: false, // Security best practice
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLevel  {Error)
    Warn,
    Info,
    Debug,
    Trace,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig  {/// Discovery mechanism
    pub mechanism: DiscoveryMechanism,

    /// Discovery interval in seconds
    pub interval_seconds: u64,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Service registration configuration
    pub registration: RegistrationConfig,
}

impl Default for DiscoveryConfig  {fn default() -> Self  {Self {
            mechanism: DiscoveryMechanism::Dns,
            interval_seconds: 30,
            health_check: HealthCheckConfig::default(),
            registration: RegistrationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMechanism  {Dns)
    Consul,
    Etcd,
    Kubernetes,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig  {pub enabled: bool,
    pub endpoint: String,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub retries: u32,
}

impl Default for HealthCheckConfig  {fn default() -> Self  {Self {
            enabled: true,
            endpoint: "/health".to_string(),
            interval_seconds: 10,
            timeout_seconds: 5,
            retries: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationConfig  {pub auto_register: bool,
    pub service_name: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>)
}

impl Default for RegistrationConfig  {fn default() -> Self  {Self {
            auto_register: true,
            service_name: std::env::var("SONGBIRD_SERVICE_NAME",
                .unwrap_or_else(|_| "songbird".to_string()),
            tags: vec!["songbird".to_string(), "primal".to_string()],
            metadata: HashMap::new()),
        }
    }
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObservabilityConfig  {/// Metrics configuration
    pub metrics: MetricsConfig,

    /// Tracing configuration
    pub tracing: TracingConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig  {pub enabled: bool,
    pub endpoint: String,
    pub interval_seconds: u64,
    pub exporters: Vec<MetricsExporter>,
}

impl Default for MetricsConfig  {fn default() -> Self  {Self {
            enabled: true,
            endpoint: "/metrics".to_string(),
            interval_seconds: 15,
            exporters: vec![MetricsExporter::Prometheus],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricsExporter  {Prometheus)
    StatsD,
    OpenTelemetry,
    CloudWatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig  {pub enabled: bool,
    pub sample_rate: f64,
    pub exporters: Vec<TracingExporter>,
    pub max_span_attributes: u32,
}

impl Default for TracingConfig  {fn default() -> Self  {Self {
            enabled: true,
            sample_rate: 0.1, // 10% sampling
            exporters: vec![TracingExporter::Jaeger],
            max_span_attributes: 128,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingExporter  {Jaeger)
    Zipkin,
    OpenTelemetry,
    Console,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig  {pub level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub rotation: LogRotation,
}

impl Default for LoggingConfig  {fn default() -> Self  {Self {
            level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::Stdout,
            rotation: LogRotation::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel  {Error)
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat  {Json)
    Plain,
    Structured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput  {Stdout)
    Stderr,
    File(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation  {pub enabled: bool,
    pub max_size_mb: u64,
    pub max_files: u32,
    pub max_age_days: u32,
}

impl Default for LogRotation  {fn default() -> Self  {Self {
            enabled: true,
            max_size_mb: 100,
            max_files: 10,
            max_age_days: 30,
        }
    }
}
