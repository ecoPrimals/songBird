//! Security Providers
//!
//! Contains authentication and authorization provider traits and implementations.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::security::types::{
    Action, AuthToken, Permission, PermissionEffect, Resource, SecurityConfig, SubjectType,
};
use songbird_errors::{AuthError, Result, SongbirdError};

// ============================================================================
// PROVIDER TRAITS
// ============================================================================

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

// ============================================================================
// IN-MEMORY IMPLEMENTATIONS
// ============================================================================

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
    /// Create new in-memory authentication provider
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            users: HashMap::new(),
            tokens: HashMap::new(),
            config,
        }
    }

    /// Add user to the provider
    pub fn add_user(
        &mut self,
        username: String,
        password: String,
        permissions: Vec<String>,
    ) -> Result<()> {
        // Validate password against policy
        self.validate_password(&password)?;

        // Hash password (simplified - use proper hashing in production)
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

    fn validate_password(&self, password: &str) -> Result<()> {
        let policy = &self.config.password_policy;

        if password.len() < policy.min_length as usize {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: format!("Password must be at least {} characters", policy.min_length),
                provider: Some("PasswordPolicy".to_string()),
            })));
        }

        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                provider: Some("SecurityProvider".to_string()),
                message: "Password must contain at least one uppercase letter".to_string(),
            })));
        }

        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                provider: Some("SecurityProvider".to_string()),
                message: "Password must contain at least one lowercase letter".to_string(),
            })));
        }

        if policy.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                provider: Some("SecurityProvider".to_string()),
                message: "Password must contain at least one number".to_string(),
            })));
        }

        if policy.require_special_chars
            && !password
                .chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
        {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                provider: Some("SecurityProvider".to_string()),
                message: "Password must contain at least one special character".to_string(),
            })));
        }

        Ok(())
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        // Simplified hash - use proper hashing (bcrypt, argon2, etc.) in production
        Ok(format!("hash_{password}"))
    }

    fn verify_password(&self, password: &str, hash: &str) -> bool {
        // Simplified verification - use proper verification in production
        hash == format!("hash_{password}")
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
                return Ok(token);
            }
        }

        Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
            provider: Some("SecurityProvider".to_string()),
            message: "Invalid credentials".to_string(),
        })))
    }

    async fn validate_token(&self, token: &str) -> Result<AuthToken> {
        if let Some(auth_token) = self.tokens.get(token) {
            if !auth_token.is_expired() {
                return Ok(auth_token.clone());
            }
        }

        // Try to extract username from token for better error message
        let username = token.split('_').nth(1).unwrap_or("unknown");

        Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
            provider: Some("SecurityProvider".to_string()),
            message: "Invalid or expired token".to_string(),
        })))
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
