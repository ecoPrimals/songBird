// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for lineage relay system

use thiserror::Error;

/// Result type for lineage relay operations
pub type Result<T> = std::result::Result<T, LineageRelayError>;

/// Errors that can occur in the lineage relay system
#[derive(Debug, Error)]
pub enum LineageRelayError {
    /// Lineage verification failed
    #[error("Lineage verification failed: {0}")]
    LineageVerificationFailed(String),

    /// No ancestors available for relay
    #[error("No ancestors available for relay: {0}")]
    NoRelayAvailable(String),

    /// `BirdSong` encryption/decryption failed
    #[error("BirdSong operation failed: {0}")]
    BirdSongError(String),

    /// Relay authorization denied
    #[error("Relay authorization denied: {0}")]
    RelayDenied(String),

    /// Direct connection failed (expected, not an error condition)
    #[error("Direct connection not possible: {0}")]
    DirectConnectionFailed(String),

    /// Relay session error
    #[error("Relay session error: {0}")]
    SessionError(String),

    /// Relay session not found
    #[error("Relay session not found: {0}")]
    SessionNotFound(String),

    /// Network communication error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Invalid protocol message
    #[error("Invalid protocol: {0}")]
    InvalidProtocol(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<String> for LineageRelayError {
    fn from(msg: String) -> Self {
        Self::Other(msg)
    }
}

impl From<&str> for LineageRelayError {
    fn from(msg: &str) -> Self {
        Self::Other(msg.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::LineageRelayError;

    #[test]
    fn display_lineage_verification() {
        let e = LineageRelayError::LineageVerificationFailed("bad".into());
        assert!(e.to_string().contains("bad"));
    }

    #[test]
    fn display_relay_denied() {
        let e = LineageRelayError::RelayDenied("no".into());
        assert!(e.to_string().contains("no"));
    }

    #[test]
    fn from_string_and_str() {
        let e: LineageRelayError = "hello".into();
        assert!(matches!(e, LineageRelayError::Other(ref s) if s == "hello"));
        let e2: LineageRelayError = String::from("x").into();
        assert!(matches!(e2, LineageRelayError::Other(_)));
    }

    #[test]
    fn session_and_network_errors() {
        assert!(LineageRelayError::SessionNotFound("s".into()).to_string().contains('s'));
        assert!(LineageRelayError::NetworkError("n".into()).to_string().contains('n'));
        assert!(LineageRelayError::Timeout("t".into()).to_string().contains('t'));
    }

    #[test]
    fn protocol_and_config_errors() {
        assert!(LineageRelayError::InvalidProtocol("p".into()).to_string().contains('p'));
        assert!(LineageRelayError::ConfigError("c".into()).to_string().contains('c'));
        assert!(LineageRelayError::SessionError("e".into()).to_string().contains('e'));
    }

    #[test]
    fn no_relay_available_and_birdsong_error_display() {
        let e = LineageRelayError::NoRelayAvailable("east".into());
        assert!(e.to_string().contains("east"));
        let b = LineageRelayError::BirdSongError("decrypt".into());
        assert!(b.to_string().contains("decrypt"));
    }

    #[test]
    fn direct_connection_failed_display() {
        let e = LineageRelayError::DirectConnectionFailed("nat".into());
        assert!(e.to_string().contains("nat"));
    }

    #[test]
    fn serialization_error_from_serde_value() {
        let err = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let e: LineageRelayError = err.into();
        assert!(e.to_string().contains("Serialization"), "expected serde wrapper, got {e}");
    }

    #[test]
    fn io_error_from_into() {
        let io = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe");
        let e: LineageRelayError = io.into();
        assert!(e.to_string().contains("pipe"), "{e}");
    }

    #[test]
    fn other_variant_preserves_message() {
        let e = LineageRelayError::Other("custom failure".into());
        assert_eq!(e.to_string(), "custom failure");
    }

    #[test]
    fn lineage_verification_failed_contains_prefix() {
        let e = LineageRelayError::LineageVerificationFailed("sig bad".into());
        assert!(e.to_string().contains("Lineage verification failed"));
        assert!(e.to_string().contains("sig bad"));
    }

    #[test]
    fn no_relay_available_contains_detail() {
        let e = LineageRelayError::NoRelayAvailable("no path".into());
        assert!(e.to_string().contains("No ancestors available"));
        assert!(e.to_string().contains("no path"));
    }

    #[test]
    fn session_error_and_not_found_distinct() {
        let s = LineageRelayError::SessionError("alloc".into());
        let n = LineageRelayError::SessionNotFound("sid".into());
        assert!(s.to_string().contains("Relay session error"));
        assert!(s.to_string().contains("alloc"));
        assert!(n.to_string().contains("Relay session not found"));
    }
}
