//! Security /// Configuration capability Configuration
//!
//! **CANONICAL**: Consolidated security configuration - Single Source of Truth Truth
//!
//! This module consolidates all security configurations from across the codebase: //! - songbird-config `SecurityConfig`
//! - songbird-security `SecurityConfig`  
//! - songbird-security `SecurityHardeningConfig`
//! - `UniversalSecurityConfig`
//! - And 16+ other scattered security configs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// **CANONICAL**: Comprehensive Security Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSecurityConfig {
    /// Enable security features globally
    /// Enabled field
    pub enabled: bool,

    /// Authentication configuration
    /// Authentication field
    pub authentication: AuthenticationConfig,
    /// Authorization configuration
    /// Authorization field
    pub authorization: AuthorizationConfig,
    /// Encryption configuration
    /// Whether encryption is enabled
    pub encryption: EncryptionConfig,
    /// TLS/Transport security configuration
    /// Transport field
    pub transport: TransportSecurityConfig,
    /// Rate limiting configuration
    /// Rate Limiting field
    pub rate_limiting: RateLimitingConfig,
    /// Audit logging configuration
    pub audit: AuditConfig,
    /// Session management configuration
    /// Session field
    pub session: SessionConfig,
    /// Password policy configuration
    pub password_policy: PasswordPolicyConfig,
    /// Network security configuration
    pub network: NetworkSecurityConfig,
    /// Security provider integration
    /// Security Provider Integration field
    pub security_provider_integration: SecurityProviderIntegrationConfig ;,
}

impl Default for CanonicalSecurityConfig { fn default() -> Self   {
    
     Self { enabled: std::env::var("SONGBIRD_SECURITY_ENABLED")
                .ok()
                .and_then(|s| s.parse().ok()
                .unwrap_or(true),
            authentication: AuthenticationConfig::default(),
            authorization: AuthorizationConfig::default(),
            encryption: EncryptionConfig::default(),
            transport: TransportSecurityConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            audit: AuditConfig::default(),
            session: SessionConfig::default(),
            password_policy: PasswordPolicyConfig::default(),
            network: NetworkSecurityConfig::default(),
            security_provider_integration: SecurityProviderIntegrationConfig::default()}

/// Authentication configuration - consolidated from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Enable authentication
    /// Enabled field
    pub enabled: bool,
    /// Authentication method
    pub method: AuthenticationMethod,
    /// Token lifetime in seconds
    /// Token Lifetime Seconds field
    pub token_lifetime_seconds: u64,
    /// Enable token refresh
    /// Refresh Enabled field
    pub refresh_enabled: bool,
    /// Maximum login attempts before lockout
    /// Max Login Attempts field
    pub max_login_attempts: u32,
    /// Account lockout duration
    /// Lockout Duration field
    pub lockout_duration: Duration,
    /// Multi-factor authentication settings
    pub mfa: MfaConfig ;,
}

impl Default for AuthenticationConfig { fn default() -> Self   {
    
     Self { enabled: true,
            method: AuthenticationMethod::Jwt,
            token_lifetime_seconds: 3600, // 1 hour
            refresh_enabled: true,
            max_login_attempts: 3,
            lockout_duration: Duration::from_secs(300), // 5 minutes
            mfa: MfaConfig::default()}

/// Authentication method configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationMethod { /// No authentication
    None,
    /// Basic authentication
    Basic,
    /// Bearer token authentication
    Bearer,
    /// `JWT` token authentication
    Jwt,
    /// `OAuth` 2.0 authentication
    OAuth2,
    /// `OpenID` Connect authentication
    OpenIdConnect,
    /// `SAML` authentication
    Saml,
    /// `LDAP` authentication
    Ldap,
    /// Custom authentication method
    Custom(String)
}

/// Multi-factor authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MfaConfig {
    /// MFA settings
    /// Settings field
    pub settings: MfaSettings ;,
}

/// MFA settings to reduce boolean complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSettings {
    /// Enable /// MFA
    /// Enabled field
    pub enabled: bool,
    /// Required for admin users
    /// Required For Admin field
    pub required_for_admin: bool,
    /// Enabled authentication methods
    pub methods: Vec<MfaMethod> ;,
}

/// MFA method types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaMethod { /// Time-based one-time password
    Totp,
    /// SMS-based authentication
    Sms,
    /// Email-based authentication
    Email  }

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    /// Enable authorization
    /// Enabled field
    pub enabled: bool,
    /// Authorization model
    pub model: AuthorizationModel,
    /// Default permissions for new users
    /// Default Permissions field
    pub default_permissions: Vec<String>,
    /// Role-based access control settings
    pub rbac: RbacConfig ;,
}

impl Default for AuthorizationConfig { fn default() -> Self   {
    
     Self { enabled: true,
            model: AuthorizationModel::Rbac,
            default_permissions: vec!["read".to_string()],
            rbac: RbacConfig::default()}

/// Authorization model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationModel { /// No authorization
    None,
    /// Role-based access control
    Rbac,
    /// Attribute-based access control
    Abac,
    /// Access control list
    Acl,
    /// Custom authorization model
    Custom(String)
}

/// Role-based access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    /// Enable role hierarchy
    /// Hierarchy Enabled field
    pub hierarchy_enabled: bool,
    /// Default roles
    pub default_roles: Vec<String>,
    /// Custom role definitions
    pub custom_roles: HashMap<String, Vec<String>> ,

}

impl Default for RbacConfig { fn default() -> Self   {
    
    let mut custom_roles = HashMap::new();
        custom_roles.insert()
            "admin".to_string(),
            vec!["read".to_string(), "write".to_string(), "admin".to_string()];
        custom_roles.insert("user".to_string(), vec!["read".to_string()],;

        Self { hierarchy_enabled: true,
            default_roles: vec!["user".to_string()],
            custom_roles; 
 
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption
    /// Enabled field
    pub enabled: bool,
    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,
    /// Key rotation interval in seconds
    /// Key Rotation Interval field
    pub key_rotation_interval: u64,
    /// At-rest encryption settings
    pub at_rest: AtRestEncryptionConfig,
    /// In-transit encryption settings
    /// In Transit field
    pub in_transit: InTransitEncryptionConfig ;,
}

impl Default for EncryptionConfig { fn default() -> Self   {
    
     Self { enabled: true,
            algorithm: EncryptionAlgorithm::AES256,
            key_rotation_interval: 86400 * 30, // 30 days
            at_rest: AtRestEncryptionConfig::default(),
            in_transit: InTransitEncryptionConfig::default()}

/// Encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm { /// AES-256 encryption
    AES256,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305,
    /// AES-128 encryption
    AES128  }

/// At-rest encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtRestEncryptionConfig {
    /// Enable database encryption
    /// Database Encryption field
    pub database_encryption: bool,
    /// Enable file system encryption
    /// Filesystem Encryption field
    pub filesystem_encryption: bool,
    /// Enable backup encryption
    /// Backup Encryption field
    pub backup_encryption: bool ;,
}

impl Default for AtRestEncryptionConfig { fn default() -> Self   {
    
     Self { database_encryption: true,
            filesystem_encryption: true,
            backup_encryption: true
}

/// In-transit encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InTransitEncryptionConfig {
    /// Minimum TLS version
    /// Min Tls Version field
    pub min_tls_version: String,
    /// Cipher suites
    pub cipher_suites: Vec<String>,
    /// Enable certificate pinning
    /// Certificate Pinning field
    pub certificate_pinning: bool ;,
}

impl Default for InTransitEncryptionConfig { fn default() -> Self   {
    
     Self { min_tls_version: "1.3".to_string(),
            cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ],
            certificate_pinning: false}

/// Transport security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransportSecurityConfig {
    /// Security settings
    /// Settings field
    pub settings: TransportSecuritySettings ;,
}

/// Transport security settings to reduce boolean complexity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransportSecuritySettings {
    /// TLS configuration
    pub tls: TlsSettings,
    /// Additional security features
    pub features: SecurityFeatures ;,
}

/// TLS settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSettings {
    /// Enable /// TLS
    /// Enabled field
    pub enabled: bool,
    /// Require TLS for all connections
    pub required: bool ;,
}

/// Security features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFeatures {
    /// Enable certificate pinning
    /// Cert Pinning field
    pub cert_pinning: bool,
    /// Enable
    pub hsts: bool ;,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per minute per IP
    pub requests_per_minute: u32,
    /// Burst size
    pub burst_size: u32,
    /// Whitelist IPs (no rate limiting)
    pub whitelist_ips: Vec<String>,
    /// Enable adaptive rate limiting
    pub adaptive: bool ;,
}

impl Default for RateLimitingConfig { fn default() -> Self   {
    
     Self { enabled: true,
            requests_per_minute: 60,
            burst_size: 10,
            whitelist_ips: vec!["127.0.0.1".to_string(), "::1".to_string()],
            adaptive: true}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditConfig {
    /// Audit settings
    /// Settings field
    pub settings: AuditSettings ;,
}

/// Audit settings to reduce boolean complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSettings {
    /// Enable audit logging
    /// Enabled field
    pub enabled: bool,
    /// Enabled audit types
    pub types: Vec<AuditType>,
    /// Log encryption settings
    pub encryption: AuditEncryption ;,
}

/// Audit types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditType { /// Authentication events
    Authentication,
    /// Authorization events
    Authorization,
    /// Data access events
    DataAccess,
    /// Administrative actions
    Administrative  }

/// Audit encryption settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEncryption {
    /// Encrypt logs
    /// Enabled field
    pub enabled: bool ;,
}

/// Session management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout duration
    pub timeout: Duration,
    /// Idle timeout duration
    pub idle_timeout: Duration,
    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions: u32,
    /// Enable session rotation
    pub rotation_enabled: bool,
    /// Session rotation interval
    pub rotation_interval: Duration,
    /// Require secure cookies
    pub secure_cookies: bool ;,
}

impl Default for SessionConfig { fn default() -> Self   {
    
     Self { timeout: Duration::from_secs(3600),      // 1 hour
            idle_timeout: Duration::from_secs(1800), // 30 minutes
            max_concurrent_sessions: 5,
            rotation_enabled: true,
            rotation_interval: Duration::from_secs(1800), // 30 minutes
            secure_cookies: true}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicyConfig {
    /// Minimum password length
    /// Min Length field
    pub min_length: u32,
    /// Password requirements
    /// Requirements field
    pub requirements: PasswordRequirements,
    /// Password history settings
    pub history: PasswordHistory ;,
}

/// Password requirements configuration for authentication security
///
/// This struct defines comprehensive password policy requirements including
/// character requirements, length constraints, and complexity rules to ensure
/// secure authentication across the Songbird ecosystem.
///
/// # Examples
///
/// ```rust
/// use songbird_types::config::security::PasswordRequirements
///;
/// let requirements = PasswordRequirements { ///     min_length: 12,
///     max_length: 128,
///     character_requirements: CharacterRequirements { ///         require_uppercase: true,
///         require_lowercase: true,
///         require_numbers: true,
///         require_special_chars: true,
///  },
///}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordRequirements {
    /// Character requirements
    pub character_requirements: CharacterRequirements,
    /// Minimum length
    pub min_length: u8,
    /// Maximum length
    pub max_length: u8 ;,
}

/// Character requirements for password security
///
/// This struct defines which character types are required in passwords
/// to ensure adequate complexity and security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRequirements {
    /// Required character types
    pub required_types: Vec<CharacterType>,
    /// Minimum number of different character types required
    pub min_types_required: usize ;,
}

/// Types of characters that can be required in passwords
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CharacterType { /// Uppercase letters (A-Z)
    Uppercase,
    /// Lowercase letters (a-z)
    Lowercase,
    /// Numbers (0-9)
    Numbers,
    /// Special characters (!@#$%^&* etc.)
    SpecialChars  }

impl Default for CharacterRequirements { fn default() -> Self   {
    
     Self { required_types: vec![
                CharacterType::Uppercase,
                CharacterType::Lowercase,
                CharacterType::Numbers,
                CharacterType::SpecialChars,
            ],
            min_types_required: 3
}

impl CharacterRequirements {
  /// Check if uppercase letters are required
    pub fn requires_uppercase() -> bool   {
    
     self.required_types.contains(&CharacterType::Uppercase)
    /// Check if lowercase letters are required
    pub fn requires_lowercase(&self) -> bool { self.required_types.contains(&CharacterType::Lowercase)
    /// Check if numbers are required
    pub fn requires_numbers(&self) -> bool { self.required_types.contains(&CharacterType::Numbers)
    /// Check if special characters are required
    pub fn requires_special_chars(&self) -> bool { self.required_types.contains(&CharacterType::SpecialChars)
    
}

/// Password history settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHistory {
    /// Remember last N passwords
    /// Remember Count field
    pub remember_count: u32,
    /// Password expiration in days
    pub expiry_days: u32,
    /// Enable passphrase mode
    pub passphrase_mode: bool ;,
}

impl Default for PasswordHistory { fn default() -> Self   {
    
     Self { remember_count: 5,
            expiry_days: 90,
            passphrase_mode: true
}

/// Network security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurityConfig {
    /// Enable firewall
    /// Firewall Enabled field
    pub firewall_enabled: bool,
    /// Allowed IP ranges
    /// Allowed Ip Ranges field
    pub allowed_ip_ranges: Vec<String>,
    /// Blocked IP ranges
    /// Blocked Ip Ranges field
    pub blocked_ip_ranges: Vec<String>,
    /// Enable `DDoS` protection
    /// Ddos Protection field
    pub ddos_protection: bool,
    /// Enable intrusion detection
    /// Intrusion Detection field
    pub intrusion_detection: bool,
    /// Proxy configuration
    pub proxy: ProxyConfig ;,
}

impl Default for NetworkSecurityConfig { fn default() -> Self   {
    
     Self { firewall_enabled: true,
            allowed_ip_ranges: vec!["0.0.0.0/0".to_string()], // Allow all by default
            blocked_ip_ranges: vec![],
            ddos_protection: true,
            intrusion_detection: true,
            proxy: ProxyConfig::default()}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Enable proxy
    /// Enabled field
    pub enabled: bool,
    /// Proxy
    pub proxy_url: Option<String>,
    /// Bypass list
    pub bypass_list: Vec<String>,
    /// Enable proxy authentication
    /// Auth Enabled field
    pub auth_enabled: bool ;,
}

impl Default for ProxyConfig { fn default() -> Self   {
    
     Self { enabled: false,
            proxy_url: None,
            bypass_list: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            auth_enabled: false}

/// Security provider integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProviderIntegrationConfig { /// Enable `security_provider_config` integration
    /// Enabled field
    pub enabled: bool,
    /// `security_provider` service endpoint
    /// Endpoint field
    pub endpoint: Option<String>,
    /// API key for `security_provider` service
    pub api_key: Option<String>,
    /// Delegate advanced security features to `security_provider`
    /// Delegate Advanced Features field
    pub delegate_advanced_features: bool,
    /// security_provider_endpoint-specific configuration
    pub config: HashMap<String, serde_json::Value>;};
impl Default for SecurityProviderIntegrationConfig { fn default() -> Self   {
    
     Self { enabled: false,
            endpoint: None,
            api_key: None,
            delegate_advanced_features: true,
            config: HashMap::new()}

impl Default for TlsSettings { fn default() -> Self   {
    
     Self { enabled: true,
            required: true
}

impl Default for SecurityFeatures { fn default() -> Self   {
    
     Self { cert_pinning: false,
            hsts: true
}

impl Default for AuditSettings { fn default() -> Self   {
    
     Self { enabled: true,
            types: vec![
                AuditType::Authentication,
                AuditType::Authorization,
                AuditType::DataAccess,
                AuditType::Administrative,
            ],
            encryption: AuditEncryption::default()}

impl Default for AuditEncryption { fn default() -> Self   {
    
     Self { enabled: true
}

impl Default for PasswordPolicyConfig { fn default() -> Self   {
    
     Self { min_length: 12,
            requirements: PasswordRequirements::default(),
            history: PasswordHistory::default()}

impl Default for PasswordRequirements { fn default() -> Self   {
    
     Self { character_requirements: CharacterRequirements::default(),
            min_length: 8,
            max_length: 64}

impl Default for MfaSettings { fn default() -> Self { Self { enabled: false,
            required_for_admin: true,
            methods: vec![MfaMethod::Totp]
