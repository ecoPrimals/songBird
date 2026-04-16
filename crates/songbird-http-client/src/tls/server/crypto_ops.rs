// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Cryptographic Operations
//!
//! Handles encryption and decryption of TLS records using handshake/application keys.

use crate::crypto::CryptoCapability;
use crate::error::{Error, Result};
use crate::tls::content_type;
use crate::tls::handshake_v2::keys::CipherSuite;
use tracing::{debug, error};

use super::core::TlsServer;

impl TlsServer {
    /// Encrypt handshake message with handshake traffic keys
    ///
    /// Reference: RFC 8446 Section 5.2 (Record Payload Protection)
    #[expect(
        clippy::cast_possible_truncation,
        reason = "TLS wire format: values are masked/bounded"
    )]
    pub(super) async fn encrypt_handshake_message(
        &self,
        plaintext: &[u8],
        key: &[u8],
        iv: &[u8],
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        // Build nonce (IV XOR sequence_number)
        let mut nonce = iv.to_vec();
        let seq_bytes = sequence_number.to_be_bytes();

        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }

        debug!("   Nonce (IV XOR seq {}): {:02x?}", sequence_number, nonce);

        // Calculate ciphertext length (plaintext + 16-byte AEAD tag)
        let ciphertext_length = plaintext.len() + 16;

        // Build AAD (TLS record header)
        let record_type = 0x17; // APPLICATION_DATA (all encrypted records use 0x17 in TLS 1.3)
        let version = [0x03, 0x03]; // TLS 1.2 compatibility
        let aad = [
            record_type,
            version[0],
            version[1],
            ((ciphertext_length >> 8) & 0xFF) as u8,
            (ciphertext_length & 0xFF) as u8,
        ];

        debug!("   AAD (TLS record header): {:02x?}", aad);

        // Encrypt via crypto provider (uses correct AEAD algorithm based on cipher suite)
        let ciphertext = match self.cipher_suite {
            CipherSuite::Aes128GcmSha256 => {
                debug!("   → Using AES-128-GCM");
                self.crypto.aes128_gcm_encrypt(key, &nonce, plaintext, &aad).await
            }
            CipherSuite::Aes256GcmSha384 => {
                debug!("   → Using AES-256-GCM");
                self.crypto.aes256_gcm_encrypt(key, &nonce, plaintext, &aad).await
            }
            CipherSuite::ChaCha20Poly1305Sha256 => {
                debug!("   → Using ChaCha20-Poly1305");
                self.crypto.encrypt(key, &nonce, plaintext, &aad).await
            }
        }
        .map_err(|e| {
            error!("❌ Encryption failed: {}", e);
            Error::TlsHandshake(format!("Failed to encrypt: {e}"))
        })?;

        debug!(
            "✅ Encrypted {} bytes → {} bytes (includes 16-byte tag)",
            plaintext.len(),
            ciphertext.len()
        );

        Ok(ciphertext)
    }

    /// Decrypt application data with application traffic keys
    ///
    /// Reference: RFC 8446 Section 5.2 (Record Payload Protection)
    #[expect(
        clippy::cast_possible_truncation,
        reason = "TLS wire format: values are masked/bounded"
    )]
    pub(super) async fn decrypt_application_data(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        iv: &[u8],
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        // Build nonce (IV XOR sequence_number)
        let mut nonce = iv.to_vec();
        let seq_bytes = sequence_number.to_be_bytes();

        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }

        debug!("   Nonce (IV XOR seq {}): {:02x?}", sequence_number, nonce);

        // Build AAD (TLS record header)
        let ciphertext_length = ciphertext.len();
        let record_type = 0x17; // APPLICATION_DATA
        let version = [0x03, 0x03]; // TLS 1.2 compatibility
        let aad = [
            record_type,
            version[0],
            version[1],
            ((ciphertext_length >> 8) & 0xFF) as u8,
            (ciphertext_length & 0xFF) as u8,
        ];

        debug!("   AAD (TLS record header): {:02x?}", aad);

        // Decrypt via crypto provider
        let plaintext = match self.cipher_suite {
            CipherSuite::Aes128GcmSha256 => {
                debug!("   → Using AES-128-GCM");
                self.crypto.aes128_gcm_decrypt(key, &nonce, ciphertext, &aad).await
            }
            CipherSuite::Aes256GcmSha384 => {
                debug!("   → Using AES-256-GCM");
                self.crypto.aes256_gcm_decrypt(key, &nonce, ciphertext, &aad).await
            }
            CipherSuite::ChaCha20Poly1305Sha256 => {
                debug!("   → Using ChaCha20-Poly1305");
                self.crypto.decrypt(key, &nonce, ciphertext, &aad).await
            }
        }
        .map_err(|e| {
            error!("❌ Decryption failed: {}", e);
            error!("   AEAD authentication failure");
            Error::TlsHandshake(format!("Failed to decrypt: {e}"))
        })?;

        debug!("✅ Decrypted {} bytes → {} bytes", ciphertext.len(), plaintext.len());

        // Strip ContentType byte (last byte)
        if plaintext.is_empty() {
            return Err(Error::TlsHandshake("Decrypted plaintext is empty".to_string()));
        }

        let content_type_byte = plaintext[plaintext.len() - 1];
        let content = &plaintext[..plaintext.len() - 1];

        debug!("   ContentType: 0x{:02x}", content_type_byte);

        Ok(content.to_vec())
    }

    /// Send encrypted handshake message
    pub(super) async fn send_encrypted_handshake_message(
        &self,
        stream: &mut tokio::net::TcpStream,
        plaintext: &[u8],
        sequence_number: u64,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let handshake_keys = self
            .handshake_keys
            .as_ref()
            .ok_or_else(|| Error::TlsHandshake("Handshake keys not available".to_string()))?;

        // Add ContentType byte for TLS 1.3
        let mut inner_plaintext = plaintext.to_vec();
        inner_plaintext.push(content_type::HANDSHAKE);

        // Encrypt using helper
        let ciphertext = self
            .encrypt_handshake_message(
                &inner_plaintext,
                &handshake_keys.server_write_key,
                &handshake_keys.server_write_iv,
                sequence_number,
            )
            .await?;

        // Wrap in TLS record
        let record = self.wrap_in_tls_record(content_type::APPLICATION_DATA, &ciphertext);

        // Send
        stream.write_all(&record).await.map_err(Error::Io)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use crate::tls::handshake_v2::keys::CipherSuite;

    /// Nonce construction matches `encrypt_handshake_message` / `decrypt_application_data` (IV XOR seq).
    fn apply_iv_xor_sequence(iv: &[u8], sequence_number: u64) -> Vec<u8> {
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

    /// AAD bytes match TLS 1.3 compatibility record header used for AEAD.
    fn tls13_compat_aad(ciphertext_length: usize) -> [u8; 5] {
        let record_type = 0x17u8;
        let version = [0x03u8, 0x03u8];
        #[allow(
            clippy::cast_possible_truncation,
            reason = "TLS record length is masked to single-byte range (& 0xFF)"
        )]
        let len_hi = ((ciphertext_length >> 8) & 0xFF) as u8;
        #[allow(
            clippy::cast_possible_truncation,
            reason = "TLS record length is masked to single-byte range (& 0xFF)"
        )]
        let len_lo = (ciphertext_length & 0xFF) as u8;
        [record_type, version[0], version[1], len_hi, len_lo]
    }

    #[test]
    fn iv_xor_sequence_zero_leaves_iv_tail() {
        let iv = [0u8; 12];
        let n = apply_iv_xor_sequence(&iv, 0);
        assert_eq!(n, iv);
    }

    #[test]
    fn iv_xor_sequence_one_flips_last_byte_of_tail() {
        let iv = vec![0u8; 12];
        let n = apply_iv_xor_sequence(&iv, 1);
        let mut expect = iv;
        expect[11] ^= 1;
        assert_eq!(n, expect);
    }

    #[test]
    fn iv_xor_sequence_max_u64_xors_all_eight_tail_bytes() {
        let iv = [0xffu8; 12];
        let n = apply_iv_xor_sequence(&iv, u64::MAX);
        let mut expect = iv.to_vec();
        for i in 0..8 {
            expect[4 + i] ^= 0xff;
        }
        assert_eq!(n, expect);
    }

    #[test]
    fn aad_length_field_matches_ciphertext_size() {
        let ct_len = 42usize;
        let aad = tls13_compat_aad(ct_len);
        let encoded = u16::from_be_bytes([aad[3], aad[4]]) as usize;
        assert_eq!(encoded, ct_len);
        assert_eq!(aad[0], 0x17);
        assert_eq!(&aad[1..3], &[0x03, 0x03]);
    }

    #[test]
    fn cipher_suite_variants_cover_tls13_suite_ids() {
        assert_eq!(CipherSuite::Aes128GcmSha256.to_u16(), 0x1301);
        assert_eq!(CipherSuite::Aes256GcmSha384.to_u16(), 0x1302);
        assert_eq!(CipherSuite::ChaCha20Poly1305Sha256.to_u16(), 0x1303);
    }

    #[test]
    fn aad_zero_length_encodes() {
        let aad = tls13_compat_aad(0);
        assert_eq!(aad[3..5], [0, 0]);
    }
}
