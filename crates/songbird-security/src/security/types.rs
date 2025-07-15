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

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub authentication_enabled: bool,
    pub authorization_enabled: bool,
    pub encryption_enabled: bool,
    pub audit_logging: bool,
    pub session_timeout: Duration,
    pub max_login_attempts: u32,
    pub password_policy: PasswordPolicy,
    /// BearDog integration configuration
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

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: u32,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_age_days: 90,
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
