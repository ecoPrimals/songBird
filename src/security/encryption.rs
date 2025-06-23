//! Encryption Module
//!
//! Production-grade encryption using AES and other strong ciphers

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

/// Encryption configuration
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,
    /// Key size in bytes
    pub key_size: usize,
    /// Key derivation settings
    pub key_derivation: KeyDerivationConfig,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::AES256GCM,
            key_size: 32, // 256 bits
            key_derivation: KeyDerivationConfig::default(),
        }
    }
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES256GCM,
    ChaCha20Poly1305,
}

/// Key derivation configuration
#[derive(Debug, Clone)]
pub struct KeyDerivationConfig {
    /// Number of iterations for PBKDF2
    pub iterations: u32,
    /// Salt length
    pub salt_length: usize,
}

impl Default for KeyDerivationConfig {
    fn default() -> Self {
        Self {
            iterations: 100_000,
            salt_length: 16,
        }
    }
}

/// Encrypted data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Encryption algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Nonce/IV used for encryption
    pub nonce: Vec<u8>,
    /// Encrypted ciphertext with authentication tag
    pub ciphertext: Vec<u8>,
    /// Salt used for key derivation (if applicable)
    pub salt: Option<Vec<u8>>,
}

/// Production encryption provider
pub struct ProductionEncryptionProvider {
    config: EncryptionConfig,
    rng: SystemRandom,
}

impl ProductionEncryptionProvider {
    /// Create a new encryption provider
    pub fn new(config: EncryptionConfig) -> Self {
        Self {
            config,
            rng: SystemRandom::new(),
        }
    }

    /// Encrypt data with the given key
    pub fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<EncryptedData, Box<dyn std::error::Error>> {
        match self.config.algorithm {
            EncryptionAlgorithm::AES256GCM => {
                self.encrypt_aes256gcm(plaintext, key)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // TODO: Implement ChaCha20-Poly1305
                Err("ChaCha20-Poly1305 not implemented yet".into())
            }
        }
    }

    /// Decrypt data with the given key
    pub fn decrypt(&self, encrypted_data: &EncryptedData, key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match encrypted_data.algorithm {
            EncryptionAlgorithm::AES256GCM => {
                self.decrypt_aes256gcm(encrypted_data, key)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // TODO: Implement ChaCha20-Poly1305
                Err("ChaCha20-Poly1305 not implemented yet".into())
            }
        }
    }

    /// Derive a key from a password using PBKDF2
    pub fn derive_key(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use ring::pbkdf2;
        
        let mut key = vec![0u8; self.config.key_size];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(self.config.key_derivation.iterations).unwrap(),
            salt,
            password.as_bytes(),
            &mut key,
        );
        
        Ok(key)
    }

    /// Generate a random key
    pub fn generate_key(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut key = vec![0u8; self.config.key_size];
        self.rng.fill(&mut key)
            .map_err(|_| "Failed to generate random key")?;
        Ok(key)
    }

    /// Generate a random salt
    pub fn generate_salt(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut salt = vec![0u8; self.config.key_derivation.salt_length];
        self.rng.fill(&mut salt)
            .map_err(|_| "Failed to generate random salt")?;
        Ok(salt)
    }

    /// Encrypt using AES-256-GCM
    fn encrypt_aes256gcm(&self, plaintext: &[u8], key: &[u8]) -> Result<EncryptedData, Box<dyn std::error::Error>> {
        if key.len() != 32 {
            return Err("Key must be 32 bytes for AES-256".into());
        }

        // Create encryption key
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|_| "Failed to create encryption key")?;
        let encryption_key = LessSafeKey::new(unbound_key);

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12]; // 96-bit nonce for GCM
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| "Failed to generate nonce")?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        // Encrypt the data
        let aad = Aad::empty();
        let mut in_out = plaintext.to_vec();
        encryption_key
            .seal_in_place_append_tag(nonce, aad, &mut in_out)
            .map_err(|_| "Encryption failed")?;

        Ok(EncryptedData {
            algorithm: EncryptionAlgorithm::AES256GCM,
            nonce: nonce_bytes.to_vec(),
            ciphertext: in_out,
            salt: None,
        })
    }

    /// Decrypt using AES-256-GCM
    fn decrypt_aes256gcm(&self, encrypted_data: &EncryptedData, key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if key.len() != 32 {
            return Err("Key must be 32 bytes for AES-256".into());
        }

        if encrypted_data.nonce.len() != 12 {
            return Err("Invalid nonce length for AES-256-GCM".into());
        }

        // Create decryption key
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|_| "Failed to create decryption key")?;
        let decryption_key = LessSafeKey::new(unbound_key);

        // Reconstruct nonce
        let nonce_bytes: [u8; 12] = encrypted_data.nonce.as_slice()
            .try_into()
            .map_err(|_| "Invalid nonce length")?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        // Decrypt the data
        let aad = Aad::empty();
        let mut in_out = encrypted_data.ciphertext.clone();
        let plaintext = decryption_key
            .open_in_place(nonce, aad, &mut in_out)
            .map_err(|_| "Decryption failed")?;

        Ok(plaintext.to_vec())
    }
}

/// Encrypt data with password-based encryption
pub fn encrypt_with_password(
    plaintext: &[u8],
    password: &str,
) -> Result<EncryptedData, Box<dyn std::error::Error>> {
    let provider = ProductionEncryptionProvider::new(EncryptionConfig::default());
    
    // Generate salt
    let salt = provider.generate_salt()?;
    
    // Derive key from password
    let key = provider.derive_key(password, &salt)?;
    
    // Encrypt data
    let mut encrypted_data = provider.encrypt(plaintext, &key)?;
    encrypted_data.salt = Some(salt);
    
    Ok(encrypted_data)
}

/// Decrypt data with password-based encryption
pub fn decrypt_with_password(
    encrypted_data: &EncryptedData,
    password: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let provider = ProductionEncryptionProvider::new(EncryptionConfig::default());
    
    // Get salt from encrypted data
    let salt = encrypted_data.salt.as_ref()
        .ok_or("No salt found in encrypted data")?;
    
    // Derive key from password
    let key = provider.derive_key(password, salt)?;
    
    // Decrypt data
    provider.decrypt(encrypted_data, &key)
}

/// Simple utility to replace XOR encryption
pub fn secure_encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let provider = ProductionEncryptionProvider::new(EncryptionConfig::default());
    let encrypted = provider.encrypt(data, key)?;
    
    // Serialize the encrypted data for storage/transmission
    bincode::serialize(&encrypted)
        .map_err(|e| format!("Failed to serialize encrypted data: {}", e).into())
}

/// Simple utility to replace XOR decryption
pub fn secure_decrypt(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let provider = ProductionEncryptionProvider::new(EncryptionConfig::default());
    
    // Deserialize the encrypted data
    let encrypted: EncryptedData = bincode::deserialize(encrypted_data)
        .map_err(|e| format!("Failed to deserialize encrypted data: {}", e))?;
    
    provider.decrypt(&encrypted, key)
} 