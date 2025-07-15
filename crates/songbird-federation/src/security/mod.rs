//! Security management for federation
//!
//! Handles BearDog integration and secure connections

use chrono::Utc;
use ring::rand::SecureRandom;
use ring::signature::KeyPair;
use ring::{rand, signature};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::*;
use songbird_errors::Result;

/// Security manager for federation
pub struct SecurityManager {
    config: SecurityConfig,
    /// Active security sessions
    sessions: Arc<RwLock<HashMap<String, SecuritySession>>>,
    /// Key pair for this node
    key_pair: signature::Ed25519KeyPair,
    /// Random number generator
    rng: rand::SystemRandom,
}

impl SecurityManager {
    /// Create new security manager
    pub async fn new(config: SecurityConfig) -> Result<Self> {
        let rng = rand::SystemRandom::new();

        // Generate a new key pair for this node
        let key_pair_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).map_err(|e| {
            songbird_errors::SongbirdError::service_error(
                "security",
                format!("Failed to generate key pair: {e}"),
            )
        })?;

        let key_pair =
            signature::Ed25519KeyPair::from_pkcs8(key_pair_bytes.as_ref()).map_err(|e| {
                songbird_errors::SongbirdError::service_error(
                    "security",
                    format!("Failed to create key pair: {e}"),
                )
            })?;

        Ok(Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            key_pair,
            rng,
        })
    }

    /// Start security manager
    pub async fn start(&self) -> Result<()> {
        // Start session cleanup task
        let sessions = self.sessions.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;

                // Remove expired sessions
                let mut sessions_guard = sessions.write().await;
                let now = Utc::now();
                sessions_guard.retain(|_, session| session.expires_at > now);
            }
        });

        Ok(())
    }

    /// Establish secure session with node
    pub async fn establish_session(&self, node: &FederationNode) -> Result<SecuritySession> {
        let session_id = Uuid::new_v4().to_string();

        // Create challenge for node authentication
        let challenge = self.create_authentication_challenge(node).await?;

        // Sign the challenge with our key
        let _signature = self.key_pair.sign(&challenge);
        let key_fingerprint = self.get_public_key_fingerprint();

        // Create session
        let session = SecuritySession {
            session_id: session_id.clone(),
            key_fingerprint,
            security_level: self.config.required_security_level.clone(),
            established_at: Utc::now(),
            expires_at: Utc::now()
                + chrono::Duration::seconds(self.config.session_timeout.as_secs() as i64),
        };

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session.clone());
        }

        tracing::info!("Established secure session with node: {}", node.name);
        Ok(session)
    }

    /// Verify session is valid
    pub async fn verify_session(&self, session: &SecuritySession) -> Result<bool> {
        let now = Utc::now();

        // Check if session is expired
        if session.expires_at <= now {
            return Ok(false);
        }

        // Check if session exists in our store
        let sessions = self.sessions.read().await;
        if let Some(stored_session) = sessions.get(&session.session_id) {
            // Verify session details match
            Ok(stored_session.key_fingerprint == session.key_fingerprint
                && stored_session.security_level == session.security_level)
        } else {
            Ok(false)
        }
    }

    /// Revoke security session
    pub async fn revoke_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if sessions.remove(session_id).is_some() {
            tracing::info!("Revoked security session: {}", session_id);
        }
        Ok(())
    }

    /// Create authentication challenge for a node
    async fn create_authentication_challenge(&self, node: &FederationNode) -> Result<Vec<u8>> {
        let challenge_data = serde_json::json!({
            "node_id": node.node_id,
            "challenge_timestamp": Utc::now().timestamp(),
            "nonce": self.generate_nonce(),
        });

        Ok(challenge_data.to_string().into_bytes())
    }

    /// Generate cryptographic nonce
    fn generate_nonce(&self) -> String {
        let mut nonce = [0u8; 32];
        self.rng.fill(&mut nonce).unwrap();
        hex::encode(nonce)
    }

    /// Get public key fingerprint
    fn get_public_key_fingerprint(&self) -> String {
        let public_key = self.key_pair.public_key();
        let digest = ring::digest::digest(&ring::digest::SHA256, public_key.as_ref());
        hex::encode(digest.as_ref())
    }

    /// Encrypt data for secure transmission
    pub async fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // For now, use a simple encryption scheme
        // In production, this would use proper symmetric encryption
        let key = ring::aead::LessSafeKey::new(
            ring::aead::UnboundKey::new(&ring::aead::AES_256_GCM, &self.derive_encryption_key()?)
                .map_err(|e| {
                songbird_errors::SongbirdError::service_error(
                    "security",
                    format!("Failed to create encryption key: {e}"),
                )
            })?,
        );

        let mut encrypted = data.to_vec();
        let nonce = ring::aead::Nonce::assume_unique_for_key([0u8; 12]); // In production, use proper nonce

        key.seal_in_place_append_tag(nonce, ring::aead::Aad::empty(), &mut encrypted)
            .map_err(|e| {
                songbird_errors::SongbirdError::service_error(
                    "security",
                    format!("Failed to encrypt data: {e}"),
                )
            })?;

        Ok(encrypted)
    }

    /// Decrypt data from secure transmission
    pub async fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let key = ring::aead::LessSafeKey::new(
            ring::aead::UnboundKey::new(&ring::aead::AES_256_GCM, &self.derive_encryption_key()?)
                .map_err(|e| {
                songbird_errors::SongbirdError::service_error(
                    "security",
                    format!("Failed to create decryption key: {e}"),
                )
            })?,
        );

        let mut decrypted = encrypted_data.to_vec();
        let nonce = ring::aead::Nonce::assume_unique_for_key([0u8; 12]); // In production, use proper nonce

        key.open_in_place(nonce, ring::aead::Aad::empty(), &mut decrypted)
            .map_err(|e| {
                songbird_errors::SongbirdError::service_error(
                    "security",
                    format!("Failed to decrypt data: {e}"),
                )
            })?;

        Ok(decrypted)
    }

    /// Derive encryption key from node key pair
    fn derive_encryption_key(&self) -> Result<[u8; 32]> {
        let public_key = self.key_pair.public_key();
        let digest = ring::digest::digest(&ring::digest::SHA256, public_key.as_ref());
        let mut key = [0u8; 32];
        key.copy_from_slice(digest.as_ref());
        Ok(key)
    }

    /// Get security statistics
    pub async fn get_security_stats(&self) -> SecurityStats {
        let sessions = self.sessions.read().await;
        SecurityStats {
            active_sessions: sessions.len(),
            total_sessions_created: sessions.len(), // Simplified
            expired_sessions_cleaned: 0,            // Would track in real implementation
            encryption_operations: 0,               // Would track in real implementation
        }
    }
}

/// Security statistics
#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub active_sessions: usize,
    pub total_sessions_created: usize,
    pub expired_sessions_cleaned: usize,
    pub encryption_operations: usize,
}
