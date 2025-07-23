//! Security Hardening Module
//!
//! Implements production-ready security validation and configuration hardening.
//! This module ensures that security configurations are properly validated
//! and that production environments have appropriate security measures.

use std::collections::HashMap;
use std::env;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use songbird_errors::{Result, SongbirdError};

/// Security hardening configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHardeningConfig {
    /// Force security features to be enabled
    pub force_security_enabled: bool,
    /// Minimum password requirements
    pub password_policy: HardenedPasswordPolicy,
    /// Session security settings
    pub session_hardening: SessionHardeningConfig,
    /// Network security settings
    pub network_hardening: NetworkHardeningConfig,
    /// Authentication hardening
    pub auth_hardening: AuthHardeningConfig,
    /// Environment validation settings
    pub environment_validation: EnvironmentValidationConfig,
}

/// Hardened password policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardenedPasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: u32,
    pub prevent_reuse_count: u32,
    pub complexity_score_required: u32,
}

/// Session hardening configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHardeningConfig {
    pub max_session_duration: Duration,
    pub idle_timeout: Duration,
    pub require_secure_cookies: bool,
    pub session_rotation_interval: Duration,
    pub concurrent_session_limit: u32,
}

/// Network hardening configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHardeningConfig {
    pub require_tls: bool,
    pub min_tls_version: String,
    pub allowed_ciphers: Vec<String>,
    pub require_certificate_validation: bool,
    pub block_insecure_protocols: bool,
    pub rate_limiting: RateLimitConfig,
}

/// Authentication hardening configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthHardeningConfig {
    pub require_mfa: bool,
    pub max_failed_attempts: u32,
    pub lockout_duration: Duration,
    pub token_expiration: Duration,
    pub require_strong_tokens: bool,
    pub audit_all_auth_events: bool,
}

/// Environment validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentValidationConfig {
    pub validate_production_settings: bool,
    pub required_environment_variables: Vec<String>,
    pub forbidden_values: HashMap<String, Vec<String>>,
    pub security_warnings: bool,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_limit: u32,
    pub block_duration: Duration,
}

/// Security validation result
#[derive(Debug, Clone)]
pub struct SecurityValidationResult {
    pub is_secure: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Security hardening manager
pub struct SecurityHardeningManager {
    config: SecurityHardeningConfig,
}

impl Default for SecurityHardeningConfig {
    fn default() -> Self {
        Self {
            force_security_enabled: true,
            password_policy: HardenedPasswordPolicy::default(),
            session_hardening: SessionHardeningConfig::default(),
            network_hardening: NetworkHardeningConfig::default(),
            auth_hardening: AuthHardeningConfig::default(),
            environment_validation: EnvironmentValidationConfig::default(),
        }
    }
}

impl Default for HardenedPasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_age_days: 90,
            prevent_reuse_count: 5,
            complexity_score_required: 3,
        }
    }
}

impl Default for SessionHardeningConfig {
    fn default() -> Self {
        Self {
            max_session_duration: Duration::from_secs(8 * 3600), // 8 hours
            idle_timeout: Duration::from_secs(30 * 60),          // 30 minutes
            require_secure_cookies: true,
            session_rotation_interval: Duration::from_secs(15 * 60), // 15 minutes
            concurrent_session_limit: 3,
        }
    }
}

impl Default for NetworkHardeningConfig {
    fn default() -> Self {
        Self {
            require_tls: true,
            min_tls_version: "1.3".to_string(),
            allowed_ciphers: vec![],
            require_certificate_validation: true,
            block_insecure_protocols: true,
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

impl Default for AuthHardeningConfig {
    fn default() -> Self {
        Self {
            require_mfa: false, // Can be enabled per deployment
            max_failed_attempts: 3,
            lockout_duration: Duration::from_secs(15 * 60), // 15 minutes
            token_expiration: Duration::from_secs(3600),    // 1 hour
            require_strong_tokens: true,
            audit_all_auth_events: true,
        }
    }
}

impl Default for EnvironmentValidationConfig {
    fn default() -> Self {
        Self {
            validate_production_settings: true,
            required_environment_variables: vec![],
            forbidden_values: {
                let mut forbidden = HashMap::new();
                forbidden.insert(
                    "SONGBIRD_SECURITY_ENABLED".to_string(),
                    vec!["false".to_string()],
                );
                forbidden.insert("SONGBIRD_ENV".to_string(), vec!["debug".to_string()]);
                forbidden
            },
            security_warnings: true,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            burst_limit: 10,
            block_duration: Duration::from_secs(5 * 60), // 5 minutes
        }
    }
}

impl SecurityHardeningManager {
    /// Create new security hardening manager
    pub fn new(config: SecurityHardeningConfig) -> Self {
        Self { config }
    }

    /// Create with default hardening configuration
    pub fn with_defaults() -> Self {
        Self::new(SecurityHardeningConfig::default())
    }

    /// Validate current security configuration
    pub fn validate_security_configuration(&self) -> SecurityValidationResult {
        let mut result = SecurityValidationResult {
            is_secure: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            recommendations: Vec::new(),
        };

        // Validate environment variables
        self.validate_environment_variables(&mut result);

        // Validate security settings
        self.validate_security_settings(&mut result);

        // Validate network security
        self.validate_network_security(&mut result);

        // Validate authentication security
        self.validate_authentication_security(&mut result);

        // Overall security assessment
        result.is_secure = result.errors.is_empty();

        result
    }

    /// Validate environment variables for security
    fn validate_environment_variables(&self, result: &mut SecurityValidationResult) {
        let env_config = &self.config.environment_validation;

        // Check required environment variables
        for env_var in &env_config.required_environment_variables {
            if env::var(env_var).is_err() {
                result.errors.push(format!(
                    "Required environment variable {env_var} is not set"
                ));
            }
        }

        // Check forbidden values
        for (env_var, forbidden_values) in &env_config.forbidden_values {
            if let Ok(value) = env::var(env_var) {
                if forbidden_values.contains(&value) {
                    result.errors.push(format!(
                        "Environment variable {env_var} has forbidden value: {value}"
                    ));
                }
            }
        }

        // Production environment validation
        if env_config.validate_production_settings {
            let environment = env::var("SONGBIRD_ENV").unwrap_or_default();

            if environment == "production" {
                // Production-specific validations
                if env::var("SONGBIRD_SECURITY_ENABLED").unwrap_or_default() != "true" {
                    result
                        .errors
                        .push("Security must be enabled in production environment".to_string());
                }

                if env::var("SONGBIRD_DEBUG").unwrap_or_default() == "true" {
                    result
                        .warnings
                        .push("Debug mode should be disabled in production".to_string());
                }

                if env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_default() == "0.0.0.0" {
                    result.warnings.push(
                        "Binding to 0.0.0.0 may expose services to external networks".to_string(),
                    );
                }
            }
        }
    }

    /// Validate security settings
    fn validate_security_settings(&self, result: &mut SecurityValidationResult) {
        // Check if security is force-enabled
        if self.config.force_security_enabled {
            let security_enabled =
                env::var("SONGBIRD_SECURITY_ENABLED").unwrap_or_default() == "true";
            if !security_enabled {
                result
                    .errors
                    .push("Security is required but not enabled".to_string());
            }
        }

        // Password policy validation
        let policy = &self.config.password_policy;
        if policy.min_length < 8 {
            result
                .warnings
                .push("Password minimum length should be at least 8 characters".to_string());
        }

        if policy.complexity_score_required < 3 {
            result
                .warnings
                .push("Password complexity requirements may be too low".to_string());
        }
    }

    /// Validate network security
    fn validate_network_security(&self, result: &mut SecurityValidationResult) {
        let network_config = &self.config.network_hardening;

        if network_config.require_tls {
            // Check TLS configuration
            if network_config.min_tls_version != "1.3" && network_config.min_tls_version != "1.2" {
                result
                    .warnings
                    .push("TLS version should be 1.2 or higher".to_string());
            }

            if network_config.allowed_ciphers.is_empty() {
                result
                    .warnings
                    .push("No allowed ciphers configured".to_string());
            }
        }

        // Rate limiting validation
        if network_config.rate_limiting.enabled {
            if network_config.rate_limiting.requests_per_minute > 1000 {
                result
                    .warnings
                    .push("Rate limiting may be too permissive".to_string());
            }
        } else {
            result
                .warnings
                .push("Rate limiting is disabled".to_string());
        }
    }

    /// Validate authentication security
    fn validate_authentication_security(&self, result: &mut SecurityValidationResult) {
        let auth_config = &self.config.auth_hardening;

        // Check authentication settings
        if auth_config.max_failed_attempts > 5 {
            result
                .warnings
                .push("Maximum failed attempts threshold may be too high".to_string());
        }

        if auth_config.token_expiration > Duration::from_secs(24 * 3600) {
            result
                .warnings
                .push("Token expiration time may be too long".to_string());
        }

        if !auth_config.audit_all_auth_events {
            result
                .warnings
                .push("Authentication auditing is disabled".to_string());
        }

        // MFA recommendations
        if !auth_config.require_mfa {
            result
                .recommendations
                .push("Consider enabling multi-factor authentication".to_string());
        }
    }

    /// Apply security hardening measures
    pub fn apply_security_hardening(&self) -> Result<()> {
        info!("🔒 Applying security hardening measures...");

        // Validate configuration first
        let validation_result = self.validate_security_configuration();

        // Log validation results
        for warning in &validation_result.warnings {
            warn!("Security warning: {}", warning);
        }

        for error in &validation_result.errors {
            error!("Security error: {}", error);
        }

        for recommendation in &validation_result.recommendations {
            info!("Security recommendation: {}", recommendation);
        }

        // Fail if there are critical security errors
        if !validation_result.is_secure {
            return Err(SongbirdError::Security {
                message: "Security validation failed".to_string(),
                context: Some("security_hardening".to_string()),
                severity: Some("medium".to_string()),
                suggestion: Some("Check security configuration".to_string()),
            });
        }

        // Apply hardening measures
        self.apply_environment_hardening()?;
        self.apply_network_hardening()?;
        self.apply_authentication_hardening()?;

        info!("✅ Security hardening measures applied successfully");
        Ok(())
    }

    /// Apply environment hardening
    fn apply_environment_hardening(&self) -> Result<()> {
        info!("🔧 Applying environment hardening...");

        // Force security enabled if required
        if self.config.force_security_enabled
            && env::var("SONGBIRD_SECURITY_ENABLED").unwrap_or_default() != "true"
        {
            warn!("Forcing security to be enabled");
            env::set_var("SONGBIRD_SECURITY_ENABLED", "true");
        }

        // Set secure defaults
        let environment = env::var("SONGBIRD_ENV").unwrap_or_default();
        if environment == "production" {
            // Production hardening
            env::set_var("SONGBIRD_DEBUG", "false");
            env::set_var("SONGBIRD_AUDIT_ENABLED", "true");
            env::set_var("SONGBIRD_RATE_LIMITING_ENABLED", "true");
        }

        Ok(())
    }

    /// Apply network hardening
    fn apply_network_hardening(&self) -> Result<()> {
        info!("🌐 Applying network hardening...");

        let network_config = &self.config.network_hardening;

        // Set TLS requirements
        if network_config.require_tls {
            env::set_var("SONGBIRD_TLS_REQUIRED", "true");
            env::set_var("SONGBIRD_MIN_TLS_VERSION", &network_config.min_tls_version);
        }

        // Set rate limiting
        if network_config.rate_limiting.enabled {
            env::set_var("SONGBIRD_RATE_LIMIT_ENABLED", "true");
            env::set_var(
                "SONGBIRD_RATE_LIMIT_RPM",
                network_config.rate_limiting.requests_per_minute.to_string(),
            );
        }

        Ok(())
    }

    /// Apply authentication hardening
    fn apply_authentication_hardening(&self) -> Result<()> {
        info!("🔐 Applying authentication hardening...");

        let auth_config = &self.config.auth_hardening;

        // Set authentication requirements
        env::set_var(
            "SONGBIRD_AUTH_MAX_ATTEMPTS",
            auth_config.max_failed_attempts.to_string(),
        );
        env::set_var(
            "SONGBIRD_AUTH_LOCKOUT_DURATION",
            auth_config.lockout_duration.as_secs().to_string(),
        );
        env::set_var(
            "SONGBIRD_TOKEN_EXPIRATION",
            auth_config.token_expiration.as_secs().to_string(),
        );

        if auth_config.require_strong_tokens {
            env::set_var("SONGBIRD_STRONG_TOKENS_REQUIRED", "true");
        }

        if auth_config.audit_all_auth_events {
            env::set_var("SONGBIRD_AUDIT_AUTH_EVENTS", "true");
        }

        Ok(())
    }

    /// Get security hardening status
    pub fn get_security_status(&self) -> HashMap<String, String> {
        let mut status = HashMap::new();

        // Environment status
        status.insert(
            "environment".to_string(),
            env::var("SONGBIRD_ENV").unwrap_or_default(),
        );
        status.insert(
            "security_enabled".to_string(),
            env::var("SONGBIRD_SECURITY_ENABLED").unwrap_or_default(),
        );

        // Security features status
        status.insert(
            "tls_required".to_string(),
            env::var("SONGBIRD_TLS_REQUIRED").unwrap_or_default(),
        );
        status.insert(
            "rate_limiting".to_string(),
            env::var("SONGBIRD_RATE_LIMIT_ENABLED").unwrap_or_default(),
        );
        status.insert(
            "audit_enabled".to_string(),
            env::var("SONGBIRD_AUDIT_ENABLED").unwrap_or_default(),
        );

        // Authentication status
        status.insert(
            "auth_max_attempts".to_string(),
            env::var("SONGBIRD_AUTH_MAX_ATTEMPTS").unwrap_or_default(),
        );
        status.insert(
            "strong_tokens".to_string(),
            env::var("SONGBIRD_STRONG_TOKENS_REQUIRED").unwrap_or_default(),
        );

        status
    }
}

/// Utility function to validate secure environment variable access
pub fn get_secure_env_var(key: &str, default: &str) -> Result<String> {
    match env::var(key) {
        Ok(value) => {
            // Log access to security-sensitive variables
            if key.contains("SECURITY") || key.contains("AUTH") || key.contains("TOKEN") {
                info!("Accessing security environment variable: {}", key);
            }
            Ok(value)
        }
        Err(_) => {
            // Log when falling back to defaults for security variables
            if key.contains("SECURITY") || key.contains("AUTH") || key.contains("TOKEN") {
                warn!(
                    "Security environment variable {} not set, using default",
                    key
                );
            }
            Ok(default.to_string())
        }
    }
}

/// Utility function to validate production environment
pub fn validate_production_environment() -> Result<()> {
    let environment = env::var("SONGBIRD_ENV").unwrap_or_default();

    if environment == "production" {
        // Critical production validations
        if env::var("SONGBIRD_SECURITY_ENABLED").unwrap_or_default() != "true" {
            return Err(SongbirdError::Security {
                message: "Security must be enabled in production".to_string(),
                context: Some("production_validation".to_string()),
                severity: Some("medium".to_string()),
                suggestion: Some("Check security configuration".to_string()),
            });
        }

        if env::var("SONGBIRD_DEBUG").unwrap_or_default() == "true" {
            return Err(SongbirdError::Security {
                message: "Debug mode must be disabled in production".to_string(),
                context: Some("production_validation".to_string()),
                severity: Some("medium".to_string()),
                suggestion: Some("Check security configuration".to_string()),
            });
        }

        info!("✅ Production environment validation passed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_hardening_config() {
        let config = SecurityHardeningConfig::default();
        assert!(config.force_security_enabled);
        assert_eq!(config.password_policy.min_length, 12);
        assert!(config.network_hardening.require_tls);
    }

    #[test]
    fn test_security_validation() {
        let manager = SecurityHardeningManager::with_defaults();
        let result = manager.validate_security_configuration();
        // Should have some warnings about missing env vars in test environment
        assert!(!result.warnings.is_empty() || !result.errors.is_empty());
    }

    #[test]
    fn test_secure_env_var_access() {
        env::set_var("TEST_SECURITY_VAR", "test_value");
        let result = get_secure_env_var("TEST_SECURITY_VAR", "default");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_value");
    }
}
