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
    pub authentication_enabled: bool,
    pub authorization_enabled: bool,
    pub encryption_enabled: bool,
    pub audit_logging: bool,
    pub session_timeout: Duration,
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
}

/// Traditional password policy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraditionalPasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    /// Minimum number of character categories required (out of 4: upper, lower, numbers, special)
    pub min_character_categories: u32,
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
            min_length: 12, // Longer for traditional passwords
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            min_character_categories: 3, // Require at least 3 of 4 categories
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
                min_character_categories: 1,
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
            validation_strategy: PasswordValidationStrategy::Traditional,
            traditional_policy: Some(TraditionalPasswordPolicy {
                min_length: 8,
                require_uppercase: true,
                require_lowercase: true,
                require_numbers: true,
                require_special_chars: true,
                min_character_categories: 4,
            }),
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SubjectType {
    User,
    Service,
    Role,
    Group,
}

/// Resource for authorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resource {
    pub resource_type: String,
    pub resource_id: String,
    pub attributes: HashMap<String, String>,
}

/// Action for authorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Action {
    pub action_type: String,
    pub attributes: HashMap<String, String>,
}

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub subject: String,
    pub subject_type: SubjectType,
    pub resource: Resource,
    pub action: Action,
    pub effect: PermissionEffect,
    pub conditions: Vec<Condition>,
}

/// Permission effect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionEffect {
    Allow,
    Deny,
}

/// Condition for permission evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub attribute: String,
    pub operator: ConditionOperator,
    pub value: String,
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    InList,
    NotInList,
}

// ============================================================================
// AUTHENTICATION TYPES
// ============================================================================

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub subject: String,
    pub subject_type: SubjectType,
    pub issued_at: u64,
    pub expires_at: u64,
    pub permissions: Vec<String>,
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
