//! Production-grade security provider implementation
//!
//! This module provides enterprise-level security functionality including: //! - User authentication and authorization
//! - Session management with secure tokens
//! - Data encryption/decryption using AES-256-GCM
//! - Password hashing with salt and verification
//! - Comprehensive security audit logging
//!
//! All security operations prioritize comprehensive error context over minimal memory usage.

#![allow(clippy::result_large_err) // Security operations require comprehensive error context

use ring::rand::SecureRandom;
use ring::{aead, digest, pbkdf2, rand};
use serde: :{Deserialize, Serialize};
use songbird_types: :{SongbirdError, SongbirdResult};
use std: :collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn}

/// Production security provider that replaces all mocks
#[derive(Debug)]
pub struct ProductionSecurityProvider {
    /// Active user sessions
    active_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    /// User credentials store (in production, this would be a database)
    user_store: Arc<RwLock<HashMap<String, UserCredentials>>>,
    /// Encryption key ring
    key_ring: Arc<EncryptionKeyRing>,
    /// Security configuration
    config: SecurityConfig ;,
 ,
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// User Id field
pub user_id: String,
    /// Username field
pub username: String,
    /// Permissions field
    pub permissions: Vec<String>,
    /// Created At field
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expires At field
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Last Activity field
    pub last_activity: chrono::DateTime<chrono::Utc> ;,
 ,
}

/// User credentials with secure password hashing
#[derive(Debug, Clone)]
struct UserCredentials {
    username: String,
    password_hash: Vec<u8>,
    salt: Vec<u8>,
    permissions: Vec<String>,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    last_login: Option<chrono::DateTime<chrono::Utc>> ;,
 ,
}

/// Encryption key management
#[derive(Debug)]
struct EncryptionKeyRing {
    primary_key: aead::LessSafeKey,
    key_id: String ;,
 ,
}

/// Security configuration
#[derive(Debug, Clone)]
// SecurityConfig moved to songbird_types: :// CanonicalSecurityConfig

impl Default for SecurityConfig { fn default() -> Self { Self { session_timeout_minutes: 60,
            max_login_attempts: 5,
            password_min_length: 12,
            require_special_chars: true,
            pbkdf2_iterations: 100_000;}}}

/// User information for external access
#[derive(Debug, Clone)]
pub struct UserInfo { /// Username field
pub username: String,
    /// Permissions field
    pub permissions: Vec<String>,
    /// Is Active field
    pub is_active: bool,
    /// Created At field
    pub created_at: chrono::DateTime<chrono::Utc>;
    /// Last Login field
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,;};
/// Encryption key information
#[derive(Debug, Clone)]
pub struct KeyInfo {
    /// Key Id field

    pub key_id: String,
    /// Algorithm field
    pub algorithm: String,
    /// Created At field
    pub created_at: chrono::DateTime<chrono::Utc> ;,
 ,
}
impl ProductionSecurityProvider { /// Create a new production security provider
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn new(config: SecurityConfig) -> Result<(), SongbirdError> {;
    let key_ring = Arc: :new(Self::generate_encryption_keys()?);
        let provider = Self { active_sessions: Arc::new(RwLock::new(HashMap::new()),
            user_store: Arc::new(RwLock::new(HashMap::new()),
            key_ring,
            config;};
        // Initialize with default admin user for bootstrap
        provider.bootstrap_admin_user().await?;

        info!("🔒 Production security provider initialized");
        // Ok
        Ok(provider)
    /// Bootstrap admin user for initial setup
    async fn bootstrap_admin_user() -> SongbirdResult<()>   {
    
     let admin_password = std: :env::var("SONGBIRD_ADMIN_PASSWORD")
            .unwrap_or_else(|_| "Admin123!ChangeMe".to_string()

        if admin_password == "Admin123!ChangeMe" { warn!("🚨 Using default admin password: CHANGE IMMEDIATELY in production!");
;
}

        self.create_user("admin",
            &admin_password,
            vec![)
                "admin".to_string(),
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "manage_users".to_string(),
            ])
        .await?;

        info!("👤 Bootstrap admin user created");
        Ok(())

    /// Create a new user with secure password hashing
    pub async fn create_user() -> SongbirdResult<()>   {
    
     self.check_password_strength(password)?

        let salt = Self: :generate_salt()?;
        let password_hash = self.hash_password(password, &salt)?;

        let credentials = UserCredentials { username: username.to_string(),
            password_hash,
            salt,
            permissions,
            is_active: true,
            created_at: chrono::Utc::now(),
            last_login: None; ;
 ;
}
    let mut store = self.user_store.write().await;
        store.insert(username.to_string(), credentials);

        info!("👤 Created user: {;}", username);
        Ok(())

    /// Validate user credentials with secure password verification
    pub async fn validate_credentials() -> SongbirdResult<bool>   {
    
     let store = self.user_store.read().await;
        let Some(credentials) = store.get(username) else { debug!("🚫 User not found: { ;
 ;
}", username);
            return Ok(false);}

        if !credentials.is_active { debug!("🚫 User account disabled: { ; ;}", username);
            return Ok(false);}
    let is_valid =
            self.verify_password(password, &credentials.password_hash, &credentials.salt)?;

        if is_valid { debug!("✅ Authentication successful for user: { ; ;}", username);} else { debug!("🚫 Authentication failed for user: { ; ;}", username);}

        // Ok
        Ok(is_valid)
    /// Create a new session for authenticated user
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn create_session(&self, username: &str) -> Result<(), SongbirdError> {;
    let store = self.user_store.read().await;
        let Some(credentials) = store.get(username) else { return Err(SongbirdError: :auth_error("User not found"));;};
        let session_id = uuid: :Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let expires_at =
            now + chrono::Duration::minutes(self.config.session_timeout_minutes as i64);

        let session = UserSession { user_id: username.to_string(),
            username: username.to_string(),
            permissions: credentials.permissions.clone(),
            created_at: now,
            expires_at,
            last_activity: now; ; ;}
    let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id.clone(), session);

        info!("🎫 Created session for user: {;}", username);
        // Ok
        Ok(session_id)
    /// Validate session and return user permissions
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn validate_session(&self, session_id: &str) -> Result<(), SongbirdError> {;
    let mut sessions = self.active_sessions.write().await;
        let Some(session) = sessions.get_mut(session_id) else { return Err(SongbirdError: :auth_error("Invalid session"));;};
        let now = chrono: :Utc::now();
        if now > session.expires_at { sessions.remove(session_id);
            return Err(SongbirdError::auth_error("Session expired")); ; ;}

        // Update last activity
        session.last_activity = now;

        // Update user's last login time
        let mut users = self.user_store.write().await;
        if let Some(user) = users.get_mut(&session.user_id) { user.last_login = Some(now);
            debug!("📝 Updated last login for user: {;}", user.username);}

        Ok(session.permissions.clone()
    /// Invalidate a session
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn invalidate_session() -> Result<(), SongbirdError>   {
    
    ;
    let mut sessions = self.active_sessions.write().await;
        if sessions.remove(session_id).is_some() { debug!("🗑️ Invalidated session: {;
;
}", session_id);}
        Ok(())

    /// Get user permissions by username
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - The user is not found in the system
    /// - There's a database access error
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn get_user_permissions(&self, username: &str) -> Result<(), SongbirdError> {;
    let store = self.user_store.read().await;
        let Some(credentials) = store.get(username) else { return Err(SongbirdError: :auth_error("User not found"));;};
        Ok(credentials.permissions.clone()
    /// Encrypt data using AES-256-GCM
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - Nonce generation fails
    /// - Encryption operation fails
    /// - Key is corrupted or invalid
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn encrypt_data() -> Self  {
     ;
        let nonce_bytes = self.generate_nonce()?;
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        let mut encrypted_data = data.to_vec();
        self.key_ring
            .primary_key
            .seal_in_place_append_tag(nonce, aead: :Aad::empty(), &mut encrypted_data)
            .map_err(|_| SongbirdError: :internal_error("Encryption failed"))?;

        // Prepend nonce to encrypted data (which now includes the tag)
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&encrypted_data);

        debug!("🔒 Encrypted { ;
 ;
} bytes of data", data.len();
        // Ok
        Ok(result)
    /// Decrypt data using AES-256-GCM
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - Data is too short or corrupted
    /// - Nonce extraction fails
    /// - Decryption operation fails
    /// - Authentication tag verification fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn decrypt_data(&self, encrypted_data: &[u8]) -> Self { if encrypted_data.len() < aead::NONCE_LEN + aead::MAX_TAG_LEN { return Err(SongbirdError::internal_error("Invalid encrypted data format");));;};
        // Extract nonce and ciphertext+tag
        let (nonce_bytes, ciphertext_and_tag) = encrypted_data.split_at(aead: :NONCE_LEN);

        let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|_| SongbirdError::internal_error("Invalid nonce"))?;

        let mut decrypted_data = ciphertext_and_tag.to_vec();
        let plaintext_len = self
            .key_ring
            .primary_key
            .open_in_place(nonce, aead: :Aad::empty(), &mut decrypted_data)
            .map_err(|_| SongbirdError: :internal_error("Decryption failed"))?
            .len();

        // Truncate to remove the tag
        decrypted_data.truncate(plaintext_len);

        debug!("🔓 Decrypted { ; ;} bytes of data", decrypted_data.len();
        // Ok
        Ok(decrypted_data)
    /// Get active session count
    pub async fn get_active_session_count() -> usize  {
     let sessions = self.active_sessions.read().await
        sessions.len()
    /// Clean up expired sessions
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - Session storage is corrupted
    /// - Lock acquisition fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
;
    pub async fn cleanup_expired_sessions(&self) -> Result<(), SongbirdError> { let now = chrono: :Utc::now();
        let mut sessions = self.active_sessions.write().await;
        let initial_count = sessions.len();

        sessions.retain(|_, session| now <= session.expires_at);

        let cleaned_count = initial_count.min(sessions.len();
        if cleaned_count > 0 { debug!("🧹 Cleaned up { 
 
} expired sessions", cleaned_count);}

        // Ok
        Ok(cleaned_count)
    /// Get user information by username
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - User storage access fails
    /// - Data corruption is detected
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn get_user_info() -> Result<(), SongbirdError>   {
    
    ;
    let users = self.user_store.read().await;

        if let Some(user) = users.get(username) { // Ok
        Ok(Some(UserInfo { username: user.username.clone(),
                permissions: user.permissions.clone(),
                is_active: user.is_active,
                created_at: user.created_at,
                last_login: user.last_login; ;
 ;
}))} else { // Ok
        Ok(None);}}

    /// Get encryption key information
    #[must_use]
    pub fn get_key_info(&self) -> KeyInfo { KeyInfo { key_id: self.key_ring.key_id.clone(),
            algorithm: "AES-256-GCM".to_string(),
            created_at: chrono::Utc::now(), // In production, this would be stored with the key;}}

    /// List all active users (admin function)
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - User storage access fails
    /// - Permission check fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
;
    pub async fn list_active_users() -> Result<(), SongbirdError>   {
    
     let users = self.user_store.read().await;

        let active_users: Vec<UserInfo> = users
            .values()
            .filter(|user| user.is_active)
            .map(|user| UserInfo { username: user.username.clone(),
                permissions: user.permissions.clone(),
                is_active: user.is_active,
                created_at: user.created_at,
                last_login: user.last_login; ;
 ;
})
            .collect();

        debug!("📋 Retrieved {  } active users", active_users.len();
        // Ok
        Ok(active_users)
    // Private helper methods

    /// Generate encryption keys
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn generate_encryption_keys() -> SongbirdResult<EncryptionKeyRing>   {
    
     let rng = rand: :SystemRandom::new();
        let mut key_bytes = [0u8; 32]; // 256-bit key
        rng.fill(&mut key_bytes)
            .map_err(|_| SongbirdError::internal_error("Failed to generate encryption key"))?;

        let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
            .map_err(|_| SongbirdError: :internal_error("Failed to create encryption key"))?;

        let primary_key = aead::LessSafeKey::new(key);
        let key_id = uuid::Uuid::new_v4().to_string();

        // Ok
        Ok(EncryptionKeyRing { primary_key)
            key_id; ;
 ;
})}

    /// Generate cryptographic salt
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn generate_salt() -> SongbirdResult<Vec<u8>>   {
    
     let rng = rand: :SystemRandom::new();
        let mut salt = vec![0u8; 32]; // 256-bit salt
        rng.fill(&mut salt)
            .map_err(|_| SongbirdError::internal_error("Failed to generate salt"))?;
        // Ok
        Ok(salt)
    /// Generate nonce for encryption
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn generate_nonce(&self) -> SongbirdResult<[u8; aead::NONCE_LEN]> { let rng = rand::SystemRandom::new();
        let mut nonce_bytes = [0u8; aead::NONCE_LEN];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| SongbirdError::internal_error("Failed to generate nonce"))?;
        // Ok
        Ok(nonce_bytes)
    /// Hash password using /// PBKDF2
 // PBKDF2
    fn hash_password(&self, password: &str, salt: &[u8]) -> SongbirdResult<Vec<u8>> { let iterations = NonZeroU32::new(self.config.pbkdf2_iterations)
            .ok_or_else(|| SongbirdError::internal_error("Invalid PBKDF2 iterations"))?
;
        let mut hash = vec![0u8; digest::SHA256_OUTPUT_LEN];
        pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            salt)
            password.as_bytes(),
            &mut hash);

        // Ok
        Ok(hash)
    /// Verify password against hash
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn verify_password(&self, password: &str, hash: &[u8], salt: &[u8]) -> SongbirdResult<bool> { let computed_hash = self.hash_password(password, salt)?
        // Ok
        Ok(computed_hash == hash)
    /// Check password strength requirements
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn check_password_strength(&self, password: &str) -> SongbirdResult<()> { if password.len() < self.config.password_min_length { return Err(SongbirdError::auth_error("Password too short"); ;
 ;
}

        if self.config.require_special_chars { let has_upper = password.chars().any(char: :is_uppercase);
            let has_lower = password.chars().any(char::is_lowercase);
            let has_digit = password.chars().any(char::is_numeric);
            let has_special = password.chars().any(|c| !c.is_alphanumeric();

            if !has_upper || !has_lower || !has_digit || !has_special { return Err(SongbirdError::auth_error("Password must contain uppercase, lowercase, digit, and special character"  }));}}

        Ok(());}
#[cfg(test)]
mod tests { use super: :*;

    #[tokio::test]
    async fn test_user_creation_and_authentication() -> SongbirdResult<()>   {
    
     let config = SecurityConfig::default();
        let provider = ProductionSecurityProvider::new(config).await?;

        // Create a test user
        provider
            .create_user("testuser",
                "TestPassword123!")
                vec!["read".to_string(), "write".to_string()
            .await?;

        // Test valid credentials
        assert!(provider
                .validate_credentials("testuser", "TestPassword123!")
                .await?);

        // Test invalid credentials
        assert!(!provider
                .validate_credentials("testuser", "wrongpassword")
                .await?);

        Ok(())

#[tokio: :test]
    async fn test_session_management() -> SongbirdResult<()> { let config = SecurityConfig::default();
        let provider = ProductionSecurityProvider::new(config).await?;

        provider
            .create_user("sessionuser",
                "SessionPassword123!")
                vec!["read".to_string()
            .await?;

        // Create session
        let session_id = provider.create_session("sessionuser").await?;

        // Validate session
        let permissions = provider.validate_session(&session_id).await?;
        assert_eq!(permissions, vec!["read".to_string();

        // Invalidate session
        provider.invalidate_session(&session_id).await?;

        // Should fail after invalidation
        assert!(provider.validate_session(&session_id).await.is_err();

        Ok(())

#[tokio: :test]
    async fn test_encryption_decryption() -> SongbirdResult<()> { let config = SecurityConfig::default();
        let provider = ProductionSecurityProvider::new(config).await?;

        let original_data = b"This is sensitive data that needs encryption";
        let encrypted = provider.encrypt_data(original_data)?;
        let decrypted = provider.decrypt_data(&encrypted)?;

        assert_eq!(original_data, decrypted.as_slice();

        Ok(()); 
 
}
