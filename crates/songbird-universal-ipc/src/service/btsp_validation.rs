// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP Session Token Validation
//!
//! Validates BTSP session tokens on virtual relay requests:
//! - Structural integrity (base64 payload + signature)
//! - Timestamp freshness (5-minute window, 60s future skew tolerance)
//! - Extracts node identity and signature bytes for Phase 3.5 cryptographic verification
//!
//! Token format: `{payload_b64}.{signature_b64}` where payload JSON contains
//! `node_id`, `ts` (Unix timestamp), and optionally `nonce`.

/// BTSP token validation result.
#[derive(Debug)]
pub enum BtspValidation {
    /// No token present — allowed for backward compatibility.
    NoToken,
    /// Token present and structurally valid.
    Valid {
        node_id: Option<String>,
        /// Raw payload bytes for Phase 3.5 signature verification.
        payload_bytes: Vec<u8>,
        /// Raw signature bytes (empty if Phase 2 single-segment token).
        signature_bytes: Vec<u8>,
    },
}

const MAX_TOKEN_AGE_SECS: u64 = 300;
const MAX_FUTURE_SKEW_SECS: u64 = 60;

/// Validate BTSP session token on a relay request.
///
/// Validation steps:
/// 1. Parse JSON — if not JSON or no `_btsp_session`, returns `NoToken`
/// 2. Empty token → reject with error
/// 3. Single-segment token → accept for Phase 2 backward compatibility
/// 4. Two-segment (payload.signature): decode, validate JSON, check timestamp freshness
/// 5. Returns extracted `node_id` and signature bytes for cryptographic verification
pub fn validate_btsp_session(request_line: &str) -> Result<BtspValidation, serde_json::Value> {
    use base64::Engine as _;

    let parsed: serde_json::Value = match serde_json::from_str(request_line) {
        Ok(v) => v,
        Err(_) => return Ok(BtspValidation::NoToken),
    };

    let Some(session) = parsed.get("_btsp_session").and_then(serde_json::Value::as_str) else {
        return Ok(BtspValidation::NoToken);
    };

    let id = parsed.get("id").cloned().unwrap_or(serde_json::Value::Null);

    if session.is_empty() {
        return Err(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "BTSP session token is empty"},
            "id": id
        }));
    }

    let Some((payload_b64, sig_b64)) = session.split_once('.') else {
        return Ok(BtspValidation::Valid {
            node_id: None,
            payload_bytes: Vec::new(),
            signature_bytes: Vec::new(),
        });
    };

    let payload_bytes = base64::engine::general_purpose::STANDARD
        .decode(payload_b64)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload_b64))
        .map_err(|_| {
            serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32600, "message": "BTSP token payload is not valid base64"},
                "id": id
            })
        })?;

    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).map_err(|_| {
        serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "BTSP token payload is not valid JSON"},
            "id": id
        })
    })?;

    let ts = payload.get("ts").and_then(serde_json::Value::as_u64).ok_or_else(|| {
        serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "BTSP token missing required 'ts' field"},
            "id": id
        })
    })?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if now.saturating_sub(ts) > MAX_TOKEN_AGE_SECS {
        return Err(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "BTSP token expired"},
            "id": id
        }));
    }
    if ts.saturating_sub(now) > MAX_FUTURE_SKEW_SECS {
        return Err(serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "BTSP token timestamp is in the future"},
            "id": id
        }));
    }

    let node_id = payload.get("node_id").and_then(serde_json::Value::as_str).map(String::from);

    let signature_bytes = base64::engine::general_purpose::STANDARD
        .decode(sig_b64)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(sig_b64))
        .unwrap_or_default();

    Ok(BtspValidation::Valid {
        node_id,
        payload_bytes,
        signature_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_no_session() {
        let request = r#"{"jsonrpc":"2.0","method":"test","params":{},"id":1}"#;
        let result = validate_btsp_session(request).unwrap();
        assert!(matches!(result, BtspValidation::NoToken));
    }

    #[test]
    fn allows_simple_session_token() {
        let request =
            r#"{"jsonrpc":"2.0","method":"test","params":{},"id":1,"_btsp_session":"abc123"}"#;
        let result = validate_btsp_session(request).unwrap();
        assert!(matches!(result, BtspValidation::Valid { .. }));
    }

    #[test]
    fn rejects_empty_session() {
        let request = r#"{"jsonrpc":"2.0","method":"test","params":{},"id":1,"_btsp_session":""}"#;
        let result = validate_btsp_session(request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err["error"]["code"], -32600);
    }

    #[test]
    fn validates_structured_token() {
        use base64::Engine as _;

        let now =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let payload = serde_json::json!({"node_id": "east-gate", "ts": now, "nonce": "abc"});
        let payload_b64 =
            base64::engine::general_purpose::STANDARD.encode(payload.to_string().as_bytes());
        let sig_b64 = base64::engine::general_purpose::STANDARD.encode(b"fake-signature");
        let token = format!("{payload_b64}.{sig_b64}");

        let request = format!(
            r#"{{"jsonrpc":"2.0","method":"test","params":{{}},"id":1,"_btsp_session":"{token}"}}"#
        );
        let result = validate_btsp_session(&request).unwrap();
        match result {
            BtspValidation::Valid {
                node_id,
                signature_bytes,
                ..
            } => {
                assert_eq!(node_id.as_deref(), Some("east-gate"));
                assert!(!signature_bytes.is_empty());
            }
            _ => panic!("Expected Valid"),
        }
    }

    #[test]
    fn rejects_expired_token() {
        use base64::Engine as _;

        let old_ts =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                - 600;
        let payload = serde_json::json!({"node_id": "old-gate", "ts": old_ts});
        let payload_b64 =
            base64::engine::general_purpose::STANDARD.encode(payload.to_string().as_bytes());
        let sig_b64 = base64::engine::general_purpose::STANDARD.encode(b"sig");
        let token = format!("{payload_b64}.{sig_b64}");

        let request = format!(
            r#"{{"jsonrpc":"2.0","method":"test","params":{{}},"id":1,"_btsp_session":"{token}"}}"#
        );
        let result = validate_btsp_session(&request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err["error"]["message"].as_str().unwrap().contains("expired"));
    }

    #[test]
    fn rejects_invalid_base64_payload() {
        let token = "!!!not-base64!!!.c2ln";
        let request = format!(
            r#"{{"jsonrpc":"2.0","method":"test","params":{{}},"id":1,"_btsp_session":"{token}"}}"#
        );
        let result = validate_btsp_session(&request);
        assert!(result.is_err());
    }
}
