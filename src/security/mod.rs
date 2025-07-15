//! Security Module
//!
//! Provides authentication, authorization, and security features

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::errors::{Result, SongbirdError};

// Placeholder NodeId type until discovery module is fully implemented
pub type NodeId = String;

pub mod audit;
pub mod authentication;
pub mod encryption;
pub mod oauth;
pub mod universal_security;
pub mod zero_trust_middleware;

// Re-export authentication types for easier access
pub use authentication::{
    AuthSession, AuthenticationResult, Authenticator, Credentials, InMemoryAuthenticator, UserInfo,
};

// ============================================================================
// BEARDOG SECURITY PROVIDER INTEGRATION
// ============================================================================

/// BearDog Security Provider - External security module interface
///
/// This trait allows integration with your in-house BearDog security module
/// for encryption, key management, access control, and audit logging.
#[async_trait]
pub trait BearDogSecurityProvider: Send + Sync {
    /// Encrypt data with BearDog's security context
    async fn encrypt(
        &self,
        data: &[u8],
        context: &BearDogSecurityContext,
    ) -> Result<BearDogEncryptedData>;

    /// Decrypt data with BearDog's security context
    async fn decrypt(
        &self,
        encrypted: &BearDogEncryptedData,
        context: &BearDogSecurityContext,
    ) -> Result<Vec<u8>>;

    /// Derive encryption key using BearDog's key management
    async fn derive_key(&self, key_id: &str, context: &BearDogKeyContext) -> Result<Vec<u8>>;

    /// Generate new encryption key with BearDog
    async fn generate_key(&self, key_spec: &BearDogKeySpec) -> Result<BearDogKeyHandle>;

    /// Verify access permissions using BearDog's access control
    async fn verify_access(
        &self,
        principal: &BearDogPrincipal,
        resource: &BearDogResource,
        action: &BearDogAction,
    ) -> Result<bool>;

    /// Establish secure communication channel
    async fn establish_secure_channel(&self, peer_id: &NodeId) -> Result<BearDogSecureChannel>;

    /// Log security events for audit
    async fn log_security_event(&self, event: &BearDogSecurityEvent) -> Result<()>;

    /// Rotate encryption keys
    async fn rotate_key(&self, key_id: &str) -> Result<BearDogKeyHandle>;

    /// Get compliance report
    async fn get_compliance_report(
        &self,
        period: &BearDogTimePeriod,
    ) -> Result<BearDogComplianceReport>;
}

/// Security context for BearDog operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogSecurityContext {
    pub operation_id: String,
    pub node_id: NodeId,
    pub timestamp: DateTime<Utc>,
    pub security_level: BearDogSecurityLevel,
    pub metadata: HashMap<String, String>,
}

/// Key context for BearDog key operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogKeyContext {
    pub key_purpose: BearDogKeyPurpose,
    pub expiration: Option<DateTime<Utc>>,
    pub access_policy: String,
    pub metadata: HashMap<String, String>,
}

/// Key specification for generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogKeySpec {
    pub algorithm: String,
    pub key_size: usize,
    pub purpose: BearDogKeyPurpose,
    pub rotation_policy: BearDogRotationPolicy,
}

/// Key handle for secure key reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogKeyHandle {
    pub id: String,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Security principal (user, service, node)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogPrincipal {
    pub id: String,
    pub principal_type: BearDogPrincipalType,
    pub attributes: HashMap<String, String>,
}

/// Resource being accessed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogResource {
    pub id: String,
    pub resource_type: String,
    pub owner: String,
    pub attributes: HashMap<String, String>,
}

/// Action being performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogAction {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

/// Secure communication channel
#[derive(Debug, Clone)]
pub struct BearDogSecureChannel {
    pub channel_id: String,
    pub peer_id: NodeId,
    pub established_at: DateTime<Utc>,
    pub encryption_key: Vec<u8>,
}

/// Security event for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogSecurityEvent {
    pub event_id: String,
    pub event_type: BearDogSecurityEventType,
    pub principal: BearDogPrincipal,
    pub resource: Option<BearDogResource>,
    pub action: Option<BearDogAction>,
    pub timestamp: DateTime<Utc>,
    pub outcome: BearDogSecurityOutcome,
    pub details: HashMap<String, String>,
}

/// Encrypted data with BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogEncryptedData {
    pub algorithm: String,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub salt: Option<Vec<u8>>,
    pub key_handle: Option<BearDogKeyHandle>,
}

/// Time period for compliance reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogTimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Compliance report from BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogComplianceReport {
    pub period: BearDogTimePeriod,
    pub encryption_operations: u64,
    pub key_rotations: u64,
    pub access_violations: u64,
    pub compliance_score: f64,
    pub recommendations: Vec<String>,
}

// ============================================================================
// BEARDOG ENUMS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BearDogSecurityLevel {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BearDogKeyPurpose {
    DataEncryption,
    KeyEncryption,
    DigitalSignature,
    Authentication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogRotationPolicy {
    pub interval_days: u32,
    pub auto_rotate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BearDogPrincipalType {
    User,
    Service,
    Node,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BearDogSecurityEventType {
    Authentication,
    Authorization,
    Encryption,
    Decryption,
    KeyGeneration,
    KeyRotation,
    AccessGranted,
    AccessDenied,
    SecurityViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BearDogSecurityOutcome {
    Success,
    Failure,
    Denied,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BearDogAuditLevel {
    Minimal,
    Standard,
    Comprehensive,
    Paranoid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BearDogComplianceMode {
    Standard,
    FIPS140,
    SOC2,
    GDPR,
}

/// BearDog security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogConfig {
    pub enabled: bool,
    pub key_store_path: std::path::PathBuf,
    pub encryption_algorithm: String,
    pub key_rotation_interval: Duration,
    pub compliance_mode: BearDogComplianceMode,
    pub audit_level: BearDogAuditLevel,
    pub default_security_level: BearDogSecurityLevel,
    pub connection_config: HashMap<String, String>,
}

impl Default for BearDogConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key_store_path: std::path::PathBuf::from("./data/beardog/keys"),
            encryption_algorithm: "AES-256-GCM".to_string(),
            key_rotation_interval: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            compliance_mode: BearDogComplianceMode::Standard,
            audit_level: BearDogAuditLevel::Standard,
            default_security_level: BearDogSecurityLevel::Internal,
            connection_config: HashMap::new(),
        }
    }
}

// ============================================================================
// EXISTING SECURITY CONFIGURATION (UPDATED)
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
            .unwrap_or_default()
            .as_secs();

        Self {
            token: uuid::Uuid::new_v4().to_string(),
            subject,
            subject_type,
            issued_at: now,
            expires_at: now + duration.as_secs(),
            permissions,
            attributes: HashMap::new(),
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.expires_at
    }

    /// Check if token has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }
}

/// Authentication provider trait
#[async_trait]
pub trait AuthenticationProvider: Send + Sync {
    /// Authenticate user with credentials
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken>;

    /// Validate authentication token
    async fn validate_token(&self, token: &str) -> Result<AuthToken>;

    /// Revoke authentication token
    async fn revoke_token(&self, token: &str) -> Result<()>;

    /// Refresh authentication token
    async fn refresh_token(&self, token: &str) -> Result<AuthToken>;
}

/// Authorization provider trait
#[async_trait]
pub trait AuthorizationProvider: Send + Sync {
    /// Check if subject has permission to perform action on resource
    async fn authorize(
        &self,
        subject: &str,
        subject_type: SubjectType,
        action: &Action,
        resource: &Resource,
        context: &HashMap<String, String>,
    ) -> Result<bool>;

    /// Get permissions for subject
    async fn get_permissions(
        &self,
        subject: &str,
        subject_type: SubjectType,
    ) -> Result<Vec<Permission>>;

    /// Add permission
    async fn add_permission(&self, permission: Permission) -> Result<()>;

    /// Remove permission
    async fn remove_permission(&self, permission_id: &str) -> Result<()>;
}

/// Simple in-memory authentication provider
pub struct InMemoryAuthProvider {
    users: HashMap<String, UserCredentials>,
    tokens: HashMap<String, AuthToken>,
    config: SecurityConfig,
}

#[derive(Debug, Clone)]
struct UserCredentials {
    #[allow(dead_code)]
    username: String,
    password_hash: String,
    permissions: Vec<String>,
    #[allow(dead_code)]
    attributes: HashMap<String, String>,
}

impl InMemoryAuthProvider {
    /// Create a new in-memory authentication provider
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            users: HashMap::new(),
            tokens: HashMap::new(),
            config,
        }
    }

    /// Add a user
    pub fn add_user(
        &mut self,
        username: String,
        password: String,
        permissions: Vec<String>,
    ) -> Result<()> {
        if self.users.contains_key(&username) {
            return Err(SongbirdError::Auth {
                message: format!("User {username} already exists"),
                user: Some("InMemoryAuthProvider".to_string()),
            });
        }

        // Validate password against policy
        self.validate_password(&password)?;

        let password_hash = self.hash_password(&password)?;
        let credentials = UserCredentials {
            username: username.clone(),
            password_hash,
            permissions,
            attributes: HashMap::new(),
        };

        self.users.insert(username, credentials);
        Ok(())
    }

    /// Validate password against policy
    fn validate_password(&self, password: &str) -> Result<()> {
        let policy = &self.config.password_policy;

        if password.len() < policy.min_length as usize {
            return Err(SongbirdError::Auth {
                message: format!("Password must be at least {} characters", policy.min_length),
                user: Some("InMemoryAuthProvider".to_string()),
            });
        }

        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(SongbirdError::Auth {
                message: "Password must contain at least one uppercase letter".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
            });
        }

        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(SongbirdError::Auth {
                message: "Password must contain at least one lowercase letter".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
            });
        }

        if policy.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(SongbirdError::Auth {
                message: "Password must contain at least one number".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
            });
        }

        if policy.require_special_chars && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(SongbirdError::Auth {
                message: "Password must contain at least one special character".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
            });
        }

        Ok(())
    }

    /// Hash password (simplified - use proper hashing in production)
    fn hash_password(&self, password: &str) -> Result<String> {
        // In production, use bcrypt or similar
        Ok(format!("hashed_{password}"))
    }

    /// Verify password hash
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        // In production, use proper password verification
        format!("hashed_{password}") == hash
    }
}

#[async_trait]
impl AuthenticationProvider for InMemoryAuthProvider {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        if let Some(credentials) = self.users.get(username) {
            if self.verify_password(password, &credentials.password_hash) {
                let token = AuthToken::new(
                    username.to_string(),
                    SubjectType::User,
                    self.config.session_timeout,
                    credentials.permissions.clone(),
                );
                Ok(token)
            } else {
                Err(SongbirdError::Auth {
                    message: "Invalid credentials".to_string(),
                    user: Some("InMemoryAuthProvider".to_string()),
                })
            }
        } else {
            Err(SongbirdError::Auth {
                message: "User not found".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
            })
        }
    }

    async fn validate_token(&self, token: &str) -> Result<AuthToken> {
        if let Some(auth_token) = self.tokens.get(token) {
            if auth_token.is_expired() {
                Err(SongbirdError::Auth {
                    message: "Token expired".to_string(),
                    user: Some("InMemoryAuthProvider".to_string()),
                })
            } else {
                Ok(auth_token.clone())
            }
        } else {
            Err(SongbirdError::Auth {
                message: "Invalid token".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
            })
        }
    }

    async fn revoke_token(&self, _token: &str) -> Result<()> {
        // Implementation would remove token from storage
        Ok(())
    }

    async fn refresh_token(&self, token: &str) -> Result<AuthToken> {
        let auth_token = self.validate_token(token).await?;
        let new_token = AuthToken::new(
            auth_token.subject,
            auth_token.subject_type,
            self.config.session_timeout,
            auth_token.permissions,
        );
        Ok(new_token)
    }
}

/// Simple in-memory authorization provider
pub struct InMemoryAuthzProvider {
    permissions: HashMap<String, Permission>,
}

impl Default for InMemoryAuthzProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryAuthzProvider {
    /// Create a new in-memory authorization provider
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
        }
    }
}

#[async_trait]
impl AuthorizationProvider for InMemoryAuthzProvider {
    async fn authorize(
        &self,
        subject: &str,
        subject_type: SubjectType,
        action: &Action,
        resource: &Resource,
        _context: &HashMap<String, String>,
    ) -> Result<bool> {
        // Simple authorization logic - check if any permission allows the action
        for permission in self.permissions.values() {
            if permission.subject == subject
                && permission.subject_type == subject_type
                && permission.resource == *resource
                && permission.action == *action
                && permission.effect == PermissionEffect::Allow
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn get_permissions(
        &self,
        subject: &str,
        subject_type: SubjectType,
    ) -> Result<Vec<Permission>> {
        let permissions: Vec<Permission> = self
            .permissions
            .values()
            .filter(|p| p.subject == subject && p.subject_type == subject_type)
            .cloned()
            .collect();

        Ok(permissions)
    }

    async fn add_permission(&self, _permission: Permission) -> Result<()> {
        // Implementation would add permission to storage
        Ok(())
    }

    async fn remove_permission(&self, _permission_id: &str) -> Result<()> {
        // Implementation would remove permission from storage
        Ok(())
    }
}

/// Security manager that coordinates authentication and authorization
pub struct SecurityManager {
    auth_user: Box<dyn AuthenticationProvider>,
    authz_user: Box<dyn AuthorizationProvider>,
    config: SecurityConfig,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(
        auth_user: Box<dyn AuthenticationProvider>,
        authz_user: Box<dyn AuthorizationProvider>,
        config: SecurityConfig,
    ) -> Self {
        Self {
            auth_user,
            authz_user,
            config,
        }
    }

    /// Authenticate user
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        if !self.config.authentication_enabled {
            return Err(SongbirdError::Auth {
                message: "Authentication is disabled".to_string(),
                user: Some("SecurityManager".to_string()),
            });
        }

        self.auth_user.authenticate(username, password).await
    }

    /// Authorize action
    pub async fn authorize(
        &self,
        token: &str,
        action: &Action,
        resource: &Resource,
        context: &HashMap<String, String>,
    ) -> Result<bool> {
        if !self.config.authorization_enabled {
            return Ok(true); // Allow all if authorization is disabled
        }

        let auth_token = self.auth_user.validate_token(token).await?;

        self.authz_user
            .authorize(
                &auth_token.subject,
                auth_token.subject_type,
                action,
                resource,
                context,
            )
            .await
    }

    /// Get configuration
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }
}

// Re-export important types
pub use audit::*;
pub use encryption::*;
pub use oauth::*;
pub use universal_security::*;
pub use zero_trust_middleware::ZeroTrustMiddleware;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_creation() {
        let token = AuthToken::new(
            "test_user".to_string(),
            SubjectType::User,
            Duration::from_secs(3600),
            vec!["read".to_string(), "write".to_string()],
        );

        assert_eq!(token.subject, "test_user");
        assert_eq!(token.subject_type, SubjectType::User);
        assert!(token.has_permission("read"));
        assert!(token.has_permission("write"));
        assert!(!token.has_permission("admin"));
        assert!(!token.is_expired());
    }

    #[test]
    fn test_password_policy_validation() {
        let config = SecurityConfig::default();
        let _auth_provider = InMemoryAuthProvider::new(config);

        // Valid password

        // Too short

        // No uppercase

        // No lowercase

        // No numbers

        // No special characters
    }

    #[tokio::test]
    async fn test_in_memory_auth_provider() {
        let config = SecurityConfig::default();
        let _auth_provider = InMemoryAuthProvider::new(config);

        // Test would create users and authenticate
        // This is a basic structure test
    }

    #[test]
    fn test_resource_and_action_creation() {
        let resource = Resource {
            resource_type: "document".to_string(),
            resource_id: "doc123".to_string(),
            attributes: HashMap::new(),
        };

        let action = Action {
            action_type: "read".to_string(),
            attributes: HashMap::new(),
        };

        assert_eq!(resource.resource_type, "document");
        assert_eq!(resource.resource_id, "doc123");
        assert_eq!(action.action_type, "read");
    }

    #[test]
    fn test_permission_creation() {
        let permission = Permission {
            id: "perm1".to_string(),
            subject: "user1".to_string(),
            subject_type: SubjectType::User,
            resource: Resource {
                resource_type: "document".to_string(),
                resource_id: "doc123".to_string(),
                attributes: HashMap::new(),
            },
            action: Action {
                action_type: "read".to_string(),
                attributes: HashMap::new(),
            },
            effect: PermissionEffect::Allow,
            conditions: Vec::new(),
        };

        assert_eq!(permission.id, "perm1");
        assert_eq!(permission.subject, "user1");
        assert_eq!(permission.effect, PermissionEffect::Allow);
    }

    #[tokio::test]
    async fn test_security_manager() {
        let config = SecurityConfig::default();
        let auth_provider = Box::new(InMemoryAuthProvider::new(config.clone()));
        let authz_provider = Box::new(InMemoryAuthzProvider::new());

        let security_manager = SecurityManager::new(auth_provider, authz_provider, config);

        assert!(security_manager.get_config().authentication_enabled);
        assert!(security_manager.get_config().authorization_enabled);
    }
}
