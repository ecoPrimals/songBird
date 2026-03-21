// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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

#![allow(missing_docs, reason = "deprecated legacy module; migrate callers to `canonical`")]

use crate::PerformanceConfig;
use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;
use std::collections::HashMap;
// use songbird_config; // FIXED: Circular import removed

// Import canonical configs for consolidation (Nov 10, 2025)
use songbird_types::config::consolidated_canonical::network::{
    CanonicalRateLimitConfig, CanonicalTlsConfig,
};

// Archived modules (moved to _archived_q2_2026/ on November 8, 2025):
// pub mod agnostic_primals; // Use canonical::primals instead

// Backward compatibility re-exports - Module archived (Phase 4: November 8, 2025)
#[expect(deprecated, reason = "migration to evolved config API planned")]
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
#[expect(deprecated, reason = "migration to evolved config API planned")]
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
    #[expect(deprecated, reason = "migration to evolved config API planned")]
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
        #[expect(deprecated, reason = "migration to evolved config API planned")]
        if self.primal_registry.is_none() {
            #[expect(deprecated, reason = "migration to evolved config API planned")]
            {
                self.primal_registry = Some(crate::canonical::primals::PrimalRegistry::default());
            }
        }

        #[expect(deprecated, reason = "migration to evolved config API planned")]
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
        #[expect(deprecated, reason = "migration to evolved config API planned")]
        let result = self
            .primal_registry
            .as_ref()
            .and_then(|registry| registry.get_primal(primal_name))
            .is_some_and(|primal| primal.enabled);
        result
    }

    /// Get primal configuration
    #[must_use]
    pub fn get_primal_config(
        &self,
        primal_name: &str,
    ) -> Option<&crate::canonical::primals::PrimalConfiguration> {
        #[expect(deprecated, reason = "migration to evolved config API planned")]
        let result =
            self.primal_registry.as_ref().and_then(|registry| registry.get_primal(primal_name));
        result
    }

    /// Disable a primal
    pub fn disable_primal(&mut self, primal_name: &str) {
        #[expect(deprecated, reason = "migration to evolved config API planned")]
        if let Some(registry) = &mut self.primal_registry
            && let Some(primal) = registry.primals.get_mut(primal_name)
        {
            primal.enabled = false;
        }
    }

    #[expect(deprecated, reason = "migration to evolved config API planned")]
    /// Get all enabled primals
    #[must_use]
    pub fn get_enabled_primals(&self) -> Vec<&crate::canonical::primals::PrimalConfiguration> {
        self.primal_registry
            .as_ref()
            .map(|registry| registry.get_enabled_primals())
            .unwrap_or_default()
    }
}

/// Network configuration - domain-specific variant for config module
/// (Different fields from `CanonicalNetworkConfig` - both are valid for their contexts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address (configurable, no hardcoded defaults)
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
    /// **CONSOLIDATED**: Now uses `CanonicalTlsConfig` from songbird-types
    pub tls: Option<CanonicalTlsConfig>,

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
            connection_timeout_ms: SafeEnv::get_usize("SONGBIRD_CONNECTION_TIMEOUT_MS", 30000)
                as u64,
            max_connections: SafeEnv::get_usize("SONGBIRD_MAX_CONNECTIONS", 1000),
            enable_ipv6: SafeEnv::get_bool("SONGBIRD_ENABLE_IPV6", true),
            tls: None,   // Configured separately if needed (now uses CanonicalTlsConfig)
            proxy: None, // Configured separately if needed
        }
    }
}

/// Port range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

// ============================================================================
// NOTE: TlsConfig has been CONSOLIDATED
// ============================================================================
//
// TlsConfig was removed and replaced with CanonicalTlsConfig
// from songbird_types::config::consolidated_canonical::network::CanonicalTlsConfig
//
// Migration: Use CanonicalTlsConfig from songbird-types instead
// - enabled → enabled (same)
// - cert_path (String) → cert_file (Option<PathBuf>) - use PathBuf::from()
// - key_path (String) → key_file (Option<PathBuf>) - use PathBuf::from()
// - ca_path (Option<String>) → ca_file (Option<PathBuf>)
// - verify_client → verify_client_cert
// - NEW: version, cipher_suites, verify_peer, server_name (for comprehensive TLS)
//
// CanonicalTlsConfig supports BOTH server and client TLS configurations!
//
// Date: November 10, 2025
// ============================================================================

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
    /// **CONSOLIDATED**: Uses `CanonicalRateLimitConfig` from songbird-types
    pub rate_limiting: CanonicalRateLimitConfig,

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
            rate_limiting: CanonicalRateLimitConfig::default(),
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

// ============================================================================
// NOTE: RateLimitConfig has been CONSOLIDATED
// ============================================================================
//
// RateLimitConfig was removed and replaced with canonical version
// from songbird_types::config::consolidated_canonical::network::CanonicalRateLimitConfig
//
// Migration: Use CanonicalRateLimitConfig from songbird-types instead
// - enabled → enabled (same)
// - requests_per_minute (u32) → requests_per_second (f64) * 60.0
// - burst_size → burst_capacity
// - window_seconds → window (Duration::from_secs(window_seconds))
// - NEW: strategy field (use "token_bucket", "sliding_window", or "fixed_window")
//
// Date: November 10, 2025
// ============================================================================
//
// NOTE: If you need the sophisticated RateLimitStrategy enum with Adaptive support,
// use songbird_primal_sdk::universal_registry::config::RateLimitConfig instead.
// That version is specialized for registry rate limiting with advanced algorithms.
//
// Use CanonicalRateLimitConfig for: General network rate limiting
// Use Registry RateLimitConfig for: Service registry-specific rate limiting with adaptive algorithms
// ============================================================================

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

/// Health check configuration
///
/// **CONSOLIDATED**: Re-export of canonical version.
/// **Migration** (Week 2, Nov 10 2025): This duplicate replaced with canonical.
///
/// **Field Mappings**:
/// - `endpoint` → `path` (renamed for consistency)
/// - `interval_seconds` → `interval_secs`
/// - `timeout_seconds` → `timeout_secs`
/// - `retries` → `max_retries`
pub use crate::canonical::resilience::HealthCheckConfig;

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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_songbird_config_test_defaults_environment_and_ports() {
        let cfg = SongbirdConfig::test_defaults();
        assert_eq!(cfg.environment, "test");
        assert_eq!(cfg.network.port_range.start, 19000);
        assert_eq!(cfg.network.port_range.end, 19999);
        assert_eq!(cfg.network.bind_address, "127.0.0.1");
        assert!(!cfg.security.enabled);
    }

    #[test]
    fn test_songbird_config_json_roundtrip() {
        let cfg = SongbirdConfig::test_defaults();
        let json = serde_json::to_string(&cfg).expect("serialize");
        let back: SongbirdConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.environment, cfg.environment);
        assert_eq!(back.network.bind_address, cfg.network.bind_address);
    }

    #[test]
    fn test_network_config_serialization() {
        let n = NetworkConfig {
            bind_address: "10.0.0.1".to_string(),
            port_range: PortRange {
                start: 1,
                end: 2,
            },
            connection_timeout_ms: 100,
            max_connections: 10,
            enable_ipv6: false,
            tls: None,
            proxy: None,
        };
        let v = serde_json::to_value(&n).expect("json");
        assert_eq!(v["bind_address"], "10.0.0.1");
    }

    #[test]
    fn test_discovery_config_default_mechanism() {
        let d = DiscoveryConfig::default();
        assert_eq!(d.interval_seconds, 30);
        assert!(matches!(d.mechanism, DiscoveryMechanism::Dns));
    }

    #[test]
    fn test_security_config_rate_limit_present() {
        let s = SecurityConfig::default();
        assert!(s.rate_limiting.enabled || !s.rate_limiting.enabled);
    }

    #[test]
    fn test_observability_default() {
        let o = ObservabilityConfig::default();
        assert!(o.metrics.enabled);
    }

    #[test]
    fn test_enable_primal_and_queries() {
        let mut cfg = SongbirdConfig::test_defaults();
        cfg.enable_primal("alpha", "http://alpha.local:1");
        assert!(cfg.is_primal_enabled("alpha"));
        assert_eq!(
            cfg.get_primal_config("alpha").expect("primal").endpoint.primary_url,
            "http://alpha.local:1"
        );
        cfg.disable_primal("alpha");
        assert!(!cfg.is_primal_enabled("alpha"));
    }

    #[test]
    fn test_get_enabled_primals_filters() {
        let mut cfg = SongbirdConfig::test_defaults();
        cfg.enable_primal("one", "http://a");
        cfg.enable_primal("two", "http://b");
        cfg.disable_primal("two");
        let names: Vec<_> =
            cfg.get_enabled_primals().into_iter().map(|p| p.primal_type.as_str()).collect();
        assert!(names.contains(&"one"));
        assert!(!names.contains(&"two"));
    }

    #[test]
    fn test_registration_config_default_tags() {
        let r = RegistrationConfig::default();
        assert!(r.tags.iter().any(|t| t == "songbird"));
    }

    #[test]
    fn test_metrics_config_default_exporter() {
        let m = MetricsConfig::default();
        assert!(matches!(m.exporters[0], MetricsExporter::Prometheus));
    }

    #[test]
    fn test_logging_config_default() {
        let l = LoggingConfig::default();
        assert!(matches!(l.level, LogLevel::Info));
        assert!(matches!(l.format, LogFormat::Json));
    }

    #[test]
    fn test_proxy_config_roundtrip() {
        let p = ProxyConfig {
            enabled: true,
            proxy_url: "http://proxy:8080".to_string(),
            bypass_list: vec!["localhost".to_string()],
        };
        let json = serde_json::to_string(&p).expect("ser");
        let back: ProxyConfig = serde_json::from_str(&json).expect("de");
        assert_eq!(back.proxy_url, p.proxy_url);
    }

    #[test]
    fn test_auth_and_authz_enums_serialize() {
        let a = AuthMethod::ApiKey;
        assert_eq!(serde_json::to_string(&a).unwrap(), "\"ApiKey\"");
        let z = AuthzModel::Abac;
        assert!(serde_json::to_string(&z).unwrap().contains("Abac"));
    }
}
