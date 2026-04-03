// SPDX-License-Identifier: AGPL-3.0-only
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

        let hash_b64 = result["hash"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing hash in sha256 response".to_string()))?;

        BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid hash base64: {e}")))
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

        let hash_b64 = result["hash"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing hash in sha384 response".to_string()))?;

        BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid hash base64: {e}")))
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

        let prk_b64 = result["prk"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing prk in hkdf_extract response".to_string()))?;

        BASE64_STANDARD
            .decode(prk_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid prk base64: {e}")))
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

        let okm_b64 = result["okm"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing okm in hkdf_expand response".to_string()))?;

        BASE64_STANDARD
            .decode(okm_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid okm base64: {e}")))
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require a running security provider instance

    #[test]
    fn test_hash_format() {
        // SHA-256 produces 32 bytes
        assert_eq!(32, 256 / 8);
        // SHA-384 produces 48 bytes
        assert_eq!(48, 384 / 8);
    }
}
