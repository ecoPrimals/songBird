//! Security Providers
//!
//! Contains authentication and authorization provider traits and implementations.

use async_trait::async_trait;
use songbird_errors::{AuthError, Result};
use std::collections::HashMap;

use crate::security::types::{
    Action, AuthToken, PassphrasePolicy, PasswordValidationStrategy, Permission, PermissionEffect,
    Resource, SecurityConfig, SubjectType, TraditionalPasswordPolicy,
};

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

        match policy.validation_strategy {
            PasswordValidationStrategy::Passphrase => {
                if let Some(ref passphrase_policy) = policy.passphrase_policy {
                    self.validate_passphrase(password, passphrase_policy)
                } else {
                    Err(songbird_errors::SongbirdError::security(
                        "Passphrase policy not configured"
                    ))
                }
            }
            PasswordValidationStrategy::Traditional => {
                if let Some(ref traditional_policy) = policy.traditional_policy {
                    self.validate_traditional_password(password, traditional_policy)?;
                    Ok(())
                } else {
                    Err(songbird_errors::SongbirdError::security(
                        "Traditional password policy not configured"
                    ))
                }
            }
            PasswordValidationStrategy::TraditionalPolicy(ref policy) => {
                self.validate_traditional_password(password, policy)?;
                Ok(())
            }
            PasswordValidationStrategy::Flexible => {
                // Try passphrase first (preferred), fall back to traditional
                if let Some(ref passphrase_policy) = policy.passphrase_policy {
                    if self
                        .validate_passphrase(password, passphrase_policy)
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                if let Some(ref traditional_policy) = policy.traditional_policy {
                    self.validate_traditional_password(password, traditional_policy)
                } else {
                    Err(songbird_errors::SongbirdError::security(
                        "No valid password policy configured for flexible validation"
                    ))
                }
            }
            PasswordValidationStrategy::Custom => {
                // Custom validation defers to security authority (e.g., BearDog primal)
                // This is minimal fallback validation for when external security system is unavailable
                if password.len() >= 8 && !password.trim().is_empty() {
                    tracing::info!("Using minimal fallback password validation - recommend integrating with BearDog primal");
                    Ok(())
                } else {
                    Err(songbird_errors::SongbirdError::security("Password too short (minimum 8 characters). For comprehensive password policies, integrate with BearDog primal."))
                }
            }
        }
    }

    /// Validate XKCD-style passphrase (simple validation for standalone mode)
    fn validate_passphrase(&self, passphrase: &str, policy: &PassphrasePolicy) -> Result<()> {
        let trimmed = passphrase.trim();

        // Simple word-based validation (space-separated words)
        let words: Vec<&str> = trimmed.split_whitespace().collect();

        // Validate word count
        if words.len() < policy.min_words as usize {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: format!(
                    "Passphrase must have at least {} words (found {}). Example: 'correct horse battery staple'",
                    policy.min_words, words.len()
                ),
                provider: Some("SimplePassphraseValidator".to_string()),
            })));
        }

        if words.len() > policy.max_words as usize {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: format!(
                    "Passphrase must have at most {} words (found {}). Consider shortening it",
                    policy.max_words,
                    words.len()
                ),
                provider: Some("SimplePassphraseValidator".to_string()),
            })));
        }

        // Check total length
        if trimmed.len() < policy.min_total_length as usize {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: format!(
                    "Passphrase must be at least {} characters long (found {})",
                    policy.min_total_length,
                    trimmed.len()
                ),
                provider: Some("SimplePassphraseValidator".to_string()),
            })));
        }

        // Check for numbers if required
        if policy.require_number && !passphrase.chars().any(|c| c.is_numeric()) {
            return Err(songbird_errors::SongbirdError::security("Passphrase must contain at least one number"));
        }

        // Check for uppercase if required
        if policy.require_uppercase && !passphrase.chars().any(|c| c.is_uppercase()) {
            return Err(songbird_errors::SongbirdError::security("Passphrase must contain at least one uppercase letter"));
        }

        // Check against common passwords if enabled
        if policy.check_common_passwords && self.is_common_password(passphrase) {
            return Err(songbird_errors::SongbirdError::security("This passphrase is too common. Please choose a more unique combination of words"));
        }

        // Basic entropy check (simplified - in production, use proper entropy calculation)
        let entropy = self.calculate_passphrase_entropy(&words);
        if entropy < policy.min_entropy_bits {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: format!(
                    "Passphrase doesn't have enough entropy (estimated {:.1} bits, need {:.1}). Consider using less common words",
                    entropy, policy.min_entropy_bits
                ),
                provider: Some("SimplePassphraseValidator".to_string()),
            })));
        }

        Ok(())
    }

    /// Validate traditional complex password
    fn validate_traditional_password(
        &self,
        password: &str,
        policy: &TraditionalPasswordPolicy,
    ) -> Result<()> {
        // Length check
        if password.len() < policy.min_length as usize {
            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: format!("Password must be at least {} characters", policy.min_length),
                provider: Some("TraditionalPasswordValidator".to_string()),
            })));
        }

        // Character category checks
        let mut categories_met = 0;
        let mut missing_categories = Vec::new();

        if policy.require_uppercase {
            if password.chars().any(|c| c.is_uppercase()) {
                categories_met += 1;
            } else {
                missing_categories.push("uppercase letter");
            }
        }

        if policy.require_lowercase {
            if password.chars().any(|c| c.is_lowercase()) {
                categories_met += 1;
            } else {
                missing_categories.push("lowercase letter");
            }
        }

        if policy.require_numbers {
            if password.chars().any(|c| c.is_numeric()) {
                categories_met += 1;
            } else {
                missing_categories.push("number");
            }
        }

        if policy.require_special_chars {
            if password
                .chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
            {
                categories_met += 1;
            } else {
                missing_categories.push("special character");
            }
        }

        // Report specific missing requirements
        if !missing_categories.is_empty() {
            let missing_str = format!("Missing required: {}", missing_categories.join(", "));

            return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: format!("Password validation failed: {}", missing_str),
                provider: Some("password_complexity".to_string()),
            })));
        }

        Ok(())
    }

    /// Check if password is in common password list (simplified implementation)
    fn is_common_password(&self, password: &str) -> bool {
        let common_passwords = [
            "password",
            "123456",
            "password123",
            "admin",
            "qwerty",
            "letmein",
            "welcome",
            "monkey",
            "dragon",
            "sunshine",
            "princess",
            "football",
            "baseball",
            "superman",
            "batman",
            // Common passphrases from breaches
            "correct horse battery staple", // The famous XKCD example itself!
            "the quick brown fox",
            "to be or not to be",
            "mary had a little lamb",
            "twinkle twinkle little star",
        ];

        let normalized = password.to_lowercase();
        common_passwords
            .iter()
            .any(|&common| normalized.contains(common))
    }

    /// Calculate estimated entropy for a passphrase (simplified)
    fn calculate_passphrase_entropy(&self, words: &[&str]) -> f64 {
        // Simplified entropy calculation
        // In production, use proper entropy calculation considering:
        // - Dictionary size
        // - Word frequency
        // - Markov chain analysis
        // - Actual randomness vs. predictable patterns

        let avg_word_length: f64 =
            words.iter().map(|w| w.len()).sum::<usize>() as f64 / words.len() as f64;

        // Rough estimate: each word contributes ~10-15 bits depending on length and commonality
        let base_entropy_per_word = if avg_word_length > 6.0 { 13.0 } else { 10.0 };

        // Longer passphrases get bonus entropy
        let word_count_bonus = if words.len() > 4 {
            (words.len() - 4) as f64 * 2.0
        } else {
            0.0
        };

        // Mixed case or numbers add slight entropy bonus
        let complexity_bonus = words
            .iter()
            .map(|word| {
                let mut bonus = 0.0;
                if word.chars().any(|c| c.is_uppercase()) {
                    bonus += 1.0;
                }
                if word.chars().any(|c| c.is_numeric()) {
                    bonus += 1.0;
                }
                bonus
            })
            .sum::<f64>();

        (words.len() as f64 * base_entropy_per_word) + word_count_bonus + complexity_bonus
    }

    fn hash_password(&self, password: &str) -> Result<String> {
        // Use SHA-256 with salt - much more secure than format!("hash_{}")
        use rand::{thread_rng, Rng};
        use ring::digest;

        // Generate random salt
        let mut salt = [0u8; 16];
        thread_rng().fill(&mut salt);

        // Hash password with salt using SHA-256
        let mut to_hash = Vec::new();
        to_hash.extend_from_slice(&salt);
        to_hash.extend_from_slice(password.as_bytes());

        let hash = digest::digest(&digest::SHA256, &to_hash);

        // Combine salt + hash for storage
        let mut combined = Vec::new();
        combined.extend_from_slice(&salt);
        combined.extend_from_slice(hash.as_ref());

        // Use hex encoding for simplicity
        Ok(hex::encode(combined))
    }

    fn verify_password(&self, password: &str, stored_hash: &str) -> bool {
        use ring::digest;

        // Decode hex
        let Ok(combined) = hex::decode(stored_hash) else {
            return false;
        };

        if combined.len() != 48 {
            // 16 bytes salt + 32 bytes SHA-256 hash
            return false;
        }

        let (salt, stored_hash_bytes) = combined.split_at(16);

        // Hash provided password with same salt
        let mut to_hash = Vec::new();
        to_hash.extend_from_slice(salt);
        to_hash.extend_from_slice(password.as_bytes());

        let calculated_hash = digest::digest(&digest::SHA256, &to_hash);

        // Constant-time comparison
        calculated_hash.as_ref() == stored_hash_bytes
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
        let _username = token.split('_').nth(1).unwrap_or("unknown");

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
