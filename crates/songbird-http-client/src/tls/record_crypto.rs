// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 record-layer cryptographic helpers
//!
//! Nonce construction (RFC 8446 §5.3) and cipher-suite dispatch extracted
//! from `record.rs` for cohesion and deduplication.

use crate::crypto::CryptoCapability;
use crate::error::{Error, Result};
use std::sync::Arc;
use tracing::{debug, error};

/// Build a per-record nonce per RFC 8446 §5.3.
///
/// `nonce = iv XOR sequence_number` (sequence right-aligned in the IV length).
#[must_use]
pub fn build_nonce(iv: &[u8], sequence_number: u64) -> Vec<u8> {
    let mut nonce = iv.to_vec();
    let seq_bytes = sequence_number.to_be_bytes();

    if nonce.len() >= 8 {
        for (i, &byte) in seq_bytes.iter().enumerate() {
            let nonce_idx = nonce.len() - 8 + i;
            nonce[nonce_idx] ^= byte;
        }
    }

    nonce
}

/// Encrypt `plaintext` using the negotiated cipher suite.
pub async fn cipher_encrypt(
    crypto: &Arc<dyn CryptoCapability>,
    cipher_suite: u16,
    key: &[u8],
    nonce: &[u8],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    match cipher_suite {
        0x1301 => {
            debug!("   → Using AES-128-GCM for application data");
            crypto.aes128_gcm_encrypt(key, nonce, plaintext, aad).await
        }
        0x1302 => {
            debug!("   → Using AES-256-GCM for application data");
            crypto.aes256_gcm_encrypt(key, nonce, plaintext, aad).await
        }
        0x1303 => {
            debug!("   → Using ChaCha20-Poly1305 for application data");
            crypto.encrypt(key, nonce, plaintext, aad).await
        }
        _ => {
            error!("Unsupported cipher suite: 0x{cipher_suite:04x}");
            Err(Error::TlsRecord(format!("Unsupported TLS 1.3 cipher suite: 0x{cipher_suite:04x}")))
        }
    }
}

/// Decrypt `ciphertext` using the negotiated cipher suite.
pub async fn cipher_decrypt(
    crypto: &Arc<dyn CryptoCapability>,
    cipher_suite: u16,
    key: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    match cipher_suite {
        0x1301 => {
            debug!("   → Using AES-128-GCM for application data");
            crypto.aes128_gcm_decrypt(key, nonce, ciphertext, aad).await
        }
        0x1302 => {
            debug!("   → Using AES-256-GCM for application data");
            crypto.aes256_gcm_decrypt(key, nonce, ciphertext, aad).await
        }
        0x1303 => {
            debug!("   → Using ChaCha20-Poly1305 for application data");
            crypto.decrypt(key, nonce, ciphertext, aad).await
        }
        _ => {
            error!("Unsupported cipher suite: 0x{cipher_suite:04x}");
            Err(Error::TlsRecord(format!("Unsupported TLS 1.3 cipher suite: 0x{cipher_suite:04x}")))
        }
    }
}

/// Human-readable name for a cipher suite code
#[must_use]
pub const fn cipher_suite_name(code: u16) -> &'static str {
    match code {
        0x1301 => "TLS_AES_128_GCM_SHA256",
        0x1302 => "TLS_AES_256_GCM_SHA384",
        0x1303 => "TLS_CHACHA20_POLY1305_SHA256",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn nonce_xor_with_zero_sequence() {
        let iv = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let nonce = build_nonce(&iv, 0);
        assert_eq!(nonce, iv, "seq 0 XOR should be identity");
    }

    #[test]
    fn nonce_changes_with_sequence() {
        let iv = vec![0xFF; 12];
        let n0 = build_nonce(&iv, 0);
        let n1 = build_nonce(&iv, 1);
        assert_ne!(n0, n1);
    }

    #[test]
    fn short_iv_skips_xor() {
        let iv = vec![1, 2, 3, 4]; // < 8 bytes
        let nonce = build_nonce(&iv, 42);
        assert_eq!(nonce, vec![1, 2, 3, 4]);
    }

    #[test]
    fn cipher_suite_names() {
        assert_eq!(cipher_suite_name(0x1301), "TLS_AES_128_GCM_SHA256");
        assert_eq!(cipher_suite_name(0x1302), "TLS_AES_256_GCM_SHA384");
        assert_eq!(cipher_suite_name(0x1303), "TLS_CHACHA20_POLY1305_SHA256");
        assert_eq!(cipher_suite_name(0xFFFF), "UNKNOWN");
    }
}
