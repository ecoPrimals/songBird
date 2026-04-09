// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP (BearDog Secure Tunnel Protocol) session methods
//!
//! Wraps BearDog's BTSP JSON-RPC methods for handshake-as-a-service.
//! Consumer primals call these to delegate BTSP crypto to BearDog.
//!
//! ## Protocol
//!
//! See `BTSP_PROTOCOL_STANDARD.md` v1.0 for the full specification.
//! BearDog implements the crypto primitives; consumer primals (like Songbird)
//! call these methods during socket accept to authenticate incoming connections.

use super::core::SecurityRpcClient;
use crate::error::{Error, Result};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;

/// Result of `btsp.session.create` — BearDog generates server-side session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspSessionCreated {
    pub session_id: String,
    pub server_ephemeral_pub: Vec<u8>,
    pub handshake_key: Vec<u8>,
}

/// Result of `btsp.session.verify` — BearDog verifies the client's challenge response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspSessionVerified {
    pub verified: bool,
    pub session_key: Option<Vec<u8>>,
}

/// Result of `btsp.negotiate` — cipher suite negotiation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspNegotiation {
    pub cipher: String,
    pub allowed: bool,
}

/// BTSP cipher suites as defined in the standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BtspCipher {
    #[serde(rename = "chacha20_poly1305")]
    ChaCha20Poly1305,
    #[serde(rename = "hmac_plain")]
    HmacPlain,
    #[serde(rename = "null")]
    Null,
}

impl std::fmt::Display for BtspCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChaCha20Poly1305 => write!(f, "chacha20_poly1305"),
            Self::HmacPlain => write!(f, "hmac_plain"),
            Self::Null => write!(f, "null"),
        }
    }
}

impl SecurityRpcClient {
    /// Create a BTSP session (server-side).
    ///
    /// Called when a new connection arrives. BearDog generates the server's
    /// ephemeral keypair, derives the handshake key from the family seed,
    /// and returns session state.
    ///
    /// # Errors
    ///
    /// Returns an error if BearDog is unreachable or the session cannot be created.
    pub async fn btsp_session_create(
        &self,
        client_ephemeral_pub: &[u8],
        challenge: &[u8],
    ) -> Result<BtspSessionCreated> {
        debug!("BTSP: creating session via security provider");

        let result = self
            .call(
                "btsp.session.create",
                json!({
                    "family_seed_ref": "env:FAMILY_SEED",
                    "client_ephemeral_pub": BASE64_STANDARD.encode(client_ephemeral_pub),
                    "challenge": BASE64_STANDARD.encode(challenge),
                }),
            )
            .await?;

        let session_id = result["session_id"]
            .as_str()
            .ok_or_else(|| Error::SecurityProviderRpc("Missing session_id".to_string()))?
            .to_string();

        let server_ephemeral_pub = BASE64_STANDARD
            .decode(result["server_ephemeral_pub"].as_str().ok_or_else(|| {
                Error::SecurityProviderRpc("Missing server_ephemeral_pub".to_string())
            })?)
            .map_err(|e| {
                Error::SecurityProviderRpc(format!("Invalid server_ephemeral_pub base64: {e}"))
            })?;

        let handshake_key =
            BASE64_STANDARD
                .decode(result["handshake_key"].as_str().ok_or_else(|| {
                    Error::SecurityProviderRpc("Missing handshake_key".to_string())
                })?)
                .map_err(|e| {
                    Error::SecurityProviderRpc(format!("Invalid handshake_key base64: {e}"))
                })?;

        Ok(BtspSessionCreated {
            session_id,
            server_ephemeral_pub,
            handshake_key,
        })
    }

    /// Verify a client's BTSP challenge response.
    ///
    /// After the server sends `ServerHello` with a challenge, the client responds
    /// with an HMAC proving family membership. BearDog verifies this.
    ///
    /// # Errors
    ///
    /// Returns an error if BearDog is unreachable or the response is malformed.
    pub async fn btsp_session_verify(
        &self,
        session_id: &str,
        client_response: &[u8],
        client_ephemeral_pub: &[u8],
        server_ephemeral_pub: &[u8],
        challenge: &[u8],
    ) -> Result<BtspSessionVerified> {
        debug!("BTSP: verifying challenge response for session {session_id}");

        let result = self
            .call(
                "btsp.session.verify",
                json!({
                    "session_id": session_id,
                    "client_response": BASE64_STANDARD.encode(client_response),
                    "client_ephemeral_pub": BASE64_STANDARD.encode(client_ephemeral_pub),
                    "server_ephemeral_pub": BASE64_STANDARD.encode(server_ephemeral_pub),
                    "challenge": BASE64_STANDARD.encode(challenge),
                }),
            )
            .await?;

        let verified = result["verified"].as_bool().ok_or_else(|| {
            Error::SecurityProviderRpc(
                "Missing 'verified' field in btsp.session.verify response".to_string(),
            )
        })?;

        let session_key = if verified {
            let key_b64 = result["session_key"].as_str().ok_or_else(|| {
                Error::SecurityProviderRpc("Missing session_key on verified session".to_string())
            })?;
            Some(BASE64_STANDARD.decode(key_b64).map_err(|e| {
                Error::SecurityProviderRpc(format!("Invalid session_key base64: {e}"))
            })?)
        } else {
            None
        };

        Ok(BtspSessionVerified {
            verified,
            session_key,
        })
    }

    /// Negotiate cipher suite for an authenticated BTSP session.
    ///
    /// After handshake verification succeeds, both parties negotiate which
    /// cipher to use. The minimum cipher is enforced per `BondingPolicy`.
    ///
    /// # Errors
    ///
    /// Returns an error if BearDog is unreachable or negotiation fails.
    pub async fn btsp_negotiate(
        &self,
        session_id: &str,
        preferred_cipher: BtspCipher,
        bond_type: &str,
    ) -> Result<BtspNegotiation> {
        debug!("BTSP: negotiating cipher for session {session_id}");

        let result = self
            .call(
                "btsp.negotiate",
                json!({
                    "session_id": session_id,
                    "preferred_cipher": preferred_cipher.to_string(),
                    "bond_type": bond_type,
                }),
            )
            .await?;

        let cipher = result["cipher"]
            .as_str()
            .ok_or_else(|| {
                Error::SecurityProviderRpc("Missing cipher in btsp.negotiate".to_string())
            })?
            .to_string();

        let allowed = result["allowed"].as_bool().ok_or_else(|| {
            Error::SecurityProviderRpc("Missing 'allowed' in btsp.negotiate".to_string())
        })?;

        Ok(BtspNegotiation {
            cipher,
            allowed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btsp_cipher_display() {
        assert_eq!(BtspCipher::ChaCha20Poly1305.to_string(), "chacha20_poly1305");
        assert_eq!(BtspCipher::HmacPlain.to_string(), "hmac_plain");
        assert_eq!(BtspCipher::Null.to_string(), "null");
    }

    #[test]
    fn btsp_cipher_serde_roundtrip() {
        let c = BtspCipher::ChaCha20Poly1305;
        let json = serde_json::to_string(&c).expect("serialize");
        assert_eq!(json, "\"chacha20_poly1305\"");
        let back: BtspCipher = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, c);
    }

    #[test]
    fn btsp_session_created_deserialize() {
        let json = serde_json::json!({
            "session_id": "abc123",
            "server_ephemeral_pub": vec![0u8; 32],
            "handshake_key": vec![1u8; 32],
        });
        let s: BtspSessionCreated = serde_json::from_value(json).expect("deser");
        assert_eq!(s.session_id, "abc123");
        assert_eq!(s.server_ephemeral_pub.len(), 32);
        assert_eq!(s.handshake_key.len(), 32);
    }

    #[test]
    fn btsp_session_verified_success() {
        let json = serde_json::json!({
            "verified": true,
            "session_key": vec![2u8; 32],
        });
        let v: BtspSessionVerified = serde_json::from_value(json).expect("deser");
        assert!(v.verified);
        assert!(v.session_key.is_some());
    }

    #[test]
    fn btsp_session_verified_failure() {
        let json = serde_json::json!({
            "verified": false,
        });
        let v: BtspSessionVerified = serde_json::from_value(json).expect("deser");
        assert!(!v.verified);
        assert!(v.session_key.is_none());
    }

    #[test]
    fn btsp_negotiation_deserialize() {
        let json = serde_json::json!({
            "cipher": "chacha20_poly1305",
            "allowed": true,
        });
        let n: BtspNegotiation = serde_json::from_value(json).expect("deser");
        assert_eq!(n.cipher, "chacha20_poly1305");
        assert!(n.allowed);
    }
}
