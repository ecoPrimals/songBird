// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Relay Security — Phase 3.5 Ed25519 signature verification via `CryptoProvider`.
//!
//! Provides `CryptoProviderVerifier` which calls the security provider's
//! `crypto.verify.ed25519` via UDS JSON-RPC. Used by the virtual relay when
//! BTSP tokens contain signed payloads.

use super::virtual_relay::BtspSignatureVerifier;

/// Phase 3.5 verifier: calls the security provider's `crypto.verify.ed25519` via UDS JSON-RPC.
///
/// The relay passes `node_id` alongside the message/signature. The security provider
/// resolves the peer's Ed25519 public key from its `TrustedIssuerRegistry` and performs
/// the verification. If the provider is offline, returns `Err` (caller rejects the request).
pub struct CryptoProviderVerifier {
    socket_path: String,
}

impl CryptoProviderVerifier {
    /// Create a verifier targeting a security provider crypto socket.
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
                .ok_or_else(|| String::from("Missing 'valid' field in verify response"))
        })
    }
}

/// Low-level UDS JSON-RPC call to the crypto provider.
async fn call_crypto_rpc(
    socket_path: &str,
    request: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("crypto provider connect ({socket_path}): {e}"))?;

    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    let mut req_bytes = serde_json::to_vec(request).map_err(|e| format!("Serialize: {e}"))?;
    req_bytes.push(b'\n');

    writer.write_all(&req_bytes).await.map_err(|e| format!("crypto provider write: {e}"))?;

    let mut response_line = String::new();
    let read_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        buf_reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| String::from("crypto provider verify timeout (5s)"))?
    .map_err(|e| format!("crypto provider read: {e}"))?;

    if read_result == 0 {
        return Err(String::from("crypto provider closed connection"));
    }

    let parsed: serde_json::Value = serde_json::from_str(response_line.trim())
        .map_err(|e| format!("crypto provider response parse: {e}"))?;

    if let Some(error) = parsed.get("error") {
        let msg =
            error.get("message").and_then(serde_json::Value::as_str).unwrap_or("unknown error");
        return Err(format!("crypto provider RPC error: {msg}"));
    }

    parsed
        .get("result")
        .cloned()
        .ok_or_else(|| String::from("Missing 'result' in crypto provider response"))
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
                "Relay: signature verifier unavailable — rejecting signed request"
            );
            let id = serde_json::from_str::<serde_json::Value>(raw_request)
                .ok()
                .and_then(|v| v.get("id").cloned())
                .unwrap_or(serde_json::Value::Null);
            Some(serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32603, "message": "Signature verifier unavailable"},
                "id": id
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn crypto_provider_verifier_fails_gracefully_on_missing_socket() {
        let verifier = CryptoProviderVerifier::new(String::from("/run/nonexistent/security.sock"));
        let result = verifier.verify("test-node", b"msg", b"sig").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("crypto provider connect"));
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

    /// Mock verifier that simulates crypto provider offline.
    struct UnavailableTestVerifier;
    impl BtspSignatureVerifier for UnavailableTestVerifier {
        fn verify(
            &self,
            _: &str,
            _: &[u8],
            _: &[u8],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send + '_>>
        {
            Box::pin(async { Err(String::from("crypto provider offline")) })
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
    async fn verify_rejects_when_verifier_unavailable() {
        let verifier = UnavailableTestVerifier;
        let result = verify_relay_signature(
            &verifier,
            Some("gate"),
            b"payload",
            b"some-sig",
            r#"{"id":1}"#,
            "/tmp/x.sock",
        )
        .await;
        assert!(result.is_some());
        let rej = result.unwrap();
        assert_eq!(rej["error"]["code"], -32603);
        assert!(rej["error"]["message"].as_str().unwrap().contains("unavailable"));
    }
}
