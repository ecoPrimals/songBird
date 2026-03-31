// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Pure Rust JWT Implementation
//!
//! 100% Pure Rust JWT encoding/decoding using `RustCrypto` (hmac + sha2).
//! Zero C dependencies - full ecoBin compliance!
//!
//! Uses:
//! - `hmac` for HMAC-SHA256 (audited, Pure Rust)
//! - `sha2` for SHA-256 (audited, Pure Rust)
//! - `base64` for encoding (Pure Rust)
//! - `serde_json` for JSON (Pure Rust)

use anyhow::{Result, anyhow};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use songbird_crypto_provider::CryptoProvider;

type HmacSha256 = Hmac<Sha256>;

fn hmac_sha256_local(secret: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    let mut mac =
        HmacSha256::new_from_slice(secret).map_err(|e| anyhow!("Invalid secret key: {e}"))?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

pub async fn hmac_sha256_via_provider(
    key: &[u8],
    message: &[u8],
    provider: &CryptoProvider,
) -> Result<Vec<u8>> {
    let key_b64 = STANDARD.encode(key);
    let data_b64 = STANDARD.encode(message);
    let result = provider
        .call("crypto.hmac.sha256", json!({ "key": key_b64, "data": data_b64 }))
        .await
        .map_err(|e| anyhow!("{}", e))?;
    let mac_b64 = result
        .get("mac")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing mac in HMAC response"))?;
    STANDARD.decode(mac_b64).map_err(|e| anyhow!("Invalid mac: {e}"))
}

/// JWT Header
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtHeader {
    alg: String,
    typ: String,
}

impl Default for JwtHeader {
    fn default() -> Self {
        Self {
            alg: "HS256".to_string(),
            typ: "JWT".to_string(),
        }
    }
}

/// Encode JWT token (Pure Rust implementation)
///
/// Creates a JWT token with HMAC-SHA256 signature using Pure Rust crypto.
///
/// # Arguments
/// * `claims` - The claims to encode
/// * `secret` - Secret key for HMAC-SHA256
///
/// # Returns
/// JWT token string in format: `header.payload.signature`
/// # Errors
///
/// Returns an error if the operation fails.
pub fn encode<T: Serialize>(claims: &T, secret: &[u8]) -> Result<String> {
    let header = JwtHeader::default();
    let header_json =
        serde_json::to_string(&header).map_err(|e| anyhow!("Failed to serialize header: {e}"))?;
    let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());

    let payload_json =
        serde_json::to_string(claims).map_err(|e| anyhow!("Failed to serialize claims: {e}"))?;
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

    let signing_input = format!("{header_b64}.{payload_b64}");

    let signature = hmac_sha256_local(secret, signing_input.as_bytes())?;
    let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);

    Ok(format!("{signing_input}.{signature_b64}"))
}

pub async fn encode_with_crypto<T: Serialize>(
    claims: &T,
    secret: &[u8],
    crypto: Option<&CryptoProvider>,
) -> Result<String> {
    let header = JwtHeader::default();
    let header_json =
        serde_json::to_string(&header).map_err(|e| anyhow!("Failed to serialize header: {e}"))?;
    let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());

    let payload_json =
        serde_json::to_string(claims).map_err(|e| anyhow!("Failed to serialize claims: {e}"))?;
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json.as_bytes());

    let signing_input = format!("{header_b64}.{payload_b64}");

    let signature = if let Some(provider) = crypto {
        hmac_sha256_via_provider(secret, signing_input.as_bytes(), provider).await?
    } else {
        tracing::warn!(
            "JWT signing using local HMAC-SHA256; BearDog crypto provider not configured"
        );
        hmac_sha256_local(secret, signing_input.as_bytes())?
    };
    let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);

    Ok(format!("{signing_input}.{signature_b64}"))
}

/// Decode JWT token (Pure Rust implementation)
///
/// Verifies and decodes a JWT token using HMAC-SHA256 with Pure Rust crypto.
///
/// # Arguments
/// * `token` - JWT token string
/// * `secret` - Secret key for HMAC-SHA256 verification
///
/// # Returns
/// Decoded claims if token is valid
/// # Errors
///
/// Returns an error if the operation fails.
pub fn decode<T: for<'de> Deserialize<'de>>(token: &str, secret: &[u8]) -> Result<T> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(anyhow!("Invalid JWT format: expected 3 parts, got {}", parts.len()));
    }

    let header_b64 = parts[0];
    let payload_b64 = parts[1];
    let signature_b64 = parts[2];

    let signing_input = format!("{header_b64}.{payload_b64}");

    let expected_signature = hmac_sha256_local(secret, signing_input.as_bytes())?;
    let provided_signature = URL_SAFE_NO_PAD
        .decode(signature_b64)
        .map_err(|e| anyhow!("Invalid signature encoding: {e}"))?;

    if expected_signature.len() != provided_signature.len() {
        return Err(anyhow!("Invalid signature"));
    }

    let mut is_valid = true;
    for (a, b) in expected_signature.iter().zip(provided_signature.iter()) {
        is_valid &= a == b;
    }

    if !is_valid {
        return Err(anyhow!("Invalid signature"));
    }

    let payload_json = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|e| anyhow!("Invalid payload encoding: {e}"))?;
    let claims: T = serde_json::from_slice(&payload_json)
        .map_err(|e| anyhow!("Failed to deserialize claims: {e}"))?;

    Ok(claims)
}

pub async fn decode_with_crypto<T: for<'de> Deserialize<'de>>(
    token: &str,
    secret: &[u8],
    crypto: Option<&CryptoProvider>,
) -> Result<T> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(anyhow!("Invalid JWT format: expected 3 parts, got {}", parts.len()));
    }

    let header_b64 = parts[0];
    let payload_b64 = parts[1];
    let signature_b64 = parts[2];

    let signing_input = format!("{header_b64}.{payload_b64}");

    let expected_signature = if let Some(provider) = crypto {
        hmac_sha256_via_provider(secret, signing_input.as_bytes(), provider).await?
    } else {
        tracing::warn!(
            "JWT verification using local HMAC-SHA256; BearDog crypto provider not configured"
        );
        hmac_sha256_local(secret, signing_input.as_bytes())?
    };
    let provided_signature = URL_SAFE_NO_PAD
        .decode(signature_b64)
        .map_err(|e| anyhow!("Invalid signature encoding: {e}"))?;

    if expected_signature.len() != provided_signature.len() {
        return Err(anyhow!("Invalid signature"));
    }

    let mut is_valid = true;
    for (a, b) in expected_signature.iter().zip(provided_signature.iter()) {
        is_valid &= a == b;
    }

    if !is_valid {
        return Err(anyhow!("Invalid signature"));
    }

    let payload_json = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|e| anyhow!("Invalid payload encoding: {e}"))?;
    let claims: T = serde_json::from_slice(&payload_json)
        .map_err(|e| anyhow!("Failed to deserialize claims: {e}"))?;

    Ok(claims)
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use hmac::Mac;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestClaims {
        sub: String,
        iat: i64,
        exp: i64,
    }

    #[test]
    fn test_pure_rust_jwt_encode_decode() {
        let secret = b"test-secret-for-jwt";
        let claims = TestClaims {
            sub: "user-123".to_string(),
            iat: 1234567890,
            exp: 1234571490,
        };

        // Encode
        let token = encode(&claims, secret).unwrap();
        assert!(token.contains('.'));
        assert_eq!(token.matches('.').count(), 2); // header.payload.signature

        // Decode
        let decoded: TestClaims = decode(&token, secret).unwrap();
        assert_eq!(decoded, claims);
    }

    #[test]
    fn test_pure_rust_jwt_invalid_signature() {
        let secret = b"test-secret";
        let wrong_secret = b"wrong-secret";

        let claims = TestClaims {
            sub: "user-123".to_string(),
            iat: 1234567890,
            exp: 1234571490,
        };

        let token = encode(&claims, secret).unwrap();

        // Should fail with wrong secret
        let result: Result<TestClaims> = decode(&token, wrong_secret);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid signature"));
    }

    #[test]
    fn test_pure_rust_jwt_invalid_format() {
        let secret = b"test-secret";

        // Invalid format (missing parts)
        let result: Result<TestClaims> = decode("invalid", secret);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid JWT format"));

        // Invalid format (too many parts)
        let result: Result<TestClaims> = decode("a.b.c.d", secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_pure_rust_jwt_different_secrets() {
        let secret1 = b"secret1";
        let secret2 = b"secret2";

        let claims = TestClaims {
            sub: "user-123".to_string(),
            iat: 1234567890,
            exp: 1234571490,
        };

        let token1 = encode(&claims, secret1).unwrap();
        let token2 = encode(&claims, secret2).unwrap();

        // Tokens should be different
        assert_ne!(token1, token2);

        // Each should decode with correct secret
        let decoded1: TestClaims = decode(&token1, secret1).unwrap();
        let decoded2: TestClaims = decode(&token2, secret2).unwrap();
        assert_eq!(decoded1, claims);
        assert_eq!(decoded2, claims);

        // Each should fail with wrong secret
        assert!(decode::<TestClaims>(&token1, secret2).is_err());
        assert!(decode::<TestClaims>(&token2, secret1).is_err());
    }

    #[test]
    fn test_pure_rust_jwt_empty_secret() {
        let secret = b"";
        let claims = TestClaims {
            sub: "user-123".to_string(),
            iat: 1234567890,
            exp: 1234571490,
        };

        // Should still work (though not recommended in production)
        let token = encode(&claims, secret).unwrap();
        let decoded: TestClaims = decode(&token, secret).unwrap();
        assert_eq!(decoded, claims);
    }

    #[test]
    fn test_pure_rust_jwt_complex_claims() {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct ComplexClaims {
            sub: String,
            iat: i64,
            exp: i64,
            roles: Vec<String>,
            metadata: std::collections::HashMap<String, String>,
        }

        let secret = b"test-secret";
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("course_id".to_string(), "CSE-847".to_string());
        metadata.insert("semester".to_string(), "Fall2024".to_string());

        let claims = ComplexClaims {
            sub: "student-123".to_string(),
            iat: 1234567890,
            exp: 1234571490,
            roles: vec!["student".to_string(), "ta".to_string()],
            metadata,
        };

        let token = encode(&claims, secret).unwrap();
        let decoded: ComplexClaims = decode(&token, secret).unwrap();
        assert_eq!(decoded, claims);
    }

    #[test]
    fn decode_invalid_payload_json() {
        let secret = b"secret";
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(b"not-json");
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        let signing_input = format!("{header}.{payload}");
        mac.update(signing_input.as_bytes());
        let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        let token = format!("{signing_input}.{sig}");
        let r: Result<TestClaims> = decode(&token, secret);
        assert!(r.is_err());
        let msg = r.expect_err("err").to_string();
        assert!(msg.contains("deserialize") || msg.contains("Failed to deserialize"), "{msg}");
    }
}
