//! Core Security Types
//!
//! Contains fundamental security types and configurations used throughout the security module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::security::beardog::BearDogConfig;

// ============================================================================
// SECURITY CONFIGURATION
// ============================================================================

/// Songbird Security Configuration
///
/// Note: This is for service orchestration security. For comprehensive security features
/// (advanced authentication, encryption, key management), integrate with BearDog primal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable user authentication
    pub authentication_enabled: bool,
    /// Enable resource authorization
    pub authorization_enabled: bool,
    /// Enable data encryption
    pub encryption_enabled: bool,
    /// Enable security audit logging
    pub audit_logging: bool,
    /// Session timeout duration
    pub session_timeout: Duration,
    /// Maximum login attempts before lockout
    pub max_login_attempts: u32,
    /// Minimal password policy - delegates to BearDog when available
    pub password_policy: PasswordPolicy,
    /// BearDog security primal integration configuration
    pub beardog: BearDogConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            authentication_enabled: true,
            authorization_enabled: true,
            encryption_enabled: true,
            audit_logging: true,
            session_timeout: Duration::from_secs(3600), // 1 hour
            max_login_attempts: 3,
            password_policy: PasswordPolicy::default(),
            beardog: BearDogConfig::default(),
        }
    }
}

/// Password policy configuration with support for both traditional and passphrase approaches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Password validation strategy - default is passphrase for humans
    pub validation_strategy: PasswordValidationStrategy,
    /// Traditional password requirements (when using Traditional strategy)
    pub traditional_policy: Option<TraditionalPasswordPolicy>,
    /// Passphrase requirements (when using Passphrase strategy)  
    pub passphrase_policy: Option<PassphrasePolicy>,
    /// Maximum password age in days
    pub max_age_days: u32,
    /// Whether to store password history to prevent reuse
    pub prevent_password_reuse: bool,
    /// Number of previous passwords to remember
    pub password_history_count: u32,
}

/// Password validation strategies for Songbird orchestrator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PasswordValidationStrategy {
    /// XKCD-style passphrase (for standalone deployments only)
    Passphrase,
    /// Traditional complex passwords (for standalone deployments only)
    Traditional,
    /// Allow either passphrase OR traditional (for migration scenarios)
    Flexible,
    /// Custom validation - defers to security authority like BearDog (recommended)
    Custom,
    /// Traditional password policy (for standalone deployments only)
    TraditionalPolicy(TraditionalPasswordPolicy),
}

/// Traditional password policy settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraditionalPasswordPolicy {
    /// Minimum password length
    pub min_length: u32,
    /// Require at least one uppercase letter
    pub require_uppercase: bool,
    /// Require at least one lowercase letter
    pub require_lowercase: bool,
    /// Require at least one number
    pub require_numbers: bool,
    /// Require at least one special character
    pub require_special_chars: bool,
}

/// Passphrase policy settings (XKCD-style: "correct horse battery staple")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassphrasePolicy {
    /// Minimum number of words (recommended: 4+)
    pub min_words: u32,
    /// Maximum number of words (to prevent excessively long passphrases)
    pub max_words: u32,
    /// Minimum total length (including spaces)
    pub min_total_length: u32,
    /// Whether to require at least one number somewhere in the passphrase
    pub require_number: bool,
    /// Whether to require at least one uppercase letter (for proper nouns, etc.)
    pub require_uppercase: bool,
    /// Common word validation - reject dictionary attacks
    pub check_common_passwords: bool,
    /// Whether to allow common words (if false, requires uncommon word combinations)
    pub allow_common_words: bool,
    /// Entropy requirements (minimum bits of entropy)
    pub min_entropy_bits: f64,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            // Default to Custom validation - defers to BearDog when available
            validation_strategy: PasswordValidationStrategy::Custom,
            traditional_policy: Some(TraditionalPasswordPolicy::default()),
            passphrase_policy: Some(PassphrasePolicy::default()),
            max_age_days: 90, // Conservative for orchestrator fallback scenarios
            prevent_password_reuse: true,
            password_history_count: 3, // Minimal for orchestrator scenarios
        }
    }
}

impl Default for TraditionalPasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: false,
        }
    }
}

impl Default for PassphrasePolicy {
    fn default() -> Self {
        Self {
            min_words: 4,                 // "correct horse battery staple" = 4 words
            max_words: 12,                // Reasonable upper bound
            min_total_length: 20,         // Including spaces
            require_number: false,        // Numbers not required for passphrase
            require_uppercase: false,     // Case not required for passphrase
            check_common_passwords: true, // Always check against breached password lists
            allow_common_words: true,     // Common words are fine in good combinations
            min_entropy_bits: 50.0,       // Good security threshold
        }
    }
}

impl PasswordPolicy {
    /// Create BearDog-integrated policy (recommended - delegates to security authority)
    pub fn beardog_integrated() -> Self {
        Self {
            validation_strategy: PasswordValidationStrategy::Custom,
            traditional_policy: None, // BearDog handles policy details
            passphrase_policy: None,  // BearDog handles policy details
            max_age_days: 90,         // Conservative fallback
            prevent_password_reuse: true,
            password_history_count: 3, // Minimal for orchestrator scenarios
        }
    }

    /// Create minimal fallback policy (when BearDog unavailable)
    pub fn orchestrator_fallback() -> Self {
        Self {
            validation_strategy: PasswordValidationStrategy::Custom,
            traditional_policy: Some(TraditionalPasswordPolicy {
                min_length: 8,
                require_uppercase: false, // Minimal requirements for fallback
                require_lowercase: false,
                require_numbers: false,
                require_special_chars: false,
            }),
            passphrase_policy: None,
            max_age_days: 30, // Short expiry to encourage proper security setup
            prevent_password_reuse: false,
            password_history_count: 0,
        }
    }

    /// Create legacy/traditional policy (for standalone deployments only)
    pub fn standalone_traditional() -> Self {
        Self {
            validation_strategy: PasswordValidationStrategy::TraditionalPolicy(
                TraditionalPasswordPolicy {
                    min_length: 12,
                    require_uppercase: true,
                    require_lowercase: true,
                    require_numbers: true,
                    require_special_chars: true,
                },
            ),
            traditional_policy: None,
            passphrase_policy: None,
            max_age_days: 90,
            prevent_password_reuse: true,
            password_history_count: 5,
        }
    }

    /// Create flexible policy that allows either approach (for migration scenarios)
    pub fn migration_flexible() -> Self {
        Self {
            validation_strategy: PasswordValidationStrategy::Flexible,
            traditional_policy: Some(TraditionalPasswordPolicy::default()),
            passphrase_policy: Some(PassphrasePolicy::default()),
            max_age_days: 60, // Shorter to encourage migration to BearDog
            prevent_password_reuse: true,
            password_history_count: 3,
        }
    }
}

// ============================================================================
// AUTHORIZATION TYPES
// ============================================================================

/// Subject types for authorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectType {
    /// Individual user account
    User,
    /// System service account
    Service,
    /// User role or group role
    Role,
    /// User group or organizational unit
    Group,
}

/// Resource being accessed in authorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    /// Type of resource (file, database, service, etc.)
    pub resource_type: String,
    /// Unique identifier for the resource
    pub resource_id: String,
    /// Additional resource attributes for context
    pub attributes: HashMap<String, String>,
}

/// Action being performed on a resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// Type of action (read, write, delete, execute, etc.)
    pub action_type: String,
    /// Additional action attributes for context
    pub attributes: HashMap<String, String>,
}

/// Permission granting or denying access to a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Unique permission identifier
    pub id: String,
    /// Subject (user/service/role) this permission applies to
    pub subject: String,
    /// Type of subject
    pub subject_type: SubjectType,
    /// Resource this permission covers
    pub resource: Resource,
    /// Action this permission allows or denies
    pub action: Action,
    /// Whether this permission allows or denies access
    pub effect: PermissionEffect,
    /// Conditions that must be met for this permission to apply
    pub conditions: Vec<Condition>,
}

/// Effect of a permission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionEffect {
    /// Grant access
    Allow,
    /// Deny access
    Deny,
}

/// Condition that must be met for a permission to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Attribute name to evaluate
    pub attribute: String,
    /// Comparison operator
    pub operator: ConditionOperator,
    /// Value to compare against
    pub value: String,
}

/// Operators for condition evaluation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionOperator {
    /// Exact equality
    Equals,
    /// Not equal
    NotEquals,
    /// String contains
    Contains,
    /// String does not contain
    NotContains,
    /// Numeric greater than
    GreaterThan,
    /// Numeric less than
    LessThan,
    /// Value is in list
    InList,
    /// Value is not in list
    NotInList,
}

// ============================================================================
// AUTHENTICATION TYPES
// ============================================================================

/// Authentication token with user information and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// The actual token string
    pub token: String,
    /// Subject (user/service) this token represents
    pub subject: String,
    /// Type of subject
    pub subject_type: SubjectType,
    /// When the token was issued (Unix timestamp)
    pub issued_at: u64,
    /// When the token expires (Unix timestamp)
    pub expires_at: u64,
    /// List of permission strings for quick access checks
    pub permissions: Vec<String>,
    /// Additional token attributes
    pub attributes: HashMap<String, String>,
}

impl AuthToken {
    /// Create a new authentication token
    pub fn new(
        subject: String,
        subject_type: SubjectType,
        duration: Duration,
        permissions: Vec<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_at = now + duration.as_secs();

        // Generate a simple token (in production, use a proper JWT library)
        let token = format!("token_{subject}_{now}");

        Self {
            token,
            subject,
            subject_type,
            issued_at: now,
            expires_at,
            permissions,
            attributes: HashMap::new(),
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }

    /// Check if token has specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }
}
