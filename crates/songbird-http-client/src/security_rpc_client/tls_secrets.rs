// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 key derivation operations
//!
//! Implements RFC 8446 key schedule for handshake and application traffic secrets.

use super::core::SecurityRpcClient;
use super::types::TlsSecrets;
use crate::error::{Error, Result};
use base64::prelude::*;
use serde_json::json;
use tracing::{debug, error, info, trace, warn};

impl SecurityRpcClient {
    /// Derive TLS handshake traffic secrets (for encrypting handshake messages)
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    ///
    /// RFC 8446 Section 7.1: Handshake traffic secrets are derived using:
    /// - ECDH shared secret
    /// - Client random
    /// - Server random
    /// - Transcript hash of `ClientHello` + `ServerHello`
    ///
    /// These keys are used to encrypt/decrypt handshake messages AFTER `ServerHello`:
    /// - `EncryptedExtensions`
    /// - `Certificate`
    /// - `CertificateVerify`
    /// - `Finished`
    pub async fn tls_derive_handshake_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsSecrets> {
        info!("🔑 Calling tls_derive_handshake_secrets via Neural API (RFC 8446 Section 7.1)");
        debug!("  → pre_master_secret: {} bytes", shared_secret.len());
        debug!("  → client_random: {} bytes", client_random.len());
        debug!("  → server_random: {} bytes", server_random.len());
        debug!(
            "  → transcript_hash: {} bytes (SHA-256 of ClientHello + ServerHello)",
            transcript_hash.len()
        );
        debug!("  → cipher_suite: 0x{:04x}", cipher_suite);
        trace!("  → transcript_hash (hex): {}", hex::encode(transcript_hash));

        let result = self
            .call(
                "tls.derive_handshake_secrets",
                json!({
                    "pre_master_secret": BASE64_STANDARD.encode(shared_secret),
                    "client_random": BASE64_STANDARD.encode(client_random),
                    "server_random": BASE64_STANDARD.encode(server_random),
                    "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
                    "cipher_suite": cipher_suite
                }),
            )
            .await
            .map_err(|e| {
                error!("❌ tls_derive_handshake_secrets RPC call failed: {}", e);
                e
            })?;

        debug!("✅ Got response from tls_derive_handshake_secrets");
        trace!(
            "  Response JSON: {}",
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "unable to serialize".to_string())
        );

        debug!("📋 Parsing handshake traffic keys from response...");

        let client_write_key = BASE64_STANDARD
            .decode(result["client_write_key"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing client_write_key in response".to_string())
            })?)
            .map_err(|e| Error::BearDogRpc(format!("Invalid client_write_key base64: {e}")))?;
        debug!("  ✅ client_handshake_key: {} bytes", client_write_key.len());

        let server_write_key = BASE64_STANDARD
            .decode(result["server_write_key"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing server_write_key in response".to_string())
            })?)
            .map_err(|e| Error::BearDogRpc(format!("Invalid server_write_key base64: {e}")))?;
        debug!("  ✅ server_handshake_key: {} bytes", server_write_key.len());

        let client_write_iv = BASE64_STANDARD
            .decode(result["client_write_iv"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing client_write_iv in response".to_string())
            })?)
            .map_err(|e| Error::BearDogRpc(format!("Invalid client_write_iv base64: {e}")))?;
        debug!("  ✅ client_handshake_iv: {} bytes", client_write_iv.len());

        let server_write_iv = BASE64_STANDARD
            .decode(result["server_write_iv"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing server_write_iv in response".to_string())
            })?)
            .map_err(|e| Error::BearDogRpc(format!("Invalid server_write_iv base64: {e}")))?;
        debug!("  ✅ server_handshake_iv: {} bytes", server_write_iv.len());

        // Parse traffic secrets (needed for Finished message - RFC 8446 Section 4.4.4)
        let client_handshake_secret = BASE64_STANDARD
            .decode(result["client_handshake_secret"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing client_handshake_secret in response".to_string())
            })?)
            .map_err(|e| {
                Error::BearDogRpc(format!("Invalid client_handshake_secret base64: {e}"))
            })?;
        debug!("  ✅ client_handshake_secret: {} bytes", client_handshake_secret.len());

        let server_handshake_secret = BASE64_STANDARD
            .decode(result["server_handshake_secret"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing server_handshake_secret in response".to_string())
            })?)
            .map_err(|e| {
                Error::BearDogRpc(format!("Invalid server_handshake_secret base64: {e}"))
            })?;
        debug!("  ✅ server_handshake_secret: {} bytes", server_handshake_secret.len());

        // HEX DUMPS for derived keys (cross-verify with RFC 8448 or server expectations)
        info!("🔍 DERIVED HANDSHAKE KEYS - FULL HEX DUMPS:");
        info!("   client_write_key: {}", hex::encode(&client_write_key));
        info!("   server_write_key: {}", hex::encode(&server_write_key));
        info!("   client_write_iv: {}", hex::encode(&client_write_iv));
        info!("   server_write_iv: {}", hex::encode(&server_write_iv));
        info!("   client_handshake_secret: {}", hex::encode(&client_handshake_secret));
        info!("   server_handshake_secret: {}", hex::encode(&server_handshake_secret));

        info!("✅ Handshake traffic secrets derived successfully (RFC 8446 Section 7.1 compliant)");

        Ok(TlsSecrets {
            client_write_key,
            server_write_key,
            client_write_iv,
            server_write_iv,
            client_handshake_secret,
            server_handshake_secret,
        })
    }

    /// Derive TLS application traffic secrets (for encrypting HTTP data)
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    ///
    /// This implements the TLS 1.3 key schedule to derive application traffic keys
    /// from the handshake secret. These keys are used for HTTP data encryption/decryption.
    ///
    /// RFC 8446 Section 7.1: After the handshake completes, derive master secret and
    /// then derive application traffic secrets for encrypting application data.
    pub async fn tls_derive_application_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsSecrets> {
        info!("🔑 Calling tls_derive_application_secrets via Neural API (RFC 8446 compliant)");
        debug!("  → pre_master_secret: {} bytes", shared_secret.len());
        debug!("  → client_random: {} bytes", client_random.len());
        debug!("  → server_random: {} bytes", server_random.len());
        debug!(
            "  → transcript_hash: {} bytes (SHA-256 of all handshake messages)",
            transcript_hash.len()
        );
        debug!("  → cipher_suite: 0x{:04x}", cipher_suite);
        trace!("  → transcript_hash (hex): {}", hex::encode(transcript_hash));

        let result = self
            .call(
                "tls.derive_application_secrets",
                json!({
                    "pre_master_secret": BASE64_STANDARD.encode(shared_secret),
                    "client_random": BASE64_STANDARD.encode(client_random),
                    "server_random": BASE64_STANDARD.encode(server_random),
                    "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
                    "cipher_suite": u64::from(cipher_suite)
                }),
            )
            .await
            .map_err(|e| {
                error!("❌ tls_derive_application_secrets RPC call failed: {}", e);
                e
            })?;

        debug!("✅ Got response from tls_derive_application_secrets");
        trace!(
            "  Response JSON: {}",
            serde_json::to_string_pretty(&result)
                .unwrap_or_else(|_| "unable to serialize".to_string())
        );

        debug!("📋 Parsing application traffic keys from response...");

        let client_write_key = BASE64_STANDARD
            .decode(result["client_write_key"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing client_write_key in response".to_string())
            })?)
            .map_err(|e| Error::BearDogRpc(format!("Invalid client_write_key base64: {e}")))?;
        debug!("  ✅ client_write_key: {} bytes", client_write_key.len());

        let server_write_key = BASE64_STANDARD
            .decode(result["server_write_key"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing server_write_key in response".to_string())
            })?)
            .map_err(|e| Error::BearDogRpc(format!("Invalid server_write_key base64: {e}")))?;
        debug!("  ✅ server_write_key: {} bytes", server_write_key.len());

        let client_write_iv = BASE64_STANDARD
            .decode(result["client_write_iv"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing client_write_iv in response".to_string())
            })?)
            .map_err(|e| Error::BearDogRpc(format!("Invalid client_write_iv base64: {e}")))?;
        debug!("  ✅ client_write_iv: {} bytes", client_write_iv.len());

        let server_write_iv = BASE64_STANDARD
            .decode(result["server_write_iv"].as_str().ok_or_else(|| {
                Error::BearDogRpc("Missing server_write_iv in response".to_string())
            })?)
            .map_err(|e| Error::BearDogRpc(format!("Invalid server_write_iv base64: {e}")))?;
        debug!("  ✅ server_write_iv: {} bytes", server_write_iv.len());

        info!("🎉 Application traffic keys successfully derived and parsed");

        Ok(TlsSecrets {
            client_write_key,
            server_write_key,
            client_write_iv,
            server_write_iv,
            // Application secrets don't need handshake secrets (those were for Finished message)
            client_handshake_secret: vec![],
            server_handshake_secret: vec![],
        })
    }

    /// Compute TLS 1.3 Finished message `verify_data` (RFC 8446 Section 4.4.4)
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    ///
    /// The Finished message authenticates the entire handshake using HMAC:
    /// ```text
    /// verify_data = HMAC(finished_key, Transcript-Hash(Handshake Context))
    /// ```
    pub async fn tls_compute_finished_verify_data(
        &self,
        client_handshake_traffic_secret: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<Vec<u8>> {
        info!("🔐 Computing TLS 1.3 Finished verify_data (RFC 8446 Section 4.4.4)");
        debug!(
            "  → client_handshake_traffic_secret: {} bytes",
            client_handshake_traffic_secret.len()
        );
        debug!("  → transcript_hash: {} bytes", transcript_hash.len());
        debug!("  → cipher_suite: 0x{:04x}", cipher_suite);
        trace!("  → transcript_hash (hex): {}", hex::encode(transcript_hash));

        let result = self
            .call(
                "tls.compute_finished_verify_data",
                json!({
                    "base_key": BASE64_STANDARD.encode(client_handshake_traffic_secret),
                    "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
                    "cipher_suite": format!("0x{:04x}", cipher_suite)
                }),
            )
            .await
            .map_err(|e| {
                error!("❌ tls_compute_finished_verify_data RPC call failed: {}", e);
                e
            })?;

        let verify_data = result["verify_data"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing verify_data in response".to_string()))?;

        let decoded = BASE64_STANDARD
            .decode(verify_data)
            .map_err(|e| Error::BearDogRpc(format!("Invalid verify_data base64: {e}")))?;

        info!("✅ Finished verify_data computed: {} bytes", decoded.len());
        debug!("   Verify data (hex): {}", hex::encode(&decoded));
        Ok(decoded)
    }

    /// Legacy alias for backwards compatibility
    /// DEPRECATED: Use `tls_derive_application_secrets` instead
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    #[deprecated(
        since = "5.6.0",
        note = "Use tls_derive_application_secrets with transcript_hash parameter"
    )]
    pub async fn tls_derive_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
    ) -> Result<TlsSecrets> {
        warn!(
            "Using deprecated tls_derive_secrets without transcript hash - not RFC 8446 compliant!"
        );
        // For backwards compatibility, create empty transcript hash (NOT RFC 8446 compliant!)
        self.tls_derive_application_secrets(
            shared_secret,
            client_random,
            server_random,
            &[],    // Empty transcript hash
            0x1303, // Default to ChaCha20-Poly1305
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_secrets_structure() {
        let secrets = TlsSecrets {
            client_write_key: vec![0u8; 32],
            server_write_key: vec![0u8; 32],
            client_write_iv: vec![0u8; 12],
            server_write_iv: vec![0u8; 12],
            client_handshake_secret: vec![0u8; 32],
            server_handshake_secret: vec![0u8; 32],
        };

        assert_eq!(secrets.client_write_key.len(), 32);
        assert_eq!(secrets.client_write_iv.len(), 12);
    }
}
