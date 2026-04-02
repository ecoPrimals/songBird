// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! NFC error types

use thiserror::Error;

/// Result type for NFC operations
pub type Result<T> = std::result::Result<T, NfcError>;

/// NFC protocol errors
#[derive(Debug, Error)]
pub enum NfcError {
    /// Protocol version mismatch
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u8),

    /// Invalid message type
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),

    /// Payload too large
    #[error("Payload too large: {0} bytes (max: {1})")]
    PayloadTooLarge(usize, usize),

    /// Malformed frame
    #[error("Malformed frame: {0}")]
    MalformedFrame(String),

    /// Crypto error (`BearDog` delegation)
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Platform error (NFC hardware/driver)
    #[error("Platform error: {0}")]
    Platform(String),

    /// NFC stack not integrated for this target (JNI, CoreNFC, libnfc, etc.)
    #[error("Platform unsupported: {0}")]
    PlatformUnsupported(String),

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// Connection lost
    #[error("NFC connection lost")]
    ConnectionLost,

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn unsupported_version_display() {
        let e = NfcError::UnsupportedVersion(0x02);
        assert_eq!(
            e.to_string(),
            "Unsupported protocol version: 2",
            "u8 should format as decimal in error message"
        );
    }

    #[test]
    fn invalid_message_type_display() {
        let e = NfcError::InvalidMessageType(0xab);
        assert_eq!(e.to_string(), "Invalid message type: 171");
    }

    #[test]
    fn payload_too_large_display() {
        let e = NfcError::PayloadTooLarge(2048, 1024);
        assert_eq!(e.to_string(), "Payload too large: 2048 bytes (max: 1024)");
    }

    #[test]
    fn malformed_frame_display() {
        let e = NfcError::MalformedFrame("bad crc".to_string());
        assert_eq!(e.to_string(), "Malformed frame: bad crc");
    }

    #[test]
    fn crypto_platform_and_timeout_display() {
        assert_eq!(NfcError::Crypto("x".to_string()).to_string(), "Crypto error: x");
        assert_eq!(
            NfcError::Platform("nfc off".to_string()).to_string(),
            "Platform error: nfc off"
        );
        assert_eq!(
            NfcError::PlatformUnsupported("no driver".to_string()).to_string(),
            "Platform unsupported: no driver"
        );
        assert_eq!(NfcError::Timeout.to_string(), "Operation timed out");
        assert_eq!(NfcError::ConnectionLost.to_string(), "NFC connection lost");
    }

    #[test]
    fn io_error_roundtrips_via_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let e: NfcError = io_err.into();
        assert!(
            e.to_string().contains("missing"),
            "Io variant should wrap message: {}",
            e
        );
    }

    #[test]
    fn serialization_error_from_json() {
        let err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let e: NfcError = err.into();
        assert!(
            e.to_string().starts_with("Serialization error:"),
            "expected serde prefix, got {e}"
        );
    }
}
