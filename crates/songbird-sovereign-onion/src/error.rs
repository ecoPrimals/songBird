//! Error types for sovereign onion protocol

use thiserror::Error;

/// Result type alias for onion operations
pub type Result<T> = std::result::Result<T, OnionError>;

/// Errors that can occur in onion operations
#[derive(Debug, Error)]
pub enum OnionError {
    /// Invalid .onion address format
    #[error("Invalid .onion address format")]
    InvalidFormat,

    /// Invalid base32 encoding
    #[error("Invalid base32 encoding")]
    InvalidEncoding,

    /// Invalid address length
    #[error("Invalid address length: expected 35, got {0}")]
    InvalidLength(usize),

    /// Unsupported onion address version
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u8),

    /// Invalid Ed25519 public key
    #[error("Invalid Ed25519 public key")]
    InvalidPublicKey,

    /// Checksum mismatch in .onion address
    #[error("Checksum mismatch in .onion address")]
    ChecksumMismatch,

    /// Connection timeout
    #[error("Connection timeout")]
    ConnectionTimeout,

    /// Handshake failed
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),

    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(String),

    /// Invalid protocol message
    #[error("Invalid protocol message: {0}")]
    InvalidMessage(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Sled database error
    #[error("Database error: {0}")]
    Database(#[from] sled::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Ed25519 signature error
    #[error("Signature error: {0}")]
    Signature(String),

    /// X25519 key exchange error
    #[error("Key exchange error: {0}")]
    KeyExchange(String),

    /// AEAD encryption/decryption error
    #[error("AEAD error: {0}")]
    Aead(String),

    /// Generic error
    #[error("Onion error: {0}")]
    Other(String),

    // =========================================================================
    // BearDog Crypto Client Errors (TRUE PRIMAL pattern)
    // =========================================================================

    /// JSON-RPC error from BearDog
    #[error("RPC error: {0}")]
    RpcError(String),

    /// Connection error to BearDog socket
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Configuration error (missing socket, env vars, etc.)
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Generic crypto error
    #[error("Crypto error: {0}")]
    CryptoError(String),
}

impl From<ed25519_dalek::SignatureError> for OnionError {
    fn from(e: ed25519_dalek::SignatureError) -> Self {
        OnionError::Signature(e.to_string())
    }
}

// Note: base32 v0.5 doesn't have DecodeError type, it returns Option<Vec<u8>>
// Error handling is done at call site with ok_or(OnionError::InvalidEncoding)
