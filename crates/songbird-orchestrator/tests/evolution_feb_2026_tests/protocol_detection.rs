// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP vs TLS first-byte heuristics (same-port protocol detection).

#[test]
fn test_tls_handshake_detection() {
    // TLS ClientHello starts with content type 0x16 (Handshake)
    let tls_client_hello_start: [u8; 5] = [
        0x16, // Content type: Handshake
        0x03, 0x01, // Version: TLS 1.0 (compat)
        0x00, 0x05, // Length (placeholder)
    ];

    assert_eq!(tls_client_hello_start[0], 0x16, "TLS record starts with 0x16");
    assert!(is_tls_record(tls_client_hello_start[0]));
}

#[test]
fn test_http_method_detection() {
    // HTTP methods start with ASCII characters
    let http_methods = vec![
        ("GET", 0x47),
        ("POST", 0x50),
        ("PUT", 0x50),
        ("HEAD", 0x48),
        ("DELETE", 0x44),
        ("OPTIONS", 0x4F),
        ("PATCH", 0x50),
        ("CONNECT", 0x43),
    ];

    for (method, first_byte) in http_methods {
        assert_eq!(
            method.as_bytes()[0],
            first_byte,
            "{method} should start with 0x{first_byte:02X}"
        );
        assert!(!is_tls_record(first_byte), "{method} should not be detected as TLS");
    }
}

#[test]
fn test_protocol_detection_boundary() {
    // Edge cases for protocol detection
    assert!(is_tls_record(0x16), "0x16 is TLS Handshake");
    assert!(!is_tls_record(0x17), "0x17 is TLS Application Data (not handshake start)");
    assert!(!is_tls_record(0x14), "0x14 is TLS Change Cipher Spec");
    assert!(!is_tls_record(0x15), "0x15 is TLS Alert");

    // ASCII printable range (HTTP)
    for byte in 0x20..=0x7E {
        if byte != 0x16 {
            assert!(!is_tls_record(byte), "ASCII byte 0x{byte:02X} should not be TLS");
        }
    }
}

#[test]
fn test_http_request_first_bytes() {
    // Actual HTTP request first bytes
    let http_requests: Vec<&[u8]> = vec![
        b"GET / HTTP/1.1\r\n",
        b"POST /api HTTP/1.1\r\n",
        b"PUT /resource HTTP/1.1\r\n",
        b"HEAD /status HTTP/1.1\r\n",
    ];

    for request in http_requests {
        assert!(!is_tls_record(request[0]), "HTTP request should not be detected as TLS");
    }
}

#[test]
fn test_tls_record_types() {
    // All TLS record types
    let tls_record_types = vec![
        (0x14, "ChangeCipherSpec"),
        (0x15, "Alert"),
        (0x16, "Handshake"),
        (0x17, "ApplicationData"),
    ];

    for (byte, name) in tls_record_types {
        // Only Handshake (0x16) should trigger TLS detection for initial connection
        if byte == 0x16 {
            assert!(is_tls_record(byte), "{name} should be detected as TLS handshake");
        } else {
            // Other record types wouldn't be the first byte of a new TLS connection
            assert!(!is_tls_record(byte), "{name} should not be first byte");
        }
    }
}

/// Helper: Check if first byte indicates TLS record (handshake)
const fn is_tls_record(byte: u8) -> bool {
    byte == 0x16 // TLS Handshake content type
}
