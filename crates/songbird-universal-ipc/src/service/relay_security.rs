// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Relay Security — Phase 3.5 Ed25519 signature verification via `CryptoProvider`.
//!
//! Provides `CryptoProviderVerifier` which calls bearDog's `crypto.verify.ed25519`
//! via UDS JSON-RPC. Used by the virtual relay when BTSP tokens contain signed payloads.

use super::virtual_relay::BtspSignatureVerifier;

/// Phase 3.5 verifier: calls bearDog's `crypto.verify.ed25519` via UDS JSON-RPC.
///
/// The relay passes `node_id` alongside the message/signature. bearDog resolves
/// the peer's Ed25519 public key from its `TrustedIssuerRegistry` and performs
/// the verification. If bearDog is offline, returns `Err` (caller degrades gracefully).
pub struct CryptoProviderVerifier {
    socket_path: String,
}

impl CryptoProviderVerifier {
    /// Create a verifier targeting a bearDog signing socket.
    #[must_use]
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
        }
    }
}

impl BtspSignatureVerifier for CryptoProviderVerifier {
    fn verify(
        &self,
        node_id: &str,
        payload_bytes: &[u8],
        signature_bytes: &[u8],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send + '_>>
    {
        use base64::Engine;
        let socket_path = self.socket_path.clone();
        let message_b64 = base64::engine::general_purpose::STANDARD.encode(payload_bytes);
        let signature_b64 = base64::engine::general_purpose::STANDARD.encode(signature_bytes);
        let node_id = node_id.to_string();

        Box::pin(async move {
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "crypto.verify.ed25519",
                "params": {
                    "message": message_b64,
                    "signature": signature_b64,
                    "node_id": node_id
                },
                "id": 1
            });

            let response = call_crypto_rpc(&socket_path, &request).await?;

            response
                .get("valid")
                .and_then(serde_json::Value::as_bool)
                .ok_or_else(|| "Missing 'valid' field in verify response".to_string())
        })
    }
}

/// Low-level UDS JSON-RPC call to the crypto provider (bearDog signing socket).
async fn call_crypto_rpc(
    socket_path: &str,
    request: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("bearDog connect ({socket_path}): {e}"))?;

    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    let mut req_bytes = serde_json::to_vec(request).map_err(|e| format!("Serialize: {e}"))?;
    req_bytes.push(b'\n');

    writer.write_all(&req_bytes).await.map_err(|e| format!("bearDog write: {e}"))?;

    let mut response_line = String::new();
    let read_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        buf_reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| "bearDog verify timeout (5s)".to_string())?
    .map_err(|e| format!("bearDog read: {e}"))?;

    if read_result == 0 {
        return Err("bearDog closed connection".to_string());
    }

    let parsed: serde_json::Value = serde_json::from_str(response_line.trim())
        .map_err(|e| format!("bearDog response parse: {e}"))?;

    if let Some(error) = parsed.get("error") {
        let msg =
            error.get("message").and_then(serde_json::Value::as_str).unwrap_or("unknown error");
        return Err(format!("bearDog RPC error: {msg}"));
    }

    parsed.get("result").cloned().ok_or_else(|| "Missing 'result' in bearDog response".to_string())
}

/// Phase 3.5 signature verification for a relay request.
///
/// Returns `Some(rejection_json)` if signature is invalid (tampered),
/// `None` if verification passed or was skipped (no sig / verifier unavailable).
pub async fn verify_relay_signature(
    verifier: &dyn BtspSignatureVerifier,
    node_id: Option<&str>,
    payload_bytes: &[u8],
    signature_bytes: &[u8],
    raw_request: &str,
    native_target: &str,
) -> Option<serde_json::Value> {
    if signature_bytes.is_empty() {
        tracing::debug!(
            target: "relay_audit",
            peer = node_id.unwrap_or("unknown"),
            native = native_target,
            "Relay: authenticated request (no signature — Phase 2 token)"
        );
        return None;
    }

    let peer = node_id.unwrap_or("unknown");
    match verifier.verify(peer, payload_bytes, signature_bytes).await {
        Ok(true) => {
            tracing::debug!(
                target: "relay_audit",
                peer,
                native = native_target,
                "Relay: signature verified"
            );
            None
        }
        Ok(false) => {
            tracing::warn!(
                target: "relay_audit",
                peer,
                "Relay: BTSP signature verification FAILED — tampered token"
            );
            let id = serde_json::from_str::<serde_json::Value>(raw_request)
                .ok()
                .and_then(|v| v.get("id").cloned())
                .unwrap_or(serde_json::Value::Null);
            Some(serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32603, "message": "BTSP signature verification failed"},
                "id": id
            }))
        }
        Err(e) => {
            tracing::warn!(
                target: "relay_audit",
                peer,
                error = %e,
                "Relay: signature verifier unavailable, accepting on trust"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn crypto_provider_verifier_fails_gracefully_on_missing_socket() {
        let verifier = CryptoProviderVerifier::new("/run/nonexistent/beardog.sock".to_string());
        let result = verifier.verify("test-node", b"msg", b"sig").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("bearDog connect"));
    }

    /// Mock verifier that always rejects.
    struct RejectingVerifier;
    impl BtspSignatureVerifier for RejectingVerifier {
        fn verify(
            &self,
            _: &str,
            _: &[u8],
            _: &[u8],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send + '_>>
        {
            Box::pin(async { Ok(false) })
        }
    }

    /// Mock verifier that simulates bearDog offline.
    struct UnavailableVerifier;
    impl BtspSignatureVerifier for UnavailableVerifier {
        fn verify(
            &self,
            _: &str,
            _: &[u8],
            _: &[u8],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send + '_>>
        {
            Box::pin(async { Err("bearDog offline".to_string()) })
        }
    }

    #[tokio::test]
    async fn verify_rejects_invalid_signature() {
        let verifier = RejectingVerifier;
        let result = verify_relay_signature(
            &verifier,
            Some("evil-gate"),
            b"payload",
            b"bad-sig",
            r#"{"jsonrpc":"2.0","method":"test","id":42}"#,
            "/tmp/native.sock",
        )
        .await;
        assert!(result.is_some());
        let rej = result.unwrap();
        assert_eq!(rej["error"]["code"], -32603);
        assert_eq!(rej["id"], 42);
    }

    #[tokio::test]
    async fn verify_skips_empty_signature() {
        let verifier = RejectingVerifier;
        let result = verify_relay_signature(
            &verifier,
            Some("gate"),
            b"payload",
            b"",
            r#"{"id":1}"#,
            "/tmp/x.sock",
        )
        .await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn verify_degrades_when_verifier_unavailable() {
        let verifier = UnavailableVerifier;
        let result = verify_relay_signature(
            &verifier,
            Some("gate"),
            b"payload",
            b"some-sig",
            r#"{"id":1}"#,
            "/tmp/x.sock",
        )
        .await;
        assert!(result.is_none());
    }
}
