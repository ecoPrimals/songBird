// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-Based Crypto Provider Abstraction
//!
//! This module defines the `CryptoProvider` trait for capability-based crypto operations.
//!
//! # TRUE PRIMAL Principles
//!
//! - **Self-Knowledge Only**: Songbird only knows it needs "crypto" capability
//! - **Capability Discovery**: Discovers ANY primal offering crypto at runtime
//! - **No Hardcoding**: No hardcoded primal names (not "`BearDog`", not "`ToadStool`")
//! - **Runtime Discovery**: Orchestrator guides via env vars, or automatic discovery
//!
//! # Architecture
//!
//! ```text
//! Songbird (only knows itself)
//!     ↓
//! CryptoProvider trait (capability abstraction)
//!     ↓
//! discover_crypto_provider() (runtime discovery)
//!     ↓
//! UnixSocketCryptoProvider (implementation)
//!     ↓
//! ANY primal offering "crypto" capability (BearDog, custom, etc.)
//! ```

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Crypto Provider Capability
///
/// This trait defines the cryptographic operations that a primal can provide.
/// Songbird discovers ANY primal implementing this capability at runtime.
///
/// # Implementations
///
/// - `UnixSocketCryptoProvider`: Communicates via Unix socket JSON-RPC
/// - Future: `HttpCryptoProvider`, `GrpcCryptoProvider`, etc.
#[async_trait]
pub trait CryptoProvider: Send + Sync + std::fmt::Debug {
    /// Compute BLAKE3 hash of data
    ///
    /// # Arguments
    /// * `data` - Data to hash
    ///
    /// # Returns
    /// * `Ok(hash)` - BLAKE3 hash (32 bytes)
    async fn blake3_hash(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Compute HMAC-SHA256
    ///
    /// # Arguments
    /// * `key` - Secret key
    /// * `data` - Data to authenticate
    ///
    /// # Returns
    /// * `Ok(mac)` - HMAC-SHA256 MAC (32 bytes)
    async fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>>;

    /// Sign message with Ed25519
    ///
    /// # Arguments
    /// * `message` - Message to sign
    /// * `key_id` - Key identifier
    /// * `purpose` - Purpose for audit logging
    ///
    /// # Returns
    /// * `Ok(signature)` - Ed25519 signature (64 bytes)
    async fn sign_ed25519(&self, message: &[u8], key_id: &str, purpose: &str) -> Result<Vec<u8>>;

    /// Verify Ed25519 signature
    ///
    /// # Arguments
    /// * `message` - Message that was signed
    /// * `signature` - Signature to verify
    /// * `public_key` - Public key (32 bytes)
    ///
    /// # Returns
    /// * `Ok(true)` - Signature is valid
    /// * `Ok(false)` - Signature is invalid
    async fn verify_ed25519(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool>;

    /// Generate ephemeral X25519 key pair
    ///
    /// # Arguments
    /// * `purpose` - Purpose for audit logging
    ///
    /// # Returns
    /// * `Ok((public_key, secret_key))` - Both keys as bytes (32 bytes each)
    async fn x25519_generate_ephemeral(&self, purpose: &str) -> Result<(Vec<u8>, Vec<u8>)>;

    /// Derive X25519 shared secret (Diffie-Hellman)
    ///
    /// # Arguments
    /// * `our_secret_key` - Our secret key (32 bytes)
    /// * `their_public_key` - Their public key (32 bytes)
    ///
    /// # Returns
    /// * `Ok(shared_secret)` - Shared secret (32 bytes)
    async fn x25519_derive_secret(
        &self,
        our_secret_key: &[u8],
        their_public_key: &[u8],
    ) -> Result<Vec<u8>>;

    /// Encrypt with ChaCha20-Poly1305 AEAD
    ///
    /// # Arguments
    /// * `plaintext` - Plaintext to encrypt
    /// * `key` - Encryption key (32 bytes)
    /// * `aad` - Additional authenticated data (optional)
    ///
    /// # Returns
    /// * `Ok((ciphertext, nonce, tag))` - Ciphertext, nonce (12 bytes), auth tag (16 bytes)
    async fn chacha20_poly1305_encrypt(
        &self,
        plaintext: &[u8],
        key: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)>;

    /// Decrypt with ChaCha20-Poly1305 AEAD
    ///
    /// # Arguments
    /// * `ciphertext` - Ciphertext to decrypt
    /// * `key` - Decryption key (32 bytes)
    /// * `nonce` - Nonce (12 bytes)
    /// * `tag` - Authentication tag (16 bytes)
    /// * `aad` - Additional authenticated data (must match encryption)
    ///
    /// # Returns
    /// * `Ok(plaintext)` - Decrypted plaintext
    async fn chacha20_poly1305_decrypt(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
        tag: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<Vec<u8>>;
}

/// Unix Socket Crypto Provider
///
/// Implements `CryptoProvider` by delegating to a primal via Unix socket JSON-RPC.
/// Does NOT know which primal it's talking to - just that it offers crypto capability.
#[derive(Debug)]
pub struct UnixSocketCryptoProvider {
    socket_path: String,
}

impl UnixSocketCryptoProvider {
    /// Create a new Unix socket crypto provider
    ///
    /// # Arguments
    /// * `socket_path` - Path to Unix socket offering crypto capability
    #[must_use]
    pub const fn new(socket_path: String) -> Self {
        Self {
            socket_path,
        }
    }

    /// Get the socket path
    #[must_use]
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }
}

#[async_trait]
impl CryptoProvider for UnixSocketCryptoProvider {
    async fn blake3_hash(&self, data: &[u8]) -> Result<Vec<u8>> {
        super::beardog_crypto_client::blake3_hash(&self.socket_path, data).await
    }

    async fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        super::beardog_crypto_client::hmac_sha256(&self.socket_path, key, data).await
    }

    async fn sign_ed25519(&self, message: &[u8], key_id: &str, purpose: &str) -> Result<Vec<u8>> {
        super::beardog_crypto_client::sign_ed25519(&self.socket_path, message, key_id, purpose)
            .await
    }

    async fn verify_ed25519(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool> {
        super::beardog_crypto_client::verify_ed25519(
            &self.socket_path,
            message,
            signature,
            public_key,
        )
        .await
    }

    async fn x25519_generate_ephemeral(&self, purpose: &str) -> Result<(Vec<u8>, Vec<u8>)> {
        super::beardog_crypto_client::x25519_generate_ephemeral(&self.socket_path, purpose).await
    }

    async fn x25519_derive_secret(
        &self,
        our_secret_key: &[u8],
        their_public_key: &[u8],
    ) -> Result<Vec<u8>> {
        super::beardog_crypto_client::x25519_derive_secret(
            &self.socket_path,
            our_secret_key,
            their_public_key,
        )
        .await
    }

    async fn chacha20_poly1305_encrypt(
        &self,
        plaintext: &[u8],
        key: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        super::beardog_crypto_client::chacha20_poly1305_encrypt(
            &self.socket_path,
            plaintext,
            key,
            aad,
        )
        .await
    }

    async fn chacha20_poly1305_decrypt(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
        tag: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        super::beardog_crypto_client::chacha20_poly1305_decrypt(
            &self.socket_path,
            ciphertext,
            key,
            nonce,
            tag,
            aad,
        )
        .await
    }
}

/// Discover crypto provider via capability-based discovery
///
/// # TRUE PRIMAL Principles
///
/// - Does NOT look for "`BearDog`" specifically
/// - Looks for ANY primal offering "crypto" capability
/// - Uses orchestrator guidance (env vars) when available
/// - Falls back to automatic discovery
///
/// # Discovery Strategy
///
/// 1. Check `CRYPTO_PROVIDER_SOCKET` (orchestrator guidance, preferred)
/// 2. Check `CRYPTO_PROVIDER` (alternative env var)
/// 3. Check `BEARDOG_CRYPTO_SOCKET` (for compatibility during migration)
/// 4. Search common socket paths
/// 5. Query via mDNS/BirdSong for "crypto" capability (future)
///
/// # Returns
///
/// * `Ok(provider)` - Discovered crypto provider
/// * `Err` - No crypto provider found
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover_crypto_provider() -> Result<Arc<dyn CryptoProvider>> {
    // Use primal-agnostic discovery (TRUE PRIMAL)
    let socket_path = crate::primal_discovery::discover_crypto_provider().await?;

    Ok(Arc::new(UnixSocketCryptoProvider::new(socket_path)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    // Mock provider for unit testing
    #[derive(Debug)]
    struct MockCryptoProvider;

    #[async_trait]
    impl CryptoProvider for MockCryptoProvider {
        async fn blake3_hash(&self, data: &[u8]) -> Result<Vec<u8>> {
            // Mock: return deterministic hash
            Ok(vec![0u8; 32])
        }

        async fn hmac_sha256(&self, _key: &[u8], _data: &[u8]) -> Result<Vec<u8>> {
            Ok(vec![1u8; 32])
        }

        async fn sign_ed25519(
            &self,
            _message: &[u8],
            _key_id: &str,
            _purpose: &str,
        ) -> Result<Vec<u8>> {
            Ok(vec![2u8; 64])
        }

        async fn verify_ed25519(
            &self,
            _message: &[u8],
            _signature: &[u8],
            _public_key: &[u8],
        ) -> Result<bool> {
            Ok(true)
        }

        async fn x25519_generate_ephemeral(&self, _purpose: &str) -> Result<(Vec<u8>, Vec<u8>)> {
            Ok((vec![3u8; 32], vec![4u8; 32]))
        }

        async fn x25519_derive_secret(
            &self,
            _our_secret_key: &[u8],
            _their_public_key: &[u8],
        ) -> Result<Vec<u8>> {
            Ok(vec![5u8; 32])
        }

        async fn chacha20_poly1305_encrypt(
            &self,
            plaintext: &[u8],
            _key: &[u8],
            _aad: Option<&[u8]>,
        ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
            Ok((plaintext.to_vec(), vec![6u8; 12], vec![7u8; 16]))
        }

        async fn chacha20_poly1305_decrypt(
            &self,
            ciphertext: &[u8],
            _key: &[u8],
            _nonce: &[u8],
            _tag: &[u8],
            _aad: Option<&[u8]>,
        ) -> Result<Vec<u8>> {
            Ok(ciphertext.to_vec())
        }
    }

    #[tokio::test]
    async fn test_mock_provider_blake3() {
        let provider = MockCryptoProvider;
        let hash = provider.blake3_hash(b"test").await.unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[tokio::test]
    async fn test_mock_provider_hmac() {
        let provider = MockCryptoProvider;
        let mac = provider.hmac_sha256(b"key", b"data").await.unwrap();
        assert_eq!(mac.len(), 32);
    }

    #[tokio::test]
    async fn test_mock_provider_ed25519() {
        let provider = MockCryptoProvider;
        let sig = provider.sign_ed25519(b"msg", "key1", "test").await.unwrap();
        assert_eq!(sig.len(), 64);

        let valid = provider.verify_ed25519(b"msg", &sig, &[0u8; 32]).await.unwrap();
        assert!(valid);
    }

    #[tokio::test]
    async fn test_mock_provider_x25519() {
        let provider = MockCryptoProvider;
        let (pk, sk) = provider.x25519_generate_ephemeral("test").await.unwrap();
        assert_eq!(pk.len(), 32);
        assert_eq!(sk.len(), 32);

        let shared = provider.x25519_derive_secret(&sk, &pk).await.unwrap();
        assert_eq!(shared.len(), 32);
    }

    #[tokio::test]
    async fn test_mock_provider_chacha20() {
        let provider = MockCryptoProvider;
        let plaintext = b"secret message";

        let (ct, nonce, tag) =
            provider.chacha20_poly1305_encrypt(plaintext, &[0u8; 32], None).await.unwrap();
        assert_eq!(ct.len(), plaintext.len());
        assert_eq!(nonce.len(), 12);
        assert_eq!(tag.len(), 16);

        let decrypted =
            provider.chacha20_poly1305_decrypt(&ct, &[0u8; 32], &nonce, &tag, None).await.unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
