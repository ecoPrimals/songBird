// SPDX-License-Identifier: AGPL-3.0-or-later
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
    /// Returns [`TlsError::CryptoUnavailable`] when no provider was configured or the security provider is unreachable.
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
    /// Returns [`TlsError::CryptoUnavailable`] when no provider was configured or the security provider is unreachable.
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

/// TLS record AEAD encrypt: security provider `crypto.aead_encrypt` (semantic wire name).
///
/// # Errors
///
/// Returns [`TlsError::CryptoUnavailable`] when the RPC to the security provider fails.
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
        warn!(error = %e, "TLS record AEAD encrypt: security provider unavailable");
        TlsError::CryptoUnavailable
    })?;
    let ciphertext_b64 = result["ciphertext"].as_str().ok_or_else(|| {
        TlsError::CryptoError("Security provider aead_encrypt: missing ciphertext".to_string())
    })?;
    let tag_b64 = result["tag"].as_str().ok_or_else(|| {
        TlsError::CryptoError("Security provider aead_encrypt: missing tag".to_string())
    })?;
    let mut ciphertext = general_purpose::STANDARD
        .decode(ciphertext_b64)
        .map_err(|e| TlsError::CryptoError(format!("decode ciphertext: {e}")))?;
    let tag = general_purpose::STANDARD
        .decode(tag_b64)
        .map_err(|e| TlsError::CryptoError(format!("decode tag: {e}")))?;
    ciphertext.extend_from_slice(&tag);
    Ok(ciphertext)
}

/// TLS record AEAD decrypt: security provider `crypto.aead_decrypt` (semantic wire name).
///
/// # Errors
///
/// Returns [`TlsError::CryptoUnavailable`] when the RPC to the security provider fails.
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
        warn!(error = %e, "TLS record AEAD decrypt: security provider unavailable");
        TlsError::CryptoUnavailable
    })?;
    let plaintext_b64 = result["plaintext"].as_str().ok_or_else(|| {
        TlsError::CryptoError("Security provider aead_decrypt: missing plaintext".to_string())
    })?;
    general_purpose::STANDARD
        .decode(plaintext_b64)
        .map_err(|e| TlsError::CryptoError(format!("decode plaintext: {e}")))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use std::sync::Arc;

    use base64::Engine;
    use base64::engine::general_purpose;
    use songbird_crypto_provider::CryptoProvider;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::messages::ContentType;

    async fn spawn_mock_jsonrpc_server(response_body: String) -> String {
        let dir = std::env::temp_dir().join(format!(
            "songbird-tls-crypto-prov-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        ));
        let _ = std::fs::create_dir_all(&dir);
        let sock_path = dir.join("mock.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
        let path = sock_path.to_string_lossy().into_owned();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 16_384];
                let _ = stream.read(&mut buf).await;
                stream.write_all(response_body.as_bytes()).await.ok();
                let _ = stream.shutdown().await;
            }
        });
        tokio::task::yield_now().await;
        path
    }

    async fn spawn_mock_aead_server() -> String {
        let dir = std::env::temp_dir().join(format!(
            "songbird-tls-aead-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        ));
        let _ = std::fs::create_dir_all(&dir);
        let sock_path = dir.join("aead.sock");
        let _ = std::fs::remove_file(&sock_path);
        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
        let path = sock_path.to_string_lossy().into_owned();
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = vec![0u8; 16_384];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    continue;
                }
                let req: serde_json::Value =
                    serde_json::from_slice(&buf[..n]).unwrap_or_else(|_| json!({}));
                let method = req["method"].as_str().unwrap_or("");
                let params = &req["params"];

                let response = if method.contains("encrypt") {
                    let plaintext_b64 = params["plaintext"].as_str().unwrap_or("");
                    let plaintext =
                        general_purpose::STANDARD.decode(plaintext_b64).unwrap_or_default();
                    let ciphertext: Vec<u8> = plaintext.iter().map(|b| b ^ 0xAA).collect();
                    let tag = vec![0xBBu8; 16];
                    json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "ciphertext": general_purpose::STANDARD.encode(&ciphertext),
                            "tag": general_purpose::STANDARD.encode(&tag),
                        },
                        "id": req["id"],
                    })
                } else {
                    let ciphertext_b64 = params["ciphertext"].as_str().unwrap_or("");
                    let ciphertext =
                        general_purpose::STANDARD.decode(ciphertext_b64).unwrap_or_default();
                    let plaintext: Vec<u8> = ciphertext.iter().map(|b| b ^ 0xAA).collect();
                    json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "plaintext": general_purpose::STANDARD.encode(&plaintext),
                        },
                        "id": req["id"],
                    })
                };

                let body = serde_json::to_string(&response).unwrap();
                stream.write_all(body.as_bytes()).await.ok();
                let _ = stream.shutdown().await;
            }
        });
        tokio::task::yield_now().await;
        path
    }

    #[test]
    fn tls_record_nonce_from_iv_sequence_zero() {
        let iv = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C];
        let nonce = tls_record_nonce_from_iv(&iv, 0).unwrap();
        assert_eq!(nonce, iv);
    }

    #[test]
    fn tls_record_nonce_from_iv_xor_sequence_into_last_eight_bytes() {
        let iv = [0xFF; 12];
        let nonce = tls_record_nonce_from_iv(&iv, 1).unwrap();
        assert_eq!(&nonce[..4], &[0xFF; 4]);
        assert_eq!(&nonce[4..], &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE]);
    }

    #[test]
    fn tls_record_nonce_from_iv_large_sequence() {
        let iv = [0u8; 12];
        let seq = 0x0102_0304_0506_0708_u64;
        let nonce = tls_record_nonce_from_iv(&iv, seq).unwrap();
        assert_eq!(&nonce[..4], &[0u8; 4]);
        assert_eq!(&nonce[4..], &seq.to_be_bytes());
    }

    #[test]
    fn tls_record_nonce_from_iv_rejects_short_iv() {
        let err = tls_record_nonce_from_iv(&[0u8; 11], 0).unwrap_err();
        assert!(matches!(err, TlsError::InvalidParameter(_)));
    }

    #[test]
    fn tls_record_nonce_from_iv_rejects_long_iv() {
        let err = tls_record_nonce_from_iv(&[0u8; 13], 0).unwrap_err();
        assert!(matches!(err, TlsError::InvalidParameter(_)));
    }

    #[test]
    fn tls_record_nonce_from_iv_max_sequence() {
        let iv = [0xAB; 12];
        let nonce = tls_record_nonce_from_iv(&iv, u64::MAX).unwrap();
        let mut expected = iv;
        for i in 0..8 {
            expected[4 + i] ^= 0xFF;
        }
        assert_eq!(nonce, expected);
    }

    #[tokio::test]
    async fn record_aead_decrypt_rejects_ciphertext_shorter_than_tag() {
        let provider = CryptoProvider::new("/nonexistent.sock");
        let err =
            record_aead_decrypt_via_provider(&provider, &[0u8; 15], &[0u8; 32], &[0u8; 12], None)
                .await
                .unwrap_err();
        assert!(matches!(err, TlsError::DecryptError));
    }

    #[tokio::test]
    async fn record_aead_decrypt_rejects_empty_ciphertext() {
        let provider = CryptoProvider::new("/nonexistent.sock");
        let err = record_aead_decrypt_via_provider(&provider, &[], &[0u8; 32], &[0u8; 12], None)
            .await
            .unwrap_err();
        assert!(matches!(err, TlsError::DecryptError));
    }

    #[tokio::test]
    async fn record_aead_encrypt_missing_ciphertext_field() {
        let path = spawn_mock_jsonrpc_server(
            r#"{"jsonrpc":"2.0","result":{"tag":"AAAA"},"id":1}"#.to_string(),
        )
        .await;

        let provider = CryptoProvider::new(path);
        let err =
            record_aead_encrypt_via_provider(&provider, b"plaintext", &[0u8; 32], &[0u8; 12], None)
                .await
                .unwrap_err();
        assert!(matches!(err, TlsError::CryptoError(ref m) if m.contains("ciphertext")));
    }

    #[tokio::test]
    async fn record_aead_encrypt_missing_tag_field() {
        let path = spawn_mock_jsonrpc_server(
            r#"{"jsonrpc":"2.0","result":{"ciphertext":"AAAA"},"id":1}"#.to_string(),
        )
        .await;

        let provider = CryptoProvider::new(path);
        let err =
            record_aead_encrypt_via_provider(&provider, b"plaintext", &[0u8; 32], &[0u8; 12], None)
                .await
                .unwrap_err();
        assert!(matches!(err, TlsError::CryptoError(ref m) if m.contains("tag")));
    }

    #[tokio::test]
    async fn record_aead_encrypt_malformed_base64_ciphertext() {
        let path = spawn_mock_jsonrpc_server(
            r#"{"jsonrpc":"2.0","result":{"ciphertext":"!!!","tag":"AAAA"},"id":1}"#.to_string(),
        )
        .await;

        let provider = CryptoProvider::new(path);
        let err =
            record_aead_encrypt_via_provider(&provider, b"plaintext", &[0u8; 32], &[0u8; 12], None)
                .await
                .unwrap_err();
        assert!(matches!(err, TlsError::CryptoError(ref m) if m.contains("decode ciphertext")));
    }

    #[tokio::test]
    async fn record_aead_decrypt_missing_plaintext_field() {
        let path =
            spawn_mock_jsonrpc_server(r#"{"jsonrpc":"2.0","result":{},"id":1}"#.to_string()).await;

        let provider = CryptoProvider::new(path);
        let payload = [0u8; 16]; // 0-byte ciphertext + 16-byte tag
        let err =
            record_aead_decrypt_via_provider(&provider, &payload, &[0u8; 32], &[0u8; 12], None)
                .await
                .unwrap_err();
        assert!(matches!(err, TlsError::CryptoError(ref m) if m.contains("plaintext")));
    }

    #[tokio::test]
    async fn record_aead_encrypt_decrypt_roundtrip_with_mock_provider() {
        let path = spawn_mock_aead_server().await;

        let provider = CryptoProvider::new(path);
        let key = [0xCCu8; 32];
        let iv = [0xDDu8; 12];
        let nonce = tls_record_nonce_from_iv(&iv, 0).unwrap();
        let plaintext = b"TLS record payload";

        let encrypted = record_aead_encrypt_via_provider(&provider, plaintext, &key, &nonce, None)
            .await
            .unwrap();
        assert!(encrypted.len() >= 16);

        let decrypted = record_aead_decrypt_via_provider(&provider, &encrypted, &key, &nonce, None)
            .await
            .unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn encrypt_record_delegated_requires_crypto_provider() {
        let mut record_layer = RecordLayer::new();
        let err = record_layer
            .encrypt_record_delegated(ContentType::ApplicationData, b"data", &[0u8; 32], &[0u8; 12])
            .await
            .unwrap_err();
        assert!(matches!(err, TlsError::CryptoUnavailable));
    }

    #[tokio::test]
    async fn decrypt_record_delegated_requires_crypto_provider() {
        let mut record_layer = RecordLayer::new();
        let err = record_layer
            .decrypt_record_delegated(&[0u8; 32], &[0u8; 32], &[0u8; 12])
            .await
            .unwrap_err();
        assert!(matches!(err, TlsError::CryptoUnavailable));
    }

    #[tokio::test]
    async fn encrypt_record_delegated_rejects_invalid_iv_length() {
        let mut record_layer =
            RecordLayer::with_crypto_provider(Arc::new(CryptoProvider::new("/unused.sock")));
        let err = record_layer
            .encrypt_record_delegated(ContentType::ApplicationData, b"data", &[0u8; 32], &[0u8; 11])
            .await
            .unwrap_err();
        assert!(matches!(err, TlsError::InvalidParameter(_)));
    }

    #[tokio::test]
    async fn decrypt_record_delegated_strips_trailing_padding_zeros() {
        let mut inner = b"hello".to_vec();
        inner.push(ContentType::Handshake as u8);
        inner.extend([0u8; 3]);
        let body = format!(
            r#"{{"jsonrpc":"2.0","result":{{"plaintext":"{}"}},"id":1}}"#,
            general_purpose::STANDARD.encode(&inner),
        );
        let path = spawn_mock_jsonrpc_server(body).await;

        let mut record_layer =
            RecordLayer::with_crypto_provider(Arc::new(CryptoProvider::new(path)));
        let payload = [0u8; 16];
        let (content_type, plaintext) =
            record_layer.decrypt_record_delegated(&payload, &[0u8; 32], &[0u8; 12]).await.unwrap();
        assert_eq!(content_type, ContentType::Handshake);
        assert_eq!(plaintext, b"hello");
        assert_eq!(record_layer.read_sequence(), 1);
    }
}
