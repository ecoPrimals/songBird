// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
    // security provider Crypto Client Errors (TRUE PRIMAL pattern)
    // =========================================================================
    /// JSON-RPC error from `security provider`
    #[error("RPC error: {0}")]
    RpcError(String),

    /// Connection error to `security provider` socket
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Configuration error (missing socket, env vars, etc.)
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Generic crypto error
    #[error("Crypto error: {0}")]
    CryptoError(String),
}

// Only available in standalone/test mode (where ed25519_dalek is available)
#[cfg(feature = "standalone")]
impl From<ed25519_dalek::SignatureError> for OnionError {
    fn from(e: ed25519_dalek::SignatureError) -> Self {
        Self::Signature(e.to_string())
    }
}

// Note: base32 v0.5 doesn't have DecodeError type, it returns Option<Vec<u8>>
// Error handling is done at call site with ok_or(OnionError::InvalidEncoding)

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn display_invalid_format_and_checksum() {
        assert!(OnionError::InvalidFormat.to_string().contains("format"));
        assert!(OnionError::ChecksumMismatch.to_string().contains("Checksum"));
    }

    #[test]
    fn display_invalid_length_and_version() {
        assert!(OnionError::InvalidLength(3).to_string().contains('3'));
        assert!(OnionError::UnsupportedVersion(2).to_string().contains('2'));
    }

    #[test]
    fn display_crypto_and_config_errors() {
        assert!(OnionError::RpcError("e".into()).to_string().contains("RPC"));
        assert!(OnionError::ConfigError("missing".into()).to_string().contains("Configuration"));
    }

    #[test]
    fn io_error_maps() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
        let e: OnionError = io.into();
        assert!(matches!(e, OnionError::Io(_)), "expected Io variant");
    }

    #[test]
    fn display_handshake_encryption_decryption_and_invalid_message() {
        assert!(
            OnionError::HandshakeFailed("h".into()).to_string().contains("Handshake"),
            "handshake display"
        );
        assert!(
            OnionError::EncryptionError("e".into()).to_string().contains("Encryption"),
            "encryption display"
        );
        assert!(
            OnionError::DecryptionError("d".into()).to_string().contains("Decryption"),
            "decryption display"
        );
        assert!(
            OnionError::InvalidMessage("m".into()).to_string().contains("protocol message"),
            "invalid message display"
        );
    }

    #[test]
    fn serialization_error_maps() {
        let e: OnionError = serde_json::from_str::<i32>("not json").unwrap_err().into();
        assert!(matches!(e, OnionError::Serialization(_)), "expected Serialization");
    }

    #[test]
    fn display_signature_keyexchange_aead_and_other() {
        assert!(OnionError::Signature("s".into()).to_string().contains("Signature"));
        assert!(OnionError::KeyExchange("k".into()).to_string().contains("Key exchange"));
        assert!(OnionError::Aead("a".into()).to_string().contains("AEAD"));
        assert!(OnionError::Other("o".into()).to_string().contains("Onion error"));
    }

    #[test]
    fn display_connection_and_crypto_errors() {
        assert!(
            OnionError::ConnectionError("c".into()).to_string().contains("Connection error"),
            "connection error display"
        );
        assert!(
            OnionError::CryptoError("x".into()).to_string().contains("Crypto error"),
            "crypto error display"
        );
    }

    #[test]
    fn connection_timeout_display() {
        assert!(OnionError::ConnectionTimeout.to_string().contains("timeout"), "timeout display");
    }

    #[test]
    fn invalid_public_key_display() {
        assert!(
            OnionError::InvalidPublicKey.to_string().contains("public key"),
            "invalid pubkey display"
        );
    }
}
