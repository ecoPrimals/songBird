//! Key exchange operations
//!
//! X25519 key generation and ECDH key derivation.

use super::core::BearDogClient;
use crate::error::{Error, Result};
use base64::prelude::*;
use serde_json::json;
use tracing::debug;

impl BearDogClient {
    /// Generate X25519 keypair
    ///
    /// Returns (public_key, private_key) as raw bytes.
    pub async fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        debug!("🔑 Generating x25519 keypair via BearDog");

        let result = self
            .call(
                "crypto.generate_keypair",
                json!({
                    "algorithm": "x25519"
                }),
            )
            .await?;

        let public_key = result["public_key"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing public_key".to_string()))?;
        let private_key = result["secret_key"] // BearDog returns "secret_key", not "private_key"
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing secret_key in BearDog response".to_string()))?;

        let public_key = BASE64_STANDARD
            .decode(public_key)
            .map_err(|e| Error::BearDogRpc(format!("Invalid public_key base64: {}", e)))?;
        let private_key = BASE64_STANDARD
            .decode(private_key)
            .map_err(|e| Error::BearDogRpc(format!("Invalid private_key base64: {}", e)))?;

        Ok((public_key, private_key))
    }

    /// Perform ECDH key exchange
    ///
    /// Derives a shared secret from our private key and their public key.
    pub async fn ecdh_derive(&self, private_key: &[u8], public_key: &[u8]) -> Result<Vec<u8>> {
        debug!("🔐 Performing ECDH via BearDog");

        // BearDog expects: "our_secret" and "their_public"
        let result = self
            .call(
                "crypto.ecdh_derive",
                json!({
                    "our_secret": BASE64_STANDARD.encode(private_key),
                    "their_public": BASE64_STANDARD.encode(public_key)
                }),
            )
            .await?;

        let shared_secret = result["shared_secret"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing shared_secret".to_string()))?;

        BASE64_STANDARD
            .decode(shared_secret)
            .map_err(|e| Error::BearDogRpc(format!("Invalid shared_secret base64: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require a running BearDog instance
    // Unit tests verify data formatting

    #[test]
    fn test_base64_encoding() {
        use base64::prelude::*;
        let data = vec![0u8; 32];
        let encoded = BASE64_STANDARD.encode(&data);
        assert_eq!(encoded, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    }
}

