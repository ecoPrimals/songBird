// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 application data encryption and decryption
//!
//! Handles encryption and decryption of HTTP application data after the
//! TLS handshake is complete, using application traffic keys.
//!
//! ## RFC 8446 Compliance
//!
//! - Record layer encryption (Section 5.2)
//! - AEAD nonce construction (Section 5.3)
//! - Sequence number tracking

use super::core::TlsHandshake;
use crate::crypto::CryptoCapability;
use crate::error::Result;
use crate::tls::session::SessionKeys;
use tracing::{debug, trace};

impl TlsHandshake {
    /// Encrypt application-layer data as a TLS 1.3 record.
    ///
    /// # Errors
    ///
    /// Returns an error if the record is too large for the wire format or crypto fails.
    pub async fn encrypt_application_data(
        &self,
        plaintext: &[u8],
        keys: &SessionKeys,
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        trace!(
            "🔐 Encrypting {} bytes of application data (seq={})",
            plaintext.len(),
            sequence_number
        );

        // Calculate ciphertext length (plaintext + 16-byte AEAD tag)
        let ciphertext_length = plaintext.len() + 16;

        // Construct TLS record header (this becomes the AAD)
        let record_type = 0x17; // ContentType: APPLICATION_DATA
        let version = [0x03, 0x03]; // TLS 1.2 (compatibility mode for TLS 1.3)
        let length = u16::try_from(ciphertext_length)
            .map_err(|_| crate::error::Error::TlsHandshake("Record too large".into()))?;

        let aad = [record_type, version[0], version[1], (length >> 8) as u8, (length & 0xFF) as u8];

        trace!("AAD (TLS record header): {:02x?}", aad);

        // Construct nonce: IV XOR sequence_number (RFC 8446 Section 5.3)
        // The sequence number is XORed with the IV (right-aligned)
        let mut nonce = keys.client_write_iv.clone();
        let seq_bytes = sequence_number.to_be_bytes();

        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }

        trace!("Nonce (IV XOR seq): {:02x?}", &nonce[..std::cmp::min(12, nonce.len())]);

        // Encrypt via crypto provider
        let ciphertext =
            self.crypto.encrypt(&keys.client_write_key, &nonce, plaintext, &aad).await?;

        debug!(
            "✅ Encrypted {} bytes → {} bytes (includes 16-byte tag)",
            plaintext.len(),
            ciphertext.len()
        );

        // Construct complete TLS record: header + ciphertext (includes tag)
        let mut record = Vec::new();
        record.extend_from_slice(&aad);
        record.extend_from_slice(&ciphertext);

        Ok(record)
    }

    /// Decrypt application-layer data from a TLS 1.3 record.
    ///
    /// # Errors
    ///
    /// Returns an error if decryption or authentication fails.
    pub async fn decrypt_application_data(
        &self,
        record_header: &[u8; 5],
        ciphertext: &[u8],
        keys: &SessionKeys,
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        trace!(
            "🔓 Decrypting {} bytes of application data (seq={})",
            ciphertext.len(),
            sequence_number
        );
        trace!("Record header (AAD): {:02x?}", record_header);

        // AAD = TLS record header (all 5 bytes: type, version, length)
        let aad = record_header;

        // Construct nonce: IV XOR sequence_number (RFC 8446 Section 5.3)
        let mut nonce = keys.server_write_iv.clone();
        let seq_bytes = sequence_number.to_be_bytes();

        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }

        trace!("Nonce (IV XOR seq): {:02x?}", &nonce[..std::cmp::min(12, nonce.len())]);

        // Decrypt via crypto provider (will handle AEAD tag validation)
        let plaintext =
            self.crypto.decrypt(&keys.server_write_key, &nonce, ciphertext, aad).await?;

        debug!(
            "✅ Decrypted {} bytes → {} bytes (AEAD authentication succeeded)",
            ciphertext.len(),
            plaintext.len()
        );

        Ok(plaintext)
    }
}
