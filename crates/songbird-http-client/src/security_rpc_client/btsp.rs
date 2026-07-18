// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP (`BearDog` Secure Tunnel Protocol) session methods
//!
//! Wraps `BearDog`'s BTSP JSON-RPC methods for handshake-as-a-service.
//! Consumer primals call these to delegate BTSP crypto to `BearDog`.
//!
//! ## Protocol
//!
//! See `BTSP_PROTOCOL_STANDARD.md` v1.0 for the full specification.
//! `BearDog` implements the crypto primitives; consumer primals (like Songbird)
//! call these methods during socket accept to authenticate incoming connections.

use super::core::SecurityRpcClient;
use crate::error::{Error, Result};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;

/// Result of `btsp.session.create` — `BearDog` generates server-side session state.
///
/// Aligned with `BearDog`'s `SessionCreateResponse`: returns an opaque
/// `session_token` (used to reference the session in subsequent calls),
/// the server's ephemeral public key, and a random challenge for the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspSessionCreated {
    pub session_token: String,
    pub server_ephemeral_pub: Vec<u8>,
    pub challenge: Vec<u8>,
}

/// Result of `btsp.session.verify` — `BearDog` verifies the client's challenge response.
///
/// Aligned with `BearDog`'s `SessionVerifyResponse`: on success, returns
/// the promoted `session_id` and negotiated `cipher`. Session keys are
/// obtained separately via `btsp.server.export_keys` when encryption is needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspSessionVerified {
    pub verified: bool,
    pub session_id: Option<String>,
    pub cipher: Option<String>,
}

/// Result of `btsp.session.negotiate` — cipher suite negotiation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspNegotiation {
    pub cipher: String,
    pub accepted: bool,
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
    /// Called when a new connection arrives. `BearDog` generates the server's
    /// ephemeral keypair, derives the handshake key from the family seed,
    /// and returns session state including a random challenge for the client.
    ///
    /// `family_seed` must be base64-encoded. `BearDog`'s `btsp.session.create`
    /// base64-decodes this parameter internally. The caller should read the
    /// raw `FAMILY_SEED` env var, trim whitespace, and base64-encode the
    /// resulting bytes before passing them here.
    ///
    /// # Errors
    ///
    /// Returns an error if `BearDog` is unreachable or the session cannot be created.
    pub async fn btsp_session_create(&self, family_seed: &str) -> Result<BtspSessionCreated> {
        debug!("BTSP: creating session via security provider");

        let result = self
            .call(
                "btsp.session.create",
                json!({
                    "family_seed": family_seed,
                }),
            )
            .await?;

        let session_token = result["session_token"]
            .as_str()
            .ok_or_else(|| Error::SecurityProviderRpc(String::from("Missing session_token")))?
            .to_string();

        let server_ephemeral_pub = BASE64_STANDARD
            .decode(result["server_ephemeral_pub"].as_str().ok_or_else(|| {
                Error::SecurityProviderRpc(String::from("Missing server_ephemeral_pub"))
            })?)
            .map_err(|e| {
                Error::SecurityProviderRpc(format!("Invalid server_ephemeral_pub base64: {e}"))
            })?;

        let challenge = BASE64_STANDARD
            .decode(
                result["challenge"]
                    .as_str()
                    .ok_or_else(|| Error::SecurityProviderRpc(String::from("Missing challenge")))?,
            )
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid challenge base64: {e}")))?;

        Ok(BtspSessionCreated {
            session_token,
            server_ephemeral_pub,
            challenge,
        })
    }

    /// Verify a client's BTSP challenge response.
    ///
    /// After the server sends `ServerHello` with a challenge, the client responds
    /// with an HMAC proving family membership. `BearDog` verifies the HMAC using
    /// the session state stored under `session_token`.
    ///
    /// On success, `BearDog` promotes the session and returns a `session_id` and
    /// the negotiated `cipher`. Session keys are obtained separately via
    /// `btsp.server.export_keys` when stream encryption is needed.
    ///
    /// # Errors
    ///
    /// Returns an error if `BearDog` is unreachable or the response is malformed.
    pub async fn btsp_session_verify(
        &self,
        session_token: &str,
        client_ephemeral_pub: &[u8],
        response: &[u8],
        preferred_cipher: &str,
    ) -> Result<BtspSessionVerified> {
        debug!("BTSP: verifying challenge response for session_token {session_token}");

        let result = self
            .call(
                "btsp.session.verify",
                json!({
                    "session_token": session_token,
                    "client_ephemeral_pub": BASE64_STANDARD.encode(client_ephemeral_pub),
                    "response": BASE64_STANDARD.encode(response),
                    "preferred_cipher": preferred_cipher,
                }),
            )
            .await?;

        let verified = result["verified"].as_bool().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from(
                "Missing 'verified' field in btsp.session.verify response",
            ))
        })?;

        let session_id = result["session_id"].as_str().map(ToString::to_string);
        let cipher = result["cipher"].as_str().map(ToString::to_string);

        Ok(BtspSessionVerified {
            verified,
            session_id,
            cipher,
        })
    }

    /// Export the handshake key for a verified BTSP session.
    ///
    /// After Phase 1 handshake completes, the `handshake_key` (derived from
    /// the X25519 shared secret during `btsp.session.create`/`verify`) is
    /// held by `BearDog`. This method retrieves it so Songbird can derive
    /// Phase 3 session keys locally via HKDF.
    ///
    /// # Errors
    ///
    /// Returns an error if `BearDog` is unreachable, the session is unknown,
    /// or the response is malformed.
    pub async fn btsp_export_keys(&self, session_id: &str) -> Result<[u8; 32]> {
        debug!("BTSP: exporting handshake key for session {session_id}");

        let result = self
            .call(
                "btsp.server.export_keys",
                json!({
                    "session_id": session_id,
                }),
            )
            .await?;

        let handshake_key_b64 = result["handshake_key"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from(
                "Missing 'handshake_key' in btsp.server.export_keys response",
            ))
        })?;

        let raw = BASE64_STANDARD.decode(handshake_key_b64).map_err(|e| {
            Error::SecurityProviderRpc(format!("Invalid handshake_key base64: {e}"))
        })?;

        let key: [u8; 32] = raw.try_into().map_err(|v: Vec<u8>| {
            Error::SecurityProviderRpc(format!(
                "handshake_key wrong length: expected 32, got {}",
                v.len()
            ))
        })?;

        Ok(key)
    }

    /// Negotiate cipher suite for an authenticated BTSP session.
    ///
    /// After handshake verification succeeds, both parties negotiate which
    /// cipher to use. `BearDog`'s verify already includes cipher negotiation,
    /// so this is typically only needed for re-negotiation.
    ///
    /// # Errors
    ///
    /// Returns an error if `BearDog` is unreachable or negotiation fails.
    pub async fn btsp_negotiate(
        &self,
        session_token: &str,
        cipher: BtspCipher,
    ) -> Result<BtspNegotiation> {
        debug!("BTSP: negotiating cipher for session_token {session_token}");

        let result = self
            .call(
                "btsp.session.negotiate",
                json!({
                    "session_token": session_token,
                    "cipher": cipher.to_string(),
                }),
            )
            .await?;

        let negotiated_cipher = result["cipher"]
            .as_str()
            .ok_or_else(|| {
                Error::SecurityProviderRpc(String::from("Missing cipher in btsp.session.negotiate"))
            })?
            .to_string();

        let accepted = result["accepted"].as_bool().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing 'accepted' in btsp.session.negotiate"))
        })?;

        Ok(BtspNegotiation {
            cipher: negotiated_cipher,
            accepted,
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
            "session_token": "tok-abc123",
            "server_ephemeral_pub": vec![0u8; 32],
            "challenge": vec![1u8; 32],
        });
        let s: BtspSessionCreated = serde_json::from_value(json).expect("deser");
        assert_eq!(s.session_token, "tok-abc123");
        assert_eq!(s.server_ephemeral_pub.len(), 32);
        assert_eq!(s.challenge.len(), 32);
    }

    #[test]
    fn btsp_session_verified_success() {
        let json = serde_json::json!({
            "verified": true,
            "session_id": "sess-001",
            "cipher": "chacha20_poly1305",
        });
        let v: BtspSessionVerified = serde_json::from_value(json).expect("deser");
        assert!(v.verified);
        assert_eq!(v.session_id.as_deref(), Some("sess-001"));
        assert_eq!(v.cipher.as_deref(), Some("chacha20_poly1305"));
    }

    #[test]
    fn btsp_session_verified_failure() {
        let json = serde_json::json!({
            "verified": false,
        });
        let v: BtspSessionVerified = serde_json::from_value(json).expect("deser");
        assert!(!v.verified);
        assert!(v.session_id.is_none());
        assert!(v.cipher.is_none());
    }

    #[test]
    fn btsp_negotiation_deserialize() {
        let json = serde_json::json!({
            "cipher": "chacha20_poly1305",
            "accepted": true,
        });
        let n: BtspNegotiation = serde_json::from_value(json).expect("deser");
        assert_eq!(n.cipher, "chacha20_poly1305");
        assert!(n.accepted);
    }
}
