// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::json;
use sha2::{Digest, Sha256};
use songbird_crypto_provider::CryptoProvider;

fn sha256_local(data: &[u8]) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().to_vec()
}

fn decode_hash_b64(v: &serde_json::Value) -> Option<Vec<u8>> {
    let b64 = v.get("hash")?.as_str()?;
    BASE64.decode(b64).ok()
}

pub async fn sha256_hash(crypto: Option<&CryptoProvider>, data: &[u8]) -> Vec<u8> {
    if let Some(p) = crypto {
        match p.call("crypto.sha256", json!({ "data": BASE64.encode(data) })).await {
            Ok(v) => {
                if let Some(hash) = decode_hash_b64(&v) {
                    return hash;
                }
            }
            Err(e) => {
                tracing::warn!(target: "songbird_discovery", "crypto.sha256 failed: {e}; using local sha2");
            }
        }
    } else {
        tracing::warn!(target: "songbird_discovery", "SHA-256 without CryptoProvider; using local sha2");
    }
    sha256_local(data)
}

pub fn sha256_hash_sync(crypto: Option<&CryptoProvider>, data: &[u8]) -> Vec<u8> {
    crypto.map_or_else(
        || {
            tracing::warn!(target: "songbird_discovery", "SHA-256 without CryptoProvider; using local sha2");
            sha256_local(data)
        },
        |p| {
            if tokio::runtime::Handle::try_current().is_ok() {
                tracing::warn!(
                    target: "songbird_discovery",
                    "SHA-256: CryptoProvider set but sync context cannot await security provider; using local sha2"
                );
                sha256_local(data)
            } else {
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt.block_on(sha256_hash(Some(p), data)),
                    Err(e) => {
                        tracing::warn!(
                            target: "songbird_discovery",
                            "SHA-256: cannot create runtime for security provider: {e}; using local sha2"
                        );
                        sha256_local(data)
                    }
                }
            }
        },
    )
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
}
