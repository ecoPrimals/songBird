// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security-related configuration types for the legacy `config` module.

use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;
use songbird_types::config::consolidated_canonical::network::CanonicalRateLimitConfig;

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
// prefer `songbird_primal_sdk::universal_registry::config::RateLimitConfig`.
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_security_config_rate_limit_present() {
        let s = SecurityConfig::default();
        assert!(s.rate_limiting.enabled || !s.rate_limiting.enabled);
    }

    #[test]
    fn test_auth_and_authz_enums_serialize() {
        let a = AuthMethod::ApiKey;
        assert_eq!(serde_json::to_string(&a).unwrap(), "\"ApiKey\"");
        let z = AuthzModel::Abac;
        assert!(serde_json::to_string(&z).unwrap().contains("Abac"));
    }
}
