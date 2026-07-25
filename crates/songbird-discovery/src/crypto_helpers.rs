// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Crypto helpers with bearDog UDS delegation.
//!
//! All hashing routes through `CryptoProvider` (bearDog's `crypto.sha256`
//! capability via UDS). The `local-crypto-fallback` feature provides a
//! degraded local implementation for bootstrap/offline scenarios.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::json;
use songbird_crypto_provider::CryptoProvider;

#[cfg(feature = "local-crypto-fallback")]
use sha2::{Digest, Sha256};

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

/// Compute SHA-256 via bearDog UDS delegation.
///
/// Delegates to `crypto.sha256` when `CryptoProvider` is available.
/// Falls back to local `sha2` crate only with `local-crypto-fallback` feature.
pub async fn sha256_hash(crypto: Option<&CryptoProvider>, data: &[u8]) -> Vec<u8> {
    if let Some(p) = crypto {
        match p.call("crypto.sha256", json!({ "data": BASE64.encode(data) })).await {
            Ok(v) => {
                if let Some(hash) = decode_hash_b64(&v) {
                    return hash;
                }
            }
            Err(e) => {
                tracing::warn!(target: "songbird_discovery", "crypto.sha256 delegation failed: {e}");
            }
        }
    }

    #[cfg(feature = "local-crypto-fallback")]
    {
        tracing::debug!(target: "songbird_discovery", "SHA-256: using local fallback (bearDog unavailable)");
        sha256_local(data)
    }

    #[cfg(not(feature = "local-crypto-fallback"))]
    {
        tracing::error!(target: "songbird_discovery", "SHA-256: bearDog delegation failed and local-crypto-fallback disabled");
        Vec::new()
    }
}

/// Synchronous SHA-256 — uses local fallback in sync contexts.
///
/// Attempts delegation via `CryptoProvider` if a tokio runtime is available,
/// otherwise uses local fallback. Returns empty vec if both paths are unavailable.
pub fn sha256_hash_sync(crypto: Option<&CryptoProvider>, data: &[u8]) -> Vec<u8> {
    crypto.map_or_else(
        || {
            #[cfg(feature = "local-crypto-fallback")]
            {
                tracing::debug!(target: "songbird_discovery", "SHA-256 sync: local fallback (no provider)");
                sha256_local(data)
            }
            #[cfg(not(feature = "local-crypto-fallback"))]
            {
                tracing::error!(target: "songbird_discovery", "SHA-256 sync: no provider and local-crypto-fallback disabled");
                Vec::new()
            }
        },
        |p| {
            if tokio::runtime::Handle::try_current().is_ok() {
                #[cfg(feature = "local-crypto-fallback")]
                {
                    tracing::debug!(target: "songbird_discovery", "SHA-256 sync: local fallback (in async context)");
                    sha256_local(data)
                }
                #[cfg(not(feature = "local-crypto-fallback"))]
                {
                    tracing::error!(target: "songbird_discovery", "SHA-256 sync: cannot delegate in sync context");
                    Vec::new()
                }
            } else {
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt.block_on(sha256_hash(Some(p), data)),
                    Err(e) => {
                        tracing::error!(target: "songbird_discovery", "SHA-256 sync: runtime creation failed: {e}");
                        #[cfg(feature = "local-crypto-fallback")]
                        { sha256_local(data) }
                        #[cfg(not(feature = "local-crypto-fallback"))]
                        { Vec::new() }
                    }
                }
            }
        },
    )
}

/// Compute BLAKE3 hash via bearDog UDS delegation (`crypto.hash.blake3`).
///
/// Falls back to local `blake3` crate only with `local-crypto-fallback` feature.
pub async fn blake3_hash(crypto: Option<&CryptoProvider>, data: &[u8]) -> Vec<u8> {
    if let Some(p) = crypto {
        match p.call("crypto.hash.blake3", json!({ "data": BASE64.encode(data) })).await {
            Ok(v) => {
                if let Some(hash) = decode_hash_b64(&v) {
                    return hash;
                }
            }
            Err(e) => {
                tracing::warn!(target: "songbird_discovery", "crypto.hash.blake3 delegation failed: {e}");
            }
        }
    }

    #[cfg(feature = "local-crypto-fallback")]
    {
        tracing::debug!(target: "songbird_discovery", "BLAKE3: using local fallback (bearDog unavailable)");
        blake3::hash(data).as_bytes().to_vec()
    }

    #[cfg(not(feature = "local-crypto-fallback"))]
    {
        tracing::error!(target: "songbird_discovery", "BLAKE3: bearDog delegation failed and local-crypto-fallback disabled");
        Vec::new()
    }
}

/// Synchronous BLAKE3 hash — local fallback for sync contexts.
///
/// Used by `dark_forest_beacon::hash_capabilities` which is a sync function.
/// When bearDog UDS delegation matures, this will attempt async delegation
/// if a tokio runtime is available, otherwise falls back to local.
#[cfg(feature = "local-crypto-fallback")]
pub fn blake3_hash_sync(data: &[u8]) -> [u8; 32] {
    let hash = blake3::hash(data);
    *hash.as_bytes()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    fn to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }

    #[tokio::test]
    async fn sha256_hash_empty_input() {
        let hash = sha256_hash(None, b"").await;
        assert_eq!(hash.len(), 32);
        assert_eq!(to_hex(&hash), EMPTY_SHA256);
    }

    #[tokio::test]
    async fn sha256_hash_hello_known_vector() {
        let hash = sha256_hash(None, b"hello").await;
        assert_eq!(to_hex(&hash), HELLO_SHA256);
    }

    #[tokio::test]
    async fn sha256_hash_is_deterministic() {
        let data = b"deterministic payload";
        let first = sha256_hash(None, data).await;
        let second = sha256_hash(None, data).await;
        assert_eq!(first, second);
    }

    #[test]
    fn sha256_hash_sync_empty_input() {
        let hash = sha256_hash_sync(None, b"");
        assert_eq!(hash.len(), 32);
        assert_eq!(to_hex(&hash), EMPTY_SHA256);
    }

    #[tokio::test]
    async fn sha256_hash_sync_matches_async_without_provider() {
        let data = b"sync vs async parity";
        let async_hash = sha256_hash(None, data).await;
        let sync_hash = sha256_hash_sync(None, data);
        assert_eq!(async_hash, sync_hash);
    }

    #[test]
    fn sha256_hash_sync_produces_32_byte_digest() {
        let hash = sha256_hash_sync(None, b"payload");
        assert_eq!(hash.len(), 32);
        assert_ne!(to_hex(&hash), EMPTY_SHA256);
    }

    #[tokio::test]
    async fn sha256_hash_different_inputs_produce_different_hashes() {
        let a = sha256_hash(None, b"alpha").await;
        let b = sha256_hash(None, b"beta").await;
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn blake3_hash_produces_32_byte_digest() {
        let hash = blake3_hash(None, b"hello beacon").await;
        assert_eq!(hash.len(), 32);
    }

    #[tokio::test]
    async fn blake3_hash_is_deterministic() {
        let a = blake3_hash(None, b"deterministic").await;
        let b = blake3_hash(None, b"deterministic").await;
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn blake3_hash_different_inputs_differ() {
        let a = blake3_hash(None, b"alpha").await;
        let b = blake3_hash(None, b"beta").await;
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn blake3_hash_matches_local_crate() {
        let data = b"parity check";
        let delegated = blake3_hash(None, data).await;
        let local = blake3::hash(data).as_bytes().to_vec();
        assert_eq!(delegated, local);
    }
}
