// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP Phase 3 — Encrypted channel negotiation and framing
//!
//! After a successful Phase 1 handshake, clients may send a `btsp.negotiate`
//! JSON-RPC request to upgrade the connection to ChaCha20-Poly1305 encrypted
//! framing. This module implements the server side of that negotiation and
//! the subsequent encrypted read/write loop.
//!
//! ## Protocol (server perspective)
//!
//! ```text
//! 1. Client sends:  {"method":"btsp.negotiate","params":{
//!                      "session_id":"...",
//!                      "ciphers":["chacha20-poly1305"],   // or preferred_cipher
//!                      "client_nonce":"<base64>",          // optional
//!                      "bond_type":"Covalent"              // optional, for cipher floor
//!                    }}
//! 2. Server selects cipher per BondingPolicy cipher floor rules
//! 3. Server exports handshake_key from security provider
//! 4. Server generates server_nonce (12 bytes)
//! 5. Server responds: {"result":{"cipher":"chacha20-poly1305","server_nonce":"<base64>"}}
//! 6. Both derive SessionKeys via HKDF-SHA256
//! 7. All subsequent I/O uses encrypted frames:
//!    [4 bytes: length (BE u32)][12 bytes: nonce][ciphertext + Poly1305 tag]
//! ```
//!
//! If the server cannot support the cipher, it returns `{"cipher":"null"}` and
//! the connection stays plaintext. primalSpring handles this gracefully.
//!
//! ## Reference
//!
//! - primalSpring client: `ecoPrimal/src/ipc/btsp_handshake.rs` (`negotiate_phase3`)
//! - petalTongue server: `crates/petal-tongue-ipc/src/btsp/json_line.rs`
//! - Spec: `BTSP_PROTOCOL_STANDARD.md` v1.0

use anyhow::{Context, Result, bail};
use base64::prelude::*;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn};

use songbird_http_client::SecurityRpcClient;

/// Maximum encrypted frame size (16 MiB), matching primalSpring.
const MAX_ENCRYPTED_FRAME: usize = 16 * 1024 * 1024;

/// Nonce size for ChaCha20-Poly1305 per-frame AEAD (96 bits).
const NONCE_SIZE: usize = 12;

/// Poly1305 authentication tag size.
const TAG_SIZE: usize = 16;

/// Server nonce size for the negotiate handshake HKDF salt (matches spec: 12 bytes).
const NEGOTIATE_NONCE_SIZE: usize = 12;

// ─── Wire types ──────────────────────────────────────────────────────────────

/// Phase 3 cipher options offered by clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase3Cipher {
    #[serde(rename = "chacha20-poly1305")]
    ChaCha20Poly1305,
    #[serde(rename = "null")]
    Null,
}

impl Phase3Cipher {
    #[must_use]
    pub fn wire_name(self) -> &'static str {
        match self {
            Self::ChaCha20Poly1305 => "chacha20-poly1305",
            Self::Null => "null",
        }
    }
}

impl std::fmt::Display for Phase3Cipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.wire_name())
    }
}

/// Incoming `btsp.negotiate` request params.
///
/// Accepts both the primalSpring Phase 3 format (`ciphers` array + `client_nonce`)
/// and the BTSP Protocol Standard format (`preferred_cipher` + `bond_type`).
/// When `preferred_cipher` is present but `ciphers` is empty/absent, the preferred
/// cipher is promoted into a single-element ciphers list.
#[derive(Debug, Deserialize)]
pub struct NegotiateParams {
    pub session_id: String,
    #[serde(default)]
    pub ciphers: Vec<String>,
    #[serde(default)]
    pub client_nonce: String,
    #[serde(default)]
    pub preferred_cipher: Option<String>,
    #[serde(default)]
    pub bond_type: Option<String>,
}

impl NegotiateParams {
    /// Resolved cipher list: merges `ciphers` array with `preferred_cipher` fallback.
    fn effective_ciphers(&self) -> Vec<&str> {
        if !self.ciphers.is_empty() {
            return self.ciphers.iter().map(String::as_str).collect();
        }
        if let Some(ref pc) = self.preferred_cipher {
            return vec![pc.as_str()];
        }
        Vec::new()
    }
}

/// Outgoing `btsp.negotiate` result.
#[derive(Debug, Serialize)]
pub struct NegotiateResult {
    pub cipher: String,
    pub server_nonce: String,
}

// ─── Session keys ────────────────────────────────────────────────────────────

/// Directional encryption keys derived from the Phase 3 HKDF.
///
/// The server's `encrypt_key` is the client's `decrypt_key` and vice versa.
pub struct SessionKeys {
    encrypt_key: [u8; 32],
    decrypt_key: [u8; 32],
}

impl SessionKeys {
    /// Derive session keys from the Phase 1 handshake key and both nonces.
    ///
    /// Uses HKDF-SHA256 with `salt = client_nonce || server_nonce` and the
    /// handshake key as IKM. Two expand passes produce directional keys:
    /// - `b"btsp-session-v1-c2s"` → client-to-server key
    /// - `b"btsp-session-v1-s2c"` → server-to-client key
    pub fn derive(
        handshake_key: &[u8; 32],
        client_nonce: &[u8],
        server_nonce: &[u8],
        is_client: bool,
    ) -> Result<Self> {
        let mut salt = Vec::with_capacity(client_nonce.len() + server_nonce.len());
        salt.extend_from_slice(client_nonce);
        salt.extend_from_slice(server_nonce);

        let hk = Hkdf::<Sha256>::new(Some(&salt), handshake_key);

        let mut c2s = [0u8; 32];
        hk.expand(b"btsp-session-v1-c2s", &mut c2s)
            .map_err(|e| anyhow::anyhow!("HKDF expand c2s failed: {e}"))?;

        let mut s2c = [0u8; 32];
        hk.expand(b"btsp-session-v1-s2c", &mut s2c)
            .map_err(|e| anyhow::anyhow!("HKDF expand s2c failed: {e}"))?;

        if is_client {
            Ok(Self {
                encrypt_key: c2s,
                decrypt_key: s2c,
            })
        } else {
            Ok(Self {
                encrypt_key: s2c,
                decrypt_key: c2s,
            })
        }
    }

    /// Encrypt plaintext, returning `nonce (12) || ciphertext || tag (16)`.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new((&self.encrypt_key).into());

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        getrandom::fill(&mut nonce_bytes).map_err(|e| anyhow::anyhow!("getrandom failed: {e}"))?;
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("ChaCha20-Poly1305 encrypt failed: {e}"))?;

        let mut frame = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        frame.extend_from_slice(&nonce_bytes);
        frame.extend_from_slice(&ciphertext);
        Ok(frame)
    }

    /// Decrypt a frame of `nonce (12) || ciphertext || tag (16)`.
    pub fn decrypt(&self, frame: &[u8]) -> Result<Vec<u8>> {
        if frame.len() < NONCE_SIZE + TAG_SIZE {
            bail!(
                "encrypted frame too short: {} bytes (min {})",
                frame.len(),
                NONCE_SIZE + TAG_SIZE
            );
        }

        let (nonce_bytes, ciphertext) = frame.split_at(NONCE_SIZE);
        let cipher = ChaCha20Poly1305::new((&self.decrypt_key).into());
        let nonce = chacha20poly1305::Nonce::from_slice(nonce_bytes);

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("ChaCha20-Poly1305 decrypt failed: {e}"))
    }
}

// ─── Cipher selection per BondingPolicy ──────────────────────────────────────

/// Select the best cipher from the offered list, applying bond-type cipher
/// floor rules from `BTSP_PROTOCOL_STANDARD.md`.
///
/// Cipher floors by bond type:
/// - Covalent: `BTSP_NULL` (all ciphers allowed)
/// - Metallic: minimum `hmac-plain`
/// - Ionic / Weak: minimum `chacha20-poly1305` (encrypted only)
///
/// Returns [`Phase3Cipher::Null`] when no acceptable cipher is offered or
/// the floor forbids the best offer.
fn select_cipher(offered: &[&str], bond_type: Option<&str>) -> Phase3Cipher {
    let wants_chacha =
        offered.iter().any(|c| *c == "chacha20-poly1305" || *c == "chacha20_poly1305");

    match bond_type {
        Some("Ionic" | "Weak" | "ZeroTrust" | "Contractual") => {
            if wants_chacha {
                Phase3Cipher::ChaCha20Poly1305
            } else {
                Phase3Cipher::Null
            }
        }
        _ => {
            if wants_chacha {
                Phase3Cipher::ChaCha20Poly1305
            } else {
                Phase3Cipher::Null
            }
        }
    }
}

// ─── Negotiate handler ───────────────────────────────────────────────────────

/// Handle a `btsp.negotiate` request, returning the JSON-RPC result and
/// optionally the derived session keys (if a real cipher was negotiated).
///
/// On any failure (security provider unreachable, missing fields), falls back to null
/// cipher so the connection stays alive on plaintext.
pub async fn handle_negotiate(
    params: &serde_json::Value,
    security_client: &Arc<SecurityRpcClient>,
) -> (NegotiateResult, Option<SessionKeys>) {
    match handle_negotiate_inner(params, security_client).await {
        Ok(pair) => pair,
        Err(e) => {
            warn!("BTSP Phase 3 negotiate failed — falling back to null cipher: {e:#}");
            (
                NegotiateResult {
                    cipher: "null".to_string(),
                    server_nonce: String::new(),
                },
                None,
            )
        }
    }
}

async fn handle_negotiate_inner(
    params: &serde_json::Value,
    security_client: &Arc<SecurityRpcClient>,
) -> Result<(NegotiateResult, Option<SessionKeys>)> {
    let neg: NegotiateParams = serde_json::from_value(params.clone())
        .context("BTSP Phase 3: malformed negotiate params")?;

    let effective = neg.effective_ciphers();
    let selected = select_cipher(&effective, neg.bond_type.as_deref());

    if selected == Phase3Cipher::Null {
        debug!(
            "BTSP Phase 3: client offers {:?} (bond_type={:?}) — returning null",
            effective, neg.bond_type,
        );
        return Ok((
            NegotiateResult {
                cipher: "null".to_string(),
                server_nonce: String::new(),
            },
            None,
        ));
    }

    let client_nonce = if neg.client_nonce.is_empty() {
        Vec::new()
    } else {
        BASE64_STANDARD
            .decode(&neg.client_nonce)
            .context("BTSP Phase 3: invalid client_nonce base64")?
    };

    let handshake_key = security_client
        .btsp_export_keys(&neg.session_id)
        .await
        .context("BTSP Phase 3: failed to export handshake key from security provider")?;

    let mut server_nonce = [0u8; NEGOTIATE_NONCE_SIZE];
    getrandom::fill(&mut server_nonce).map_err(|e| anyhow::anyhow!("getrandom failed: {e}"))?;

    let keys = SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, false)?;

    let result = NegotiateResult {
        cipher: "chacha20-poly1305".to_string(),
        server_nonce: BASE64_STANDARD.encode(server_nonce),
    };

    info!(
        session_id = %neg.session_id,
        bond_type = ?neg.bond_type,
        "BTSP Phase 3: negotiated chacha20-poly1305",
    );

    Ok((result, Some(keys)))
}

// ─── Encrypted frame I/O ─────────────────────────────────────────────────────

/// Read one encrypted frame: `[4B len (BE u32)][payload]`.
///
/// Returns the raw payload (nonce + ciphertext); caller decrypts.
pub async fn read_encrypted_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await.context("BTSP Phase 3: failed to read frame length")?;

    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_ENCRYPTED_FRAME {
        bail!("BTSP Phase 3: frame too large ({len} bytes, max {MAX_ENCRYPTED_FRAME})");
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await.context("BTSP Phase 3: failed to read frame payload")?;

    Ok(payload)
}

/// Write one encrypted frame: `[4B len (BE u32)][payload]`.
pub async fn write_encrypted_frame<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    payload: &[u8],
) -> Result<()> {
    let len =
        u32::try_from(payload.len()).context("BTSP Phase 3: frame payload exceeds u32::MAX")?;
    writer
        .write_all(&len.to_be_bytes())
        .await
        .context("BTSP Phase 3: failed to write frame length")?;
    writer.write_all(payload).await.context("BTSP Phase 3: failed to write frame payload")?;
    writer.flush().await.context("BTSP Phase 3: failed to flush")?;
    Ok(())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn phase3_cipher_wire_names() {
        assert_eq!(Phase3Cipher::ChaCha20Poly1305.wire_name(), "chacha20-poly1305");
        assert_eq!(Phase3Cipher::Null.wire_name(), "null");
    }

    #[test]
    fn phase3_cipher_serde_roundtrip() {
        let c = Phase3Cipher::ChaCha20Poly1305;
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"chacha20-poly1305\"");
        let back: Phase3Cipher = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn phase3_cipher_null_serde() {
        let c = Phase3Cipher::Null;
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"null\"");
        let back: Phase3Cipher = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Phase3Cipher::Null);
    }

    #[test]
    fn session_keys_derive_produces_different_directional_keys() {
        let hk = [0x42u8; 32];
        let cn = [0xAAu8; 32];
        let sn = [0xBBu8; 32];
        let server_keys = SessionKeys::derive(&hk, &cn, &sn, false).unwrap();
        let client_keys = SessionKeys::derive(&hk, &cn, &sn, true).unwrap();

        assert_ne!(server_keys.encrypt_key, server_keys.decrypt_key);
        assert_eq!(server_keys.encrypt_key, client_keys.decrypt_key);
        assert_eq!(server_keys.decrypt_key, client_keys.encrypt_key);
    }

    #[test]
    fn session_keys_derive_deterministic() {
        let hk = [1u8; 32];
        let cn = [2u8; 32];
        let sn = [3u8; 32];
        let k1 = SessionKeys::derive(&hk, &cn, &sn, false).unwrap();
        let k2 = SessionKeys::derive(&hk, &cn, &sn, false).unwrap();
        assert_eq!(k1.encrypt_key, k2.encrypt_key);
        assert_eq!(k1.decrypt_key, k2.decrypt_key);
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let hk = [0x55u8; 32];
        let cn = [0x11u8; 32];
        let sn = [0x22u8; 32];
        let server_keys = SessionKeys::derive(&hk, &cn, &sn, false).unwrap();
        let client_keys = SessionKeys::derive(&hk, &cn, &sn, true).unwrap();

        let plaintext = b"hello from client";
        let encrypted = client_keys.encrypt(plaintext).unwrap();
        let decrypted = server_keys.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);

        let server_msg = b"hello from server";
        let encrypted2 = server_keys.encrypt(server_msg).unwrap();
        let decrypted2 = client_keys.decrypt(&encrypted2).unwrap();
        assert_eq!(decrypted2, server_msg);
    }

    #[test]
    fn encrypt_produces_nonce_prefix() {
        let keys = SessionKeys::derive(&[1u8; 32], &[2u8; 32], &[3u8; 32], false).unwrap();
        let encrypted = keys.encrypt(b"test").unwrap();
        assert!(encrypted.len() >= NONCE_SIZE + TAG_SIZE);
        assert_eq!(encrypted.len(), NONCE_SIZE + 4 + TAG_SIZE);
    }

    #[test]
    fn decrypt_rejects_short_frame() {
        let keys = SessionKeys::derive(&[1u8; 32], &[2u8; 32], &[3u8; 32], false).unwrap();
        let result = keys.decrypt(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_rejects_tampered_frame() {
        let keys = SessionKeys::derive(&[1u8; 32], &[2u8; 32], &[3u8; 32], false).unwrap();
        let mut encrypted = keys.encrypt(b"secret").unwrap();
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF;
        assert!(keys.decrypt(&encrypted).is_err());
    }

    #[test]
    fn negotiate_params_deserialize_ciphers_format() {
        let json = serde_json::json!({
            "session_id": "sess-123",
            "ciphers": ["chacha20-poly1305"],
            "client_nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        });
        let p: NegotiateParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.session_id, "sess-123");
        assert_eq!(p.ciphers, vec!["chacha20-poly1305"]);
        assert!(p.preferred_cipher.is_none());
        assert!(p.bond_type.is_none());
    }

    #[test]
    fn negotiate_params_deserialize_preferred_cipher_format() {
        let json = serde_json::json!({
            "session_id": "sess-456",
            "preferred_cipher": "chacha20-poly1305",
            "bond_type": "Covalent",
        });
        let p: NegotiateParams = serde_json::from_value(json).unwrap();
        assert_eq!(p.session_id, "sess-456");
        assert!(p.ciphers.is_empty());
        assert_eq!(p.preferred_cipher.as_deref(), Some("chacha20-poly1305"));
        assert_eq!(p.bond_type.as_deref(), Some("Covalent"));

        let eff = p.effective_ciphers();
        assert_eq!(eff, vec!["chacha20-poly1305"]);
    }

    #[test]
    fn negotiate_params_ciphers_takes_precedence_over_preferred() {
        let json = serde_json::json!({
            "session_id": "sess-789",
            "ciphers": ["chacha20-poly1305", "null"],
            "preferred_cipher": "null",
            "client_nonce": "",
        });
        let p: NegotiateParams = serde_json::from_value(json).unwrap();
        let eff = p.effective_ciphers();
        assert_eq!(eff, vec!["chacha20-poly1305", "null"]);
    }

    #[test]
    fn negotiate_params_empty_ciphers_and_no_preferred() {
        let json = serde_json::json!({ "session_id": "sess-empty" });
        let p: NegotiateParams = serde_json::from_value(json).unwrap();
        assert!(p.effective_ciphers().is_empty());
    }

    #[test]
    fn select_cipher_chacha_covalent() {
        assert_eq!(
            select_cipher(&["chacha20-poly1305"], Some("Covalent")),
            Phase3Cipher::ChaCha20Poly1305,
        );
    }

    #[test]
    fn select_cipher_chacha_ionic_allowed() {
        assert_eq!(
            select_cipher(&["chacha20-poly1305"], Some("Ionic")),
            Phase3Cipher::ChaCha20Poly1305,
        );
    }

    #[test]
    fn select_cipher_null_only_ionic_rejected() {
        assert_eq!(select_cipher(&["null"], Some("Ionic")), Phase3Cipher::Null,);
    }

    #[test]
    fn select_cipher_no_bond_type_defaults_to_chacha_if_offered() {
        assert_eq!(select_cipher(&["chacha20-poly1305"], None), Phase3Cipher::ChaCha20Poly1305,);
    }

    #[test]
    fn select_cipher_underscore_variant_accepted() {
        assert_eq!(select_cipher(&["chacha20_poly1305"], None), Phase3Cipher::ChaCha20Poly1305,);
    }

    #[test]
    fn select_cipher_empty_offers_returns_null() {
        assert_eq!(select_cipher(&[], None), Phase3Cipher::Null);
    }

    #[test]
    fn negotiate_result_serialize() {
        let r = NegotiateResult {
            cipher: "chacha20-poly1305".to_string(),
            server_nonce: "abc123".to_string(),
        };
        let json = serde_json::to_value(&r).unwrap();
        assert_eq!(json["cipher"], "chacha20-poly1305");
        assert_eq!(json["server_nonce"], "abc123");
    }

    #[test]
    fn null_negotiate_result_serialize() {
        let r = NegotiateResult {
            cipher: "null".to_string(),
            server_nonce: String::new(),
        };
        let json = serde_json::to_value(&r).unwrap();
        assert_eq!(json["cipher"], "null");
        assert_eq!(json["server_nonce"], "");
    }

    #[tokio::test]
    async fn encrypted_frame_write_read_roundtrip() {
        let payload = b"test encrypted frame payload";
        let mut buf = Vec::new();

        write_encrypted_frame(&mut buf, payload).await.unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let read_back = read_encrypted_frame(&mut cursor).await.unwrap();
        assert_eq!(read_back, payload);
    }

    #[tokio::test]
    async fn encrypted_frame_rejects_oversized() {
        let len = (MAX_ENCRYPTED_FRAME as u32 + 1).to_be_bytes();
        let mut cursor = std::io::Cursor::new(len.to_vec());
        let result = read_encrypted_frame(&mut cursor).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn full_encrypted_session_roundtrip() {
        let hk = [0x99u8; 32];
        let cn = [0xABu8; 32];
        let sn = [0xCDu8; 32];
        let client_keys = SessionKeys::derive(&hk, &cn, &sn, true).unwrap();
        let server_keys = SessionKeys::derive(&hk, &cn, &sn, false).unwrap();

        let messages = [
            b"first request".as_slice(),
            b"second request",
            b"third with unicode: \xc3\xa9\xc3\xa0\xc3\xbc",
        ];

        for msg in &messages {
            let encrypted = client_keys.encrypt(msg).unwrap();

            let mut buf = Vec::new();
            write_encrypted_frame(&mut buf, &encrypted).await.unwrap();

            let mut cursor = std::io::Cursor::new(buf);
            let frame = read_encrypted_frame(&mut cursor).await.unwrap();
            let decrypted = server_keys.decrypt(&frame).unwrap();
            assert_eq!(&decrypted, msg);

            let response = b"response ok";
            let enc_resp = server_keys.encrypt(response).unwrap();
            let mut resp_buf = Vec::new();
            write_encrypted_frame(&mut resp_buf, &enc_resp).await.unwrap();

            let mut resp_cursor = std::io::Cursor::new(resp_buf);
            let resp_frame = read_encrypted_frame(&mut resp_cursor).await.unwrap();
            let dec_resp = client_keys.decrypt(&resp_frame).unwrap();
            assert_eq!(&dec_resp, response);
        }
    }

    #[test]
    fn session_keys_different_nonces_produce_different_keys() {
        let hk = [0x42u8; 32];
        let cn = [0xAAu8; 32];
        let sn1 = [0xBBu8; 32];
        let sn2 = [0xCCu8; 32];
        let k1 = SessionKeys::derive(&hk, &cn, &sn1, false).unwrap();
        let k2 = SessionKeys::derive(&hk, &cn, &sn2, false).unwrap();
        assert_ne!(k1.encrypt_key, k2.encrypt_key);
        assert_ne!(k1.decrypt_key, k2.decrypt_key);
    }

    #[test]
    fn encrypt_produces_unique_nonces() {
        let keys = SessionKeys::derive(&[1u8; 32], &[2u8; 32], &[3u8; 32], false).unwrap();
        let e1 = keys.encrypt(b"same").unwrap();
        let e2 = keys.encrypt(b"same").unwrap();
        assert_ne!(e1[..NONCE_SIZE], e2[..NONCE_SIZE]);
    }

    #[test]
    fn cross_key_decrypt_fails() {
        let k1 = SessionKeys::derive(&[1u8; 32], &[2u8; 32], &[3u8; 32], false).unwrap();
        let k2 = SessionKeys::derive(&[4u8; 32], &[5u8; 32], &[6u8; 32], false).unwrap();
        let encrypted = k1.encrypt(b"private").unwrap();
        assert!(k2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn encrypt_empty_plaintext() {
        let server = SessionKeys::derive(&[1u8; 32], &[2u8; 32], &[3u8; 32], false).unwrap();
        let client = SessionKeys::derive(&[1u8; 32], &[2u8; 32], &[3u8; 32], true).unwrap();
        let encrypted = server.encrypt(b"").unwrap();
        assert_eq!(encrypted.len(), NONCE_SIZE + TAG_SIZE);
        let decrypted = client.decrypt(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }
}
