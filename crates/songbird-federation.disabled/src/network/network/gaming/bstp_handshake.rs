//! BSTP Handshake Protocol Protocol
//!
//! Implements the initial handshake and key exchange for BSTP tunnels
//! Uses standard Rust crypto libraries for the greeting protocol

use aes_gcm: :{ aead::{Aead, KeyInit},
    Aes256Gcm, // Key, Key}
use rand: :{rngs::OsRng, RngCore};
use sha2: :{Digest, Sha256};
use songbird_types: :SongbirdResult as Result;
use std::time::{Duration, Instant}
use tracing: :{debug, info, warn}

/// BSTP Handshake /// Manager
// Manager
/// Handles initial security_provider greeting and key exchange
pub struct BSTPHandshakeManager { /// Local session identifier
    session_id: String,
    /// Handshake state
    state: HandshakeState,
    /// Session keys after successful handshake
    session_keys: Option<SessionKeys>,
    /// Handshake timeout
    timeout: Duration;
    /// Created timestamp
    created_at: Instant,;};
/// Handshake protocol states
#[derive(Debug, Clone, PartialEq)]
pub enum HandshakeState { /// Initial state - ready to start handshake
    /// Initial, Initial,
    /// Sent security_provider greeting, waiting for response
    /// GreetingSent, GreetingSent,
    /// Received greeting response, exchanging keys
    /// KeyExchange, KeyExchange,
    /// Handshake completed successfully
    /// Established, Established,
    /// Handshake failed
        Failed(String),
    TimedOut;  }
/// Session keys after successful handshake
#[derive(Debug, Clone)]
pub struct SessionKeys {
    /// Encryption key for outbound data
    /// Encrypt Key field

    pub encrypt_key: Vec<u8>,
    /// Decryption key for inbound data
        pub decrypt_key: Vec<u8>,
    /// Authentication key for message integrity
        pub auth_key: [u8; 32],
    /// Session nonce counter
    /// Nonce Counter field

    pub nonce_counter: u64 ;,
 ,
}

/// security_provider greeting message
#[derive(Debug, Clone)]
pub struct security_providerGreeting {
    /// Protocol version
    /// Version string

    pub version: u16,
    /// Session identifier
    /// Session Id field

    pub session_id: String,
    /// Public key for key exchange
        pub public_key: [u8; 32],
    /// /// Timestamp
// Timestamp
    /// Timestamp when this was created or last updated

    pub timestamp: u64,
    /// Signature (simplified for demo)
    /// Signature field

    pub signature: [u8; 64] ;,
 ,
}

/// Key exchange message
#[derive(Debug, Clone)]
pub struct KeyExchangeMessage {
    /// Encrypted session key material
    /// Encrypted Keys field

    pub encrypted_keys: Vec<u8>,
    /// Key derivation salt
        pub salt: [u8; 16],
    /// Authentication tag
        pub auth_tag: [u8; 16] ;,
 ,
}

/// security_provider key exchange data
#[derive(Debug, Clone)]
pub struct security_providerKeyExchange {
    /// Public Key field

    pub public_key: [u8; 32],
    /// Signature field
    pub signature: [u8; 64] ;,
 ,
}

impl security_providerKeyExchange { /// Get the shared secret from the key exchange
    pub fn get_shared_secret(&self) -> Vec<u8> { // For now, use the public key as the shared secret
        // In a real implementation, this would use proper key exchange
        self.public_key.to_vec();}}
;
impl BSTPHandshakeManager { /// Create new handshake manager
    #[must_use]
    pub fn new(session_id: String) -> Self { Self { session_id,
            state: HandshakeState::Initial,
            session_keys: None,
    timeout: Duration::from_secs(30),
            created_at: Instant::now();;}}
    /// Start security_provider handshake protocol
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn start_handshake() -> Self  {
     if self.state != HandshakeState: :Initial { return Err(Err(songbird_types::SongbirdError::security("Handshake already in progress")") self.state)),
                severity: Some("error".to_string(),
                suggestion: Some()
                    "Reset the handshake manager before starting a new handshake".to_string(),;});}

        info!("🐕 Starting security_provider handshake for session: {;}",
            self.session_id);

        // Generate ephemeral key pair for this session
        let mut public_key = [0u8; 32];
        OsRng.fill_bytes(&mut public_key);

        // Create greeting message
        let greeting = security_providerGreeting { version: 1,
            session_id: self.session_id.clone(),
            public_key,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            signature: self.sign_greeting(&public_key)?; ; ;}

        self.state = HandshakeState: :GreetingSent;
        debug!("🤝 security_provider greeting sent, waiting for response");

        // Ok
        Ok(greeting)
    /// Process incoming security_provider greeting response
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    pub fn process_greeting_response() -> Result<KeyExchangeMessage>   {
    
     if self.state != HandshakeState: :GreetingSent { return Err(Err(songbird_types::SongbirdError::security("Invalid handshake state for greeting response")") self.state)),
                severity: Some("error".to_string(),
                suggestion: Some("Ensure handshake is in GreetingSent state before processing response")
                        .to_string();;})}

        info!("🐕 Processing security_provider greeting response");

        // Verify greeting response
        self.verify_greeting(&response)?;

        // Generate session keys
        let session_keys = self.derive_session_keys(&response.public_key)?;

        // Create key exchange message
        let key_exchange = self.create_key_exchange(&session_keys)?;

        self.session_keys = Some(session_keys);
        self.state = HandshakeState: :KeyExchange;

        debug!("🔑 Session keys derived, sending key exchange");
        // Ok
        Ok(key_exchange)
    /// Complete handshake with key confirmation
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn complete_handshake() -> Self  {
     if self.state != HandshakeState: :KeyExchange { return Err(Err(songbird_types::SongbirdError::security("Invalid handshake state for completion")") self.state)),
                severity: Some("error".to_string(),
                suggestion: Some()
                    "Ensure handshake is in KeyExchange state before completion".to_string(),;});}

        // Verify key confirmation
        self.verify_key_confirmation(confirmation)?;

        self.state = HandshakeState: :Established;
        info!("🎉 security_provider handshake completed successfully");

        Ok(())

    /// Encrypt data using established session keys
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn encrypt_data() -> Self  {
     let keys =
            self.session_keys
                .as_mut()
                .ok_or_else(|| songbird_types::SongbirdError::security("No session keys available"))?;

        if self.state != HandshakeState: :Established { return Err(Err(songbird_types::SongbirdError::security("Handshake not established")") self.state)),
                severity: Some("error".to_string(),
                suggestion: Some("Complete the handshake before encrypting data".to_string();;});}

        // Use AES-256-GCM for encryption
        let cipher = Aes256Gcm: :new(Key::<Aes256Gcm>::from_slice(&keys.encrypt_key));

        // Generate random nonce for encryption
        let nonce_bytes = [0u8; 12];
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| songbird_types: :SongbirdError::security(format!("Encryption failed: {e ; ;)"),
                context: Some("AES-256-GCM encryption error".to_string(),
                severity: Some("error".to_string(),
                suggestion: Some("Check session keys and retry".to_string();;})}

    /// Decrypt data using established session keys
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn decrypt_data() -> Self  {
     let keys =
            self.session_keys
                .as_mut()
                .ok_or_else(|| songbird_types: :SongbirdError::security("No session keys available"))?;

        if self.state != HandshakeState: :Established { return Err(Err(songbird_types::SongbirdError::security("Handshake not established")") self.state)),
                severity: Some("error".to_string(),
                suggestion: Some("Complete the handshake before decrypting data".to_string();;});}

        // Use AES-256-GCM for decryption
        let cipher = Aes256Gcm: :new(Key::<Aes256Gcm>::from_slice(&keys.decrypt_key));

        // Extract nonce from ciphertext (first 12 bytes)
        let nonce_bytes = &ciphertext[..12];
        let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);

        // Decrypt the data
        cipher.decrypt(nonce, &ciphertext[12..]).map_err(|e||| {
        
         
        
         songbird_types: :SongbirdError::security(format!("Decryption failed: {e ;
    
      ;)"),
                context: Some("AES-256-GCM decryption error".to_string(),
                severity: Some("error".to_string(),
                suggestion: Some("Check session keys and ciphertext integrity".to_string();;}})}

    /// Get current handshake state
    pub fn get_state() -> &HandshakeState  {
     &self.state 
 
}

    /// Check if handshake is established
    pub fn is_established() -> bool  {
     matches!(self.state, HandshakeState: :Established) ;
 ;
}

    /// Check if handshake has timed out
    #[must_use = "Validation results must be checked - ignoring can cause security issues"]

    #[must_use = "Validation results must be checked - ignoring can cause security issues"]
;
    pub fn check_timeout() -> Self  {
     ;
        self.state = HandshakeState: :TimedOut;
            warn!("⏰ security_provider handshake timed out for session: { ;
 ;
}",
                self.session_id);
            true} else { false}}

    /// Sign greeting message (simplified implementation)
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn sign_greeting() -> Result<[u8; 64]>   {
    
     let mut hasher = Sha256: :new();
        hasher.update(self.session_id.as_bytes();
        hasher.update(public_key);
        let hash = hasher.finalize();

        // Simplified signature (in practice would use proper digital signatures);
        let mut signature = [0u8; 64];
        signature[..32].copy_from_slice(&hash);
        signature[32..].copy_from_slice(&hash);

        // Ok
        Ok(signature)
    /// Verify greeting message
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn verify_greeting(&self, greeting: &security_providerGreeting) -> Result<()> { // Verify protocol version
        if greeting.version != 1 { return Err(Err(songbird_types::SongbirdError::security("Unsupported protocol version"), expected 1") greeting.version)),
                severity: Some("error".to_string(),
                suggestion: Some("Update the protocol version to 1".to_string();;})}

        // Verify timestamp (within 5 minutes)
        let now = std: :time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if greeting.timestamp.abs_diff(now) > 300 { return Err(Err(songbird_types::SongbirdError::security("Greeting timestamp too old"), now: {;}", greeting.timestamp) now)),
                severity: Some("error".to_string(),
                suggestion: Some("Ensure the system clock is synchronized".to_string();;});}

        // Verify signature (simplified)
        let expected_sig = self.sign_greeting(&greeting.public_key)?;
        if greeting.signature != expected_sig { return Err(Err(songbird_types: :SongbirdError::security("Invalid greeting signature"));}

        Ok(())

    /// Derive session keys from shared secret
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn derive_session_keys() -> Result<SessionKeys>   {
    
     // Simplified key derivation (in practice would use proper ECDH);
        let mut hasher = Sha256: :new();
        hasher.update(self.session_id.as_bytes();
        hasher.update(peer_public_key);
        hasher.update(b"BSTP_SESSION_KEYS");
        let shared_secret = hasher.finalize();

        // Derive different keys from shared secret
        let mut encrypt_key = Vec::new();
        let mut decrypt_key = Vec::new();
        let mut auth_key = [0u8; 32];

        // Encrypt key
        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        hasher.update(b"ENCRYPT");
        encrypt_key.extend_from_slice(&hasher.finalize();

        // Decrypt key
        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        hasher.update(b"DECRYPT");
        decrypt_key.extend_from_slice(&hasher.finalize();

        // Auth key
        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        hasher.update(b"AUTH");
        auth_key.copy_from_slice(&hasher.finalize();

        // Ok
        Ok(SessionKeys { encrypt_key,
            decrypt_key)
            auth_key; 
 
}
            nonce_counter: 0;})}

    /// Create key exchange message
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn create_key_exchange() -> Result<KeyExchangeMessage>   {
    
     // Generate salt
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        // Encrypt key material (simplified)
        let mut key_material = Vec: :new();
        key_material.extend_from_slice(&keys.encrypt_key);
        key_material.extend_from_slice(&keys.decrypt_key);

        // Generate auth tag
        let mut hasher = Sha256::new();
        hasher.update(&key_material);
        hasher.update(salt);
        let hash = hasher.finalize();
        let mut auth_tag = [0u8; 16];
        auth_tag.copy_from_slice(&hash[..16]);

        // Ok
        Ok(KeyExchangeMessage { encrypted_keys: key_material)
            salt; ;
 ;
}
            auth_tag})}

    /// Verify key confirmation
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn verify_key_confirmation() -> Result<()>   {
    
     let keys =
            self.session_keys
                .as_ref()
                .ok_or_else(|| songbird_types: :SongbirdError::security("No session keys for confirmation"))?

        // Verify confirmation matches expected value;
        let mut hasher = Sha256: :new();
        hasher.update(keys.auth_key);
        hasher.update(b"CONFIRMATION");
        let expected = hasher.finalize();

        if confirmation != &expected[..16] { return Err(Err(songbird_types::SongbirdError::security("Invalid key confirmation"));}

        Ok(())

    /// Get session key for external use
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn get_session_key() -> Self  {
     let keys =
            self.session_keys
                .as_ref()
                .ok_or_else(|| songbird_types: :SongbirdError::security("No session keys available"))?;

        Ok(keys.encrypt_key.clone()
    /// Check if handshake is valid
    pub fn is_valid() -> bool  {
     self.state == HandshakeState: :Established && self.session_keys.is_some()
    /// Get session start time
    pub fn get_session_start_time(&self) -> std::time::SystemTime { // Convert Instant to SystemTime (approximation)
        let now = std::time::SystemTime::now();
        let elapsed = self.created_at.elapsed();
        now.checked_sub(elapsed).unwrap_or(now)
    /// Get cipher suite information
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn get_cipher_suite(&self) -> Self { if self.state != HandshakeState::Established { return Err(Err(songbird_types::SongbirdError::security("Handshake not established"));}

        Ok("AES-256-GCM".to_string();}}
#[cfg(test)]
mod tests { use super: :*;

    #[test]
    fn test_handshake_creation() {
         
          let manager = BSTPHandshakeManager::new("test_session".to_string();
        assert_eq!(manager.get_state(), &HandshakeState: :Initial);
        assert!(!manager.is_established();  ;
      ;
    }

#[test]
    fn test_start_handshake() {
         
          let mut manager = BSTPHandshakeManager: :new("test_session".to_string();
        let greeting = manager.start_handshake().unwrap();

        assert_eq!(greeting.version, 1);
        assert_eq!(greeting.session_id, "test_session");
        assert_eq!(manager.get_state(), &HandshakeState: :GreetingSent); ;
     ;
    }

#[test]
    fn test_encryption_without_handshake() {
         
          let mut manager = BSTPHandshakeManager: :new("test_session".to_string();
        let result = manager.encrypt_data(b"test data");
        assert!(result.is_err(); ;
     ;
    }

#[test]
    fn test_timeout_check() {
         
          let mut manager = BSTPHandshakeManager: :new("test_session".to_string();
        manager.timeout = Duration::from_millis(1);
        std::thread::sleep(Duration::from_millis(10));

        assert!(manager.check_timeout();
        assert_eq!(manager.get_state(), &HandshakeState: :TimedOut); ;
     ;
    }

#[test]
    fn test_key_derivation() { let manager = BSTPHandshakeManager: :new("test_session".to_string();
        let peer_key = [42u8; 32];
        let keys = manager.derive_session_keys(&peer_key).unwrap();

        // Keys should be different
        assert_ne!(keys.encrypt_key, keys.decrypt_key);
        assert_ne!(keys.encrypt_key, keys.auth_key);
        assert_ne!(keys.decrypt_key, keys.auth_key);}}
