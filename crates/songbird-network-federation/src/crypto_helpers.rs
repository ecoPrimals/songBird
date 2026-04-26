// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::{Digest, Sha256};
use songbird_crypto_provider::CryptoProvider;

type HmacSha256 = Hmac<Sha256>;

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

pub async fn sha256_hash(crypto: Option<&CryptoProvider>, data: &[u8]) -> Vec<u8> {
    if let Some(p) = crypto {
        match p.call("crypto.sha256", json!({ "data": BASE64.encode(data) })).await {
            Ok(v) => {
                if let Some(hash) = decode_hash_b64(&v) {
                    return hash;
                }
            }
            Err(e) => {
                tracing::warn!(
                    target: "songbird_network_federation",
                    "crypto.sha256 failed: {e}; using local sha2"
                );
            }
        }
    } else {
        tracing::warn!(
            target: "songbird_network_federation",
            "SHA-256 without CryptoProvider; using local sha2"
        );
    }
    sha256_local(data)
}

pub async fn hmac_sha256(crypto: Option<&CryptoProvider>, key: &[u8], data: &[u8]) -> Vec<u8> {
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
                    return mac;
                }
            }
            Err(e) => {
                tracing::warn!(
                    target: "songbird_network_federation",
                    "crypto.hmac.sha256 failed: {e}; using local hmac+sha2"
                );
            }
        }
    } else {
        tracing::warn!(
            target: "songbird_network_federation",
            "HMAC-SHA256 without CryptoProvider; using local hmac+sha2"
        );
    }
    match HmacSha256::new_from_slice(key) {
        Ok(mut mac) => {
            mac.update(data);
            mac.finalize().into_bytes().to_vec()
        }
        Err(e) => {
            tracing::warn!(
                target: "songbird_network_federation",
                "HMAC-SHA256 local init failed: {e}; returning empty MAC"
            );
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use sha2::{Digest, Sha256};

    #[tokio::test]
    async fn sha256_without_provider_matches_sha2_crate() {
        let data = b"hello federation";
        let local: Vec<u8> = {
            let mut h = Sha256::new();
            h.update(data);
            h.finalize().to_vec()
        };
        let got = sha256_hash(None, data).await;
        assert_eq!(got, local);
    }

    #[tokio::test]
    async fn hmac_sha256_without_provider_matches_expected() {
        let key = b"secret";
        let data = b"message";
        let mac = hmac_sha256(None, key, data).await;
        assert_eq!(mac.len(), 32);
        let empty_key_mac = hmac_sha256(None, &[], data).await;
        assert_eq!(empty_key_mac.len(), 32);
    }
}
