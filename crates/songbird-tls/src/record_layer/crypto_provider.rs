// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use base64::{Engine as _, engine::general_purpose};
use serde_json::json;
use songbird_crypto_provider::CryptoProvider;
use tracing::warn;

use crate::error::{Result, TlsError};
use crate::messages::ContentType;

use super::layer::RecordLayer;

impl RecordLayer {
    /// Encrypt a record using `crypto.aead_encrypt` via [`CryptoProvider`].
    ///
    /// # Errors
    ///
    /// Returns [`TlsError::CryptoUnavailable`] when no provider was configured or BearDog is unreachable.
    pub async fn encrypt_record_delegated(
        &mut self,
        content_type: ContentType,
        plaintext: &[u8],
        key: &[u8],
        iv: &[u8],
    ) -> Result<Vec<u8>> {
        let provider = self.crypto_provider.as_ref().ok_or(TlsError::CryptoUnavailable)?;
        let mut inner = Vec::with_capacity(plaintext.len() + 1);
        inner.extend_from_slice(plaintext);
        inner.push(content_type.into());
        let seq = self.write_sequence;
        let nonce = tls_record_nonce_from_iv(iv, seq)?;
        let ciphertext =
            record_aead_encrypt_via_provider(provider.as_ref(), &inner, key, &nonce, None).await?;
        self.increment_write_sequence();
        self.frame_plaintext(ContentType::ApplicationData, &ciphertext)
    }

    /// Decrypt a record using `crypto.aead_decrypt` via [`CryptoProvider`].
    ///
    /// # Errors
    ///
    /// Returns [`TlsError::CryptoUnavailable`] when no provider was configured or BearDog is unreachable.
    pub async fn decrypt_record_delegated(
        &mut self,
        ciphertext: &[u8],
        key: &[u8],
        iv: &[u8],
    ) -> Result<(ContentType, Vec<u8>)> {
        let provider = self.crypto_provider.as_ref().ok_or(TlsError::CryptoUnavailable)?;
        let seq = self.read_sequence;
        let nonce = tls_record_nonce_from_iv(iv, seq)?;
        let mut inner =
            record_aead_decrypt_via_provider(provider.as_ref(), ciphertext, key, &nonce, None)
                .await?;
        self.increment_read_sequence();
        while !inner.is_empty() && inner[inner.len() - 1] == 0 {
            inner.pop();
        }
        if inner.is_empty() {
            return Err(TlsError::DecryptError);
        }
        let content_type_byte = inner.pop().ok_or(TlsError::DecryptError)?;
        let content_type = ContentType::from(content_type_byte);
        Ok((content_type, inner))
    }
}

fn tls_record_nonce_from_iv(iv: &[u8], sequence: u64) -> Result<[u8; 12]> {
    if iv.len() != 12 {
        return Err(TlsError::InvalidParameter("TLS record AEAD IV must be 12 bytes".to_string()));
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(iv);
    let seq_bytes = sequence.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= seq_bytes[i];
    }
    Ok(nonce)
}

/// TLS record AEAD encrypt: BearDog `crypto.aead_encrypt` (semantic wire name).
///
/// # Errors
///
/// Returns [`TlsError::CryptoUnavailable`] when the RPC to BearDog fails.
pub async fn record_aead_encrypt_via_provider(
    provider: &CryptoProvider,
    plaintext: &[u8],
    key: &[u8],
    nonce: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>> {
    let mut params = json!({
        "plaintext": general_purpose::STANDARD.encode(plaintext),
        "key": general_purpose::STANDARD.encode(key),
        "nonce": general_purpose::STANDARD.encode(nonce),
    });
    if let Some(a) = aad {
        params["aad"] = json!(general_purpose::STANDARD.encode(a));
    }
    let result = provider.call("crypto.aead_encrypt", params).await.map_err(|e| {
        warn!(error = %e, "TLS record AEAD encrypt: BearDog unavailable");
        TlsError::CryptoUnavailable
    })?;
    let ciphertext_b64 = result["ciphertext"].as_str().ok_or_else(|| {
        TlsError::CryptoError("BearDog aead_encrypt: missing ciphertext".to_string())
    })?;
    let tag_b64 = result["tag"]
        .as_str()
        .ok_or_else(|| TlsError::CryptoError("BearDog aead_encrypt: missing tag".to_string()))?;
    let mut ciphertext = general_purpose::STANDARD
        .decode(ciphertext_b64)
        .map_err(|e| TlsError::CryptoError(format!("decode ciphertext: {e}")))?;
    let tag = general_purpose::STANDARD
        .decode(tag_b64)
        .map_err(|e| TlsError::CryptoError(format!("decode tag: {e}")))?;
    ciphertext.extend_from_slice(&tag);
    Ok(ciphertext)
}

/// TLS record AEAD decrypt: BearDog `crypto.aead_decrypt` (semantic wire name).
///
/// # Errors
///
/// Returns [`TlsError::CryptoUnavailable`] when the RPC to BearDog fails.
pub async fn record_aead_decrypt_via_provider(
    provider: &CryptoProvider,
    ciphertext_with_tag: &[u8],
    key: &[u8],
    nonce: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>> {
    if ciphertext_with_tag.len() < 16 {
        return Err(TlsError::DecryptError);
    }
    let tag_start = ciphertext_with_tag.len() - 16;
    let ciphertext = &ciphertext_with_tag[..tag_start];
    let tag = &ciphertext_with_tag[tag_start..];
    let mut params = json!({
        "ciphertext": general_purpose::STANDARD.encode(ciphertext),
        "key": general_purpose::STANDARD.encode(key),
        "nonce": general_purpose::STANDARD.encode(nonce),
        "tag": general_purpose::STANDARD.encode(tag),
    });
    if let Some(a) = aad {
        params["aad"] = json!(general_purpose::STANDARD.encode(a));
    }
    let result = provider.call("crypto.aead_decrypt", params).await.map_err(|e| {
        warn!(error = %e, "TLS record AEAD decrypt: BearDog unavailable");
        TlsError::CryptoUnavailable
    })?;
    let plaintext_b64 = result["plaintext"].as_str().ok_or_else(|| {
        TlsError::CryptoError("BearDog aead_decrypt: missing plaintext".to_string())
    })?;
    general_purpose::STANDARD
        .decode(plaintext_b64)
        .map_err(|e| TlsError::CryptoError(format!("decode plaintext: {e}")))
}
