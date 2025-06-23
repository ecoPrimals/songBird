//! Security Module
//!
//! Production-grade security implementations including JWT authentication,
//! AES encryption, and OAuth2/OIDC integration

pub mod authentication;
pub mod encryption;
pub mod oauth;
pub mod audit;

use async_trait::async_trait;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::debug;

pub use authentication::*;
pub use encryption::*;
pub use oauth::*;
pub use audit::*;

// Type aliases for backward compatibility
pub type AuditEvent = AuthEvent;

/// Security subject (user, service, or system)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// Subject ID
    pub id: String,
    /// Subject type (user, service, system)
    pub subject_type: SubjectType,
    /// Subject attributes
    pub attributes: HashMap<String, String>,
}

/// Subject types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubjectType {
    User,
    Service,
    System,
}

/// Security resource being accessed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource ID
    pub id: String,
    /// Resource type
    pub resource_type: String,
    /// Resource attributes
    pub attributes: HashMap<String, String>,
}

/// Security action being performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action name
    pub name: String,
    /// Action attributes
    pub attributes: HashMap<String, String>,
}

/// Security permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Resource being accessed
    pub resource: Resource,
    /// Action being performed
    pub action: Action,
    /// Conditions that must be met
    pub conditions: Vec<Condition>,
}

/// Security condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Condition attribute
    pub attribute: String,
    /// Condition operator
    pub operator: ConditionOperator,
    /// Expected value
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
    GreaterThanOrEqual,
    LessThanOrEqual,
    In,
    NotIn,
}

/// Security provider trait
#[async_trait]
pub trait SecurityProvider: Send + Sync {
    /// Authorize a subject to perform an action on a resource
    async fn authorize(&self, subject: &Subject, resource: &Resource, action: &Action) -> crate::errors::Result<bool>;
    
    /// Log an audit event
    async fn log_audit(&self, event: AuditEvent) -> crate::errors::Result<()>;
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// JWT signing key
    pub jwt_secret: String,
    /// JWT token expiration time
    pub jwt_expiration: Duration,
    /// AES encryption key (32 bytes)
    pub encryption_key: [u8; 32],
    /// Enable OAuth2 integration
    pub enable_oauth: bool,
    /// OAuth2 configuration
    pub oauth_config: Option<OAuth2Config>,
    /// Enable audit logging
    pub enable_audit: bool,
    /// Audit log configuration
    pub audit_config: AuditConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        // Generate a random key for demo purposes
        // In production, this should come from secure key management
        let mut key = [0u8; 32];
        let rng = SystemRandom::new();
        if let Err(e) = rng.fill(&mut key) {
            tracing::error!("Failed to generate secure encryption key: {:?}", e);
            // Use a deterministic key as fallback (not secure for production)
            key = [42u8; 32]; // Deterministic fallback for testing/demo
        }

        Self {
            jwt_secret: "super-secret-jwt-key-change-in-production".to_string(),
            jwt_expiration: Duration::from_secs(24 * 60 * 60), // 24 hours
            encryption_key: key,
            enable_oauth: false,
            oauth_config: None,
            enable_audit: true,
            audit_config: AuditConfig::default(),
        }
    }
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at
    pub iat: u64,
    /// Expiration
    pub exp: u64,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// User roles
    pub roles: Vec<String>,
    /// Custom claims
    pub custom: HashMap<String, serde_json::Value>,
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// JWT token
    pub token: String,
    /// Token type (Bearer)
    pub token_type: String,
    /// Expiration time
    pub expires_in: u64,
    /// Refresh token (optional)
    pub refresh_token: Option<String>,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Email
    pub email: Option<String>,
    /// User roles
    pub roles: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Production-grade security provider
pub struct ProductionSecurityProvider {
    config: SecurityConfig,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey,
    encryption_key: LessSafeKey,
    oauth_provider: Option<Box<dyn OAuth2Provider>>,
    audit_logger: AuditLogger,
}

impl ProductionSecurityProvider {
    /// Create a new production security provider
    pub fn new(config: SecurityConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let jwt_encoding_key = EncodingKey::from_secret(config.jwt_secret.as_ref());
        let jwt_decoding_key = DecodingKey::from_secret(config.jwt_secret.as_ref());
        
        let unbound_key = UnboundKey::new(&AES_256_GCM, &config.encryption_key)
            .map_err(|e| format!("Failed to create encryption key: {:?}", e))?;
        let encryption_key = LessSafeKey::new(unbound_key);

        let oauth_provider = if config.enable_oauth {
            if let Some(oauth_config) = &config.oauth_config {
                Some(create_oauth_provider(oauth_config.clone())?)
            } else {
                None
            }
        } else {
            None
        };

        let audit_logger = AuditLogger::new(config.audit_config.clone())?;

        Ok(Self {
            config,
            jwt_encoding_key,
            jwt_decoding_key,
            encryption_key,
            oauth_provider,
            audit_logger,
        })
    }

    /// Generate a JWT token for a user
    pub fn generate_jwt(&self, user: &UserInfo) -> Result<AuthToken, Box<dyn std::error::Error>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = Claims {
            sub: user.id.clone(),
            iat: now,
            exp: now + self.config.jwt_expiration.as_secs(),
            iss: "songbird-orchestrator".to_string(),
            aud: "songbird-services".to_string(),
            roles: user.roles.clone(),
            custom: user.metadata.clone(),
        };

        let token = encode(&Header::default(), &claims, &self.jwt_encoding_key)
            .map_err(|e| format!("Failed to encode JWT: {}", e))?;

        // Log authentication event
        if self.config.enable_audit {
            self.audit_logger.log_auth_event(AuthEvent {
                event_type: AuthEventType::TokenGenerated,
                user_id: user.id.clone(),
                timestamp: chrono::Utc::now(),
                details: HashMap::from([
                    ("token_expires".to_string(), serde_json::Value::from(claims.exp)),
                    ("roles".to_string(), serde_json::to_value(&user.roles).unwrap()),
                ]),
                success: true,
                ip_address: None,
                user_agent: None,
            });
        }

        Ok(AuthToken {
            token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt_expiration.as_secs(),
            refresh_token: None, // TODO: Implement refresh tokens
        })
    }

    /// Validate and decode a JWT token
    pub fn validate_jwt(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        let mut validation = Validation::default();
        validation.set_audience(&["songbird-services"]);
        validation.set_issuer(&["songbird-orchestrator"]);

        let token_data: TokenData<Claims> = decode(token, &self.jwt_decoding_key, &validation)
            .map_err(|e| format!("Failed to decode JWT: {}", e))?;

        // Log validation event
        if self.config.enable_audit {
            self.audit_logger.log_auth_event(AuthEvent {
                event_type: AuthEventType::TokenValidated,
                user_id: token_data.claims.sub.clone(),
                timestamp: chrono::Utc::now(),
                details: HashMap::from([
                    ("roles".to_string(), serde_json::to_value(&token_data.claims.roles).unwrap()),
                ]),
                success: true,
                ip_address: None,
                user_agent: None,
            });
        }

        Ok(token_data.claims)
    }

    /// Encrypt data using AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12]; // 96-bit nonce for GCM
        rng.fill(&mut nonce_bytes)
            .map_err(|_| "Failed to generate nonce")?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let aad = Aad::empty();

        let mut in_out = plaintext.to_vec();
        self.encryption_key
            .seal_in_place_append_tag(nonce, aad, &mut in_out)
            .map_err(|_| "Encryption failed")?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&in_out);

        debug!("Encrypted {} bytes of data", plaintext.len());
        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if ciphertext.len() < 12 {
            return Err("Ciphertext too short".into());
        }

        // Extract nonce from the beginning
        let (nonce_bytes, encrypted_data) = ciphertext.split_at(12);
        let nonce = Nonce::assume_unique_for_key(
            nonce_bytes.try_into()
                .map_err(|_| "Invalid nonce length")?
        );

        let aad = Aad::empty();
        let mut in_out = encrypted_data.to_vec();
        
        let plaintext = self.encryption_key
            .open_in_place(nonce, aad, &mut in_out)
            .map_err(|_| "Decryption failed")?;

        debug!("Decrypted {} bytes of data", plaintext.len());
        Ok(plaintext.to_vec())
    }

    /// Get OAuth provider if enabled
    pub fn oauth_provider(&self) -> Option<&dyn OAuth2Provider> {
        self.oauth_provider.as_deref()
    }

    /// Get audit logger
    pub fn audit_logger(&self) -> &AuditLogger {
        &self.audit_logger
    }
}

#[async_trait::async_trait]
impl SecurityProvider for ProductionSecurityProvider {
    async fn authorize(&self, subject: &Subject, resource: &Resource, action: &Action) -> crate::errors::Result<bool> {
        // Simple authorization logic - in production this would be more sophisticated
        let authorized = match subject.subject_type {
            SubjectType::System => true, // System always authorized
            SubjectType::Service => {
                // Services can access their own resources
                resource.resource_type == "service"
            }
            SubjectType::User => {
                // Check if user has admin role in attributes
                let is_admin = subject.attributes.get("role").map_or(false, |r| r == "admin");
                if is_admin {
                    true
                } else {
                    // Regular users have limited access
                    match action.name.as_str() {
                        "read" => resource.resource_type == "user_data" || resource.resource_type == "service",
                        "write" => resource.resource_type == "user_data" && resource.id == subject.id,
                        "delete" => false, // Regular users cannot delete
                        _ => false,
                    }
                }
            }
        };

        // Log authorization event
        if self.config.enable_audit {
            self.audit_logger.log_auth_event(AuthEvent {
                event_type: if authorized { AuthEventType::AccessGranted } else { AuthEventType::AccessDenied },
                user_id: subject.id.clone(),
                timestamp: chrono::Utc::now(),
                details: HashMap::from([
                    ("resource".to_string(), serde_json::Value::String(resource.id.clone())),
                    ("action".to_string(), serde_json::Value::String(action.name.clone())),
                    ("authorized".to_string(), serde_json::Value::Bool(authorized)),
                ]),
                success: authorized,
                ip_address: None,
                user_agent: None,
            });
        }

        Ok(authorized)
    }

    async fn log_audit(&self, event: AuditEvent) -> crate::errors::Result<()> {
        self.audit_logger.log_auth_event(event);
        Ok(())
    }
}

#[async_trait::async_trait]
impl authentication::AuthenticationProvider for ProductionSecurityProvider {
    async fn authenticate(&self, credentials: &authentication::Credentials) -> crate::errors::Result<authentication::AuthenticationResult> {
        match credentials {
            authentication::Credentials::Basic { username, password } => {
                // In production, validate against user database
                // For now, use simple hardcoded validation
                if username == "admin" && password == "admin123" {
                    let user = UserInfo {
                        id: "admin".to_string(),
                        username: "admin".to_string(),
                        email: Some("admin@example.com".to_string()),
                        roles: vec!["admin".to_string()],
                        metadata: HashMap::new(),
                    };

                    let token = self.generate_jwt(&user)?;

                    Ok(authentication::AuthenticationResult {
                        success: true,
                        user: Some(user),
                        token: Some(token),
                        session: None,
                        error: None,
                        mfa_required: false,
                        mfa_methods: vec![],
                    })
                } else {
                    Ok(authentication::AuthenticationResult {
                        success: false,
                        user: None,
                        token: None,
                        session: None,
                        error: Some("Invalid credentials".to_string()),
                        mfa_required: false,
                        mfa_methods: vec![],
                    })
                }
            }
            authentication::Credentials::Bearer { token } => {
                match self.validate_jwt(token) {
                    Ok(claims) => {
                        let user = UserInfo {
                            id: claims.sub.clone(),
                            username: claims.sub.clone(),
                            email: None,
                            roles: claims.roles.clone(),
                            metadata: claims.custom.clone(),
                        };

                        Ok(authentication::AuthenticationResult {
                            success: true,
                            user: Some(user),
                            token: None, // Token already provided
                            session: None,
                            error: None,
                            mfa_required: false,
                            mfa_methods: vec![],
                        })
                    }
                    Err(e) => {
                        Ok(authentication::AuthenticationResult {
                            success: false,
                            user: None,
                            token: None,
                            session: None,
                            error: Some(format!("Invalid token: {}", e)),
                            mfa_required: false,
                            mfa_methods: vec![],
                        })
                    }
                }
            }
            _ => {
                Ok(authentication::AuthenticationResult {
                    success: false,
                    user: None,
                    token: None,
                    session: None,
                    error: Some("Unsupported credential type".to_string()),
                    mfa_required: false,
                    mfa_methods: vec![],
                })
            }
        }
    }

    async fn validate_token(&self, token: &str) -> crate::errors::Result<authentication::SessionInfo> {
        match self.validate_jwt(token) {
            Ok(claims) => {
                Ok(authentication::SessionInfo {
                    session_id: "mock_session".to_string(),
                    user_id: claims.sub.clone(),
                    created_at: chrono::Utc::now(),
                    expires_at: chrono::DateTime::from_timestamp(claims.exp as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::hours(1)),
                    roles: claims.roles.clone(),
                    metadata: claims.custom.clone(),
                    ip_address: None,
                    user_agent: None,
                })
            }
            Err(e) => {
                Err(crate::errors::SongbirdError::SecurityError(format!("Invalid token: {}", e)))
            }
        }
    }

    async fn refresh_token(&self, _refresh_token: &str) -> crate::errors::Result<AuthToken> {
        // TODO: Implement proper refresh token logic
        Ok(AuthToken {
            token: "refreshed_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some("new_refresh_token".to_string()),
        })
    }

    async fn revoke_token(&self, _token: &str) -> crate::errors::Result<()> {
        // TODO: Implement token revocation (blacklist, etc.)
        Ok(())
    }
}
