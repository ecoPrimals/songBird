//! Security Managers
//!
//! This module contains the high-level security managers that coordinate
//! authentication and authorization providers.

use std::collections::HashMap;

use super::providers::{AuthenticationProvider, AuthorizationProvider};
use super::types::{Action, AuthToken, Resource, SecurityConfig};
use super::hardening::{SecurityHardeningManager, validate_production_environment, get_secure_env_var};
use songbird_errors::{Result, SongbirdError, NetworkError, AuthError};

// ============================================================================
// SECURITY MANAGER
// ============================================================================

/// Security manager that coordinates authentication and authorization
pub struct SecurityManager {
    auth_user: Box<dyn AuthenticationProvider>,
    authz_user: Box<dyn AuthorizationProvider>,
    config: SecurityConfig,
    hardening_manager: SecurityHardeningManager,
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
            hardening_manager: SecurityHardeningManager::with_defaults(),
        }
    }

    /// Create a new security manager with custom hardening configuration
    pub fn with_hardening(
        auth_user: Box<dyn AuthenticationProvider>,
        authz_user: Box<dyn AuthorizationProvider>,
        config: SecurityConfig,
        hardening_manager: SecurityHardeningManager,
    ) -> Self {
        Self {
            auth_user,
            authz_user,
            config,
            hardening_manager,
        }
    }

    /// Initialize security manager with hardening
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("🔐 Initializing security manager with hardening...");

        // Validate production environment
        validate_production_environment()?;

        // Apply security hardening measures
        self.hardening_manager.apply_security_hardening()?;

        // Validate security configuration
        let validation_result = self.hardening_manager.validate_security_configuration();
        
        if !validation_result.is_secure {
            return Err(SongbirdError::Security {
                severity: Some("medium".to_string()),
                suggestion: Some("Check security configuration and permissions".to_string()),
                message: "Security validation failed during initialization".to_string(),
                severity: Some("medium".to_string()),
                suggestion: Some("Check security configuration and permissions".to_string()),
                context: Some("security_manager_init".to_string()),
            });
        }

        tracing::info!("✅ Security manager initialized successfully");
        Ok(())
    }

    /// Authenticate user with enhanced security
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        // Validate production environment for authentication
        validate_production_environment()?;

        if !self.config.authentication_enabled {
            tracing::error!("SECURITY CRITICAL: Authentication disabled");
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "Authentication is disabled".to_string(),
                severity: Some("medium".to_string()),
                suggestion: Some("Check security configuration and permissions".to_string()),
                user: Some("SecurityManager".to_string()),
            })));
        }

        // Log authentication attempt for security audit
        tracing::info!("Authentication attempt for user: {}", username);

        let result = self.auth_user.authenticate(username, password).await;

        // Log authentication result for security audit
        match &result {
            Ok(token) => {
                tracing::info!("Authentication successful for user: {} (token: {})", username, token.token);
            }
            Err(e) => {
                tracing::warn!("Authentication failed for user: {} - {}", username, e);
            }
        }

        result
    }

    /// Authorize action with enhanced security
    pub async fn authorize(
        &self,
        token: &str,
        action: &Action,
        resource: &Resource,
        context: &HashMap<String, String>,
    ) -> Result<bool> {
        // Validate production environment for authorization
        validate_production_environment()?;

        if !self.config.authorization_enabled {
            tracing::error!("SECURITY CRITICAL: Authorization disabled - this should only be used in development!");
            let environment = get_secure_env_var("SONGBIRD_ENV", "development")?;
            if environment != "development" {
                return Err(songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("security".to_string()),
                    message: "Authorization cannot be disabled in production".to_string(),
                severity: Some("medium".to_string()),
                suggestion: Some("Check security configuration and permissions".to_string()),
                    details: None,
                })));
            }
            return Ok(false); // Explicit deny in production
        }

        // Log authorization attempt for security audit
        tracing::debug!("Authorization attempt for action: {:?} on resource: {:?}", action, resource);

        let auth_token = self.auth_user.validate_token(token).await?;

        let result = self.authz_user
            .authorize(
                &auth_token.subject,
                auth_token.subject_type,
                action,
                resource,
                context,
            )
            .await;

        // Log authorization result for security audit
        match &result {
            Ok(authorized) => {
                if *authorized {
                    tracing::debug!("Authorization granted for action: {:?} on resource: {:?}", action, resource);
                } else {
                    tracing::warn!("Authorization denied for action: {:?} on resource: {:?}", action, resource);
                }
            }
            Err(e) => {
                tracing::error!("Authorization error for action: {:?} on resource: {:?} - {}", action, resource, e);
            }
        }

        result
    }

    /// Get security configuration
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Get security hardening status
    pub fn get_hardening_status(&self) -> HashMap<String, String> {
        self.hardening_manager.get_security_status()
    }

    /// Validate current security configuration
    pub fn validate_security(&self) -> Result<()> {
        let validation_result = self.hardening_manager.validate_security_configuration();
        
        if !validation_result.is_secure {
            return Err(SongbirdError::Security {
                severity: Some("medium".to_string()),
                suggestion: Some("Check security configuration and permissions".to_string()),
                message: format!("Security validation failed: {:?}", validation_result.errors),
                context: Some("security_validation".to_string()),
            });
        }

        Ok(())
    }

    /// Apply security hardening measures
    pub fn apply_hardening(&self) -> Result<()> {
        self.hardening_manager.apply_security_hardening()
    }
}
