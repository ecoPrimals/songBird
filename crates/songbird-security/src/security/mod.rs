//! Security Module
//!
//! Provides authentication, authorization, and security features

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use songbird_errors::{AuthError, NetworkError, Result};

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
    pub security_level: BearDogSecurityLevel,
    pub use_bstp: bool,
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
    pub key_id: String,
    pub algorithm: String,
    pub created_at: SystemTime,
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
    pub data: Vec<u8>,
    pub algorithm: String,
    pub key_id: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogSecurityLevel {
    Standard,
    Public,
    Internal,
    High,
    Confidential,
    Secret,
    TopSecret,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogPrincipalType {
    User,
    Device,
    Service,
    Node,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogSecurityOutcome {
    Success,
    Failure,
    Denied,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogAuditLevel {
    Minimal,
    Standard,
    Detailed,
    Comprehensive,
    Paranoid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogComplianceMode {
    Standard,
    Strict,
    FIPS140,
    SOC2,
    GDPR,
}

/// BearDog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogConfig {
    pub endpoint: String,
    pub api_key: String,
    pub security_level: BearDogSecurityLevel,
    pub audit_level: BearDogAuditLevel,
    pub compliance_mode: BearDogComplianceMode,
    pub metadata: HashMap<String, String>,
}

impl Default for BearDogConfig {
    fn default() -> Self {
        Self {
            endpoint: songbird_config::config::constants::network::DEFAULT_BEARDOG_ENDPOINT
                .to_string(),
            api_key: "your_api_key".to_string(),
            security_level: BearDogSecurityLevel::Internal,
            audit_level: BearDogAuditLevel::Standard,
            compliance_mode: BearDogComplianceMode::Standard,
            metadata: HashMap::new(),
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: format!("User {} already exists", username),
                user: Some("InMemoryAuthProvider".to_string()),
                provider: Some("InMemoryAuthProvider".to_string()),
                suggestion: Some("Use a different username".to_string()),
            })));
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
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: format!("Password must be at least {} characters", policy.min_length),
                user: Some("InMemoryAuthProvider".to_string()),
                provider: Some("InMemoryAuthProvider".to_string()),
                suggestion: Some("Use a longer password".to_string()),
            })));
        }

        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "Password must contain at least one uppercase letter".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
                provider: Some("InMemoryAuthProvider".to_string()),
                suggestion: Some("Add an uppercase letter to your password".to_string()),
            })));
        }

        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "Password must contain at least one lowercase letter".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
                provider: Some("InMemoryAuthProvider".to_string()),
                suggestion: Some("Add a lowercase letter to your password".to_string()),
            })));
        }

        if policy.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "Password must contain at least one number".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
                provider: Some("InMemoryAuthProvider".to_string()),
                suggestion: Some("Add a number to your password".to_string()),
            })));
        }

        if policy.require_special_chars && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "Password must contain at least one special character".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
                provider: Some("InMemoryAuthProvider".to_string()),
                suggestion: Some("Add a special character to your password".to_string()),
            })));
        }

        Ok(())
    }

    /// Hash password (simplified - use proper hashing in production)
    fn hash_password(&self, password: &str) -> Result<String> {
        // In production, use bcrypt or similar
        Ok(format!("hashed_{}", password))
    }

    /// Verify password hash
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        // In production, use proper password verification
        format!("hashed_{}", password) == hash
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
                Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                    message: "Invalid credentials".to_string(),
                    user: Some("InMemoryAuthProvider".to_string()),
                    provider: Some("InMemoryAuthProvider".to_string()),
                    suggestion: Some("Check your username and password".to_string()),
                })))
            }
        } else {
            Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "User not found".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
                provider: Some("InMemoryAuthProvider".to_string()),
                suggestion: Some("Check your username or register a new account".to_string()),
            })))
        }
    }

    async fn validate_token(&self, token: &str) -> Result<AuthToken> {
        if let Some(auth_token) = self.tokens.get(token) {
            if auth_token.is_expired() {
                Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                    message: "Token expired".to_string(),
                    user: Some("InMemoryAuthProvider".to_string()),
                    provider: Some("InMemoryAuthProvider".to_string()),
                    suggestion: Some("Please re-authenticate to get a new token".to_string()),
                })))
            } else {
                Ok(auth_token.clone())
            }
        } else {
            Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "Invalid token".to_string(),
                user: Some("InMemoryAuthProvider".to_string()),
                provider: Some("InMemoryAuthProvider".to_string()),
                suggestion: Some("Please provide a valid authentication token".to_string()),
            })))
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
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "Authentication is disabled".to_string(),
                user: Some("SecurityManager".to_string()),
                provider: Some("SecurityManager".to_string()),
                suggestion: Some("Enable authentication in configuration".to_string()),
            })));
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
            tracing::error!("SECURITY CRITICAL: Authorization disabled - this should only be used in development!");
            if std::env::var("SONGBIRD_ENV").unwrap_or_default() != "development" {
                return Err(songbird_errors::SongbirdError::Network(Box::new(
                    NetworkError {
                        service: Some("security".to_string()),
                        message: "Authorization cannot be disabled in production".to_string(),
                        details: None,
                        endpoint: Some("security/authorize".to_string()),
                        suggestion: Some(
                            "Enable authorization in production configuration".to_string(),
                        ),
                    },
                )));
            }
            return Ok(false); // Explicit deny in production
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
        let auth_provider = InMemoryAuthProvider::new(config);

        // Valid password
        assert!(auth_provider.validate_password("SecurePass123!").is_ok());

        // Too short
        assert!(auth_provider.validate_password("Short1!").is_err());

        // No uppercase
        assert!(auth_provider.validate_password("lowercase123!").is_err());

        // No lowercase
        assert!(auth_provider.validate_password("UPPERCASE123!").is_err());

        // No numbers
        assert!(auth_provider.validate_password("NoNumbers!").is_err());

        // No special characters
        assert!(auth_provider.validate_password("NoSpecial123").is_err());
    }

    #[tokio::test]
    async fn test_in_memory_auth_provider() {
        let config = SecurityConfig::default();
        let mut auth_provider = InMemoryAuthProvider::new(config);

        // Add a test user
        let result = auth_provider.add_user(
            "testuser".to_string(),
            "SecurePass123!".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );
        assert!(result.is_ok());

        // Test authentication with correct credentials
        let token_result = auth_provider
            .authenticate("testuser", "SecurePass123!")
            .await;
        assert!(token_result.is_ok());
        let token = token_result.unwrap();
        assert_eq!(token.subject, "testuser");
        assert!(token.has_permission("read"));
        assert!(token.has_permission("write"));

        // Test authentication with wrong password
        let wrong_password_result = auth_provider
            .authenticate("testuser", "WrongPassword")
            .await;
        assert!(wrong_password_result.is_err());

        // Test authentication with non-existent user
        let no_user_result = auth_provider.authenticate("nouser", "password").await;
        assert!(no_user_result.is_err());
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

    #[tokio::test]
    async fn test_authentication_flow_comprehensive() {
        let config = SecurityConfig::default();
        let mut auth_provider = InMemoryAuthProvider::new(config);

        // Add multiple users with different permissions
        assert!(auth_provider
            .add_user(
                "admin".to_string(),
                "AdminPass123!".to_string(),
                vec!["read".to_string(), "write".to_string(), "admin".to_string()],
            )
            .is_ok());

        assert!(auth_provider
            .add_user(
                "user".to_string(),
                "UserPass123!".to_string(),
                vec!["read".to_string()],
            )
            .is_ok());

        // Test admin authentication
        let admin_token = auth_provider
            .authenticate("admin", "AdminPass123!")
            .await
            .unwrap();
        assert!(admin_token.has_permission("admin"));
        assert!(admin_token.has_permission("read"));
        assert!(admin_token.has_permission("write"));

        // Test user authentication
        let user_token = auth_provider
            .authenticate("user", "UserPass123!")
            .await
            .unwrap();
        assert!(user_token.has_permission("read"));
        assert!(!user_token.has_permission("write"));
        assert!(!user_token.has_permission("admin"));

        // Test duplicate user creation
        let duplicate_result = auth_provider.add_user(
            "admin".to_string(),
            "NewPass123!".to_string(),
            vec!["read".to_string()],
        );
        assert!(duplicate_result.is_err());
    }

    #[tokio::test]
    async fn test_token_validation_and_expiration() {
        let config = SecurityConfig::default();
        let mut auth_provider = InMemoryAuthProvider::new(config);

        // Add test user
        auth_provider
            .add_user(
                "testuser".to_string(),
                "SecurePass123!".to_string(),
                vec!["read".to_string()],
            )
            .unwrap();

        // Authenticate and get token
        let _token = auth_provider
            .authenticate("testuser", "SecurePass123!")
            .await
            .unwrap();

        // Create a fake token string for validation test
        let fake_token = "invalid_token_123";
        let validation_result = auth_provider.validate_token(fake_token).await;
        assert!(validation_result.is_err());

        // Test token refresh
        let refresh_result = auth_provider.refresh_token(fake_token).await;
        assert!(refresh_result.is_err());

        // Test token revocation (should succeed even with fake token in this implementation)
        let revoke_result = auth_provider.revoke_token(fake_token).await;
        assert!(revoke_result.is_ok());
    }

    #[tokio::test]
    async fn test_authorization_provider() {
        let authz_provider = InMemoryAuthzProvider::new();

        let resource = Resource {
            resource_type: "document".to_string(),
            resource_id: "doc123".to_string(),
            attributes: HashMap::new(),
        };

        let action = Action {
            action_type: "read".to_string(),
            attributes: HashMap::new(),
        };

        // Test authorization without any permissions (should deny)
        let auth_result = authz_provider
            .authorize(
                "user1",
                SubjectType::User,
                &action,
                &resource,
                &HashMap::new(),
            )
            .await;
        assert!(auth_result.is_ok());
        assert!(!auth_result.unwrap());

        // Test getting permissions for user (should be empty)
        let permissions = authz_provider
            .get_permissions("user1", SubjectType::User)
            .await
            .unwrap();
        assert!(permissions.is_empty());

        // Test adding permission
        let permission = Permission {
            id: "perm1".to_string(),
            subject: "user1".to_string(),
            subject_type: SubjectType::User,
            resource: resource.clone(),
            action: action.clone(),
            effect: PermissionEffect::Allow,
            conditions: Vec::new(),
        };

        let add_result = authz_provider.add_permission(permission).await;
        assert!(add_result.is_ok());

        // Test removing permission
        let remove_result = authz_provider.remove_permission("perm1").await;
        assert!(remove_result.is_ok());
    }

    #[tokio::test]
    async fn test_security_manager_authentication_disabled() {
        let mut config = SecurityConfig::default();
        config.authentication_enabled = false;

        let auth_provider = Box::new(InMemoryAuthProvider::new(config.clone()));
        let authz_provider = Box::new(InMemoryAuthzProvider::new());
        let security_manager = SecurityManager::new(auth_provider, authz_provider, config);

        // Authentication should fail when disabled
        let auth_result = security_manager.authenticate("user", "password").await;
        assert!(auth_result.is_err());
    }

    #[tokio::test]
    async fn test_security_manager_authorization_flow() {
        std::env::set_var("SONGBIRD_ENV", "development");

        let mut config = SecurityConfig::default();
        config.authorization_enabled = false;

        let auth_provider = Box::new(InMemoryAuthProvider::new(config.clone()));
        let authz_provider = Box::new(InMemoryAuthzProvider::new());
        let security_manager = SecurityManager::new(auth_provider, authz_provider, config);

        let resource = Resource {
            resource_type: "document".to_string(),
            resource_id: "doc123".to_string(),
            attributes: HashMap::new(),
        };

        let action = Action {
            action_type: "read".to_string(),
            attributes: HashMap::new(),
        };

        // Authorization should return false when disabled in development
        let auth_result = security_manager
            .authorize("fake_token", &action, &resource, &HashMap::new())
            .await;
        assert!(auth_result.is_ok());
        assert!(!auth_result.unwrap());

        // Clean up environment variable
        std::env::remove_var("SONGBIRD_ENV");
    }

    #[test]
    fn test_beardog_config_creation() {
        let config = BearDogConfig::default();

        assert_eq!(config.endpoint, "https://localhost:8443");
        assert_eq!(config.api_key, "your_api_key");
        assert_eq!(config.security_level, BearDogSecurityLevel::Internal);
        assert_eq!(config.audit_level, BearDogAuditLevel::Standard);
        assert_eq!(config.compliance_mode, BearDogComplianceMode::Standard);
        assert!(config.metadata.is_empty());
    }

    #[test]
    fn test_beardog_security_context() {
        let context = BearDogSecurityContext {
            security_level: BearDogSecurityLevel::Secret,
            use_bstp: true,
            metadata: HashMap::from([(
                "operation_type".to_string(),
                "data_encryption".to_string(),
            )]),
        };

        assert_eq!(context.security_level, BearDogSecurityLevel::Secret);
        assert!(context.use_bstp);
        assert!(!context.metadata.is_empty());
    }

    #[test]
    fn test_beardog_key_spec() {
        let key_spec = BearDogKeySpec {
            algorithm: "AES-256".to_string(),
            key_size: 256,
            purpose: BearDogKeyPurpose::DataEncryption,
            rotation_policy: BearDogRotationPolicy {
                interval_days: 30,
                auto_rotate: true,
            },
        };

        assert_eq!(key_spec.algorithm, "AES-256");
        assert_eq!(key_spec.key_size, 256);
        assert_eq!(key_spec.purpose, BearDogKeyPurpose::DataEncryption);
        assert!(key_spec.rotation_policy.auto_rotate);
    }

    #[test]
    fn test_beardog_principal_and_resource() {
        let principal = BearDogPrincipal {
            id: "user123".to_string(),
            principal_type: BearDogPrincipalType::User,
            attributes: HashMap::new(),
        };

        let resource = BearDogResource {
            id: "res456".to_string(),
            resource_type: "document".to_string(),
            owner: "user123".to_string(),
            attributes: HashMap::new(),
        };

        assert_eq!(principal.id, "user123");
        assert_eq!(principal.principal_type, BearDogPrincipalType::User);
        assert_eq!(resource.id, "res456");
        assert_eq!(resource.owner, "user123");
    }

    #[test]
    fn test_beardog_security_event() {
        let event = BearDogSecurityEvent {
            event_id: "event123".to_string(),
            event_type: BearDogSecurityEventType::Authentication,
            principal: BearDogPrincipal {
                id: "user123".to_string(),
                principal_type: BearDogPrincipalType::User,
                attributes: HashMap::new(),
            },
            resource: None,
            action: None,
            timestamp: chrono::Utc::now(),
            outcome: BearDogSecurityOutcome::Success,
            details: HashMap::new(),
        };

        assert_eq!(event.event_id, "event123");
        assert_eq!(event.event_type, BearDogSecurityEventType::Authentication);
        assert_eq!(event.outcome, BearDogSecurityOutcome::Success);
        assert!(event.resource.is_none());
        assert!(event.action.is_none());
    }

    #[test]
    fn test_beardog_encrypted_data() {
        let encrypted_data = BearDogEncryptedData {
            data: vec![1, 2, 3, 4, 5],
            algorithm: "AES-256-GCM".to_string(),
            key_id: "key123".to_string(),
        };

        assert_eq!(encrypted_data.data.len(), 5);
        assert_eq!(encrypted_data.algorithm, "AES-256-GCM");
        assert_eq!(encrypted_data.key_id, "key123");
    }

    #[test]
    fn test_condition_operators() {
        let condition = Condition {
            attribute: "role".to_string(),
            operator: ConditionOperator::Equals,
            value: "admin".to_string(),
        };

        assert_eq!(condition.attribute, "role");
        assert_eq!(condition.operator, ConditionOperator::Equals);
        assert_eq!(condition.value, "admin");

        // Test all condition operators exist
        let _ops = vec![
            ConditionOperator::Equals,
            ConditionOperator::NotEquals,
            ConditionOperator::Contains,
            ConditionOperator::NotContains,
            ConditionOperator::GreaterThan,
            ConditionOperator::LessThan,
            ConditionOperator::InList,
            ConditionOperator::NotInList,
        ];
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();

        assert!(config.authentication_enabled);
        assert!(config.authorization_enabled);
        assert!(config.encryption_enabled);
        assert!(config.audit_logging);
        assert_eq!(config.max_login_attempts, 3);

        // BearDog config
        assert_eq!(config.beardog.endpoint, "https://localhost:8443");
        assert_eq!(
            config.beardog.security_level,
            BearDogSecurityLevel::Internal
        );
    }

    #[test]
    fn test_password_policy_defaults() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy.min_length, 8);
        assert!(policy.require_uppercase);
        assert!(policy.require_lowercase);
        assert!(policy.require_numbers);
        assert!(policy.require_special_chars);
        assert_eq!(policy.max_age_days, 90);
    }

    #[test]
    fn test_subject_types() {
        let user = SubjectType::User;
        let service = SubjectType::Service;
        let role = SubjectType::Role;
        let group = SubjectType::Group;

        // Test that all subject types can be created and are distinct
        assert_ne!(user, service);
        assert_ne!(service, role);
        assert_ne!(role, group);
        assert_ne!(group, user);
    }

    #[test]
    fn test_permission_effects() {
        let allow = PermissionEffect::Allow;
        let deny = PermissionEffect::Deny;

        assert_ne!(allow, deny);
    }

    #[test]
    fn test_beardog_compliance_modes() {
        let standard = BearDogComplianceMode::Standard;
        let strict = BearDogComplianceMode::Strict;
        let fips = BearDogComplianceMode::FIPS140;
        let soc2 = BearDogComplianceMode::SOC2;
        let gdpr = BearDogComplianceMode::GDPR;

        // Test that all compliance modes are distinct
        assert_ne!(standard, strict);
        assert_ne!(strict, fips);
        assert_ne!(fips, soc2);
        assert_ne!(soc2, gdpr);
        assert_ne!(gdpr, standard);
    }

    #[test]
    fn test_beardog_audit_levels() {
        let minimal = BearDogAuditLevel::Minimal;
        let standard = BearDogAuditLevel::Standard;
        let comprehensive = BearDogAuditLevel::Comprehensive;
        let paranoid = BearDogAuditLevel::Paranoid;

        // Test that all audit levels are distinct
        assert_ne!(minimal, standard);
        assert_ne!(standard, comprehensive);
        assert_ne!(comprehensive, paranoid);
        assert_ne!(paranoid, minimal);
    }

    #[test]
    fn test_auth_token_expiration() {
        // Create token with very short duration (1 second)
        let token = AuthToken::new(
            "test_user".to_string(),
            SubjectType::User,
            Duration::from_secs(1), // Short duration that will be measurable
            vec!["read".to_string()],
        );

        // Token should not be expired immediately
        assert!(!token.is_expired());

        // Wait for token to expire
        std::thread::sleep(Duration::from_secs(2));
        assert!(token.is_expired());
    }
}
