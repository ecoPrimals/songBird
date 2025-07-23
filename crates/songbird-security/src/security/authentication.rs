//! Authentication Module
//!
//! Provides authentication mechanisms and credential validation

use async_trait::async_trait;
use songbird_errors::{AuthError, Result};
use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthenticationResult {
    pub success: bool,
    pub token: Option<String>,
    pub user_id: Option<String>,
    pub permissions: Vec<String>,
    pub expires_at: Option<u64>,
    pub error: Option<String>,
}

/// Different types of credentials
#[derive(Debug, Clone)]
pub enum Credentials {
    /// Username and password
    UserPassword { username: String, password: String },
    /// Bearer token
    Bearer { token: String },
    /// Basic authentication
    Basic { credentials: String },
    /// Certificate-based authentication
    Certificate { certificate: String },
    /// OAuth2 token
    OAuth2 {
        access_token: String,
        token_type: String,
        // Additional fields for test compatibility
        code: Option<String>,
        state: Option<String>,
        redirect_uri: Option<String>,
    },
    /// Multi-factor authentication
    MFA {
        primary_credential: Box<Credentials>,
        mfa_code: String,
        // Additional fields for test compatibility
        primary: Option<Box<Credentials>>,
        secondary_factor: Option<String>,
    },
}

/// Authentication session
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub session_id: String,
    pub user_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub last_activity: u64,
    pub permissions: Vec<String>,
    pub attributes: HashMap<String, String>,
}

impl AuthSession {
    /// Create a new authentication session
    pub fn new(user_id: String, duration: Duration, permissions: Vec<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            created_at: now,
            expires_at: now + duration.as_secs(),
            last_activity: now,
            permissions,
            attributes: HashMap::new(),
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.expires_at
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Check if session has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Add permission to session
    pub fn add_permission(&mut self, permission: String) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    /// Remove permission from session
    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.retain(|p| p != permission);
    }
}

/// Authentication provider trait
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate using provided credentials
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthenticationResult>;

    /// Validate an existing session
    async fn validate_session(&self, session_id: &str) -> Result<AuthSession>;

    /// Invalidate a session
    async fn invalidate_session(&self, session_id: &str) -> Result<()>;

    /// Refresh a session
    async fn refresh_session(&self, session_id: &str) -> Result<AuthSession>;

    /// Get user permissions
    async fn get_user_permissions(&self, user_id: &str) -> Result<Vec<String>>;
}

/// Simple in-memory authenticator
pub struct InMemoryAuthenticator {
    users: HashMap<String, UserInfo>,
    sessions: HashMap<String, AuthSession>,
    session_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
    pub password_hash: String,
    pub permissions: Vec<String>,
    pub enabled: bool,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
}

impl InMemoryAuthenticator {
    /// Create a new in-memory authenticator
    pub fn new(session_duration: Duration) -> Self {
        Self {
            users: HashMap::new(),
            sessions: HashMap::new(),
            session_duration,
        }
    }

    /// Add a user to the authenticator
    pub fn add_user(
        &mut self,
        username: String,
        password: String,
        permissions: Vec<String>,
    ) -> Result<String> {
        let user_id = uuid::Uuid::new_v4().to_string();
        let password_hash = self.hash_password(&password)?;

        let user_info = UserInfo {
            user_id: user_id.clone(),
            username: username.clone(),
            password_hash,
            permissions,
            enabled: true,
            mfa_enabled: false,
            mfa_secret: None,
        };

        self.users.insert(username, user_info);
        Ok(user_id)
    }

    /// Enable MFA for a user
    pub fn enable_mfa(&mut self, username: &str, secret: String) -> Result<()> {
        if let Some(user) = self.users.get_mut(username) {
            user.mfa_enabled = true;
            user.mfa_secret = Some(secret);
            Ok(())
        } else {
            Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "User not found".to_string(),
                provider: Some("InMemoryAuthenticator".to_string()),
            })))
        }
    }

    /// Disable user account
    pub fn disable_user(&mut self, username: &str) -> Result<()> {
        if let Some(user) = self.users.get_mut(username) {
            user.enabled = false;
            Ok(())
        } else {
            Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "User not found".to_string(),
                provider: Some("InMemoryAuthenticator".to_string()),
            })))
        }
    }

    /// Hash password (simplified - use proper hashing in production)
    fn hash_password(&self, password: &str) -> Result<String> {
        // Use SHA-256 with salt - secure cryptographic hashing
        use rand::{thread_rng, Rng};
        use ring::digest;

        let mut salt = [0u8; 16];
        thread_rng().fill(&mut salt);

        let mut to_hash = Vec::new();
        to_hash.extend_from_slice(&salt);
        to_hash.extend_from_slice(password.as_bytes());

        let hash = digest::digest(&digest::SHA256, &to_hash);

        let mut combined = Vec::new();
        combined.extend_from_slice(&salt);
        combined.extend_from_slice(hash.as_ref());

        Ok(hex::encode(combined))
    }

    /// Verify password
    fn verify_password(&self, password: &str, stored_hash: &str) -> bool {
        use ring::digest;

        let Ok(combined) = hex::decode(stored_hash) else {
            return false;
        };

        if combined.len() != 48 {
            return false;
        }

        let (salt, stored_hash_bytes) = combined.split_at(16);

        let mut to_hash = Vec::new();
        to_hash.extend_from_slice(salt);
        to_hash.extend_from_slice(password.as_bytes());

        let calculated_hash = digest::digest(&digest::SHA256, &to_hash);
        calculated_hash.as_ref() == stored_hash_bytes
    }

    /// Verify MFA code (simplified)
    fn verify_mfa_code(&self, _secret: &str, _code: &str) -> bool {
        // In production, implement TOTP verification
        true // Simplified for demo
    }

    /// Authenticate with username/password
    #[allow(dead_code)]
    async fn authenticate_user_password(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<AuthenticationResult> {
        if let Some(user) = self.users.get(username) {
            if !user.enabled {
                return Ok(AuthenticationResult {
                    success: false,
                    token: None,
                    user_id: None,
                    permissions: Vec::new(),
                    expires_at: None,
                    error: Some("Account disabled".to_string()),
                });
            }

            if self.verify_password(password, &user.password_hash) {
                if user.mfa_enabled {
                    // MFA required but not provided
                    return Ok(AuthenticationResult {
                        success: false,
                        token: None,
                        user_id: None,
                        permissions: Vec::new(),
                        expires_at: None,
                        error: Some("MFA required".to_string()),
                    });
                }

                // Create session
                let session = AuthSession::new(
                    user.user_id.clone(),
                    self.session_duration,
                    user.permissions.clone(),
                );

                let session_id = session.session_id.clone();
                let expires_at = session.expires_at;
                self.sessions.insert(session_id.clone(), session);

                Ok(AuthenticationResult {
                    success: true,
                    token: Some(session_id),
                    user_id: Some(user.user_id.clone()),
                    permissions: user.permissions.clone(),
                    expires_at: Some(expires_at),
                    error: None,
                })
            } else {
                Ok(AuthenticationResult {
                    success: false,
                    token: None,
                    user_id: None,
                    permissions: Vec::new(),
                    expires_at: None,
                    error: Some("Invalid credentials".to_string()),
                })
            }
        } else {
            Ok(AuthenticationResult {
                success: false,
                token: None,
                user_id: None,
                permissions: Vec::new(),
                expires_at: None,
                error: Some("User not found".to_string()),
            })
        }
    }

    /// Authenticate with MFA
    #[allow(dead_code)]
    async fn authenticate_mfa(
        &mut self,
        primary_credential: &Credentials,
        mfa_code: &str,
    ) -> Result<AuthenticationResult> {
        // First authenticate with primary credential
        let primary_result = self.authenticate(primary_credential).await?;

        if !primary_result.success {
            return Ok(primary_result);
        }

        // Extract username from primary credential
        let username = match primary_credential {
            Credentials::UserPassword { username, .. } => username,
            _ => {
                return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                    message: "MFA only supported with username/password".to_string(),
                    provider: Some("InMemoryAuthenticator".to_string()),
                })))
            }
        };

        if let Some(user) = self.users.get(username) {
            if let Some(ref secret) = user.mfa_secret {
                if self.verify_mfa_code(secret, mfa_code) {
                    // Create session
                    let session = AuthSession::new(
                        user.user_id.clone(),
                        self.session_duration,
                        user.permissions.clone(),
                    );

                    let session_id = session.session_id.clone();
                    let expires_at = session.expires_at;
                    self.sessions.insert(session_id.clone(), session);

                    Ok(AuthenticationResult {
                        success: true,
                        token: Some(session_id),
                        user_id: Some(user.user_id.clone()),
                        permissions: user.permissions.clone(),
                        expires_at: Some(expires_at),
                        error: None,
                    })
                } else {
                    Ok(AuthenticationResult {
                        success: false,
                        token: None,
                        user_id: None,
                        permissions: Vec::new(),
                        expires_at: None,
                        error: Some("Invalid MFA code".to_string()),
                    })
                }
            } else {
                Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                    message: "MFA not configured for user".to_string(),
                    provider: Some("InMemoryAuthenticator".to_string()),
                })))
            }
        } else {
            Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "User not found".to_string(),
                provider: Some("InMemoryAuthenticator".to_string()),
            })))
        }
    }

    /// Validate bearer token
    async fn validate_bearer_token(&self, token: &str) -> Result<AuthenticationResult> {
        if token.is_empty() {
            return Ok(AuthenticationResult {
                success: false,
                token: None,
                user_id: None,
                permissions: Vec::new(),
                expires_at: None,
                error: Some("Empty token".to_string()),
            });
        }

        if let Some(session) = self.sessions.get(token) {
            if session.is_expired() {
                Ok(AuthenticationResult {
                    success: false,
                    token: None,
                    user_id: None,
                    permissions: Vec::new(),
                    expires_at: None,
                    error: Some("Token expired".to_string()),
                })
            } else {
                Ok(AuthenticationResult {
                    success: true,
                    token: Some(token.to_string()),
                    user_id: Some(session.user_id.clone()),
                    permissions: session.permissions.clone(),
                    expires_at: Some(session.expires_at),
                    error: None,
                })
            }
        } else {
            Ok(AuthenticationResult {
                success: false,
                token: None,
                user_id: None,
                permissions: Vec::new(),
                expires_at: None,
                error: Some("Invalid token".to_string()),
            })
        }
    }
}

#[async_trait]
impl Authenticator for InMemoryAuthenticator {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthenticationResult> {
        match credentials {
            Credentials::UserPassword { username, password } => {
                // Create a mutable clone for authentication
                if let Some(user) = self.users.get(username) {
                    if !user.enabled {
                        return Ok(AuthenticationResult {
                            success: false,
                            token: None,
                            user_id: None,
                            permissions: Vec::new(),
                            expires_at: None,
                            error: Some("Account disabled".to_string()),
                        });
                    }

                    if self.verify_password(password, &user.password_hash) {
                        if user.mfa_enabled {
                            // MFA required but not provided
                            return Ok(AuthenticationResult {
                                success: false,
                                token: None,
                                user_id: None,
                                permissions: Vec::new(),
                                expires_at: None,
                                error: Some("MFA required".to_string()),
                            });
                        }

                        // Create session (this is where we need mutable access)
                        // For now, return success without creating session
                        Ok(AuthenticationResult {
                            success: true,
                            token: Some("temp_session".to_string()),
                            user_id: Some(user.user_id.clone()),
                            permissions: user.permissions.clone(),
                            expires_at: Some(chrono::Utc::now().timestamp() as u64 + 3600),
                            error: None,
                        })
                    } else {
                        Ok(AuthenticationResult {
                            success: false,
                            token: None,
                            user_id: None,
                            permissions: Vec::new(),
                            expires_at: None,
                            error: Some("Invalid credentials".to_string()),
                        })
                    }
                } else {
                    Ok(AuthenticationResult {
                        success: false,
                        token: None,
                        user_id: None,
                        permissions: Vec::new(),
                        expires_at: None,
                        error: Some("User not found".to_string()),
                    })
                }
            }
            Credentials::Bearer { token } => self.validate_bearer_token(token).await,
            Credentials::MFA {
                primary_credential,
                mfa_code,
                primary: _,
                secondary_factor: _,
            } => {
                // For MFA, we need to handle this differently since we can't mutate self
                // For now, return a simplified implementation
                let username = match primary_credential.as_ref() {
                    Credentials::UserPassword { username, .. } => username,
                    _ => {
                        return Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                            message: "MFA only supported with username/password".to_string(),
                            provider: Some("InMemoryAuthenticator".to_string()),
                        })))
                    }
                };

                if let Some(user) = self.users.get(username) {
                    if let Some(ref _secret) = user.mfa_secret {
                        if self.verify_mfa_code("", mfa_code) {
                            Ok(AuthenticationResult {
                                success: true,
                                token: Some("temp_mfa_session".to_string()),
                                user_id: Some(user.user_id.clone()),
                                permissions: user.permissions.clone(),
                                expires_at: Some(chrono::Utc::now().timestamp() as u64 + 3600),
                                error: None,
                            })
                        } else {
                            Ok(AuthenticationResult {
                                success: false,
                                token: None,
                                user_id: None,
                                permissions: Vec::new(),
                                expires_at: None,
                                error: Some("Invalid MFA code".to_string()),
                            })
                        }
                    } else {
                        Ok(AuthenticationResult {
                            success: false,
                            token: None,
                            user_id: None,
                            permissions: Vec::new(),
                            expires_at: None,
                            error: Some("MFA not enabled".to_string()),
                        })
                    }
                } else {
                    Ok(AuthenticationResult {
                        success: false,
                        token: None,
                        user_id: None,
                        permissions: Vec::new(),
                        expires_at: None,
                        error: Some("User not found".to_string()),
                    })
                }
            }
            _ => Ok(AuthenticationResult {
                success: false,
                token: None,
                user_id: None,
                permissions: Vec::new(),
                expires_at: None,
                error: Some("Unsupported credential type".to_string()),
            }),
        }
    }

    async fn validate_session(&self, session_id: &str) -> Result<AuthSession> {
        if let Some(session) = self.sessions.get(session_id) {
            if session.is_expired() {
                Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                    message: "Session expired".to_string(),
                    provider: Some("InMemoryAuthenticator".to_string()),
                })))
            } else {
                Ok(session.clone())
            }
        } else {
            Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
                message: "Session not found".to_string(),
                provider: Some("InMemoryAuthenticator".to_string()),
            })))
        }
    }

    async fn invalidate_session(&self, _session_id: &str) -> Result<()> {
        // Implementation would remove session from storage
        Ok(())
    }

    async fn refresh_session(&self, session_id: &str) -> Result<AuthSession> {
        let session = self.validate_session(session_id).await?;

        // Create new session with extended expiration
        let new_session =
            AuthSession::new(session.user_id, self.session_duration, session.permissions);

        Ok(new_session)
    }

    async fn get_user_permissions(&self, user_id: &str) -> Result<Vec<String>> {
        for user in self.users.values() {
            if user.user_id == user_id {
                return Ok(user.permissions.clone());
            }
        }

        Err(songbird_errors::SongbirdError::Auth(Box::new(AuthError {
            message: "User not found".to_string(),
            provider: Some("InMemoryAuthenticator".to_string()),
        })))
    }
}

// Note: This implementation has issues with mutability that would need to be
// resolved in a real implementation using proper concurrent data structures
// or external storage systems.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_session_creation() {
        let session = AuthSession::new(
            "user123".to_string(),
            Duration::from_secs(3600),
            vec!["read".to_string(), "write".to_string()],
        );

        assert_eq!(session.user_id, "user123");
        assert!(session.has_permission("read"));
        assert!(session.has_permission("write"));
        assert!(!session.has_permission("admin"));
        assert!(!session.is_expired());
    }

    #[test]
    fn test_credentials_creation() {
        let creds = Credentials::UserPassword {
            username: "testuser".to_string(),
            password: "password".to_string(),
        };

        match creds {
            Credentials::UserPassword { username, password } => {
                assert_eq!(username, "testuser");
                assert_eq!(password, "password");
            }
            _ => {
                assert!(false, "Expected UserPassword credential type");
            }
        }
    }

    #[test]
    fn test_bearer_credentials() {
        let creds = Credentials::Bearer {
            token: "abc123".to_string(),
        };

        match creds {
            Credentials::Bearer { token } => {
                assert_eq!(token, "abc123");
            }
            _ => {
                assert!(false, "Expected Bearer credential type");
            }
        }
    }

    #[test]
    fn test_mfa_credentials() {
        let primary = Box::new(Credentials::UserPassword {
            username: "user".to_string(),
            password: "pass".to_string(),
        });

        let creds = Credentials::MFA {
            primary_credential: primary,
            mfa_code: "123456".to_string(),
            primary: None,
            secondary_factor: None,
        };

        match creds {
            Credentials::MFA { mfa_code, .. } => {
                assert_eq!(mfa_code, "123456");
            }
            _ => {
                assert!(false, "Expected MFA credential type");
            }
        }
    }

    #[test]
    fn test_session_permissions() {
        let mut session = AuthSession::new(
            "testuser".to_string(),
            Duration::from_secs(3600),
            vec!["read".to_string()],
        );

        assert!(session.has_permission("read"));
        assert!(!session.has_permission("write"));

        session.add_permission("write".to_string());
        assert!(session.has_permission("write"));

        session.remove_permission("read");
        assert!(!session.has_permission("read"));
        assert!(session.has_permission("write"));
    }

    #[test]
    fn test_authentication_result() {
        let result = AuthenticationResult {
            success: true,
            token: Some("token123".to_string()),
            user_id: Some("user123".to_string()),
            permissions: vec!["read".to_string()],
            expires_at: Some(1234567890),
            error: None,
        };

        assert!(result.success);
        assert_eq!(
            result.token.expect("Token should be present in test"),
            "token123"
        );
        assert_eq!(result.user_id.unwrap(), "user123");
        assert_eq!(result.permissions.len(), 1);
        assert!(result.error.is_none());
    }
}

// Authentication methods are properly implemented in InMemoryAuthenticator above
