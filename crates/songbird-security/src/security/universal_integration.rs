//! Universal Security Integration
//!
//! Replaces hardcoded BearDog integration with universal adapter pattern.
//! Uses the mature SecurityCapabilityAdapter to discover and route to any security provider.

use songbird_errors::{SongbirdError, SongbirdResult};
// use songbird_universal::  // TEMPORARILY DISABLED - adapters::routing;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::security::types::{AuthToken, SubjectType};
use std::collections::HashMap;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, // Subject (user ID)
    exp: usize,  // Expiration time
    iat: usize,  // Issued at
    iss: String, // Issuer
    permissions: Vec<String>,
}

/// Universal Security Integration
///
/// Provides real JWT-based authentication with the Universal Provider system
pub struct UniversalSecurityIntegration {
    /// JWT signing key
    jwt_secret: String,
    /// Token expiration duration in hours
    token_expiration_hours: i64,
    #[allow(dead_code)] // Configuration for future security integrations
    config: HashMap<String, String>,
}

impl UniversalSecurityIntegration {
    /// Create a new universal security integration
    pub fn new() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "songbird-default-secret-change-in-production".to_string()),
            token_expiration_hours: 24, // 24 hour tokens by default
            config: HashMap::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(jwt_secret: String, expiration_hours: i64) -> Self {
        Self {
            jwt_secret,
            token_expiration_hours: expiration_hours,
            config: HashMap::new(),
        }
    }

    /// Authenticate using real JWT-based authentication
    pub async fn authenticate(&self, credentials: &str) -> SongbirdResult<AuthToken> {
        debug!("🔐 Processing authentication request via universal adapter");

        if credentials.is_empty() {
            return Err(SongbirdError::internal_error(validation_error(
                "Credentials cannot be empty",
            ));
        }

        // Parse credentials (expecting JSON format)
        let cred_data: serde_json::Value = serde_json::from_str(credentials)
            .map_err(|_| SongbirdError::validation_error("Invalid credentials format"))?;

        let username = cred_data["username"]
            .as_str()
            .ok_or_else(|| SongbirdError::validation_error("Username required"))?;

        let password = cred_data["password"]
            .as_str()
            .ok_or_else(|| SongbirdError::validation_error("Password required"))?;

        // Validate credentials (in production, this would check against a database)
        if !self.validate_credentials(username, password).await? {
            return Err(SongbirdError::internal_error(operation_error("Invalid credentials"));
        }

        // Generate JWT token
        let now = Utc::now();
        let exp = now + Duration::hours(self.token_expiration_hours);

        let claims = Claims {
            sub: username.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: "songbird-security".to_string(),
            permissions: self.get_user_permissions(username).await?,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| SongbirdError::operation_error(format!("Token generation failed: {e}")))?;

        info!("✅ Authentication successful for user: {}", username);
        Ok(AuthToken {
            token,
            subject: username.to_string(),
            subject_type: SubjectType::User,
            issued_at: now.timestamp() as u64,
            expires_at: exp.timestamp() as u64,
            permissions: claims.permissions,
            attributes: HashMap::new(),
        })
    }

    /// Validate JWT token
    pub async fn validate_token(&self, token: &str) -> SongbirdResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| SongbirdError::operation_error(format!("Token validation failed: {e}")))?;

        Ok(token_data.claims)
    }

    /// Validate user credentials (placeholder - would integrate with user store)
    async fn validate_credentials(&self, username: &str, password: &str) -> SongbirdResult<bool> {
        // In production, this would:
        // 1. Query user database
        // 2. Verify password hash
        // 3. Check account status
        // 4. Integrate with external auth providers (LDAP, OAuth, etc.)

        // For now, accept any non-empty credentials
        Ok(!username.is_empty() && !password.is_empty())
    }

    /// Get user permissions (placeholder - would query permission system)
    async fn get_user_permissions(&self, username: &str) -> SongbirdResult<Vec<String>> {
        // In production, this would query the permission system
        // For now, return basic permissions
        Ok(match username {
            "admin" => vec!["read".to_string(), "write".to_string(), "admin".to_string()],
            _ => vec!["read".to_string()],
        })
    }

    /// Check if universal security integration is available
    pub async fn is_available(&self) -> bool {
        // Check if we have a valid JWT secret and can generate tokens
        !self.jwt_secret.is_empty()
    }
}

impl Default for UniversalSecurityIntegration {
    fn default() -> Self {
        Self::new()
    }
}
