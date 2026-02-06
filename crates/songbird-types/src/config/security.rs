//! Security Configuration
//!
//! **CANONICAL**: Consolidated security configuration - Single Source of Truth
//!
//! This module consolidates all security configurations from across the codebase.

use crate::SafeEnv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// **CANONICAL**: Comprehensive Security Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSecurityConfig {
    /// Enable security features globally
    pub enabled: bool,
    /// Authentication configuration
    pub authentication: AuthenticationConfig,
    /// Authorization configuration
    pub authorization: AuthorizationConfig,
    /// Encryption settings
    pub encryption: EncryptionConfig,
    /// Security provider integration
    pub security_provider_integration: SecurityProviderIntegrationConfig,
}

impl Default for CanonicalSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: SafeEnv::get_bool("SONGBIRD_SECURITY_ENABLED", true),
            authentication: AuthenticationConfig::default(),
            authorization: AuthorizationConfig::default(),
            encryption: EncryptionConfig::default(),
            security_provider_integration: SecurityProviderIntegrationConfig::default(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Enable authentication
    pub enabled: bool,
    /// Authentication method
    pub method: AuthenticationMethod,
    /// Session timeout
    pub session_timeout: Duration,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: AuthenticationMethod::default(),
            session_timeout: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Authentication method enumeration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AuthenticationMethod {
    /// No authentication
    None,
    /// Basic authentication
    Basic,
    /// `OAuth2` authentication
    OAuth2,
    /// JWT authentication
    #[default]
    Jwt,
    /// Multi-factor authentication
    Mfa,
}

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    /// Enable authorization
    pub enabled: bool,
    /// Role-based access control
    pub rbac_enabled: bool,
    /// Default role
    pub default_role: String,
}

impl Default for AuthorizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rbac_enabled: true,
            default_role: "user".to_string(),
        }
    }
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption
    pub enabled: bool,
    /// Encryption algorithm
    pub algorithm: String,
    /// Key size in bits
    pub key_size: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "AES-256-GCM".to_string(),
            key_size: 256,
        }
    }
}

/// Security provider integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityProviderIntegrationConfig {
    /// Enable security provider integration
    pub enabled: bool,
    /// Provider configurations
    pub providers: HashMap<String, SecurityProviderConfig>,
}

/// Security provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProviderConfig {
    /// Provider name
    pub name: String,
    /// Provider endpoint
    pub endpoint: String,
    /// Provider credentials
    pub credentials: HashMap<String, String>,
}

impl Default for SecurityProviderConfig {
    fn default() -> Self {
        let security_host = SafeEnv::get_or_default("SECURITY_PROVIDER_HOST", "localhost");
        // EVOLVED (Feb 5, 2026): Respect SONGBIRD_TLS_ENABLED for default protocol
        // When TLS is disabled, use HTTP instead of HTTPS
        let tls_enabled = SafeEnv::get_bool("SONGBIRD_TLS_ENABLED", true);
        let (protocol, default_port) = if tls_enabled {
            ("https", 8443)
        } else {
            ("http", 8080)
        };
        let security_port = SafeEnv::get_port("SECURITY_PROVIDER_PORT", default_port);

        Self {
            name: "default".to_string(),
            endpoint: format!("{protocol}://{security_host}:{security_port}"),
            credentials: HashMap::new(),
        }
    }
}

/// Multi-factor authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaMethod {
    /// Time-based One-Time Password
    Totp,
    /// SMS verification
    Sms,
    /// Email verification
    Email,
    /// Hardware token
    Hardware,
}

impl Default for MfaMethod {
    fn default() -> Self {
        Self::Totp
    }
}

/// MFA settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSettings {
    /// Enable MFA
    pub enabled: bool,
    /// Require MFA for admin operations
    pub required_for_admin: bool,
    /// Available MFA methods
    pub methods: Vec<MfaMethod>,
}

impl Default for MfaSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            required_for_admin: true,
            methods: vec![MfaMethod::Totp],
        }
    }
}
