// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::{Result, TlsError};
use crate::messages::ContentType;

use super::layer::RecordLayer;

impl RecordLayer {
    /// Encrypt a TLS record (Application Data)
    ///
    /// In TLS 1.3, the actual content type is hidden inside the encrypted payload.
    /// The record content type is always `ApplicationData` (23).
    ///
    /// Format of encrypted payload:
    /// ```text
    /// struct {
    ///     opaque content[length];
    ///     ContentType type;        // Actual content type
    ///     uint8 zeros[length_of_padding];
    /// } TLSInnerPlaintext;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if encryption or framing fails.
    pub fn encrypt_record(
        &mut self,
        content_type: ContentType,
        plaintext: &[u8],
        encrypt_fn: impl FnOnce(&[u8], u64) -> Result<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        // Build inner plaintext: content + content_type + padding
        let mut inner = Vec::with_capacity(plaintext.len() + 1);
        inner.extend_from_slice(plaintext);
        inner.push(content_type.into()); // Actual content type
        // No padding for now (can be added later for traffic analysis resistance)

        // Encrypt the inner plaintext
        let ciphertext = encrypt_fn(&inner, self.write_sequence)?;

        // Increment sequence number
        self.increment_write_sequence();

        // Frame as ApplicationData record
        self.frame_plaintext(ContentType::ApplicationData, &ciphertext)
    }

    /// Decrypt a TLS record (Application Data)
    ///
    /// Extracts the hidden content type from the end of the decrypted payload.
    ///
    /// # Errors
    ///
    /// Returns an error if decryption fails or payload is invalid.
    pub fn decrypt_record(
        &mut self,
        ciphertext: &[u8],
        decrypt_fn: impl FnOnce(&[u8], u64) -> Result<Vec<u8>>,
    ) -> Result<(ContentType, Vec<u8>)> {
        // Decrypt the ciphertext
        let mut inner = decrypt_fn(ciphertext, self.read_sequence)?;

        // Increment sequence number
        self.increment_read_sequence();

        // Extract content type from the end (remove padding zeros first)
        while !inner.is_empty() && inner[inner.len() - 1] == 0 {
            inner.pop();
        }

        if inner.is_empty() {
            return Err(TlsError::DecryptError);
        }

        // Last byte is the actual content type
        let content_type_byte = inner.pop().ok_or(TlsError::DecryptError)?;
        let content_type = ContentType::from(content_type_byte);

        Ok((content_type, inner))
    }
}
