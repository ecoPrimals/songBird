//! Core configuration management for Songbird
//!
//! # ⚠️ **DEPRECATION NOTICE** - Phase 1 Complete (Nov 2025)
//!
//! This module is being consolidated into `canonical::` module as part of the
//! configuration unification effort. All new code should use `canonical::` instead.
//!
//! ## 📋 Migration Path (Simple!)
//!
//! ### Quick Migration Table
//!
//! | Old (Deprecated) | New (Canonical) | Status |
//! |------------------|-----------------|--------|
//! | `config::NetworkConfig` | `canonical::NetworkConfig` | ✅ Available |
//! | `config::environment::EnvironmentConfig` | `canonical::EnvironmentConfig` | ✅ Available |
//! | `config::ServiceConfig` | `canonical::ServiceConfig` | ✅ Available |
//! | `config::SecurityConfig` | `canonical::SecurityConfig` | 🔄 Phase 5 |
//! | `config::PrimalConfig` | `canonical::PrimalConfig` | 🔄 Phase 4 |
//!
//! ### Code Examples
//!
//! ```rust,ignore
//! // ❌ OLD (deprecated - still works but discouraged)
//! use songbird_config::config::NetworkConfig;
//! use songbird_config::config::environment::EnvironmentConfig;
//!
//! // ✅ NEW (canonical - recommended for all new code)
//! use songbird_config::canonical::{NetworkConfig, EnvironmentConfig};
//!
//! // Or with explicit aliases for clarity:
//! use songbird_config::canonical::{
//!     NetworkConfig as CanonicalNetworkConfig,
//!     EnvironmentConfig as CanonicalEnvironmentConfig,
//! };
//! ```
//!
//! ## 📅 Timeline
//! - **Nov 2025**: Phase 1 complete - Deprecation notices added
//! - **Dec 2025**: Phase 2-3 - Network & Environment consolidation
//! - **Q1 2026**: Phase 4-6 - Complete consolidation
//! - **Q2 2026**: Remove deprecated `config::` module (6 months notice)
//!
//! ## 📊 Status
//! - ✅ This module maintained for backward compatibility
//! - ✅ New code should use `canonical::` module
//! - ✅ Deprecation warnings guide migration
//! - 🔄 Active consolidation in progress
//!
//! ## 📚 See Also
//! - `crate::canonical` - **Single source of truth** for all config types
//! - `CONFIG_CONSOLIDATION_ROADMAP.md` - Detailed 6-phase consolidation plan
//! - `UNIFICATION_AUDIT_REPORT_NOV_8_2025.md` - Complete analysis

use crate::PerformanceConfig;
use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;
use std::collections::HashMap;
// use songbird_config; // FIXED: Circular import removed

// Archived modules (moved to _archived_q2_2026/ on November 8, 2025):
// pub mod agnostic_primals; // Use canonical::primals instead

// Backward compatibility re-exports - Module archived (Phase 4: November 8, 2025)
#[allow(deprecated)]
pub mod universal_primals; // DEPRECATED: Use canonical::primals instead - kept for re-exports only

pub mod constants;
// pub mod environment; // ✅ REMOVED: Fully consolidated into canonical::environment (Nov 9, 2025)
pub mod hardcoded_elimination;
// pub mod network; // ✅ REMOVED: Fully consolidated into canonical::network (Nov 9, 2025)
pub mod paths;
pub mod providers;
// TEMPORARY: Disabled due to syntax errors - fix in next session
// pub mod validation;

// Re-export commonly used types
#[allow(deprecated)]
pub use constants::get_default_bind_address;
// ✅ REMOVED: Use canonical::environment::EnvironmentConfig instead (Nov 9, 2025)
// pub use environment::EnvironmentConfig;

/// Main Songbird configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig {
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
    /// **Deprecated**: Use `canonical::primals::PrimalRegistry` directly
    #[deprecated(since = "0.2.0", note = "Use `canonical::primals::PrimalRegistry` instead")]
    pub primal_registry: Option<crate::canonical::primals::PrimalRegistry>,

    /// Custom configuration parameters
    pub custom: Option<HashMap<String, serde_json::Value>>,
    // Note: Legacy primal fields removed in favor of universal primal_registry
    // All primal configurations now use the capability-based registry system
}

impl Default for SongbirdConfig {
    #[allow(deprecated)]
    fn default() -> Self {
        Self {
            environment: SafeEnv::get_or_default("SONGBIRD_ENV", "development"),
            performance: Some(PerformanceConfig::default()),
            network: NetworkConfig::default(),
            security: SecurityConfig::default(),
            discovery: DiscoveryConfig::default(),
            observability: ObservabilityConfig::default(),
            primal_registry: Some(crate::canonical::primals::PrimalRegistry::default()),
            custom: None,
            // Note: Legacy fields removed - use primal_registry instead
        }
    }
}

impl SongbirdConfig {
    /// Create a test configuration with sensible defaults for testing
    ///
    /// This configuration uses isolated ports and directories to avoid
    /// conflicts with other tests or production instances.
    #[must_use]
    pub fn test_defaults() -> Self {
        // Create a config with test-specific overrides using struct initialization
        Self {
            environment: "test".to_string(),
            performance: Some(PerformanceConfig {
                connection_pool_size: Some(10),
                worker_threads: Some(2),
                request_timeout_ms: Some(5000),
                ..Default::default()
            }),
            network: NetworkConfig {
                port_range: PortRange {
                    start: 19000,
                    end: 19999,
                },
                bind_address: "127.0.0.1".to_string(),
                max_connections: 100,
                enable_ipv6: false,
                ..Default::default()
            },
            security: SecurityConfig {
                enabled: false,
                ..Default::default()
            },
            discovery: DiscoveryConfig {
                interval_seconds: 5,
                ..Default::default()
            },
            observability: ObservabilityConfig {
                tracing: TracingConfig {
                    enabled: false,
                    ..Default::default()
                },
                logging: LoggingConfig {
                    level: LogLevel::Info,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Enable a primal in the universal registry
    pub fn enable_primal(&mut self, primal_name: &str, endpoint: &str) {
        if self.primal_registry.is_none() {
            self.primal_registry = Some(crate::canonical::primals::PrimalRegistry::default());
        }

        if let Some(registry) = &mut self.primal_registry {
            let mut primal_config = crate::canonical::primals::PrimalConfiguration::new_template(
                primal_name,
                &format!("{} Service", primal_name.to_uppercase()),
            );
            primal_config.endpoint.primary_url = endpoint.to_string();
            primal_config.enabled = true;

            registry.register_primal(primal_config);
        }
    }

    /// Check if a primal is enabled
    #[must_use]
    pub fn is_primal_enabled(&self, primal_name: &str) -> bool {
        self.primal_registry
            .as_ref()
            .and_then(|registry| registry.get_primal(primal_name))
            .is_some_and(|primal| primal.enabled)
    }

    /// Get primal configuration
    #[must_use]
    pub fn get_primal_config(
        &self,
        primal_name: &str,
    ) -> Option<&crate::canonical::primals::PrimalConfiguration> {
        self.primal_registry.as_ref().and_then(|registry| registry.get_primal(primal_name))
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
    #[must_use]
    pub fn get_enabled_primals(&self) -> Vec<&crate::canonical::primals::PrimalConfiguration> {
        self.primal_registry
            .as_ref()
            .map(|registry| registry.get_enabled_primals())
            .unwrap_or_default()
    }
}

/// Network configuration with zero hardcoded values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address (configurable, no hardcoded `crate::constants::network::DEFAULT_HOST`)
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

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: SafeEnv::get_or_default("SONGBIRD_BIND_ADDRESS", "0.0.0.0"),
            port_range: PortRange {
                start: SafeEnv::get_port("SONGBIRD_PORT_START", 8000),
                end: SafeEnv::get_port("SONGBIRD_PORT_END", 9000),
            },
            connection_timeout_ms: SafeEnv::get_usize("SONGBIRD_CONNECTION_TIMEOUT_MS", 30000) as u64,
            max_connections: SafeEnv::get_usize("SONGBIRD_MAX_CONNECTIONS", 1000),
            enable_ipv6: SafeEnv::get_bool("SONGBIRD_ENABLE_IPV6", true),
            tls: None,   // Configured separately if needed
            proxy: None, // Configured separately if needed
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: Option<String>,
    pub verify_client: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub proxy_url: String,
    pub bypass_list: Vec<String>,
}

/// Security configuration with comprehensive options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security features
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

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: SafeEnv::get_bool("SONGBIRD_SECURITY_ENABLED", true),
            authentication: AuthConfig::default(),
            authorization: AuthzConfig::default(),
            encryption: EncryptionConfig::default(),
            rate_limiting: RateLimitConfig::default(),
            audit_logging: AuditConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub method: AuthMethod,
    pub token_lifetime_seconds: u64,
    pub refresh_enabled: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: AuthMethod::Jwt,
            token_lifetime_seconds: 3600, // 1 hour
            refresh_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Jwt,
    OAuth2,
    ApiKey,
    Mutual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzConfig {
    pub enabled: bool,
    pub model: AuthzModel,
    pub policy_file: Option<String>,
}

impl Default for AuthzConfig {
    fn default() -> Self {
        Self {
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
pub struct EncryptionConfig {
    pub at_rest: bool,
    pub in_transit: bool,
    pub algorithm: EncryptionAlgorithm,
    pub key_rotation_days: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            at_rest: true,
            in_transit: true,
            algorithm: EncryptionAlgorithm::AES256GCM,
            key_rotation_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES256GCM,
    ChaCha20Poly1305,
    AES128GCM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_seconds: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 1000,
            burst_size: 100,
            window_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_level: AuditLevel,
    pub retention_days: u32,
    pub include_payload: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_level: AuditLevel::Info,
            retention_days: 90,
            include_payload: false, // Security best practice
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Discovery mechanism
    pub mechanism: DiscoveryMechanism,

    /// Discovery interval in seconds
    pub interval_seconds: u64,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Service registration configuration
    pub registration: RegistrationConfig,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            mechanism: DiscoveryMechanism::Dns,
            interval_seconds: 30,
            health_check: HealthCheckConfig::default(),
            registration: RegistrationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMechanism {
    Dns,
    Consul,
    Etcd,
    Kubernetes,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub retries: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/health".to_string(),
            interval_seconds: 10,
            timeout_seconds: 5,
            retries: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationConfig {
    pub auto_register: bool,
    pub service_name: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Default for RegistrationConfig {
    fn default() -> Self {
        Self {
            auto_register: true,
            service_name: SafeEnv::get_or_default("SONGBIRD_SERVICE_NAME", "songbird"),
            tags: vec!["songbird".to_string(), "primal".to_string()],
            metadata: HashMap::new(),
        }
    }
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObservabilityConfig {
    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Tracing configuration
    pub tracing: TracingConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub interval_seconds: u64,
    pub exporters: Vec<MetricsExporter>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/metrics".to_string(),
            interval_seconds: 15,
            exporters: vec![MetricsExporter::Prometheus],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricsExporter {
    Prometheus,
    StatsD,
    OpenTelemetry,
    CloudWatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub sample_rate: f64,
    pub exporters: Vec<TracingExporter>,
    pub max_span_attributes: u32,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate: 0.1, // 10% sampling
            exporters: vec![TracingExporter::Jaeger],
            max_span_attributes: 128,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingExporter {
    Jaeger,
    Zipkin,
    OpenTelemetry,
    Console,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub rotation: LogRotation,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::Stdout,
            rotation: LogRotation::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Plain,
    Structured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation {
    pub enabled: bool,
    pub max_size_mb: u64,
    pub max_files: u32,
    pub max_age_days: u32,
}

impl Default for LogRotation {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_mb: 100,
            max_files: 10,
            max_age_days: 30,
        }
    }
}
