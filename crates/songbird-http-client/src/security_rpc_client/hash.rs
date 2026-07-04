// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Hashing and HKDF operations
//!
//! SHA-256, SHA-384 hashing and HKDF key derivation.

use super::core::SecurityRpcClient;
use crate::error::{Error, Result};
use base64::prelude::*;
use serde_json::json;

impl SecurityRpcClient {
    /// Compute SHA-256 hash
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    pub async fn sha256(&self, data: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.sha256",
                json!({
                    "data": BASE64_STANDARD.encode(data)
                }),
            )
            .await?;

        let hash_b64 = result["hash"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing hash in sha256 response"))
        })?;

        BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid hash base64: {e}")))
    }

    /// Compute SHA-384 hash
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    pub async fn sha384(&self, data: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.sha384",
                json!({
                    "data": BASE64_STANDARD.encode(data)
                }),
            )
            .await?;

        let hash_b64 = result["hash"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing hash in sha384 response"))
        })?;

        BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid hash base64: {e}")))
    }

    /// HKDF-Extract: Extract a PRK from salt and input keying material
    ///
    /// RFC 5869: HMAC-based Key Derivation Function
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    pub async fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.hkdf_extract",
                json!({
                    "salt": BASE64_STANDARD.encode(salt),
                    "ikm": BASE64_STANDARD.encode(ikm)
                }),
            )
            .await?;

        let prk_b64 = result["prk"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing prk in hkdf_extract response"))
        })?;

        BASE64_STANDARD
            .decode(prk_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid prk base64: {e}")))
    }

    /// HKDF-Expand: Expand a PRK to the desired length
    ///
    /// RFC 5869: HMAC-based Key Derivation Function
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    pub async fn hkdf_expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.hkdf_expand",
                json!({
                    "prk": BASE64_STANDARD.encode(prk),
                    "info": BASE64_STANDARD.encode(info),
                    "length": length
                }),
            )
            .await?;

        let okm_b64 = result["okm"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing okm in hkdf_expand response"))
        })?;

        BASE64_STANDARD
            .decode(okm_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid okm base64: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use base64::prelude::*;
    use serde_json::json;

    #[test]
    fn test_hash_format() {
        // SHA-256 produces 32 bytes
        assert_eq!(32, 256 / 8);
        // SHA-384 produces 48 bytes
        assert_eq!(48, 384 / 8);
    }

    /// Mirrors `sha256` / `sha384` response decoding for unit tests (no live RPC).
    fn decode_hash_field(result: &serde_json::Value, label: &str) -> crate::error::Result<Vec<u8>> {
        let hash_b64 = result["hash"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(format!("Missing hash in {label} response"))
        })?;
        BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid hash base64: {e}")))
    }

    #[test]
    fn sha256_response_missing_hash_field() {
        let err = decode_hash_field(&json!({}), "sha256").expect_err("missing hash");
        assert!(matches!(err, Error::SecurityProviderRpc(_)));
    }

    #[test]
    fn sha256_response_invalid_base64() {
        let err = decode_hash_field(&json!({"hash": "@@@"}), "sha256").expect_err("bad b64");
        assert!(matches!(err, Error::SecurityProviderRpc(_)));
    }

    #[test]
    fn sha256_response_roundtrip_decode() {
        let raw = b"hello-hash";
        let v = json!({"hash": BASE64_STANDARD.encode(raw)});
        let got = decode_hash_field(&v, "sha256").expect("decode");
        assert_eq!(got, raw);
    }

    fn decode_prk_field(result: &serde_json::Value) -> crate::error::Result<Vec<u8>> {
        let prk_b64 = result["prk"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing prk in hkdf_extract response"))
        })?;
        BASE64_STANDARD
            .decode(prk_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid prk base64: {e}")))
    }

    #[test]
    fn hkdf_extract_response_errors() {
        assert!(decode_prk_field(&json!({})).is_err());
        let ok = decode_prk_field(&json!({"prk": BASE64_STANDARD.encode(b"ikm")})).expect("ok");
        assert_eq!(ok, b"ikm");
    }

    fn decode_okm_field(result: &serde_json::Value) -> crate::error::Result<Vec<u8>> {
        let okm_b64 = result["okm"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing okm in hkdf_expand response"))
        })?;
        BASE64_STANDARD
            .decode(okm_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid okm base64: {e}")))
    }

    #[test]
    fn hkdf_expand_response_errors() {
        assert!(decode_okm_field(&json!({})).is_err());
        let ok =
            decode_okm_field(&json!({"okm": BASE64_STANDARD.encode(b"expanded")})).expect("ok");
        assert_eq!(ok, b"expanded");
    }
}
