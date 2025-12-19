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

/// **CANONICAL**: Security and authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSecurityConfig {
    /// Enable security features (default: true, failsafe)
    pub enabled: bool,
    
    /// Authentication method (default: "jwt")
    pub auth_method: String,
    
    /// TLS/SSL configuration
    pub tls: TlsConfig,
    
    /// Progressive trust escalation enabled (default: true)
    pub trust_escalation_enabled: bool,
    
    /// Initial trust level for new connections (default: anonymous)
    pub initial_trust_level: TrustLevel,
    
    /// Require hardware key for admin operations (default: true)
    pub require_hardware_for_admin: bool,
    
    /// Enable 2FA (default: true in production)
    pub enable_2fa: bool,
}

/// TLS/SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS (default: true, failsafe)
    pub enabled: bool,
    
    /// Auto-generate self-signed certificates if not found (default: true)
    pub auto_generate_certs: bool,
    
    /// Certificate path (auto-detected if not specified)
    pub cert_path: Option<String>,
    
    /// Private key path (auto-detected if not specified)
    pub key_path: Option<String>,
    
    /// Auto-detect SANs from hostname and local IP (default: true)
    pub auto_sans: bool,
    
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
        Self {
            enabled: env::var("SONGBIRD_SECURITY_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            auth_method: env::var("SONGBIRD_AUTH_METHOD")
                .unwrap_or_else(|_| "jwt".to_string()),
            tls: TlsConfig::default(),
            trust_escalation_enabled: env::var("SONGBIRD_TRUST_ESCALATION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            initial_trust_level: TrustLevel::Anonymous,
            require_hardware_for_admin: env::var("SONGBIRD_REQUIRE_HARDWARE_ADMIN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            enable_2fa: env::var("SONGBIRD_ENABLE_2FA")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
        }
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            // TLS enabled by default (failsafe)
            enabled: env::var("SONGBIRD_TLS_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            auto_generate_certs: env::var("SONGBIRD_TLS_AUTO_GENERATE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            cert_path: env::var("SONGBIRD_TLS_CERT").ok(),
            key_path: env::var("SONGBIRD_TLS_KEY").ok(),
            auto_sans: env::var("SONGBIRD_TLS_AUTO_SANS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            additional_sans: env::var("SONGBIRD_TLS_SANS")
                .ok()
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            require_valid_certs: env::var("SONGBIRD_TLS_REQUIRE_VALID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false), // False for LAN federation
        }
    }
}
