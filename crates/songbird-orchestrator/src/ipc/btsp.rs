// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP (BearDog Secure Tunnel Protocol) server-side handshake
//!
//! Implements the server half of the BTSP 4-step handshake on incoming
//! UDS connections, delegating all crypto to BearDog via `SecurityRpcClient`.
//!
//! ## Protocol Flow (server perspective)
//!
//! ```text
//! 1. Read   ClientHello        { version, client_ephemeral_pub }
//! 2. Send   ServerHello        { version, server_ephemeral_pub, challenge }
//! 3. Read   ChallengeResponse  { response, preferred_cipher }
//! 4. Send   HandshakeComplete  { cipher, session_id }
//!    — or —
//!    Send   HandshakeError     { error, reason }  + close
//! ```
//!
//! ## Reference
//!
//! `BTSP_PROTOCOL_STANDARD.md` v1.0 — wateringHole (ecoPrimals Core Standards)

use anyhow::{Context, Result, bail};
use base64::Engine as _;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info, warn};

use songbird_http_client::SecurityRpcClient;

// ─── Wire types ──────────────────────────────────────────────────────────────

const BTSP_VERSION: u8 = 1;
const MAX_HANDSHAKE_FRAME: u32 = 8192;
const CHALLENGE_LEN: usize = 32;

#[derive(Debug, Serialize, Deserialize)]
struct ClientHello {
    version: u8,
    client_ephemeral_pub: String, // base64
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerHello {
    version: u8,
    server_ephemeral_pub: String, // base64
    challenge: String,            // base64
}

#[derive(Debug, Serialize, Deserialize)]
struct ChallengeResponse {
    response: String,         // base64 HMAC-SHA256
    preferred_cipher: String, // e.g. "chacha20_poly1305"
}

#[derive(Debug, Serialize, Deserialize)]
struct HandshakeComplete {
    cipher: String,
    session_id: String, // hex
}

#[derive(Debug, Serialize, Deserialize)]
struct HandshakeError {
    error: String,
    reason: String,
}

/// Outcome of a successful BTSP handshake.
#[derive(Debug)]
pub struct BtspSession {
    pub session_id: String,
    pub cipher: String,
    pub session_key: Vec<u8>,
}

// ─── Length-prefixed framing ─────────────────────────────────────────────────
//
// All BTSP messages (including handshake) use 4-byte big-endian length prefix.

async fn read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await.context("BTSP: failed to read frame length")?;

    let len = u32::from_be_bytes(len_buf);
    if len > MAX_HANDSHAKE_FRAME {
        bail!("BTSP: frame too large ({len} > {MAX_HANDSHAKE_FRAME})");
    }

    let mut payload = vec![0u8; len as usize];
    reader.read_exact(&mut payload).await.context("BTSP: failed to read frame payload")?;

    Ok(payload)
}

async fn write_frame<W: AsyncWriteExt + Unpin>(writer: &mut W, payload: &[u8]) -> Result<()> {
    let len = u32::try_from(payload.len()).context("BTSP: payload exceeds u32::MAX")?;
    writer.write_all(&len.to_be_bytes()).await.context("BTSP: failed to write frame length")?;
    writer.write_all(payload).await.context("BTSP: failed to write frame payload")?;
    writer.flush().await.context("BTSP: failed to flush")?;
    Ok(())
}

// ─── Handshake ───────────────────────────────────────────────────────────────

/// Perform the server-side BTSP handshake on an accepted connection.
///
/// Delegates all crypto to BearDog via `security_client`. On success returns
/// a [`BtspSession`] with the negotiated cipher and session key. On failure,
/// sends a `HandshakeError` to the client and returns `Err`.
///
/// # Errors
///
/// Returns an error if:
/// - The client sends malformed handshake messages
/// - BearDog is unreachable or rejects the session
/// - The client fails the challenge-response (wrong family seed)
/// - Cipher negotiation is disallowed by `BondingPolicy`
pub async fn perform_server_handshake<S>(
    stream: &mut S,
    security_client: &Arc<SecurityRpcClient>,
) -> Result<BtspSession>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    // Step 1: Read ClientHello
    debug!("BTSP handshake: awaiting ClientHello");
    let hello_bytes = read_frame(stream).await?;
    let client_hello: ClientHello =
        serde_json::from_slice(&hello_bytes).context("BTSP: malformed ClientHello")?;

    if client_hello.version != BTSP_VERSION {
        let err = HandshakeError {
            error: "handshake_failed".into(),
            reason: format!(
                "unsupported BTSP version {} (expected {BTSP_VERSION})",
                client_hello.version
            ),
        };
        let _ = write_frame(stream, &serde_json::to_vec(&err)?).await;
        bail!("BTSP: client sent unsupported version {}", client_hello.version);
    }

    let client_pub = BASE64_STANDARD
        .decode(&client_hello.client_ephemeral_pub)
        .context("BTSP: invalid base64 in client_ephemeral_pub")?;

    // Generate challenge (32 random bytes via getrandom)
    let mut challenge = vec![0u8; CHALLENGE_LEN];
    getrandom::fill(&mut challenge).map_err(|e| anyhow::anyhow!("BTSP: getrandom failed: {e}"))?;

    // Step 2: Call BearDog to create session
    let session = security_client
        .btsp_session_create(&client_pub, &challenge)
        .await
        .context("BTSP: BearDog btsp.session.create failed")?;

    info!("BTSP handshake: session {} created, sending ServerHello", session.session_id);

    let server_hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: BASE64_STANDARD.encode(&session.server_ephemeral_pub),
        challenge: BASE64_STANDARD.encode(&challenge),
    };
    write_frame(stream, &serde_json::to_vec(&server_hello)?).await?;

    // Step 3: Read ChallengeResponse
    debug!("BTSP handshake: awaiting ChallengeResponse");
    let resp_bytes = read_frame(stream).await?;
    let challenge_resp: ChallengeResponse =
        serde_json::from_slice(&resp_bytes).context("BTSP: malformed ChallengeResponse")?;

    let client_response = BASE64_STANDARD
        .decode(&challenge_resp.response)
        .context("BTSP: invalid base64 in challenge response")?;

    // Step 4: Verify via BearDog
    let verification = security_client
        .btsp_session_verify(
            &session.session_id,
            &client_response,
            &client_pub,
            &session.server_ephemeral_pub,
            &challenge,
        )
        .await
        .context("BTSP: BearDog btsp.session.verify failed")?;

    if !verification.verified {
        error!("BTSP handshake: FAILED for session {} (wrong family seed)", session.session_id);
        let err = HandshakeError {
            error: "handshake_failed".into(),
            reason: "family_verification".into(),
        };
        let _ = write_frame(stream, &serde_json::to_vec(&err)?).await;
        bail!("BTSP: client failed family verification");
    }

    let session_key =
        verification.session_key.context("BTSP: verified but no session_key returned")?;

    // Negotiate cipher
    let preferred = &challenge_resp.preferred_cipher;
    let negotiation = security_client
        .btsp_negotiate(&session.session_id, parse_cipher(preferred), "Covalent")
        .await
        .context("BTSP: BearDog btsp.negotiate failed")?;

    if !negotiation.allowed {
        warn!(
            "BTSP: cipher '{}' not allowed by BondingPolicy, session {}",
            preferred, session.session_id
        );
        let err = HandshakeError {
            error: "handshake_failed".into(),
            reason: format!("cipher '{preferred}' not allowed"),
        };
        let _ = write_frame(stream, &serde_json::to_vec(&err)?).await;
        bail!("BTSP: cipher negotiation rejected");
    }

    // Send HandshakeComplete
    let complete = HandshakeComplete {
        cipher: negotiation.cipher.clone(),
        session_id: session.session_id.clone(),
    };
    write_frame(stream, &serde_json::to_vec(&complete)?).await?;

    info!(
        "BTSP handshake: COMPLETE for session {} (cipher: {})",
        session.session_id, negotiation.cipher
    );

    Ok(BtspSession {
        session_id: session.session_id,
        cipher: negotiation.cipher,
        session_key,
    })
}

fn parse_cipher(s: &str) -> songbird_http_client::BtspCipher {
    match s {
        "chacha20_poly1305" | "chacha20" => songbird_http_client::BtspCipher::ChaCha20Poly1305,
        "hmac_plain" | "hmac" => songbird_http_client::BtspCipher::HmacPlain,
        "null" | "none" => songbird_http_client::BtspCipher::Null,
        _ => songbird_http_client::BtspCipher::ChaCha20Poly1305,
    }
}

/// Check whether BTSP handshake is required for the current configuration.
///
/// Returns `true` when `FAMILY_ID` is set to a non-default, non-empty value
/// AND `BIOMEOS_INSECURE` is not set. In all other cases the connection
/// proceeds with raw newline-delimited JSON-RPC (development mode).
#[must_use]
pub fn btsp_required() -> bool {
    btsp_required_with(|k| songbird_process_env::var(k))
}

/// Injectable variant for testing.
#[must_use]
pub fn btsp_required_with<F>(env_reader: F) -> bool
where
    F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
{
    let insecure = env_reader("BIOMEOS_INSECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if insecure {
        return false;
    }

    let fid = env_reader("FAMILY_ID")
        .or_else(|_| env_reader("SONGBIRD_FAMILY_ID"))
        .or_else(|_| env_reader("BIOMEOS_FAMILY_ID"))
        .unwrap_or_default();

    !fid.is_empty() && fid != "default"
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_map(
        pairs: Vec<(&str, &str)>,
    ) -> impl Fn(&str) -> std::result::Result<String, std::env::VarError> {
        let map: HashMap<String, String> =
            pairs.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |key: &str| map.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn btsp_not_required_without_family_id() {
        assert!(!btsp_required_with(env_map(vec![])));
    }

    #[test]
    fn btsp_not_required_with_default_family() {
        assert!(!btsp_required_with(env_map(vec![("FAMILY_ID", "default")])));
    }

    #[test]
    fn btsp_required_with_production_family() {
        assert!(btsp_required_with(env_map(vec![("FAMILY_ID", "nat0")])));
    }

    #[test]
    fn btsp_not_required_when_insecure() {
        assert!(!btsp_required_with(env_map(vec![
            ("FAMILY_ID", "nat0"),
            ("BIOMEOS_INSECURE", "1"),
        ])));
    }

    #[test]
    fn btsp_required_with_songbird_family_id() {
        assert!(btsp_required_with(env_map(vec![("SONGBIRD_FAMILY_ID", "prod-family"),])));
    }

    #[test]
    fn btsp_not_required_with_empty_family() {
        assert!(!btsp_required_with(env_map(vec![("FAMILY_ID", "")])));
    }

    #[test]
    fn parse_cipher_variants() {
        assert_eq!(
            parse_cipher("chacha20_poly1305"),
            songbird_http_client::BtspCipher::ChaCha20Poly1305
        );
        assert_eq!(parse_cipher("chacha20"), songbird_http_client::BtspCipher::ChaCha20Poly1305);
        assert_eq!(parse_cipher("hmac_plain"), songbird_http_client::BtspCipher::HmacPlain);
        assert_eq!(parse_cipher("null"), songbird_http_client::BtspCipher::Null);
        assert_eq!(parse_cipher("unknown"), songbird_http_client::BtspCipher::ChaCha20Poly1305);
    }

    #[test]
    fn wire_types_serde_roundtrip() {
        let hello = ClientHello {
            version: 1,
            client_ephemeral_pub: "AAAA".to_string(),
        };
        let json = serde_json::to_vec(&hello).unwrap();
        let back: ClientHello = serde_json::from_slice(&json).unwrap();
        assert_eq!(back.version, 1);

        let server = ServerHello {
            version: 1,
            server_ephemeral_pub: "BBBB".to_string(),
            challenge: "CCCC".to_string(),
        };
        let json = serde_json::to_vec(&server).unwrap();
        let back: ServerHello = serde_json::from_slice(&json).unwrap();
        assert_eq!(back.challenge, "CCCC");

        let resp = ChallengeResponse {
            response: "DDDD".to_string(),
            preferred_cipher: "chacha20_poly1305".to_string(),
        };
        let json = serde_json::to_vec(&resp).unwrap();
        let back: ChallengeResponse = serde_json::from_slice(&json).unwrap();
        assert_eq!(back.preferred_cipher, "chacha20_poly1305");

        let complete = HandshakeComplete {
            cipher: "chacha20_poly1305".to_string(),
            session_id: "abcdef01".to_string(),
        };
        let json = serde_json::to_vec(&complete).unwrap();
        let back: HandshakeComplete = serde_json::from_slice(&json).unwrap();
        assert_eq!(back.session_id, "abcdef01");
    }

    #[tokio::test]
    async fn framing_roundtrip() {
        let payload = b"hello btsp";
        let mut buf = Vec::new();
        write_frame(&mut buf, payload).await.unwrap();
        let mut cursor = std::io::Cursor::new(buf);
        let got = read_frame(&mut cursor).await.unwrap();
        assert_eq!(got, payload);
    }

    #[tokio::test]
    async fn frame_too_large_rejected() {
        let len = (MAX_HANDSHAKE_FRAME + 1).to_be_bytes();
        let mut cursor = std::io::Cursor::new(len.to_vec());
        let result = read_frame(&mut cursor).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("too large"), "got: {err}");
    }
}
