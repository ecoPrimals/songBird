// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Crypto helpers with bearDog UDS delegation.
//!
//! All cryptographic operations route through `CryptoProvider` (bearDog's
//! `crypto.*` capabilities via UDS). The `local-crypto-fallback` feature
//! provides a degraded local implementation for bootstrap/offline scenarios.
//! Production builds without the feature will error if bearDog is unavailable.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::json;
use songbird_crypto_provider::CryptoProvider;

/// Error returned when crypto delegation fails without a local fallback.
#[derive(Debug, thiserror::Error)]
pub enum CryptoDelegationError {
    /// bearDog unavailable and `local-crypto-fallback` feature not enabled.
    #[error("crypto delegation failed: {0}")]
    Unavailable(String),
}

#[cfg(feature = "local-crypto-fallback")]
use hmac::{Hmac, Mac};
#[cfg(feature = "local-crypto-fallback")]
use sha2::{Digest, Sha256};
#[cfg(feature = "local-crypto-fallback")]
type HmacSha256 = Hmac<Sha256>;

#[cfg(feature = "local-crypto-fallback")]
fn sha256_local(data: &[u8]) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().to_vec()
}

fn decode_hash_b64(v: &serde_json::Value) -> Option<Vec<u8>> {
    let b64 = v.get("hash")?.as_str()?;
    BASE64.decode(b64).ok()
}

fn decode_mac_b64(v: &serde_json::Value) -> Option<Vec<u8>> {
    let b64 = v.get("mac")?.as_str()?;
    BASE64.decode(b64).ok()
}

/// Compute SHA-256 via bearDog UDS delegation.
///
/// When `CryptoProvider` is available, delegates to `crypto.sha256`.
/// With `local-crypto-fallback` feature: falls back to local `sha2` crate.
/// Without feature: returns empty vec (callers should use `try_sha256_hash`).
///
/// # Deprecated behavior
///
/// Returns empty `Vec` on total failure. Prefer [`try_sha256_hash`] which
/// propagates errors properly.
pub async fn sha256_hash(crypto: Option<&CryptoProvider>, data: &[u8]) -> Vec<u8> {
    try_sha256_hash(crypto, data).await.unwrap_or_default()
}

/// Compute SHA-256 via bearDog UDS delegation (error-propagating).
///
/// # Errors
///
/// Returns error if crypto delegation fails and no local fallback is available.
pub async fn try_sha256_hash(
    crypto: Option<&CryptoProvider>,
    data: &[u8],
) -> Result<Vec<u8>, CryptoDelegationError> {
    if let Some(p) = crypto {
        match p.call("crypto.sha256", json!({ "data": BASE64.encode(data) })).await {
            Ok(v) => {
                if let Some(hash) = decode_hash_b64(&v) {
                    return Ok(hash);
                }
            }
            Err(e) => {
                tracing::warn!(
                    target: "songbird_network_federation",
                    "crypto.sha256 delegation failed: {e}"
                );
            }
        }
    }

    #[cfg(feature = "local-crypto-fallback")]
    {
        tracing::debug!(
            target: "songbird_network_federation",
            "SHA-256: using local fallback (bearDog unavailable)"
        );
        Ok(sha256_local(data))
    }

    #[cfg(not(feature = "local-crypto-fallback"))]
    {
        tracing::error!(
            target: "songbird_network_federation",
            "SHA-256: bearDog crypto delegation failed and local-crypto-fallback is disabled"
        );
        Err(CryptoDelegationError::Unavailable(
            "SHA-256: no crypto provider and local-crypto-fallback disabled".into(),
        ))
    }
}

/// Compute HMAC-SHA256 via bearDog UDS delegation.
///
/// When `CryptoProvider` is available, delegates to `crypto.hmac.sha256`.
/// With `local-crypto-fallback` feature: falls back to local `hmac`+`sha2` crates.
/// Without feature: returns empty vec. Prefer [`try_hmac_sha256`].
pub async fn hmac_sha256(crypto: Option<&CryptoProvider>, key: &[u8], data: &[u8]) -> Vec<u8> {
    try_hmac_sha256(crypto, key, data).await.unwrap_or_default()
}

/// Compute HMAC-SHA256 via bearDog UDS delegation (error-propagating).
///
/// # Errors
///
/// Returns error if crypto delegation fails and no local fallback is available.
pub async fn try_hmac_sha256(
    crypto: Option<&CryptoProvider>,
    key: &[u8],
    data: &[u8],
) -> Result<Vec<u8>, CryptoDelegationError> {
    if let Some(p) = crypto {
        match p
            .call(
                "crypto.hmac.sha256",
                json!({
                    "key": BASE64.encode(key),
                    "data": BASE64.encode(data)
                }),
            )
            .await
        {
            Ok(v) => {
                if let Some(mac) = decode_mac_b64(&v) {
                    return Ok(mac);
                }
            }
            Err(e) => {
                tracing::warn!(
                    target: "songbird_network_federation",
                    "crypto.hmac.sha256 delegation failed: {e}"
                );
            }
        }
    }

    #[cfg(feature = "local-crypto-fallback")]
    {
        tracing::debug!(
            target: "songbird_network_federation",
            "HMAC-SHA256: using local fallback (bearDog unavailable)"
        );
        match HmacSha256::new_from_slice(key) {
            Ok(mut mac) => {
                mac.update(data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            Err(e) => Err(CryptoDelegationError::Unavailable(format!(
                "HMAC-SHA256 local key rejected: {e}"
            ))),
        }
    }

    #[cfg(not(feature = "local-crypto-fallback"))]
    {
        tracing::error!(
            target: "songbird_network_federation",
            "HMAC-SHA256: bearDog crypto delegation failed and local-crypto-fallback is disabled"
        );
        Err(CryptoDelegationError::Unavailable(
            "HMAC-SHA256: no crypto provider and local-crypto-fallback disabled".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn sha256_without_provider_produces_32_byte_hash() {
        let hash = sha256_hash(None, b"hello federation").await;
        assert_eq!(hash.len(), 32);
    }

    #[tokio::test]
    async fn sha256_is_deterministic() {
        let a = sha256_hash(None, b"deterministic").await;
        let b = sha256_hash(None, b"deterministic").await;
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn hmac_sha256_without_provider_produces_32_byte_mac() {
        let mac = hmac_sha256(None, b"secret", b"message").await;
        assert_eq!(mac.len(), 32);
    }

    #[tokio::test]
    async fn hmac_sha256_different_keys_produce_different_macs() {
        let a = hmac_sha256(None, b"key1", b"data").await;
        let b = hmac_sha256(None, b"key2", b"data").await;
        assert_ne!(a, b);
    }
}
