//! Canonical security types and levels
//!
//! Unified security configuration providing capability-based, primal-agnostic
//! security settings including authentication, encryption, and access control.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security level for services and endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// No security required
    None,
    /// Minimal security (basic validation)
    Minimal,
    /// Basic security (authentication)
    Basic,
    /// Low security level
    Low,
    /// Medium security level
    Medium,
    /// Standard security level
    Standard,
    /// Public access (default)
    #[default]
    Public,
    /// High security level
    High,
    /// Private access
    Private,
    /// Critical security
    Critical,
    /// Confidential data handling
    Confidential,
    /// Enhanced security
    Enhanced,
    /// Maximum security level
    Maximum,
    /// Classified information
    Classified,
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Minimal => write!(f, "minimal"),
            Self::Basic => write!(f, "basic"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::Standard => write!(f, "standard"),
            Self::Public => write!(f, "public"),
            Self::High => write!(f, "high"),
            Self::Private => write!(f, "private"),
            Self::Critical => write!(f, "critical"),
            Self::Confidential => write!(f, "confidential"),
            Self::Enhanced => write!(f, "enhanced"),
            Self::Maximum => write!(f, "maximum"),
            Self::Classified => write!(f, "classified"),
        }
    }
}

impl std::str::FromStr for SecurityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "minimal" => Ok(Self::Minimal),
            "basic" => Ok(Self::Basic),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "standard" => Ok(Self::Standard),
            "public" => Ok(Self::Public),
            "high" => Ok(Self::High),
            "private" => Ok(Self::Private),
            "critical" => Ok(Self::Critical),
            "confidential" => Ok(Self::Confidential),
            "enhanced" => Ok(Self::Enhanced),
            "maximum" => Ok(Self::Maximum),
            "classified" => Ok(Self::Classified),
            _ => Err(format!("Invalid security level: {s}")),
        }
    }
}

impl SecurityLevel {
    /// Get the numeric value of the security level (0-13)
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Minimal => 1,
            Self::Basic => 2,
            Self::Low => 3,
            Self::Medium => 4,
            Self::Standard => 5,
            Self::Public => 6,
            Self::High => 7,
            Self::Private => 8,
            Self::Critical => 9,
            Self::Confidential => 10,
            Self::Enhanced => 11,
            Self::Maximum => 12,
            Self::Classified => 13,
        }
    }

    /// Create from numeric value (0-13)
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Minimal),
            2 => Some(Self::Basic),
            3 => Some(Self::Low),
            4 => Some(Self::Medium),
            5 => Some(Self::Standard),
            6 => Some(Self::Public),
            7 => Some(Self::High),
            8 => Some(Self::Private),
            9 => Some(Self::Critical),
            10 => Some(Self::Confidential),
            11 => Some(Self::Enhanced),
            12 => Some(Self::Maximum),
            13 => Some(Self::Classified),
            _ => None,
        }
    }

    /// Check if this level requires authentication
    #[must_use]
    pub const fn requires_authentication(self) -> bool {
        !matches!(self, Self::None | Self::Public)
    }
}

// =============================================================================
// UNIFIED SECURITY CONFIGURATION (Merged from unified/security.rs)
// =============================================================================

/// Universal Security Configuration - Capability-Based, Primal-Agnostic
///
/// This configuration operates purely on security capabilities without
/// hardcoded knowledge of specific security providers (beardog, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UniversalSecurityConfig {
    /// Security capability requirements
    pub capability_requirements: SecurityCapabilityRequirements,

    /// Authentication configuration
    pub authentication: AuthenticationConfig,

    /// Encryption configuration
    pub encryption: EncryptionConfig,

    /// Access control configuration
    pub access_control: AccessControlConfig,

    /// Security provider discovery settings
    pub provider_discovery: ProviderDiscoveryConfig,
}

/// Security capability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCapabilityRequirements {
    /// Required encryption capabilities
    pub encryption_capabilities: Vec<String>,

    /// Required authentication capabilities
    pub authentication_capabilities: Vec<String>,

    /// Required access control capabilities
    pub access_control_capabilities: Vec<String>,

    /// Minimum security level required
    pub minimum_security_level: String,

    /// Preferred security level
    pub preferred_security_level: Option<String>,
}

impl Default for SecurityCapabilityRequirements {
    fn default() -> Self {
        Self {
            encryption_capabilities: vec![
                "aes_256".to_string(),
                "rsa_2048".to_string(),
                "tls_1_3".to_string(),
            ],
            authentication_capabilities: vec![
                "multi_factor".to_string(),
                "token_based".to_string(),
                "certificate_based".to_string(),
            ],
            access_control_capabilities: vec![
                "role_based".to_string(),
                "attribute_based".to_string(),
            ],
            minimum_security_level: "enterprise".to_string(),
            preferred_security_level: Some("quantum_resistant".to_string()),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Enable authentication
    pub enabled: bool,

    /// Preferred authentication methods (in order of preference)
    pub preferred_methods: Vec<AuthenticationMethod>,

    /// Token configuration
    pub token_config: TokenConfig,

    /// Session configuration
    pub session_config: SessionConfig,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preferred_methods: vec![
                AuthenticationMethod::BearerToken,
                AuthenticationMethod::Certificate,
                AuthenticationMethod::ApiKey,
            ],
            token_config: TokenConfig::default(),
            session_config: SessionConfig::default(),
        }
    }
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    /// Bearer token authentication
    BearerToken,
    /// API key authentication
    ApiKey,
    /// Certificate-based authentication
    Certificate,
    /// `OAuth2` authentication
    OAuth2,
    /// SAML authentication
    Saml,
    /// Custom authentication method
    Custom(String),
}

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Token expiration time in seconds
    pub expiration_secs: u64,

    /// Token refresh threshold (percentage of expiration time)
    pub refresh_threshold: f64,

    /// Enable token rotation
    pub enable_rotation: bool,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            expiration_secs: 3600,  // 1 hour
            refresh_threshold: 0.8, // 80%
            enable_rotation: true,
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout in seconds
    pub timeout_secs: u64,

    /// Enable session persistence
    pub persistent: bool,

    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions: Option<u32>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 28800, // 8 hours
            persistent: false,
            max_concurrent_sessions: Some(5),
        }
    }
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption
    pub enabled: bool,

    /// Preferred encryption algorithms (in order of preference)
    pub preferred_algorithms: Vec<EncryptionAlgorithm>,

    /// Key management configuration
    pub key_management: KeyManagementConfig,

    /// Transport encryption configuration
    pub transport: TransportEncryptionConfig,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preferred_algorithms: vec![
                EncryptionAlgorithm::Aes256Gcm,
                EncryptionAlgorithm::ChaCha20Poly1305,
                EncryptionAlgorithm::Aes256Cbc,
            ],
            key_management: KeyManagementConfig::default(),
            transport: TransportEncryptionConfig::default(),
        }
    }
}

/// Encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// AES-256-CBC
    Aes256Cbc,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
    /// RSA-2048
    Rsa2048,
    /// RSA-4096
    Rsa4096,
    /// ECC P-256
    EccP256,
    /// ECC P-384
    EccP384,
    /// Custom algorithm
    Custom(String),
}

/// Key management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementConfig {
    /// Key rotation interval in seconds
    pub rotation_interval_secs: u64,

    /// Enable automatic key rotation
    pub auto_rotation: bool,

    /// Key derivation function
    pub key_derivation: KeyDerivationFunction,

    /// Key storage backend preference
    pub storage_backend: KeyStorageBackend,
}

impl Default for KeyManagementConfig {
    fn default() -> Self {
        Self {
            rotation_interval_secs: 86400 * 30, // 30 days
            auto_rotation: true,
            key_derivation: KeyDerivationFunction::Pbkdf2,
            storage_backend: KeyStorageBackend::CapabilityBased,
        }
    }
}

/// Key derivation functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyDerivationFunction {
    /// PBKDF2
    Pbkdf2,
    /// Scrypt
    Scrypt,
    /// Argon2
    Argon2,
    /// Custom KDF
    Custom(String),
}

/// Key storage backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyStorageBackend {
    /// Use any available security capability provider
    CapabilityBased,

    /// Use hardware security module if available
    Hsm,

    /// Use secure enclave if available
    SecureEnclave,

    /// Custom storage backend
    Custom(String),
}

/// Transport encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportEncryptionConfig {
    /// Require TLS for all connections
    pub require_tls: bool,

    /// Minimum TLS version
    pub min_tls_version: TlsVersion,

    /// Preferred cipher suites
    pub preferred_cipher_suites: Vec<String>,

    /// Enable certificate pinning
    pub certificate_pinning: bool,
}

impl Default for TransportEncryptionConfig {
    fn default() -> Self {
        Self {
            require_tls: true,
            min_tls_version: TlsVersion::Tls13,
            preferred_cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_AES_128_GCM_SHA256".to_string(),
            ],
            certificate_pinning: false,
        }
    }
}

/// TLS versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TlsVersion {
    /// TLS 1.2
    Tls12,
    /// TLS 1.3
    Tls13,
}

/// Access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// Enable access control
    pub enabled: bool,

    /// Default access policy
    pub default_policy: AccessPolicy,

    /// Role-based access control configuration
    pub rbac: RbacConfig,

    /// Attribute-based access control configuration
    pub abac: AbacConfig,
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_policy: AccessPolicy::Deny,
            rbac: RbacConfig::default(),
            abac: AbacConfig::default(),
        }
    }
}

/// Access policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPolicy {
    /// Allow access
    Allow,
    /// Deny access
    Deny,
    /// Conditional access based on attributes
    Conditional(HashMap<String, String>),
}

/// Role-based access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    /// Enable RBAC
    pub enabled: bool,

    /// Default roles
    pub default_roles: Vec<String>,

    /// Role hierarchy
    pub role_hierarchy: HashMap<String, Vec<String>>,
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_roles: vec!["user".to_string(), "admin".to_string(), "system".to_string()],
            role_hierarchy: HashMap::from([
                ("admin".to_string(), vec!["user".to_string()]),
                ("system".to_string(), vec!["admin".to_string(), "user".to_string()]),
            ]),
        }
    }
}

/// Attribute-based access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacConfig {
    /// Enable ABAC
    pub enabled: bool,

    /// Policy evaluation engine
    pub policy_engine: PolicyEngine,

    /// Attribute sources
    pub attribute_sources: Vec<AttributeSource>,
}

impl Default for AbacConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default as it's more complex
            policy_engine: PolicyEngine::Simple,
            attribute_sources: vec![
                AttributeSource::User,
                AttributeSource::Resource,
                AttributeSource::Environment,
            ],
        }
    }
}

/// Policy engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyEngine {
    /// Simple policy evaluation
    Simple,
    /// XACML policy engine
    Xacml,
    /// Open Policy Agent
    Opa,
    /// Custom policy engine
    Custom(String),
}

/// Attribute sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeSource {
    /// User attributes
    User,
    /// Resource attributes
    Resource,
    /// Environment attributes
    Environment,
    /// External attribute source
    External(String),
}

/// Provider discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDiscoveryConfig {
    /// Enable automatic discovery of security providers
    pub auto_discovery: bool,

    /// Discovery interval in seconds
    pub discovery_interval_secs: u64,

    /// Health check interval in seconds
    pub health_check_interval_secs: u64,

    /// Provider selection strategy
    pub selection_strategy: ProviderSelectionStrategy,

    /// Fallback configuration
    pub fallback: FallbackConfig,
}

impl Default for ProviderDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_discovery: true,
            discovery_interval_secs: 60,
            health_check_interval_secs: 30,
            selection_strategy: ProviderSelectionStrategy::BestCapability,
            fallback: FallbackConfig::default(),
        }
    }
}

/// Provider selection strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderSelectionStrategy {
    /// Select provider with best matching capabilities
    BestCapability,

    /// Select fastest responding provider
    FastestResponse,

    /// Load balance across available providers
    LoadBalance,

    /// Use first available provider
    FirstAvailable,
}

/// Fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Enable fallback to built-in security implementations
    pub enable_builtin_fallback: bool,

    /// Fallback timeout in seconds
    pub fallback_timeout_secs: u64,

    /// Maximum fallback attempts
    pub max_fallback_attempts: u32,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enable_builtin_fallback: true,
            fallback_timeout_secs: 30,
            max_fallback_attempts: 3,
        }
    }
}

#[cfg(test)]
#[path = "security_tests.rs"]
mod tests;
