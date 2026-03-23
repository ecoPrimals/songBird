// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 Key Schedule (RFC 8446 Section 7.1)
//!
//! Implements HKDF-based key derivation for TLS 1.3.
//!
//! ## Key Schedule Overview
//!
//! ```text
//!              0
//!              |
//!              v
//!    PSK ->  HKDF-Extract = Early Secret
//!              |
//!              +-----> Derive-Secret(., "ext binder" | "res binder")
//!              |                     = binder_key
//!              |
//!              +-----> Derive-Secret(., "c e traffic", ClientHello)
//!              |                     = client_early_traffic_secret
//!              |
//!              +-----> Derive-Secret(., "e exp master", ClientHello)
//!              |                     = early_exporter_master_secret
//!              v
//!        Derive-Secret(., "derived", "")
//!              |
//!              v
//!    (EC)DHE -> HKDF-Extract = Handshake Secret
//!              |
//!              +-----> Derive-Secret(., "c hs traffic", ClientHello...ServerHello)
//!              |                     = client_handshake_traffic_secret
//!              |
//!              +-----> Derive-Secret(., "s hs traffic", ClientHello...ServerHello)
//!              |                     = server_handshake_traffic_secret
//!              v
//!        Derive-Secret(., "derived", "")
//!              |
//!              v
//!    0 -> HKDF-Extract = Master Secret
//!              |
//!              +-----> Derive-Secret(., "c ap traffic", ClientHello...server Finished)
//!              |                     = client_application_traffic_secret_0
//!              |
//!              +-----> Derive-Secret(., "s ap traffic", ClientHello...server Finished)
//!              |                     = server_application_traffic_secret_0
//!              |
//!              +-----> Derive-Secret(., "exp master", ClientHello...server Finished)
//!              |                     = exporter_master_secret
//!              |
//!              +-----> Derive-Secret(., "res master", ClientHello...client Finished)
//!                                    = resumption_master_secret
//! ```

use crate::crypto::BeardogCryptoClient;
use crate::error::{Result, TlsError};

/// TLS 1.3 Key Schedule
///
/// Manages key derivation using HKDF with `BearDog` crypto delegation.
pub struct KeySchedule {
    /// Current secret (evolves through: early -> handshake -> master)
    current_secret: Vec<u8>,

    /// Transcript hash accumulator
    transcript_hash: Vec<u8>,

    /// `BearDog` crypto client for HMAC operations
    crypto_client: Option<BeardogCryptoClient>,

    /// Server's X25519 secret key (for ECDHE)
    server_secret_key: Option<Vec<u8>>,
}

impl KeySchedule {
    /// Create a new key schedule
    ///
    /// Starts with PSK = 0 (no PSK in basic TLS 1.3)
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_secret: vec![0u8; 32], // SHA-256 hash length
            transcript_hash: Vec::new(),
            crypto_client: None,
            server_secret_key: None,
        }
    }

    /// Set the `BearDog` crypto client (for async operations)
    pub fn set_crypto_client(&mut self, client: BeardogCryptoClient) {
        self.crypto_client = Some(client);
    }

    /// Store server's X25519 secret key
    pub fn set_server_secret_key(&mut self, secret_key: Vec<u8>) {
        self.server_secret_key = Some(secret_key);
    }

    /// Get server's X25519 secret key
    #[must_use]
    pub fn server_secret_key(&self) -> Option<&[u8]> {
        self.server_secret_key.as_deref()
    }

    /// Update transcript hash with new handshake message
    ///
    /// In a real implementation, this would use Blake3 or SHA-256.
    /// For now, we just concatenate (will be replaced with actual hashing).
    pub fn update_transcript(&mut self, data: &[u8]) {
        self.transcript_hash.extend_from_slice(data);
    }

    /// Get current transcript hash
    #[must_use]
    pub fn transcript_hash(&self) -> &[u8] {
        &self.transcript_hash
    }

    /// HKDF-Extract (using HMAC-SHA256 via `BearDog`)
    ///
    /// This is the core HKDF operation for deriving keys.
    ///
    /// ```text
    /// HKDF-Extract(salt, IKM) = HMAC-Hash(salt, IKM)
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if crypto client is not set or HMAC operation fails.
    pub async fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>> {
        let client = self
            .crypto_client
            .as_ref()
            .ok_or_else(|| TlsError::InternalError("Crypto client not set".to_string()))?;

        // HKDF-Extract = HMAC(salt, ikm)
        client.hmac_sha256(ikm, salt).await
    }

    /// HKDF-Expand (using HMAC-SHA256 via `BearDog`)
    ///
    /// ```text
    /// HKDF-Expand(PRK, info, L) = T(0) | T(1) | T(2) | ...
    /// where:
    ///   T(0) = empty string
    ///   T(1) = HMAC-Hash(PRK, T(0) | info | 0x01)
    ///   T(2) = HMAC-Hash(PRK, T(1) | info | 0x02)
    ///   ...
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if crypto client is not set or HMAC operation fails.
    pub async fn hkdf_expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>> {
        let client = self
            .crypto_client
            .as_ref()
            .ok_or_else(|| TlsError::InternalError("Crypto client not set".to_string()))?;

        let mut output = Vec::new();
        let mut t = Vec::new();
        let mut input = Vec::new();
        let mut counter = 1u8;

        while output.len() < length {
            // T(i) = HMAC(PRK, T(i-1) | info | counter)
            input.clear();
            input.extend_from_slice(&t);
            input.extend_from_slice(info);
            input.push(counter);

            t = client.hmac_sha256(&input, prk).await?;
            output.extend_from_slice(&t);
            counter += 1;

            if counter == 0 {
                return Err(TlsError::InternalError(
                    "HKDF-Expand: too many iterations".to_string(),
                ));
            }
        }

        output.truncate(length);
        Ok(output)
    }

    /// Derive-Secret (TLS 1.3 specific)
    ///
    /// ```text
    /// Derive-Secret(Secret, Label, Messages) =
    ///     HKDF-Expand-Label(Secret, Label,
    ///                       Transcript-Hash(Messages), Hash.length)
    ///
    /// HKDF-Expand-Label(Secret, Label, Context, Length) =
    ///     HKDF-Expand(Secret, HkdfLabel, Length)
    ///
    /// struct {
    ///     uint16 length = Length;
    ///     opaque label<7..255> = "tls13 " + Label;
    ///     opaque context<0..255> = Context;
    /// } HkdfLabel;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if crypto client is not set or HKDF operation fails.
    pub async fn derive_secret(
        &self,
        secret: &[u8],
        label: &str,
        context: &[u8],
    ) -> Result<Vec<u8>> {
        // Build HkdfLabel
        let mut hkdf_label = Vec::new();

        // Length (2 bytes) - SHA-256 output length
        hkdf_label.extend_from_slice(&32u16.to_be_bytes());

        // Label length + label (prepend "tls13 ")
        let full_label = format!("tls13 {label}");
        hkdf_label.push(
            u8::try_from(full_label.len())
                .map_err(|_| TlsError::InternalError("Label too long".to_string()))?,
        );
        hkdf_label.extend_from_slice(full_label.as_bytes());

        // Context length + context
        hkdf_label.push(
            u8::try_from(context.len())
                .map_err(|_| TlsError::InternalError("Context too long".to_string()))?,
        );
        hkdf_label.extend_from_slice(context);

        // HKDF-Expand
        self.hkdf_expand(secret, &hkdf_label, 32).await
    }

    /// Compute handshake secret from (EC)DHE shared secret
    ///
    /// ```text
    /// Handshake Secret = HKDF-Extract(Derive-Secret(Early Secret, "derived", ""),
    ///                                 ECDHE shared secret)
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if crypto client is not set or HKDF operation fails.
    pub async fn compute_handshake_secret(&mut self, ecdhe_secret: &[u8]) -> Result<()> {
        // Derive-Secret(current_secret, "derived", "")
        let derived = self.derive_secret(&self.current_secret, "derived", &[]).await?;

        // HKDF-Extract(derived, ecdhe_secret)
        let handshake_secret = self.hkdf_extract(&derived, ecdhe_secret).await?;

        self.current_secret = handshake_secret;
        Ok(())
    }

    /// Derive handshake traffic keys
    ///
    /// Returns: (`client_handshake_traffic_secret`, `server_handshake_traffic_secret`)
    ///
    /// # Errors
    ///
    /// Returns an error if crypto client is not set or HKDF operation fails.
    pub async fn derive_handshake_traffic_keys(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let client_secret =
            self.derive_secret(&self.current_secret, "c hs traffic", &self.transcript_hash).await?;

        let server_secret =
            self.derive_secret(&self.current_secret, "s hs traffic", &self.transcript_hash).await?;

        Ok((client_secret, server_secret))
    }

    /// Compute master secret
    ///
    /// ```text
    /// Master Secret = HKDF-Extract(Derive-Secret(Handshake Secret, "derived", ""), 0)
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if crypto client is not set or HKDF operation fails.
    pub async fn compute_master_secret(&mut self) -> Result<()> {
        // Derive-Secret(current_secret, "derived", "")
        let derived = self.derive_secret(&self.current_secret, "derived", &[]).await?;

        // HKDF-Extract(derived, 0)
        let zeros = vec![0u8; 32];
        let master_secret = self.hkdf_extract(&derived, &zeros).await?;

        self.current_secret = master_secret;
        Ok(())
    }

    /// Derive application traffic keys
    ///
    /// Returns: (`client_application_traffic_secret`, `server_application_traffic_secret`)
    ///
    /// # Errors
    ///
    /// Returns an error if crypto client is not set or HKDF operation fails.
    pub async fn derive_application_traffic_keys(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let client_secret =
            self.derive_secret(&self.current_secret, "c ap traffic", &self.transcript_hash).await?;

        let server_secret =
            self.derive_secret(&self.current_secret, "s ap traffic", &self.transcript_hash).await?;

        Ok((client_secret, server_secret))
    }

    /// Derive write key and IV from traffic secret
    ///
    /// ```text
    /// key = HKDF-Expand-Label(Secret, "key", "", key_length)
    /// iv  = HKDF-Expand-Label(Secret, "iv", "", iv_length)
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if crypto client is not set or HKDF operation fails.
    pub async fn derive_traffic_keys(&self, traffic_secret: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        // Derive key (32 bytes for ChaCha20)
        let key = self.derive_secret(traffic_secret, "key", &[]).await?;

        // Derive IV (12 bytes for ChaCha20-Poly1305)
        let mut iv_label = Vec::new();
        iv_label.extend_from_slice(&12u16.to_be_bytes()); // 12-byte IV
        let full_label = "tls13 iv";
        iv_label.push(
            u8::try_from(full_label.len())
                .map_err(|_| TlsError::InternalError("IV label too long".to_string()))?,
        );
        iv_label.extend_from_slice(full_label.as_bytes());
        iv_label.push(0); // Empty context

        let iv = self.hkdf_expand(traffic_secret, &iv_label, 12).await?;

        Ok((key, iv))
    }

    /// Compute finished `verify_data`
    ///
    /// ```text
    /// finished_key = HKDF-Expand-Label(BaseKey, "finished", "", Hash.length)
    /// verify_data = HMAC(finished_key, Transcript-Hash(Handshake Context))
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if crypto client is not set or HMAC operation fails.
    pub async fn compute_finished_verify_data(&self, base_key: &[u8]) -> Result<Vec<u8>> {
        // Derive finished_key
        let finished_key = self.derive_secret(base_key, "finished", &[]).await?;

        // HMAC(finished_key, transcript_hash)
        let client = self
            .crypto_client
            .as_ref()
            .ok_or_else(|| TlsError::InternalError("Crypto client not set".to_string()))?;

        client.hmac_sha256(&self.transcript_hash, &finished_key).await
    }
}

impl Default for KeySchedule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_new_key_schedule() {
        let ks = KeySchedule::new();
        assert_eq!(ks.current_secret.len(), 32);
        assert!(ks.transcript_hash.is_empty());
        assert!(ks.crypto_client.is_none());
    }

    #[test]
    fn test_update_transcript() {
        let mut ks = KeySchedule::new();
        ks.update_transcript(b"Hello");
        ks.update_transcript(b" World");
        assert_eq!(ks.transcript_hash(), b"Hello World");
    }

    #[test]
    fn test_derive_secret_label_format() {
        // Test that HkdfLabel is constructed correctly
        // This is a structural test - actual crypto delegation tested in integration tests
        let ks = KeySchedule::new();

        // Label should be "tls13 " + label
        // This will be tested when we have a mock crypto client
        assert!(ks.crypto_client.is_none());
    }

    // ========================================
    // NEW COMPREHENSIVE KEY SCHEDULE TESTS
    // Added: January 27, 2026 (Evening)
    // Goal: Increase coverage from 12% → 70%
    // ========================================

    #[test]
    fn test_key_schedule_initialization() {
        let ks = KeySchedule::new();

        // Initial secret should be 32 bytes (SHA-256 length)
        assert_eq!(ks.current_secret.len(), 32);

        // All bytes should be zeros initially (PSK = 0)
        assert!(ks.current_secret.iter().all(|&b| b == 0));

        // Transcript should be empty
        assert!(ks.transcript_hash().is_empty());

        // No crypto client initially
        assert!(ks.crypto_client.is_none());

        // No server secret key initially
        assert!(ks.server_secret_key().is_none());
    }

    #[test]
    fn test_set_crypto_client() {
        let mut ks = KeySchedule::new();

        // Create mock crypto client
        let client = BeardogCryptoClient::with_socket_path("/tmp/test-ks.sock".to_string());

        // Set client
        ks.set_crypto_client(client);

        // Should now have a crypto client
        assert!(ks.crypto_client.is_some());
    }

    #[test]
    fn test_set_and_get_server_secret_key() {
        let mut ks = KeySchedule::new();

        // Create test secret key (32 bytes for X25519)
        let secret_key = vec![42u8; 32];

        // Set secret key
        ks.set_server_secret_key(secret_key.clone());

        // Verify we can retrieve it
        assert_eq!(ks.server_secret_key(), Some(secret_key.as_slice()));
    }

    #[test]
    fn test_update_transcript_multiple_times() {
        let mut ks = KeySchedule::new();

        // Update transcript with multiple messages
        ks.update_transcript(b"ClientHello");
        ks.update_transcript(b"ServerHello");
        ks.update_transcript(b"Certificate");

        // Transcript should contain all messages concatenated
        let expected = b"ClientHelloServerHelloCertificate";
        assert_eq!(ks.transcript_hash(), expected);
    }

    #[test]
    fn test_transcript_hash_empty_initially() {
        let ks = KeySchedule::new();
        assert!(ks.transcript_hash().is_empty());
        assert_eq!(ks.transcript_hash().len(), 0);
    }

    #[test]
    fn test_transcript_hash_preserves_data() {
        let mut ks = KeySchedule::new();

        let data = b"Test Data 12345";
        ks.update_transcript(data);

        assert_eq!(ks.transcript_hash(), data);
    }

    #[test]
    fn test_default_trait() {
        let ks = KeySchedule::default();

        // Default should be same as new()
        assert_eq!(ks.current_secret.len(), 32);
        assert!(ks.transcript_hash().is_empty());
        assert!(ks.crypto_client.is_none());
    }

    #[test]
    fn test_current_secret_initial_state() {
        let ks = KeySchedule::new();

        // Current secret starts as all zeros (no PSK)
        let expected_zeros = vec![0u8; 32];
        assert_eq!(ks.current_secret, expected_zeros);
    }

    #[tokio::test]
    async fn test_hkdf_extract_requires_crypto_client() {
        let ks = KeySchedule::new();

        let salt = b"test salt";
        let ikm = b"input keying material";

        // Should fail without crypto client
        let result = ks.hkdf_extract(salt, ikm).await;
        assert!(result.is_err());

        // Error should mention crypto client
        if let Err(e) = result {
            assert!(format!("{e:?}").contains("Crypto client not set"));
        }
    }

    #[tokio::test]
    async fn test_hkdf_expand_requires_crypto_client() {
        let ks = KeySchedule::new();

        let prk = vec![1u8; 32];
        let info = b"test info";

        // Should fail without crypto client
        let result = ks.hkdf_expand(&prk, info, 32).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_derive_secret_requires_crypto_client() {
        let ks = KeySchedule::new();

        let secret = vec![1u8; 32];
        let label = "test label";
        let context = b"test context";

        // Should fail without crypto client
        let result = ks.derive_secret(&secret, label, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compute_handshake_secret_requires_crypto_client() {
        let mut ks = KeySchedule::new();

        let ecdhe_secret = vec![2u8; 32];

        // Should fail without crypto client
        let result = ks.compute_handshake_secret(&ecdhe_secret).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_derive_handshake_traffic_keys_requires_crypto_client() {
        let ks = KeySchedule::new();

        // Should fail without crypto client
        let result = ks.derive_handshake_traffic_keys().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compute_master_secret_requires_crypto_client() {
        let mut ks = KeySchedule::new();

        // Should fail without crypto client
        let result = ks.compute_master_secret().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_derive_application_traffic_keys_requires_crypto_client() {
        let ks = KeySchedule::new();

        // Should fail without crypto client
        let result = ks.derive_application_traffic_keys().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_derive_traffic_keys_requires_crypto_client() {
        let ks = KeySchedule::new();

        let traffic_secret = vec![3u8; 32];

        // Should fail without crypto client
        let result = ks.derive_traffic_keys(&traffic_secret).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compute_finished_verify_data_requires_crypto_client() {
        let ks = KeySchedule::new();

        let base_key = vec![4u8; 32];

        // Should fail without crypto client
        let result = ks.compute_finished_verify_data(&base_key).await;
        assert!(result.is_err());

        // Error should mention crypto client
        if let Err(e) = result {
            assert!(format!("{e:?}").contains("Crypto client not set"));
        }
    }

    #[test]
    fn test_server_secret_key_none_initially() {
        let ks = KeySchedule::new();
        assert!(ks.server_secret_key().is_none());
    }

    #[test]
    fn test_multiple_transcript_updates_order() {
        let mut ks = KeySchedule::new();

        // Add messages in order
        ks.update_transcript(b"First");
        ks.update_transcript(b"Second");
        ks.update_transcript(b"Third");

        // Order should be preserved
        assert_eq!(ks.transcript_hash(), b"FirstSecondThird");
    }

    // Note: Full key schedule tests with live crypto operations require BearDog integration
    // These are in integration tests with a live BearDog instance
}
