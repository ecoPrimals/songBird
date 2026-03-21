// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BearDog` RPC types
//!
//! JSON-RPC 2.0 message structures and TLS secrets container.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request
#[derive(Debug, Serialize)]
pub(super) struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize)]
#[allow(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
pub(super) struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Request ID (can be null for notifications per JSON-RPC 2.0 spec)
    pub id: Option<u64>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Deserialize)]
pub(super) struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    pub data: Option<Value>,
}

/// TLS traffic secrets container
///
/// Contains derived keys and IVs for TLS 1.3 traffic:
/// - `client_write_key` / `server_write_key` - Handshake traffic (for encrypting handshake messages)
/// - Application traffic (for encrypting HTTP data)
///
/// Songbird derives application traffic keys for HTTP data encryption.
#[derive(Debug, Clone)]
pub struct TlsSecrets {
    pub client_write_key: Vec<u8>,
    pub server_write_key: Vec<u8>,
    pub client_write_iv: Vec<u8>,
    pub server_write_iv: Vec<u8>,
    /// Traffic secrets (needed for RFC 8446 Section 4.4.4 Finished message)
    pub client_handshake_secret: Vec<u8>,
    pub server_handshake_secret: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: serde_json::json!({}),
            id: 1,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test\""));
    }

    #[test]
    fn test_json_rpc_response_deserialization() {
        let json = r#"{"jsonrpc": "2.0", "result": {"key": "value"}, "id": 1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_error_response() {
        let json = r#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": 1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid Request");
    }

    #[test]
    fn test_tls_secrets_creation() {
        let secrets = TlsSecrets {
            client_write_key: vec![0u8; 32],
            server_write_key: vec![0u8; 32],
            client_write_iv: vec![0u8; 12],
            server_write_iv: vec![0u8; 12],
            client_handshake_secret: vec![0u8; 32],
            server_handshake_secret: vec![0u8; 32],
        };
        assert_eq!(secrets.client_write_key.len(), 32);
        assert_eq!(secrets.client_write_iv.len(), 12);
    }

    #[test]
    fn test_fault_zero_length_keys() {
        let secrets = TlsSecrets {
            client_write_key: vec![],
            server_write_key: vec![],
            client_write_iv: vec![],
            server_write_iv: vec![],
            client_handshake_secret: vec![],
            server_handshake_secret: vec![],
        };
        assert_eq!(secrets.client_write_key.len(), 0);
    }

    #[test]
    fn test_fault_mismatched_key_sizes() {
        let secrets = TlsSecrets {
            client_write_key: vec![0u8; 16],
            server_write_key: vec![0u8; 64],
            client_write_iv: vec![0u8; 8],
            server_write_iv: vec![0u8; 24],
            client_handshake_secret: vec![0u8; 48],
            server_handshake_secret: vec![0u8; 20],
        };
        // Validation happens at crypto layer, not here
        assert_eq!(secrets.client_write_key.len(), 16);
    }
}
