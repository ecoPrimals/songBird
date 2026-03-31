// SPDX-License-Identifier: AGPL-3.0-only
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
    match crypto {
        None => {
            tracing::warn!(target: "songbird_discovery", "SHA-256 without CryptoProvider; using local sha2");
            sha256_local(data)
        }
        Some(p) => {
            if tokio::runtime::Handle::try_current().is_ok() {
                tracing::warn!(
                    target: "songbird_discovery",
                    "SHA-256: CryptoProvider set but sync context cannot await BearDog; using local sha2"
                );
                sha256_local(data)
            } else {
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt.block_on(sha256_hash(Some(p), data)),
                    Err(e) => {
                        tracing::warn!(
                            target: "songbird_discovery",
                            "SHA-256: cannot create runtime for BearDog: {e}; using local sha2"
                        );
                        sha256_local(data)
                    }
                }
            }
        }
    }
}
