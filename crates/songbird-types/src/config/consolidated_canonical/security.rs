//! # Security Configuration Module
//!
//! **CANONICAL SECURITY CONFIGURATION** ✅
//!
//! This module provides security and authentication configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};
use std::env;

// ============================================================================
// SECURITY CONFIGURATION - Secure by Default
// ============================================================================

/// Security level - replaces enabled bool with explicit levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Minimal security (testing only!)
    Minimal,
    /// Standard security (production)
    Standard,
    /// Paranoid security (requires 2FA + hardware keys)
    Paranoid,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Standard // Secure by default
    }
}

impl SecurityLevel {
    /// Check if 2FA should be enforced
    pub fn requires_2fa(&self) -> bool {
        matches!(self, Self::Paranoid)
    }

    /// Check if hardware key required for admin
    pub fn requires_hardware_for_admin(&self) -> bool {
        !matches!(self, Self::Minimal)
    }

    /// Check if trust escalation is allowed
    pub fn allows_trust_escalation(&self) -> bool {
        !matches!(self, Self::Minimal)
    }
}

/// **CANONICAL**: Security and authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSecurityConfig {
    /// Security level (replaces multiple bools)
    pub security_level: SecurityLevel,

    /// Authentication method (default: "jwt")
    pub auth_method: String,

    /// TLS/SSL configuration
    pub tls: TlsConfig,

    /// Initial trust level for new connections (default: anonymous)
    pub initial_trust_level: TrustLevel,
}

/// TLS certificate generation policy - replaces multiple bools
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TlsCertPolicy {
    /// Use provided certificates only (fail if not found)
    ProvidedOnly,
    /// Auto-generate self-signed if not found
    AutoGenerate,
    /// Auto-generate with auto-detected SANs
    AutoGenerateWithSans,
}

impl Default for TlsCertPolicy {
    fn default() -> Self {
        Self::AutoGenerateWithSans // Convenience for development
    }
}

/// TLS/SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate generation policy (replaces multiple bools)
    pub cert_policy: TlsCertPolicy,

    /// Certificate path (auto-detected if not specified)
    pub cert_path: Option<String>,

    /// Private key path (auto-detected if not specified)
    pub key_path: Option<String>,

    /// Additional SANs for certificate generation
    pub additional_sans: Vec<String>,

    /// Require valid certificates for federation (default: false for LAN)
    pub require_valid_certs: bool,
}

/// Trust levels for progressive escalation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    /// No trust, anonymous only
    Anonymous,
    /// Capability-verified (can coordinate tasks)
    CapabilityVerified,
    /// Role-verified (can access registry)
    RoleVerified,
    /// Identity-verified (can see infrastructure)
    IdentityVerified,
    /// Hardware-verified (full admin access, BearDog)
    HardwareVerified,
}

impl Default for CanonicalSecurityConfig {
    fn default() -> Self {
        // Parse security level from environment
        let security_level = env::var("SONGBIRD_SECURITY_LEVEL")
            .ok()
            .and_then(|v| match v.to_lowercase().as_str() {
                "minimal" => Some(SecurityLevel::Minimal),
                "standard" => Some(SecurityLevel::Standard),
                "paranoid" => Some(SecurityLevel::Paranoid),
                _ => None,
            })
            .unwrap_or_default();

        Self {
            security_level,
            auth_method: env::var("SONGBIRD_AUTH_METHOD").unwrap_or_else(|_| "jwt".to_string()),
            tls: TlsConfig::default(),
            initial_trust_level: TrustLevel::Anonymous,
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        // Parse certificate policy from environment
        let cert_policy = env::var("SONGBIRD_TLS_CERT_POLICY")
            .ok()
            .and_then(|v| match v.to_lowercase().as_str() {
                "provided" => Some(TlsCertPolicy::ProvidedOnly),
                "auto" => Some(TlsCertPolicy::AutoGenerate),
                "auto_sans" => Some(TlsCertPolicy::AutoGenerateWithSans),
                _ => None,
            })
            .unwrap_or_default();

        Self {
            cert_policy,
            cert_path: env::var("SONGBIRD_TLS_CERT").ok(),
            key_path: env::var("SONGBIRD_TLS_KEY").ok(),
            additional_sans: env::var("SONGBIRD_TLS_SANS")
                .ok()
                .and_then(|s| {
                    let sans: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
                    if sans.is_empty() {
                        None
                    } else {
                        Some(sans)
                    }
                })
                .unwrap_or_default(),
            require_valid_certs: env::var("SONGBIRD_TLS_REQUIRE_VALID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false), // False for LAN federation
        }
    }
}
