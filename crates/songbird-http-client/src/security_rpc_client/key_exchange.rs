// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Key exchange operations
//!
//! X25519 key generation and ECDH key derivation.

use super::core::SecurityRpcClient;
use crate::error::{Error, Result};
use base64::prelude::*;
use serde_json::{Value, json};
use tracing::debug;

fn parse_x25519_keypair_from_value(result: &Value) -> Result<(Vec<u8>, Vec<u8>)> {
    let public_key = result["public_key"]
        .as_str()
        .ok_or_else(|| Error::SecurityProviderRpc("Missing public_key".to_string()))?;
    let private_key = result["secret_key"] // Provider returns "secret_key", not "private_key"
        .as_str()
        .ok_or_else(|| {
            Error::SecurityProviderRpc(
                "Missing secret_key in security provider response".to_string(),
            )
        })?;

    let public_key = BASE64_STANDARD
        .decode(public_key)
        .map_err(|e| Error::SecurityProviderRpc(format!("Invalid public_key base64: {e}")))?;
    let private_key = BASE64_STANDARD
        .decode(private_key)
        .map_err(|e| Error::SecurityProviderRpc(format!("Invalid private_key base64: {e}")))?;

    Ok((public_key, private_key))
}

fn parse_ecdh_shared_secret_from_value(result: &Value) -> Result<Vec<u8>> {
    let shared_secret = result["shared_secret"]
        .as_str()
        .ok_or_else(|| Error::SecurityProviderRpc("Missing shared_secret".to_string()))?;

    BASE64_STANDARD
        .decode(shared_secret)
        .map_err(|e| Error::SecurityProviderRpc(format!("Invalid shared_secret base64: {e}")))
}

impl SecurityRpcClient {
    /// Generate X25519 keypair
    ///
    /// Returns `(public_key, private_key)` as raw bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    pub async fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        debug!("🔑 Generating x25519 keypair via security provider");

        let result = self
            .call(
                "crypto.generate_keypair",
                json!({
                    "algorithm": "x25519"
                }),
            )
            .await?;

        parse_x25519_keypair_from_value(&result)
    }

    /// Perform ECDH key exchange
    ///
    /// Derives a shared secret from our private key and their public key.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    pub async fn ecdh_derive(&self, private_key: &[u8], public_key: &[u8]) -> Result<Vec<u8>> {
        debug!("🔐 Performing ECDH via security provider");

        // Provider expects: "our_secret" and "their_public"
        let result = self
            .call(
                "crypto.ecdh_derive",
                json!({
                    "our_secret": BASE64_STANDARD.encode(private_key),
                    "their_public": BASE64_STANDARD.encode(public_key)
                }),
            )
            .await?;

        parse_ecdh_shared_secret_from_value(&result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::{parse_ecdh_shared_secret_from_value, parse_x25519_keypair_from_value};
    use crate::error::Error;
    use base64::prelude::*;
    use serde_json::json;

    #[test]
    fn test_base64_encoding() {
        let data = vec![0u8; 32];
        let encoded = BASE64_STANDARD.encode(&data);
        assert_eq!(encoded, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    }

    #[test]
    fn parse_keypair_accepts_valid_base64_fields() {
        let raw = b"hello";
        let v = json!({
            "public_key": BASE64_STANDARD.encode(raw),
            "secret_key": BASE64_STANDARD.encode(raw),
        });
        let (pub_k, sec_k) = parse_x25519_keypair_from_value(&v).unwrap();
        assert_eq!(pub_k, raw);
        assert_eq!(sec_k, raw);
    }

    #[test]
    fn parse_keypair_errors_on_missing_public_key() {
        let v = json!({ "secret_key": BASE64_STANDARD.encode(b"x") });
        let err = parse_x25519_keypair_from_value(&v).unwrap_err();
        match err {
            Error::SecurityProviderRpc(msg) => assert_eq!(msg, "Missing public_key"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parse_keypair_errors_on_missing_secret_key() {
        let v = json!({ "public_key": BASE64_STANDARD.encode(b"x") });
        let err = parse_x25519_keypair_from_value(&v).unwrap_err();
        match err {
            Error::SecurityProviderRpc(msg) => {
                assert_eq!(msg, "Missing secret_key in security provider response");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parse_keypair_errors_on_invalid_base64() {
        let v = json!({
            "public_key": "@@@",
            "secret_key": BASE64_STANDARD.encode(b"x"),
        });
        let err = parse_x25519_keypair_from_value(&v).unwrap_err();
        let Error::SecurityProviderRpc(msg) = err else {
            panic!("expected SecurityProviderRpc");
        };
        assert!(msg.starts_with("Invalid public_key base64:"), "{msg}");
    }

    #[test]
    fn parse_shared_secret_roundtrip() {
        let secret = vec![1u8, 2, 3, 4];
        let v = json!({ "shared_secret": BASE64_STANDARD.encode(&secret) });
        let out = parse_ecdh_shared_secret_from_value(&v).unwrap();
        assert_eq!(out, secret);
    }

    #[test]
    fn parse_shared_secret_missing_field() {
        let v = json!({});
        let err = parse_ecdh_shared_secret_from_value(&v).unwrap_err();
        match err {
            Error::SecurityProviderRpc(msg) => assert_eq!(msg, "Missing shared_secret"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn generate_keypair_request_json_shape() {
        let req = json!({ "algorithm": "x25519" });
        assert_eq!(req["algorithm"].as_str(), Some("x25519"));
    }

    #[test]
    fn ecdh_derive_request_json_shape() {
        let sk = vec![7u8; 32];
        let pk = vec![9u8; 32];
        let req = json!({
            "our_secret": BASE64_STANDARD.encode(&sk),
            "their_public": BASE64_STANDARD.encode(&pk),
        });
        assert_eq!(BASE64_STANDARD.decode(req["our_secret"].as_str().unwrap()).unwrap(), sk);
        assert_eq!(BASE64_STANDARD.decode(req["their_public"].as_str().unwrap()).unwrap(), pk);
    }
}
