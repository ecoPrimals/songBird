// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Cryptographic Capability Abstraction
//!
//! Defines the `CryptoCapability` trait that abstracts cryptographic operations.
//! This enables Songbird to work with any crypto provider (`BearDog`, etc.) via
//! runtime discovery rather than hardcoded dependencies.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                          Songbird TLS                               │
//! │                 (Uses CryptoCapability trait)                       │
//! └─────────────────────────────┬───────────────────────────────────────┘
//!                               │
//!                               │ CryptoCapability trait
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    CryptoCapability Trait                           │
//! │  ├── generate_keypair()                                             │
//! │  ├── derive_shared_secret()                                         │
//! │  ├── encrypt() / decrypt()                                          │
//! │  ├── hash()                                                         │
//! │  └── tls_derive_*()                                                 │
//! └─────────────────────────────┬───────────────────────────────────────┘
//!                               │
//!              ┌────────────────┼────────────────┐
//!              │                │                │
//!              ▼                ▼                ▼
//!      ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
//!      │   BearDog    │ │  Future      │ │   NoOp       │
//!      │  Provider    │ │  Provider    │ │  Provider    │
//!      └──────────────┘ └──────────────┘ └──────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Discover crypto provider at runtime
//! let crypto = discover_crypto_capability().await?;
//!
//! // Use capability-based API
//! let (public, private) = crypto.generate_x25519_keypair().await?;
//! let shared = crypto.derive_x25519_shared_secret(&private, &peer_public).await?;
//! ```

use crate::error::Result;
use async_trait::async_trait;

/// TLS 1.3 Handshake Secrets (RFC 8446 Section 7.1)
#[derive(Debug, Clone)]
pub struct TlsHandshakeSecrets {
    /// Client handshake traffic secret (for Finished computation)
    pub client_handshake_secret: Vec<u8>,
    /// Server handshake traffic secret (for Finished computation)
    pub server_handshake_secret: Vec<u8>,
    /// Client handshake write key
    pub client_write_key: Vec<u8>,
    /// Client handshake write IV
    pub client_write_iv: Vec<u8>,
    /// Server handshake write key
    pub server_write_key: Vec<u8>,
    /// Server handshake write IV
    pub server_write_iv: Vec<u8>,
    /// Handshake secret (for deriving application secrets)
    pub handshake_secret: Vec<u8>,
}

/// TLS 1.3 Application Secrets (RFC 8446 Section 7.1)
#[derive(Debug, Clone)]
pub struct TlsApplicationSecrets {
    /// Client application traffic secret
    pub client_traffic_secret: Vec<u8>,
    /// Server application traffic secret
    pub server_traffic_secret: Vec<u8>,
    /// Client write key
    pub client_write_key: Vec<u8>,
    /// Client write IV
    pub client_write_iv: Vec<u8>,
    /// Server write key
    pub server_write_key: Vec<u8>,
    /// Server write IV
    pub server_write_iv: Vec<u8>,
}

/// Cryptographic Capability Trait
///
/// Abstracts all cryptographic operations needed for TLS 1.3.
/// Implementations can delegate to `BearDog`, use local crypto, or
/// route through Neural API for semantic translation.
///
/// ## Design Principles
///
/// 1. **Agnostic**: No hardcoded provider names
/// 2. **Discoverable**: Providers found at runtime
/// 3. **Semantic**: Methods named by intent, not implementation
/// 4. **Async**: All operations are async for IPC flexibility
///
/// ## Implementation Notes
///
/// - All byte arrays use `Vec<u8>` for simplicity across IPC
/// - Errors should be descriptive for debugging
/// - Implementations should be stateless where possible
#[async_trait]
pub trait CryptoCapability: Send + Sync + std::fmt::Debug {
    /// Provider name for debugging/logging
    fn name(&self) -> &str;

    /// Check if the provider is available and healthy
    async fn is_available(&self) -> bool;

    // ═══════════════════════════════════════════════════════════════════
    // Key Exchange (X25519)
    // ═══════════════════════════════════════════════════════════════════

    /// Generate X25519 key pair
    ///
    /// Returns (`public_key`, `private_key`) as 32-byte arrays.
    async fn generate_x25519_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)>;

    /// Derive shared secret using X25519 ECDH
    ///
    /// # Arguments
    /// * `our_secret` - Our private key (32 bytes)
    /// * `their_public` - Peer's public key (32 bytes)
    ///
    /// # Returns
    /// Shared secret (32 bytes)
    async fn derive_x25519_shared_secret(
        &self,
        our_secret: &[u8],
        their_public: &[u8],
    ) -> Result<Vec<u8>>;

    // ═══════════════════════════════════════════════════════════════════
    // AEAD Encryption (AES-GCM, ChaCha20-Poly1305)
    // ═══════════════════════════════════════════════════════════════════

    /// AES-128-GCM encryption
    async fn aes128_gcm_encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>>;

    /// AES-128-GCM decryption
    async fn aes128_gcm_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>>;

    /// AES-256-GCM encryption
    async fn aes256_gcm_encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>>;

    /// AES-256-GCM decryption
    async fn aes256_gcm_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>>;

    /// ChaCha20-Poly1305 encryption
    async fn chacha20_poly1305_encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>>;

    /// ChaCha20-Poly1305 decryption
    async fn chacha20_poly1305_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>>;

    // ═══════════════════════════════════════════════════════════════════
    // Hashing (SHA-256, SHA-384, Cipher-Aware)
    // ═══════════════════════════════════════════════════════════════════

    /// SHA-256 hash
    async fn sha256(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// SHA-384 hash
    async fn sha384(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Cipher-suite aware hashing (RFC 8446 compliant)
    ///
    /// Returns the appropriate hash for TLS 1.3 transcript computation:
    /// - 0x1301 (`TLS_AES_128_GCM_SHA256)`: SHA-256 (32 bytes)
    /// - 0x1302 (`TLS_AES_256_GCM_SHA384)`: SHA-384 (48 bytes)
    /// - 0x1303 (`TLS_CHACHA20_POLY1305_SHA256)`: SHA-256 (32 bytes)
    ///
    /// This is critical for correct transcript hash computation in TLS 1.3.
    async fn hash_for_cipher(&self, data: &[u8], cipher_suite: u16) -> Result<Vec<u8>>;

    // ═══════════════════════════════════════════════════════════════════
    // Key Derivation (HKDF)
    // ═══════════════════════════════════════════════════════════════════

    /// HKDF-Extract
    async fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>>;

    /// HKDF-Expand
    async fn hkdf_expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>>;

    // ═══════════════════════════════════════════════════════════════════
    // TLS 1.3 Specific Operations (RFC 8446)
    // ═══════════════════════════════════════════════════════════════════

    /// Derive TLS 1.3 handshake secrets
    ///
    /// RFC 8446 Section 7.1: Key Schedule
    ///
    /// # Parameters
    /// - `shared_secret`: ECDH shared secret (`pre_master_secret`)
    /// - `client_random`: 32-byte client random from `ClientHello`
    /// - `server_random`: 32-byte server random from `ServerHello`
    /// - `transcript_hash`: Hash of `ClientHello` + `ServerHello`
    /// - `cipher_suite`: Negotiated cipher suite (e.g., 0x1301 for `TLS_AES_128_GCM_SHA256`)
    async fn tls_derive_handshake_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsHandshakeSecrets>;

    /// Derive TLS 1.3 application secrets
    ///
    /// RFC 8446 Section 7.1: Key Schedule
    /// `cipher_suite` is required to derive correct key lengths (16 bytes for AES-128, 32 for AES-256/ChaCha)
    async fn tls_derive_application_secrets(
        &self,
        handshake_secret: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsApplicationSecrets>;

    /// Compute TLS 1.3 Finished `verify_data`
    ///
    /// RFC 8446 Section 4.4.4
    ///
    /// # Parameters
    /// - `base_key`: Handshake traffic secret
    /// - `transcript_hash`: Hash of handshake transcript (32 or 48 bytes)
    /// - `cipher_suite`: Negotiated cipher suite (for selecting HMAC algorithm)
    async fn tls_compute_finished_verify_data(
        &self,
        base_key: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<Vec<u8>>;

    // ═══════════════════════════════════════════════════════════════════
    // Generic Encryption (cipher-agnostic)
    // ═══════════════════════════════════════════════════════════════════

    /// Generic encrypt (uses default cipher - ChaCha20-Poly1305)
    async fn encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        self.chacha20_poly1305_encrypt(key, nonce, plaintext, aad).await
    }

    /// Generic decrypt (uses default cipher - ChaCha20-Poly1305)
    async fn decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        self.chacha20_poly1305_decrypt(key, nonce, ciphertext, aad).await
    }
}

/// Alias for backward compatibility
pub type CryptoProvider = dyn CryptoCapability;

#[cfg(test)]
mod tests {
    use super::*;

    // Test that trait is object-safe
    fn _assert_object_safe(_: &dyn CryptoCapability) {}
}
