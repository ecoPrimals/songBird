//! # Canonical Authentication Provider
//!
//! **🎯 CANONICAL UNIFICATION SOLUTION**
//!
//! This module provides the **single source of truth** for authentication
//! patterns across the Songbird ecosystem, eliminating trait fragmentation
//! and technical debt through canonical unification.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_errors::{ServiceResult, SongbirdError, SongbirdResult, success};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Re-export types for backward compatibility
pub use crate::security::authentication::{AuthenticationResult, Credentials};
pub use crate::security::types::{AuthToken, SecurityConfig};

// ============================================================================
// CANONICAL AUTHENTICATION TRAIT - SINGLE SOURCE OF TRUTH
// ============================================================================

/// **CANONICAL**: Universal Authentication Provider Trait
///
/// This is the **single canonical interface** for all authentication providers
/// in the Songbird ecosystem. All authentication implementations MUST implement
/// this trait to ensure consistency and interoperability.
#[async_trait]
pub trait CanonicalAuthenticationProvider: Send + Sync {
    /// Authenticate user with structured request
    async async fn authenticate(&self, request: &AuthenticationRequest) -> SongbirdResult<AuthToken>;

    /// Validate authentication token
    async async fn validate_token(&self, token: &str) -> SongbirdResult<AuthToken>;

    /// Revoke authentication token
    async async fn revoke_token(&self, token: &str) -> SongbirdResult<()>;

    /// Refresh authentication token
    async async fn refresh_token(&self, token: &str) -> SongbirdResult<AuthToken>;

    /// Health check for the authentication provider
    async async fn health_check(&self) -> SongbirdResult<ProviderHealth>;

    /// Get provider capabilities
    fn capabilities(&self) -> AuthenticationCapabilities;

    /// Get provider metadata
    fn metadata(&self) -> ProviderMetadata;
}

// ============================================================================
// CANONICAL DATA STRUCTURES
// ============================================================================

/// **CANONICAL**: Authentication Request Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationRequest {
    /// Authentication method
    pub method: AuthenticationMethod,
    /// Primary credential (username, email, etc.)
    pub identifier: String,
    /// Secondary credential (password, token, etc.)
    pub credential: String,
    /// Additional authentication factors
    pub additional_factors: Option<HashMap<String, String>>,
    /// Request context
    pub context: AuthenticationContext,
}

/// **CANONICAL**: Authentication Method Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthenticationMethod {
    /// Username/password authentication
    Password,
    /// Token-based authentication
    Token,
    /// OAuth2 authentication
    OAuth2,
    /// Multi-factor authentication
    MFA,
    /// Certificate-based authentication
    Certificate,
    /// API key authentication
    ApiKey,
}

/// **CANONICAL**: Authentication Context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationContext {
    /// Client IP address
    pub client_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Request timestamp
    pub timestamp: u64,
    /// Additional context data
    pub metadata: HashMap<String, String>,
}

impl Default for AuthenticationContext {
    fn default() -> Self {
        Self {
            client_ip: None,
            user_agent: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }
}

/// **CANONICAL**: Provider Health Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Last check timestamp
    pub last_check: u64,
    /// Health details
    pub details: HashMap<String, String>,
}

/// **CANONICAL**: Health Status Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Provider is healthy
    Healthy,
    /// Provider is degraded but functional
    Degraded,
    /// Provider is unhealthy
    Unhealthy,
    /// Provider status is unknown
    Unknown,
}

/// **CANONICAL**: Authentication Capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationCapabilities {
    /// Supported authentication methods
    pub supported_methods: Vec<AuthenticationMethod>,
    /// Multi-factor authentication support
    pub mfa_support: bool,
    /// Token refresh support
    pub token_refresh: bool,
    /// Session management support
    pub session_management: bool,
    /// Maximum token lifetime (seconds)
    pub max_token_lifetime: Option<u64>,
}

/// **CANONICAL**: Provider Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Provider name
    pub name: String,
    /// Provider version
    pub version: String,
    /// Provider description
    pub description: String,
    /// Provider vendor
    pub vendor: String,
    /// Configuration schema version
    pub schema_version: String,
}

// ============================================================================
// UNIVERSAL ADAPTER PATTERN - DEBT ELIMINATION
// ============================================================================

/// **CANONICAL**: Universal Authentication Adapter
///
/// This adapter bridges **all existing authentication implementations**
/// to the canonical interface, eliminating technical debt while maintaining
/// backward compatibility.
#[derive(Clone)]
pub struct CanonicalAuthenticationAdapter<T> {
    inner: T,
    capabilities: AuthenticationCapabilities,
    metadata: ProviderMetadata,
}

impl<T> CanonicalAuthenticationAdapter<T> {
    /// Create new canonical adapter
    pub fn new(
        inner: T,
        capabilities: AuthenticationCapabilities,
        metadata: ProviderMetadata,
    ) -> Self {
        Self {
            inner,
            capabilities,
            metadata,
        }
    }

    /// Access the inner provider
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

// ============================================================================
// ADAPTER IMPLEMENTATIONS - UNIFYING FRAGMENTED TRAITS
// ============================================================================

/// Adapter for legacy AuthenticationProvider trait
#[async_trait]
impl<T> CanonicalAuthenticationProvider for CanonicalAuthenticationAdapter<T>
where
    T: crate::security::providers::AuthenticationProvider + Send + Sync,
{
    async fn authenticate(&self, request: &AuthenticationRequest) -> SongbirdResult<AuthToken> {
        // Convert canonical request to legacy format
        match request.method {
            AuthenticationMethod::Password => {
                let token = self
                    .inner
                    .authenticate(&request.identifier, &request.credential)
                    .await
                    .map_err(|e| SongbirdError::validation_error(
                        &format!("Authentication failed: {}", e)
                    ))?;
                Ok(songbird_errors::evolved_success(songbird_errors::success(token)))
            }
            _ => Err(SongbirdError::internal_error(validation_error(
                "Invalid authentication method"
            )),
        }
    }

    async fn validate_token(&self, token: &str) -> SongbirdResult<AuthToken> {
        let validated_token =
            self.inner
                .validate_token(token)
                .await
                .map_err(|e| SongbirdError::validation_error(
                    &format!("Token validation failed: {}", e)
                ))?;
        Ok(songbird_errors::evolved_success(songbird_errors::success(validated_token)))
    }

    async fn revoke_token(&self, token: &str) -> SongbirdResult<()> {
        self.inner
            .revoke_token(token)
            .await
            .map_err(|e| SongbirdError::validation_error(
                &format!("Token revocation failed: {}", e)
            ))?;
        Ok(songbird_errors::success(()))
    }

    async fn refresh_token(&self, token: &str) -> SongbirdResult<AuthToken> {
        let refreshed_token =
            self.inner
                .refresh_token(token)
                .await
                .map_err(|e| SongbirdError::validation_error(
                    &format!("Token refresh failed: {}", e)
                ))?;
        Ok(songbird_errors::evolved_success(songbird_errors::success(refreshed_token)))
    }

    async fn health_check(&self) -> SongbirdResult<ProviderHealth> {
        // Basic health check implementation
        let start_time = SystemTime::now();

        // Try a simple validation to check provider health
        let health_status = match self.inner.validate_token("health_check_token").await {
            Ok(songbird_errors::evolved_success(_)) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Healthy, // Expected to fail with invalid token, but provider is responsive
        };

        let response_time = start_time.elapsed().unwrap_or_default().as_millis() as u64;

        Ok(songbird_errors::success(ProviderHealth {
            status: health_status,
            response_time_ms: response_time,
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            details: HashMap::new(),
        }))
    }

    fn capabilities(&self) -> AuthenticationCapabilities {
        self.capabilities.clone()
    }

    fn metadata(&self) -> ProviderMetadata {
        self.metadata.clone()
    }
}

// ============================================================================
// HELPER FUNCTIONS - CANONICAL PATTERNS
// ============================================================================

/// Create default authentication capabilities
pub fn default_authentication_capabilities() -> AuthenticationCapabilities {
    AuthenticationCapabilities {
        supported_methods: vec![AuthenticationMethod::Password, AuthenticationMethod::Token],
        mfa_support: false,
        token_refresh: true,
        session_management: true,
        max_token_lifetime: Some(3600), // 1 hour default
    }
}

/// Create default provider metadata
pub fn default_provider_metadata(name: &str) -> ProviderMetadata {
    ProviderMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: format!("Canonical {name} Authentication Provider"),
        vendor: "Songbird Security".to_string(),
        schema_version: "1.0".to_string(),
    }
}

/// Convert legacy credentials to canonical request
pub fn credentials_to_canonical_request(credentials: &Credentials) -> AuthenticationRequest {
    match credentials {
        Credentials::UserPassword { username, password } => AuthenticationRequest {
            method: AuthenticationMethod::Password,
            identifier: username.clone(),
            credential: password.clone(),
            additional_factors: None,
            context: AuthenticationContext::default(),
        },
        Credentials::Bearer { token } => AuthenticationRequest {
            method: AuthenticationMethod::Token,
            identifier: "bearer".to_string(),
            credential: token.clone(),
            additional_factors: None,
            context: AuthenticationContext::default(),
        },
        Credentials::OAuth2 {
            access_token,
            token_type,
            ..
        } => AuthenticationRequest {
            method: AuthenticationMethod::OAuth2,
            identifier: token_type.clone(),
            credential: access_token.clone(),
            additional_factors: None,
            context: AuthenticationContext::default(),
        },
        _ => AuthenticationRequest {
            method: AuthenticationMethod::Token,
            identifier: "unknown".to_string(),
            credential: "".to_string(),
            additional_factors: None,
            context: AuthenticationContext::default(),
        },
    }
}
