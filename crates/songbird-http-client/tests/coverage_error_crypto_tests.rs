// SPDX-License-Identifier: AGPL-3.0-only
//! Coverage tests for songbird_http_client::error and songbird_http_client::crypto::capability
//!
//! Tests error type construction, Display impls, From conversions,
//! and TLS secret structs.

use songbird_http_client::crypto::{TlsApplicationSecrets, TlsHandshakeSecrets};
use songbird_http_client::error::Error;

// ═══════════════════════════════════════════════════════════════════════
// Error variant construction and Display
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_error_beardog_rpc() {
    let err = Error::BearDogRpc("connection refused".to_string());
    assert!(err.to_string().contains("BearDog RPC error"));
    assert!(err.to_string().contains("connection refused"));
}

#[test]
fn test_error_tls_handshake() {
    let err = Error::TlsHandshake("certificate expired".to_string());
    assert!(err.to_string().contains("TLS handshake failed"));
    assert!(err.to_string().contains("certificate expired"));
}

#[test]
fn test_error_tls_record() {
    let err = Error::TlsRecord("record too large".to_string());
    assert!(err.to_string().contains("TLS record layer error"));
}

#[test]
fn test_error_tls_alert() {
    let err = Error::TlsAlert("bad_record_mac".to_string());
    assert!(err.to_string().contains("TLS alert"));
}

#[test]
fn test_error_http_protocol() {
    let err = Error::HttpProtocol("invalid chunked encoding".to_string());
    assert!(err.to_string().contains("HTTP protocol error"));
}

#[test]
fn test_error_connection() {
    let err = Error::Connection("refused".to_string());
    assert!(err.to_string().contains("Connection error"));
}

#[test]
fn test_error_invalid_url() {
    let err = Error::InvalidUrl("not-a-url".to_string());
    assert!(err.to_string().contains("Invalid URL"));
}

#[test]
fn test_error_timeout() {
    let err = Error::Timeout;
    assert!(err.to_string().contains("Request timeout"));
}

#[test]
fn test_error_invalid_response() {
    let err = Error::InvalidResponse("missing content-length".to_string());
    assert!(err.to_string().contains("Invalid response"));
}

#[test]
fn test_error_other() {
    let err = Error::Other("unexpected".to_string());
    assert!(err.to_string().contains("Other error"));
}

#[test]
fn test_error_hyper() {
    let err = Error::Hyper("connection reset".to_string());
    assert!(err.to_string().contains("Hyper error"));
}

// ═══════════════════════════════════════════════════════════════════════
// Error From conversions
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let err: Error = io_err.into();
    assert!(err.to_string().contains("IO error"));
}

#[test]
fn test_error_from_serde_json() {
    let json_err = serde_json::from_str::<String>("not-json").unwrap_err();
    let err: Error = json_err.into();
    assert!(err.to_string().contains("JSON error"));
}

#[test]
fn test_error_from_anyhow() {
    let anyhow_err = anyhow::anyhow!("something went wrong");
    let err: Error = anyhow_err.into();
    assert!(err.to_string().contains("Other error"));
    assert!(err.to_string().contains("something went wrong"));
}

#[test]
fn test_error_debug() {
    let err = Error::Timeout;
    let debug = format!("{:?}", err);
    assert!(debug.contains("Timeout"));
}

// ═══════════════════════════════════════════════════════════════════════
// TlsHandshakeSecrets
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tls_handshake_secrets_construction() {
    let secrets = TlsHandshakeSecrets {
        client_handshake_secret: vec![1, 2, 3, 4],
        server_handshake_secret: vec![5, 6, 7, 8],
        client_write_key: vec![9, 10, 11, 12, 13, 14, 15, 16],
        client_write_iv: vec![17, 18, 19, 20],
        server_write_key: vec![21, 22, 23, 24, 25, 26, 27, 28],
        server_write_iv: vec![29, 30, 31, 32],
        handshake_secret: vec![33, 34, 35, 36],
    };

    assert_eq!(secrets.client_handshake_secret.len(), 4);
    assert_eq!(secrets.server_handshake_secret.len(), 4);
    assert_eq!(secrets.client_write_key.len(), 8);
    assert_eq!(secrets.server_write_key.len(), 8);
    assert_eq!(secrets.handshake_secret.len(), 4);
}

#[test]
fn test_tls_handshake_secrets_clone() {
    let secrets = TlsHandshakeSecrets {
        client_handshake_secret: vec![0u8; 32],
        server_handshake_secret: vec![0u8; 32],
        client_write_key: vec![0u8; 16],
        client_write_iv: vec![0u8; 12],
        server_write_key: vec![0u8; 16],
        server_write_iv: vec![0u8; 12],
        handshake_secret: vec![0u8; 32],
    };

    let cloned = secrets.clone();
    assert_eq!(secrets.client_handshake_secret, cloned.client_handshake_secret);
    assert_eq!(secrets.handshake_secret, cloned.handshake_secret);
}

#[test]
fn test_tls_handshake_secrets_debug() {
    let secrets = TlsHandshakeSecrets {
        client_handshake_secret: vec![0u8; 32],
        server_handshake_secret: vec![0u8; 32],
        client_write_key: vec![0u8; 16],
        client_write_iv: vec![0u8; 12],
        server_write_key: vec![0u8; 16],
        server_write_iv: vec![0u8; 12],
        handshake_secret: vec![0u8; 32],
    };

    let debug = format!("{:?}", secrets);
    assert!(debug.contains("TlsHandshakeSecrets"));
}

// ═══════════════════════════════════════════════════════════════════════
// TlsApplicationSecrets
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tls_application_secrets_construction() {
    let secrets = TlsApplicationSecrets {
        client_traffic_secret: vec![0u8; 32],
        server_traffic_secret: vec![0u8; 32],
        client_write_key: vec![0u8; 16],
        client_write_iv: vec![0u8; 12],
        server_write_key: vec![0u8; 16],
        server_write_iv: vec![0u8; 12],
    };

    assert_eq!(secrets.client_traffic_secret.len(), 32);
    assert_eq!(secrets.server_traffic_secret.len(), 32);
    assert_eq!(secrets.client_write_key.len(), 16);
    assert_eq!(secrets.client_write_iv.len(), 12);
}

#[test]
fn test_tls_application_secrets_clone() {
    let secrets = TlsApplicationSecrets {
        client_traffic_secret: vec![1, 2, 3],
        server_traffic_secret: vec![4, 5, 6],
        client_write_key: vec![7, 8, 9],
        client_write_iv: vec![10, 11, 12],
        server_write_key: vec![13, 14, 15],
        server_write_iv: vec![16, 17, 18],
    };

    let cloned = secrets.clone();
    assert_eq!(secrets.client_traffic_secret, cloned.client_traffic_secret);
    assert_eq!(secrets.server_write_key, cloned.server_write_key);
}

#[test]
fn test_tls_application_secrets_debug() {
    let secrets = TlsApplicationSecrets {
        client_traffic_secret: vec![0u8; 32],
        server_traffic_secret: vec![0u8; 32],
        client_write_key: vec![0u8; 16],
        client_write_iv: vec![0u8; 12],
        server_write_key: vec![0u8; 16],
        server_write_iv: vec![0u8; 12],
    };

    let debug = format!("{:?}", secrets);
    assert!(debug.contains("TlsApplicationSecrets"));
}

